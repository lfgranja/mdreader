//! T067b — US5 property: idle-drain + zero-billing
//! (FR-015a / FR-015c / FR-027 / SC-050 / SC-050b).
//!
//! Proptest invariants, verbatim from tasks.md T067b: after
//! entering Paused-idle, the engine-spy records zero
//! synthesised characters/bytes for a simulated idle of
//! ≥1 h AND the pre-roll buffer is empty throughout; a
//! Play press after idle begins pre-roll and produces
//! audible audio within the FR-027 bound; discarded
//! pre-roll bills ≤ the smallest documented per-request
//! byte budget recorded in `docs/byte-budget.md` (T007c)
//! on a per-character engine-spy.
//!
//! Spec anchors: FR-015a (once the reader has acted to
//! hear audio, the pipeline prepares ahead of the
//! playback point without playing it — the pre-roll MUST
//! NOT advance position, MUST hold no more than 60 s at
//! any moment, MUST NOT grow while a session remains
//! paused, and MUST be abandoned at session end); FR-015c
//! (while a document sits paused and the reader has taken
//! no action, the system MUST NOT prepare audio and MUST
//! NOT spend synthesis budget until a Play or resume
//! press — pre-roll follows the reader's intent to
//! listen, not the existence of a paused session); FR-027
//! (audible audio within 3 s of the user's Play press —
//! one of the three latency bounds alongside CHK001's
//! headphone-pull pause ≤1 s and the rate-change
//! effective ≤2 s); SC-050 (a document left paused for at
//! least one hour produces zero synthesis, holds an empty
//! pre-roll buffer throughout the idle period, and bills
//! a per-character engine zero characters; pressing play
//! after that idle begins pre-roll and produces audible
//! audio within the 3-second bound); SC-050b (discarded
//! pre-roll — audio prepared but never played — bills
//! zero characters on a per-character engine beyond the
//! smallest documented per-request byte budget); and
//! `docs/byte-budget.md` (T007c, Principle X): the Google
//! Cloud TTS smallest documented per-request limit is
//! **5 000 bytes** — only a constitution amendment may
//! change the number.
//!
//! FAILS before `crates/player/src/recovery.rs` (T076)
//! and `crates/engine-core/src/fallback.rs` (T075) exist:
//! the `player::recovery` and `engine_core::fallback`
//! modules are unresolved, so this test target does not
//! build. That build failure is the required
//! pre-implementation red state (the T072 BLOCKING gate:
//! all US5 tests written and FAILING before any US5
//! implementation task begins).
//!
//! Expected API contract for T075/T076 (the RED surface
//! this file is written against — the implementation
//! builds exactly this surface, no different one):
//!
//! ```text
//! // crates/engine-core/src/fallback.rs (T075)
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! pub struct EngineDescriptor { pub id: String, pub tier: EngineTier }
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! pub struct EngineOffer { pub engine_id: String, pub tier: EngineTier }
//!
//! // crates/player/src/recovery.rs (T076)
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum RecoveryDecision {
//!     Takeover { engine_id: String, from_position_secs: f64 },
//!     PausedWithReason { reason: String },
//!     Offered { alternatives: Vec<EngineOffer> },
//! }
//! #[derive(Debug)]
//! pub struct RecoveryMachine { /* private */ }
//! impl RecoveryMachine {
//!     pub fn new(registry: Vec<EngineDescriptor>) -> Self;
//!     /// The named failure of `engine_id` at `position_secs`:
//!     /// with remaining selectable engines, a 3 s offer naming
//!     /// them (FR-023a); with none, PausedWithReason.
//!     pub fn on_engine_failure(&mut self, engine_id: &str, position_secs: f64) -> RecoveryDecision;
//!     /// The 3 s offer window elapsed with no choice: the
//!     /// auto-fallback takes over from the last spoken position
//!     /// (FR-023a), or PausedWithReason when no engine remains.
//!     pub fn on_offer_expired(&mut self) -> RecoveryDecision;
//!     /// The reader's explicit choice inside the window: the
//!     /// chosen engine continues from the last spoken position.
//!     /// `Err(PlayerError::EngineNotAvailable)` for an engine
//!     /// that is not selectable (failed, unregistered, or
//!     /// locked-out by the resolution).
//!     pub fn choose(&mut self, engine_id: &str) -> Result<RecoveryDecision, player::cli::PlayerError>;
//! }
//! ```
//!
//! The pipeline under test composes the three authorities
//! the US5 story names: the engine authority
//! (`player::recovery::RecoveryMachine`, T076 — the
//! registry of selectable engines and the failover
//! machine), the transport and position authority
//! (`player::cli::PlaybackSession`, FR-002), and the
//! pre-roll buffer in both of its views (the Play-gated
//! `player::buffer::PreRollBuffer` and the sample-capacity
//! `player::skip::PreRollRing`, FR-015a) — plus the
//! per-character engine-spy, whose billing ledger is the
//! only billing record (SC-050/SC-050b).
//!
//! Device-only gaps (traced, not asserted on host): the
//! audible confirmation that the post-idle pre-roll
//! became audible audio inside the FR-027 window on a
//! physical device, and the wall-clock idle hour — SC-050
//! is a 10-of-10 device-trial count, so the host asserts
//! the invariant on the simulated window (a synthetic
//! clock, no progress fed) instead of waiting an hour.
//! The host asserts the state machines, the billing
//! ledger, and the buffer shapes.

use proptest::prelude::*;

use engine_core::closed::{EngineError, EngineTier, PlaybackRate, TransportState};
use engine_core::fallback::EngineDescriptor;
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;
use engine_core::trait_::{AlignmentGranularity, Engine, MAX_PREROLL_SECS, SsmlLevel, StreamChunk};
use player::buffer::{BufferError, PreRollBuffer};
use player::cli::{PlaybackSession, PlayerError, ShellVerb};
use player::recovery::{RecoveryDecision, RecoveryMachine};
use player::skip::PreRollRing;

/// The smallest documented per-request byte budget
/// (`docs/byte-budget.md`, T007c): the Google Cloud TTS
/// smallest documented per-request limit, 5 000 bytes.
/// Only a constitution amendment may change this number
/// (Principle X).
const BYTE_BUDGET: u64 = 5_000;

/// Bytes per synthesised character on the per-character
/// engine-spy: the pt-BR document text the stand-ins
/// carry averages a 3-byte UTF-8 encoding per character,
/// so the spy bills 3 bytes per character — the
/// per-character billing unit of FR-024 / T007c.
const BYTES_PER_CHAR: u64 = 3;

/// Characters per second of speech on the per-character
/// engine-spy: the synthesis rate of the pt-BR document
/// corpus.
const CHARS_PER_SEC: u64 = 15;

/// The FR-027 play-to-audio bound: audible audio within
/// 3 seconds of a Play press.
const FR027_PLAY_TO_AUDIO_SECS: f64 = 3.0;

/// The minimum simulated idle window (SC-050): at least
/// one hour.
const MIN_IDLE_SECS: f64 = 3_600.0;

/// The bound on saturation rounds in the cap property:
/// the smallest strategy value (0.01 s) needs 6 000
/// rounds to reach the 60 s cap, so 10 000 rounds bound
/// every strategy value with room to spare.
const SATURATION_ROUNDS: u32 = 10_000;

/// The model tone sample: a fixed non-zero amplitude, so
/// the delivered frames model audible audio, never
/// silence. The property under test is the delivery and
/// capacity path, not the waveform.
const TONE_SAMPLE: f32 = 0.25;

/// The primary engine's registered id: `standin_a`, the
/// offline-like stand-in of the SC-051 drill — one of the
/// two `test_double` ids the 2026-10-05 clarification and
/// SC-057 permit (every `test_double` identifier is
/// `standin_a` or `standin_b`).
const PRIMARY_ENGINE_ID: &str = StandinA::ID;

/// The fallback engine's registered id: `standin_b`, the
/// cloud-like stand-in of the SC-051 drill.
const FALLBACK_ENGINE_ID: &str = StandinB::ID;

/// Simulated idle windows of at least one hour, on the
/// exact second grid: `k` s for `k ∈ [3600, 7200]` — the
/// SC-050 minimum (at least one hour) up to two hours,
/// every value an exact binary-fraction second.
#[expect(
    clippy::cast_precision_loss,
    reason = "the generated seconds are at most 7 200, an exact integer in f64; the cast is exact for every generated value and a tolerance would wrongly admit off-grid durations"
)]
fn idle_duration_strategy() -> impl Strategy<Value = f64> {
    (3_600u64..=7_200).prop_map(|secs| secs as f64)
}

/// Pre-roll request durations on the centisecond grid:
/// `k × 0.01` s for `k ∈ [1, 6000]` — the open-lower
/// interval (0, 60] seconds, every value an exact binary
/// fraction, bounded by the 60 s pre-roll cap (FR-015a).
#[expect(
    clippy::cast_precision_loss,
    reason = "the generated centiseconds are at most 6 000, exact in f64; the division by the exact constant 100.0 keeps every value on the centisecond grid"
)]
fn pre_roll_secs_strategy() -> impl Strategy<Value = f64> {
    (1u64..=6_000).prop_map(|centis| centis as f64 / 100.0)
}

/// Positions on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600] — the same grid the
/// transport-math properties use, so every position is an
/// exact binary fraction and the takeover-position
/// comparison is bit-exact.
#[expect(
    clippy::cast_precision_loss,
    reason = "the generated quarters are at most 14 400, exact in f64; the multiplication by the exact binary fraction 0.25 keeps every value on the grid"
)]
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Sample pushes at and beyond the ring's capacity: the
/// strategy generates a push count in
/// `[1 323 000, 1 400 000]` — at and just past the 60 s
/// capacity (`MAX_SECS × 22 050` = 1 323 000 samples),
/// so the cap is exercised at and beyond its boundary.
fn beyond_capacity_strategy() -> impl Strategy<Value = u64> {
    1_323_000u64..=1_400_000
}

/// The checkpoint stride of a simulated window: `total`
/// divided into `parts` equal steps. Every generated
/// grid value is an exact binary fraction, so the
/// division is exact and the cast admits no rounding.
#[expect(
    clippy::cast_precision_loss,
    reason = "parts is at most 8 and total_secs is at most 7 200.0 on the generated grids, so the quotient is exact"
)]
fn step_of(total_secs: f64, parts: u64) -> f64 {
    total_secs / parts as f64
}

/// The closed discard vocabulary (FR-015c): a rate change
/// re-prepares the held audio at the new rung (discarding
/// the stale preparation, FR-019a), and a pause drains
/// the pre-roll for idle. No other discard path exists —
/// the match on this set is exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiscardKind {
    /// A rate change discards the held pre-roll (FR-019a).
    RateChange,
    /// A pause discards the held pre-roll for idle
    /// (FR-015c).
    Pause,
}

fn discard_kind_strategy() -> impl Strategy<Value = DiscardKind> {
    prop_oneof![Just(DiscardKind::RateChange), Just(DiscardKind::Pause),]
}

/// The synthetic clock: simulated time advances only
/// through [`advance`](SyntheticClock::advance), so an
/// idle window of any length costs nothing real and no
/// progress is fed while it runs — exactly what SC-050
/// measures (an idle window of ≥1 h bills zero).
#[derive(Debug, Clone, Copy, PartialEq)]
struct SyntheticClock {
    /// Simulated time since the session began, in seconds.
    now_secs: f64,
}

impl SyntheticClock {
    /// A clock at the session start (0.0 s).
    const fn new() -> Self {
        Self { now_secs: 0.0 }
    }

    /// The current simulated time, in seconds.
    const fn now(self) -> f64 {
        self.now_secs
    }

    /// Advance the simulated time by `secs` without
    /// feeding any audible progress: time passing while
    /// idle is the idle window itself.
    fn advance(&mut self, secs: f64) {
        self.now_secs += secs;
    }
}

/// One billed synthesis request — the per-character
/// billing unit (FR-024 / T007c): the characters the
/// engine synthesised for the request and the bytes they
/// bill at [`BYTES_PER_CHAR`] per character. The `f64`
/// positions keep the type `PartialEq` only — `f64` is
/// not `Eq`, and every comparison is exact on the
/// generated grids.
#[derive(Debug, Clone, Copy, PartialEq)]
struct BillingEntry {
    /// The document position the request started at, in
    /// seconds.
    position_secs: f64,
    /// The requested synthesis duration, in seconds.
    max_secs: f64,
    /// Characters synthesised for the request.
    chars: u64,
    /// Bytes billed: `chars × BYTES_PER_CHAR`.
    bytes: u64,
}

/// The per-character engine-spy (SC-050/SC-050b): an
/// `engine_core::trait_::Engine` fixture whose `stream`
/// is the only synthesis path, recording one billing
/// entry per request. Zero entries is zero billing: an
/// idle session that synthesises nothing bills nothing
/// (FR-015c).
#[derive(Debug, Default)]
struct SpyEngine {
    /// The per-request billing ledger, in request order.
    ledger: Vec<BillingEntry>,
}

impl Engine for SpyEngine {
    /// The spy declares the coarsest pair — plain-text
    /// delivery with no alignment events (the same
    /// declaration the follow-up synthesis adapter
    /// carries).
    fn declared_capabilities(&self) -> (SsmlLevel, AlignmentGranularity) {
        (SsmlLevel::None, AlignmentGranularity::None)
    }

    /// A rate change is a pitch-preserving time-stretch
    /// inside the engine (FR-043): the spy accepts every
    /// rung of the closed ladder and never deviates from
    /// the declared handoff shape.
    fn set_rate(&mut self, rate: PlaybackRate) -> Result<(), EngineError> {
        let _ = rate;
        Ok(())
    }

    /// Synthesise `max_secs` of audio from
    /// `position_secs`: one billed request on the ledger,
    /// and a delivered chunk of mono `f32` frames at the
    /// declared 22 050 Hz (FR-033). A request beyond the
    /// pre-roll bound is rejected before any synthesis
    /// runs (FR-015a) — a named failure, never an
    /// open-ended synthesis.
    fn stream(&mut self, position_secs: f64, max_secs: f64) -> Result<StreamChunk, EngineError> {
        #[expect(
            clippy::neg_cmp_op_on_partial_ord,
            reason = "the negated >= deliberately rejects NaN inputs: NaN >= 0.0 is false, so a NaN position or bound fails closed before any synthesis runs (FR-015a)"
        )]
        if !(position_secs >= 0.0) || !(max_secs >= 0.0) || max_secs > MAX_PREROLL_SECS {
            return Err(EngineError::StartRefused);
        }
        let chars = characters_for(max_secs);
        self.ledger.push(BillingEntry {
            position_secs,
            max_secs,
            chars,
            bytes: chars * BYTES_PER_CHAR,
        });
        Ok(StreamChunk {
            start_position_secs: position_secs,
            frames: vec![TONE_SAMPLE; frame_count(max_secs)],
            boundaries: Vec::new(),
        })
    }
}

/// The characters a per-character engine synthesises for
/// `max_secs` of speech: [`CHARS_PER_SEC`] per second,
/// rounded up to the whole character (the billing unit).
#[expect(
    clippy::cast_precision_loss,
    reason = "CHARS_PER_SEC (15) and the bounded max_secs (at most 60.0) give a product of at most 900.0, an exact value in f64"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "max_secs is bounded by MAX_PREROLL_SECS (60.0) and CHARS_PER_SEC is 15, so the product is at most 900.0 — far below u64 range; the ceiling keeps a fractional second inside the billed character"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "max_secs is non-negative (checked by the caller), so the ceiling is non-negative and no sign is lost"
)]
fn characters_for(max_secs: f64) -> u64 {
    (max_secs * CHARS_PER_SEC as f64).ceil() as u64
}

/// The delivered frame count for `max_secs` of mono
/// `f32` audio at the declared 22 050 Hz (FR-033).
#[expect(
    clippy::cast_possible_truncation,
    reason = "max_secs is bounded by MAX_PREROLL_SECS (60.0), so the product is at most 1 323 000 frames — an exact integer far below usize range"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "max_secs is non-negative (checked by the caller), so the product is non-negative and no sign is lost"
)]
fn frame_count(max_secs: f64) -> usize {
    (max_secs * f64::from(PreRollRing::SAMPLE_RATE)) as usize
}

/// The pipeline's own failure surface: the closed union
/// of the engine's named failure and the buffer's named
/// rejection (Principle VII — every failure is named, never
/// silent).
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "the payloads wrap the closed EngineError and BufferError sets at the pipeline's failure seams (press_play/begin_pre_roll); this suite asserts the success paths — the failure paths are the engine-core and buffer suites' subjects — so the wrapped errors are provenance read by no assertion, and a derived Debug impl is intentionally ignored by dead-code analysis"
)]
enum PipelineError {
    /// The engine named a failure (the closed
    /// [`EngineError`] set).
    Engine(EngineError),
    /// The pre-roll buffer rejected the delivery (the closed
    /// [`BufferError`] set).
    Buffer(BufferError),
}

/// The playback pipeline under test (the US5 composition):
/// the engine authority ([`RecoveryMachine`], T076), the
/// transport and position authority ([`PlaybackSession`],
/// FR-002), the Play-gated pre-roll buffer
/// ([`PreRollBuffer`], FR-015a/FR-015c), the
/// sample-capacity ring ([`PreRollRing`], FR-015a), one
/// per-character engine-spy per registered engine (so each
/// engine's billing is separately observable,
/// SC-050/SC-050b), and the synthetic clock.
#[derive(Debug)]
struct IdlePipeline {
    /// The engine authority: the registry of selectable
    /// engines and the failover machine (T076).
    machine: RecoveryMachine,
    /// The transport and position authority (FR-002).
    session: PlaybackSession,
    /// The Play-gated pre-roll buffer (FR-015a/FR-015c).
    pre_roll: PreRollBuffer,
    /// The sample-capacity ring (FR-015a).
    ring: PreRollRing,
    /// The per-character engine-spies, in registry order.
    spies: Vec<SpyEngine>,
    /// The synthetic clock (simulated time).
    clock: SyntheticClock,
    /// The registered engine ids, in registry order — the
    /// pipeline's own copy of the machine's registry, so a
    /// takeover decision can re-bind the pipeline to the
    /// takeover engine.
    engine_ids: Vec<String>,
    /// The index of the bound engine in the registry. The
    /// binding moves only through a machine decision
    /// (FR-023a), never on its own.
    bound_index: usize,
    /// The simulated seconds from the last Play press to
    /// the pre-roll begin — the FR-027 measurement.
    press_to_pre_roll_secs: f64,
}

impl IdlePipeline {
    /// A pipeline over the single registered engine — the
    /// configuration the idle-drain trials run under.
    fn single_engine() -> Self {
        Self::from_registry(&[PRIMARY_ENGINE_ID])
    }

    /// A pipeline over both stand-ins (the SC-051 failover
    /// configuration): `standin_a` primary, `standin_b`
    /// fallback — both `test_double`, admissible only while
    /// no non-`test_double` adapter is registered
    /// (FR-023c / the 2026-10-05 clarification).
    fn two_engines() -> Self {
        Self::from_registry(&[PRIMARY_ENGINE_ID, FALLBACK_ENGINE_ID])
    }

    /// A pipeline over `engine_ids`, registered in order
    /// with the engine authority as the `test_double`
    /// stand-ins they are.
    fn from_registry(engine_ids: &[&str]) -> Self {
        let registry: Vec<EngineDescriptor> = engine_ids
            .iter()
            .map(|id| EngineDescriptor {
                id: String::from(*id),
                tier: EngineTier::TestDouble,
            })
            .collect();
        let machine = RecoveryMachine::new(registry);
        let spies: Vec<SpyEngine> = engine_ids.iter().map(|_| SpyEngine::default()).collect();
        Self {
            machine,
            session: PlaybackSession::new(),
            pre_roll: PreRollBuffer::new(),
            ring: PreRollRing::new(),
            spies,
            clock: SyntheticClock::new(),
            engine_ids: engine_ids.iter().map(|id| String::from(*id)).collect(),
            bound_index: 0,
            press_to_pre_roll_secs: 0.0,
        }
    }

    /// The bound engine's spy: the ledger the pre-roll and
    /// the audible progress are billed to.
    fn bound_spy(&mut self) -> &mut SpyEngine {
        &mut self.spies[self.bound_index]
    }

    /// The bound engine's registered id.
    fn bound_engine_id(&self) -> &str {
        &self.engine_ids[self.bound_index]
    }

    /// The `(chars, bytes)` totals of one registered
    /// engine's ledger — that engine's billing record.
    fn ledger_of(&self, engine_id: &str) -> (u64, u64) {
        let mut chars = 0u64;
        let mut bytes = 0u64;
        for (index, registered) in self.engine_ids.iter().enumerate() {
            if registered == engine_id {
                for entry in &self.spies[index].ledger {
                    chars += entry.chars;
                    bytes += entry.bytes;
                }
            }
        }
        (chars, bytes)
    }

    /// Every billing entry across all engines, in registry
    /// then request order — the pipeline's complete
    /// billing record.
    fn billing_record(&self) -> Vec<BillingEntry> {
        let mut record = Vec::new();
        for spy in &self.spies {
            record.extend_from_slice(&spy.ledger);
        }
        record
    }

    /// Whether the pre-roll is empty in both views: the
    /// Play-gated buffer and the sample ring (FR-015a /
    /// FR-015c).
    const fn pre_roll_is_empty(&self) -> bool {
        self.pre_roll.is_empty() && self.ring.buffer.is_empty()
    }

    /// The Play press (FR-015c): the transport authority
    /// lands `Playing`, the Play-gated buffer opens
    /// preparation, and pre-roll begins — the bound
    /// engine's `stream` is invoked and the delivered
    /// frames land in the ring and the buffer —
    /// immediately on the press, inside the FR-027 bound.
    /// The press is the only path that opens preparation:
    /// an idle session prepares nothing.
    fn press_play(&mut self, pre_roll_secs: f64) -> Result<(), PipelineError> {
        let press_at_secs = self.clock.now();
        let _resumed = self.session.execute(ShellVerb::Play);
        self.pre_roll.on_play_pressed();
        self.begin_pre_roll(pre_roll_secs)?;
        self.press_to_pre_roll_secs = self.clock.now() - press_at_secs;
        Ok(())
    }

    /// Begin pre-roll: the bound engine synthesises
    /// `pre_roll_secs` of audio from the current position
    /// and the delivered frames land in the ring and the
    /// Play-gated buffer (FR-015a). Pre-roll never
    /// advances the position: the clock is untouched.
    fn begin_pre_roll(&mut self, pre_roll_secs: f64) -> Result<(), PipelineError> {
        let position = self.session.snapshot().position_secs;
        let chunk = match self.bound_spy().stream(position, pre_roll_secs) {
            Ok(chunk) => chunk,
            Err(error) => return Err(PipelineError::Engine(error)),
        };
        #[expect(
            clippy::cast_precision_loss,
            reason = "the frame count is bounded by the 60 s pre-roll cap (at most 1 323 000 frames) and the sample rate is the exact constant 22 050, so the division is a bounded f64 operation"
        )]
        let delivered_secs = chunk.frames.len() as f64 / f64::from(PreRollRing::SAMPLE_RATE);
        for frame in chunk.frames {
            self.ring.push(frame);
        }
        match self.pre_roll.push_delivered(delivered_secs) {
            Ok(_held) => Ok(()),
            Err(error) => Err(PipelineError::Buffer(error)),
        }
    }

    /// Feed audible progress (the backend's clock seam,
    /// FR-002): the bound engine synthesised `secs` of
    /// audible audio — billed requests on that engine's
    /// ledger, each bounded by the engine-side pre-roll
    /// request bound — and the position authority advanced
    /// by exactly `secs`. Audible audio is consumed by the
    /// backend as it is delivered: it never enters the
    /// pre-roll ring (FR-015a — only prepared-ahead audio
    /// is buffered), and audible progress is the only
    /// position-advance path.
    fn feed_audible_progress(&mut self, secs: f64) -> Result<(), EngineError> {
        let mut remaining = secs;
        #[expect(
            clippy::while_float,
            reason = "each bounded step is min(remaining, MAX_PREROLL_SECS): while remaining > MAX the decrement is exact (a representable subtrahend), and the final step lands on exactly 0.0; > 0.0 is the only correct termination guard"
        )]
        while remaining > 0.0 {
            // One synthesis request never exceeds the
            // engine-side bound (MAX_PREROLL_SECS): audible
            // progress is fed in bounded requests, each a
            // billed entry on the engine's ledger.
            let step = remaining.min(MAX_PREROLL_SECS);
            let position = self.session.snapshot().position_secs;
            let chunk = self.bound_spy().stream(position, step)?;
            let _ = chunk;
            self.session.on_audible_progress(step);
            remaining -= step;
        }
        Ok(())
    }

    /// Enter Paused-idle (FR-015c): the transport authority
    /// lands `Paused` with the position retained, the
    /// Play-gated buffer drains for idle (empty and
    /// non-preparing), and the ring drains. From here
    /// until the next Play press the pipeline prepares
    /// nothing and bills nothing.
    fn enter_idle(&mut self) -> Result<(), PlayerError> {
        let _paused = self.session.execute(ShellVerb::Pause)?;
        self.pre_roll.drain_for_idle();
        self.ring.drain_idle();
        Ok(())
    }

    /// Simulate the idle window (SC-050): advance the
    /// synthetic clock by `secs` and feed no audible
    /// progress. Nothing else happens — that absence is
    /// the invariant: an idle session synthesises nothing,
    /// bills nothing, and holds an empty pre-roll buffer
    /// throughout.
    fn simulate_idle(&mut self, secs: f64) {
        self.clock.advance(secs);
    }

    /// Apply a takeover decision (FR-023a): the pipeline
    /// re-binds to the takeover engine — the binding moves
    /// only through a machine decision, never on its own —
    /// so the pre-roll that follows is served by the
    /// takeover engine's ledger.
    ///
    /// Returns whether the takeover engine is one of the
    /// pipeline's registered engines (it always is: the
    /// machine resolves only among its registry).
    fn apply_takeover(&mut self, engine_id: &str) -> bool {
        let mut rebound = false;
        for (index, registered) in self.engine_ids.iter().enumerate() {
            if registered == engine_id {
                self.bound_index = index;
                rebound = true;
            }
        }
        rebound
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// I1 (SC-050 / FR-015c / FR-015a): after the session
    /// enters Paused-idle, a simulated idle window of at
    /// least one hour synthesises nothing — the
    /// engine-spy's ledger records zero new characters and
    /// zero new bytes for the whole window, sampled at
    /// checkpoints throughout — and the pre-roll buffer is
    /// empty at every sample, not only at the end.
    #[test]
    fn an_idle_window_of_at_least_one_hour_bills_zero_synthesis(
        idle_secs in idle_duration_strategy()
    ) {
        prop_assert!(
            idle_secs >= MIN_IDLE_SECS,
            "the SC-050 idle window must be at least one hour, got {idle_secs} s"
        );
        let mut pipeline = IdlePipeline::single_engine();
        // Playback to a spoken position, then Paused-idle:
        // the reader pauses and walks away.
        let played = pipeline.session.execute(ShellVerb::Play);
        prop_assert!(
            played.is_ok(),
            "the Play press must succeed from the initial Paused state, got {played:?}"
        );
        let fed = pipeline.feed_audible_progress(60.0);
        prop_assert!(
            fed.is_ok(),
            "audible progress must succeed while Playing, got {fed:?}"
        );
        let idled = pipeline.enter_idle();
        prop_assert!(
            idled.is_ok(),
            "the pause into Paused-idle must succeed from Playing, got {idled:?}"
        );
        prop_assert_eq!(
            pipeline.session.snapshot().transport,
            TransportState::Paused,
            "entering Paused-idle must land the transport on Paused (FR-015c)"
        );
        // The idle window opens with the ledger at its
        // pre-idle depth and the pre-roll empty.
        let ledger_depth_at_idle = pipeline.billing_record().len();
        let ledger_at_idle = pipeline.ledger_of(PRIMARY_ENGINE_ID);
        prop_assert!(
            pipeline.pre_roll_is_empty(),
            "the pre-roll buffer must be empty when the idle window opens (FR-015c)"
        );
        // The window is sampled at checkpoints: zero new
        // synthesis and an empty pre-roll at every sample
        // (SC-050: "empty throughout the idle period").
        let checkpoints = 8u64;
        let step_secs = step_of(idle_secs, checkpoints);
        for _ in 0..checkpoints {
            pipeline.simulate_idle(step_secs);
            prop_assert_eq!(
                pipeline.billing_record().len(),
                ledger_depth_at_idle,
                "the engine-spy served a synthesis request during the {idle_secs} s idle window (SC-050)",
                idle_secs = idle_secs
            );
            prop_assert!(
                pipeline.pre_roll_is_empty(),
                "the pre-roll buffer was not empty during the {idle_secs} s idle window (SC-050)"
            );
        }
        // The ledger's totals are unchanged across the
        // window: zero synthesised characters and zero
        // bytes billed for the whole idle period (SC-050).
        prop_assert_eq!(
            pipeline.ledger_of(PRIMARY_ENGINE_ID),
            ledger_at_idle,
            "the idle window changed the bound engine's billing record (SC-050)"
        );
    }

    /// I2 (SC-050 / FR-015c / FR-027): a Play press after
    /// the idle window begins pre-roll — the bound
    /// engine's `stream` is invoked (pre-roll synthesis
    /// starts) and the delivered audio lands in the
    /// pre-roll buffer — immediately on the press, inside
    /// the FR-027 bound, and without advancing the
    /// position (FR-015a).
    #[expect(
        clippy::float_cmp,
        reason = "the compared positions are exact binary fractions on the generated grids; == is exact and a tolerance would wrongly admit off-grid positions"
    )]
    #[test]
    fn a_play_press_after_idle_begins_pre_roll_within_the_fr027_bound(
        (idle_secs, pre_roll_secs) in (idle_duration_strategy(), pre_roll_secs_strategy())
    ) {
        let mut pipeline = IdlePipeline::single_engine();
        // The reader is away for the whole idle window:
        // nothing was synthesised while away (SC-050).
        pipeline.simulate_idle(idle_secs);
        prop_assert_eq!(
            pipeline.billing_record().len(),
            0,
            "the engine-spy synthesised during the idle window before any Play press (SC-050)"
        );
        // The Play press begins pre-roll within the bound.
        let pressed = pipeline.press_play(pre_roll_secs);
        prop_assert!(
            pressed.is_ok(),
            "the Play press after idle must begin pre-roll, got {pressed:?}"
        );
        prop_assert_eq!(
            pipeline.session.snapshot().transport,
            TransportState::Playing,
            "the Play press must land the transport on Playing (FR-015c)"
        );
        prop_assert_eq!(
            pipeline.billing_record().len(),
            1,
            "the Play press must begin exactly one pre-roll synthesis request (FR-015c)"
        );
        prop_assert!(
            !pipeline.pre_roll.is_empty(),
            "the Play-gated pre-roll buffer stayed empty after the Play press (FR-015a)"
        );
        prop_assert!(
            !pipeline.ring.buffer.is_empty(),
            "the pre-roll ring stayed empty after the Play press (FR-015a)"
        );
        prop_assert!(
            pipeline.press_to_pre_roll_secs <= FR027_PLAY_TO_AUDIO_SECS,
            "pre-roll began {} s after the Play press, outside the FR-027 {} s bound",
            pipeline.press_to_pre_roll_secs,
            FR027_PLAY_TO_AUDIO_SECS
        );
        // Pre-roll never advances the position (FR-015a):
        // the press began pre-roll, and no audible audio
        // has been delivered yet.
        // the press began pre-roll, and no audible audio
        // has been delivered yet.
        // the press began pre-roll, and no audible audio
        // has been delivered yet.
        prop_assert_eq!(
            pipeline.session.snapshot().position_secs,
            0.0,
            "pre-roll advanced the position (FR-015a)"
        );
    }

    /// I3 (SC-050b / FR-015c / FR-015a / T007c): discarded
    /// pre-roll — audio prepared but never played — bills
    /// at most the smallest documented per-request byte
    /// budget (5 000 bytes, `docs/byte-budget.md`: the
    /// Google Cloud TTS smallest documented per-request
    /// limit; only an amendment may change the number) on
    /// the per-character engine-spy: every request the
    /// discarded pre-roll served respected the 60 s
    /// pre-roll bound and bills `chars × BYTES_PER_CHAR ≤
    /// 5 000` bytes.
    #[test]
    fn discarded_pre_roll_bills_within_the_documented_per_request_budget(
        (pre_roll_secs, discard) in (pre_roll_secs_strategy(), discard_kind_strategy())
    ) {
        let mut pipeline = IdlePipeline::single_engine();
        let pressed = pipeline.press_play(pre_roll_secs);
        prop_assert!(
            pressed.is_ok(),
            "the Play press must begin pre-roll, got {pressed:?}"
        );
        prop_assert_eq!(
            pipeline.billing_record().len(),
            1,
            "the Play press must have served exactly one pre-roll request"
        );
        // The pre-roll is discarded — by a rate change (the
        // buffer re-prepares at the new rung, discarding
        // the stale preparation) or by a pause (the buffer
        // drains for idle). Both are the closed discard
        // vocabulary (FR-015c / FR-019a).
        match discard {
            DiscardKind::RateChange => {
                pipeline.pre_roll.reprepare_for_rate(PlaybackRate::R1_25);
                let _stretched = pipeline.bound_spy().set_rate(PlaybackRate::R1_25);
            }
            DiscardKind::Pause => {
                let _paused = pipeline.session.execute(ShellVerb::Pause);
                pipeline.pre_roll.drain_for_idle();
                pipeline.ring.drain_idle();
            }
        }
        // Every request the discarded pre-roll served
        // respected the pre-roll bound and bills within the
        // documented byte budget (SC-050b / T007c).
        for entry in pipeline.billing_record() {
            prop_assert!(
                entry.max_secs <= MAX_PREROLL_SECS,
                "a pre-roll request asked for {} s, beyond the {} s pre-roll bound (FR-015a)",
                entry.max_secs,
                MAX_PREROLL_SECS
            );
            prop_assert!(
                entry.bytes <= BYTE_BUDGET,
                "a discarded pre-roll request billed {} bytes ({} chars × {} bytes/char), over the {}-byte documented per-request budget (SC-050b / docs/byte-budget.md / T007c)",
                entry.bytes,
                entry.chars,
                BYTES_PER_CHAR,
                BYTE_BUDGET
            );
        }
    }

    /// I4 (FR-015a): the pre-roll buffer never holds more
    /// than 60 s of engine-delivered audio — the Play-gated
    /// buffer saturates at the cap under aggregate delivery,
    /// and the sample ring drops every frame pushed past
    /// its capacity, so no delivery path can grow the
    /// buffer beyond [`PreRollRing::MAX_SECS`]. Both views
    /// read the same single cap constant (FR-044: no
    /// second cap value that can drift).
    #[test]
    fn the_pre_roll_never_holds_more_than_sixty_seconds(
        (delivered_secs, extra_pushes) in (pre_roll_secs_strategy(), beyond_capacity_strategy())
    ) {
        // View 1 — the Play-gated buffer (FR-015b): the
        // bound is a property of the pipeline measured
        // against audio actually handed to the playback
        // module, and aggregate delivery saturates at the
        // 60 s cap.
        let mut buffer = PreRollBuffer::new();
        buffer.on_play_pressed();
        let mut rounds = 0u32;
        while buffer.buffered_secs() < PreRollBuffer::capacity_secs()
            && rounds < SATURATION_ROUNDS
        {
            let _held = buffer.push_delivered(delivered_secs);
            rounds += 1;
        }
        prop_assert!(
            buffer.buffered_secs() <= PreRollBuffer::capacity_secs(),
            "the Play-gated buffer held {} s, over the {} s cap (FR-015a)",
            buffer.buffered_secs(),
            PreRollBuffer::capacity_secs()
        );
        // View 2 — the sample-capacity ring (FR-015a):
        // pushes past the capacity are dropped, so the ring
        // never holds more than MAX_SECS of audio.
        let mut ring = PreRollRing::new();
        for _ in 0..extra_pushes {
            ring.push(TONE_SAMPLE);
        }
        prop_assert!(
            ring.buffer.len() <= ring.max_samples,
            "the ring held {} samples, over its {}-sample capacity (FR-015a)",
            ring.buffer.len(),
            ring.max_samples
        );
        #[expect(
            clippy::cast_precision_loss,
            reason = "the sample count is at most 1 400 000 and the sample rate is the exact constant 22 050, so the division is a bounded f64 operation"
        )]
        let held_secs = ring.buffer.len() as f64 / f64::from(PreRollRing::SAMPLE_RATE);
        prop_assert!(
            held_secs <= PreRollRing::MAX_SECS,
            "the ring held {held_secs} s of audio, over the {} s pre-roll cap (FR-015a)",
            PreRollRing::MAX_SECS
        );
    }

    /// The US5 seam (FR-023 / FR-023a / SC-051 / SC-050):
    /// an engine failure during the idle window resolves
    /// through the engine authority ([`RecoveryMachine`],
    /// T076) and bills nothing — the failure, the 3 s
    /// offer, and the expiry are state-machine decisions,
    /// not synthesis requests, so the idle window's
    /// zero-billing invariant (SC-050) holds across the
    /// failover, and the expired offer takes over from the
    /// last spoken position locked, with no unlock (the
    /// closed `ShellVerb` set has no unlock member, so a
    /// locked resolution is mechanically unlock-free).
    #[expect(
        clippy::float_cmp,
        reason = "the compared positions are exact binary fractions on the quarter-second grid; == is exact and a tolerance would wrongly admit off-grid positions"
    )]
    #[test]
    fn a_mid_idle_engine_failure_resolves_locked_and_bills_nothing(
        (idle_secs, spoken_secs) in (idle_duration_strategy(), position_strategy())
    ) {
        let mut pipeline = IdlePipeline::two_engines();
        // Playback to the last spoken position on the bound
        // engine, then Paused-idle.
        let played = pipeline.session.execute(ShellVerb::Play);
        prop_assert!(
            played.is_ok(),
            "the Play press must succeed from the initial Paused state, got {played:?}"
        );
        let fed = pipeline.feed_audible_progress(spoken_secs);
        prop_assert!(
            fed.is_ok(),
            "audible progress must succeed while Playing, got {fed:?}"
        );
        prop_assert_eq!(
            pipeline.session.snapshot().position_secs,
            spoken_secs,
            "the position authority must hold the last spoken position (FR-002)"
        );
        let idled = pipeline.enter_idle();
        prop_assert!(
            idled.is_ok(),
            "the pause into Paused-idle must succeed from Playing, got {idled:?}"
        );
        // The idle window's opening ledger state, per engine.
        let (primary_chars, primary_bytes) = pipeline.ledger_of(PRIMARY_ENGINE_ID);
        let (fallback_chars, fallback_bytes) = pipeline.ledger_of(FALLBACK_ENGINE_ID);
        // The simulated idle window (≥1 h) — zero synthesis,
        // empty pre-roll throughout (SC-050).
        pipeline.simulate_idle(idle_secs);
        prop_assert_eq!(
            pipeline.ledger_of(PRIMARY_ENGINE_ID).0,
            primary_chars,
            "the idle window billed characters on the bound engine (SC-050)"
        );
        prop_assert_eq!(
            pipeline.ledger_of(FALLBACK_ENGINE_ID).0,
            fallback_chars,
            "the idle window billed characters on the fallback engine (SC-050)"
        );
        // The bound engine fails mid-document at the last
        // spoken position: the engine authority records the
        // named failure and opens the 3 s offer naming the
        // remaining engine (FR-023 / FR-023a).
        let offered = pipeline.machine.on_engine_failure(PRIMARY_ENGINE_ID, spoken_secs);
        let offered_alternatives = match &offered {
            RecoveryDecision::Offered { alternatives } => Some(alternatives.clone()),
            RecoveryDecision::Takeover { .. } | RecoveryDecision::PausedWithReason { .. } => None,
        };
        prop_assert!(
            offered_alternatives.is_some(),
            "a two-stand-in registry must open a 3 s offer on the first failure, got {offered:?} (FR-023a)"
        );
        let alternatives = offered_alternatives.unwrap_or_default();
        prop_assert_eq!(
            alternatives.len(),
            1,
            "the offer must name exactly the one remaining selectable engine (FR-023a)"
        );
        prop_assert_eq!(
            alternatives.first().map(|offer| offer.engine_id.as_str()),
            Some(FALLBACK_ENGINE_ID),
            "the offer must name stand-in B, the remaining stand-in (SC-051)"
        );
        prop_assert_eq!(
            alternatives.first().map(|offer| offer.tier),
            Some(EngineTier::TestDouble),
            "the stand-in offer carries the test_double tier (the 2026-10-05 clarification)"
        );
        // The transport never left Paused during the offer
        // window: a failed engine produces no audio, and a
        // playing state is never reported while the session
        // is silent (FR-005).
        prop_assert_eq!(
            pipeline.session.snapshot().transport,
            TransportState::Paused,
            "the transport must stay Paused through the offer window (FR-005)"
        );
        // The 3 s window elapses with no choice: the
        // auto-fallback takes over from the last spoken
        // position (FR-023a) — locked resolution, no
        // unlock.
        let takeover = pipeline.machine.on_offer_expired();
        let taken_over = match &takeover {
            RecoveryDecision::Takeover { engine_id, from_position_secs } => {
                Some((engine_id.clone(), *from_position_secs))
            }
            RecoveryDecision::Offered { .. } | RecoveryDecision::PausedWithReason { .. } => None,
        };
        prop_assert!(
            taken_over.is_some(),
            "an expired offer with a remaining engine must take over, got {takeover:?} (FR-023a)"
        );
        let (takeover_engine, takeover_position) =
            taken_over.unwrap_or_default();
        prop_assert_eq!(
            takeover_engine.as_str(), FALLBACK_ENGINE_ID,
            "the expired offer must take over on stand-in B (SC-051)"
        );
        prop_assert_eq!(
            takeover_position, spoken_secs,
            "the takeover must continue from the last spoken position, never restart (FR-023a)"
        );
        prop_assert!(
            pipeline.apply_takeover(&takeover_engine),
            "the takeover engine must be one of the pipeline's registered engines"
        );
        prop_assert_eq!(
            pipeline.bound_engine_id(),
            FALLBACK_ENGINE_ID,
            "the pipeline must re-bind to the takeover engine (FR-023a)"
        );
        // The whole window — idle, failure, offer, and
        // expiry included — billed zero on both engines: a
        // recovery decision is not a synthesis request
        // (SC-050 / SC-050b).
        prop_assert_eq!(
            pipeline.ledger_of(PRIMARY_ENGINE_ID).0,
            primary_chars,
            "the failure, offer, and expiry billed characters on the failed engine (SC-050)"
        );
        prop_assert_eq!(
            pipeline.ledger_of(PRIMARY_ENGINE_ID).1,
            primary_bytes,
            "the failure, offer, and expiry billed bytes on the failed engine (SC-050)"
        );
        prop_assert_eq!(
            pipeline.ledger_of(FALLBACK_ENGINE_ID).0,
            fallback_chars,
            "the failure, offer, and expiry billed characters on the takeover engine (SC-050)"
        );
        prop_assert_eq!(
            pipeline.ledger_of(FALLBACK_ENGINE_ID).1,
            fallback_bytes,
            "the failure, offer, and expiry billed bytes on the takeover engine (SC-050)"
        );
        prop_assert!(
            pipeline.pre_roll_is_empty(),
            "the pre-roll buffer must stay empty through the failover (FR-015c)"
        );
        // The reader's next Play press begins pre-roll on
        // the takeover engine within the FR-027 bound
        // (SC-050): the takeover engine's ledger serves the
        // request, and the failed engine's does not.
        let pressed = pipeline.press_play(pre_roll_after_takeover());
        prop_assert!(
            pressed.is_ok(),
            "the Play press after the takeover must begin pre-roll, got {pressed:?}"
        );
        prop_assert!(
            pipeline.press_to_pre_roll_secs <= FR027_PLAY_TO_AUDIO_SECS,
            "post-takeover pre-roll began {} s after the Play press, outside the FR-027 {} s bound",
            pipeline.press_to_pre_roll_secs,
            FR027_PLAY_TO_AUDIO_SECS
        );
        prop_assert_eq!(
            pipeline.ledger_of(PRIMARY_ENGINE_ID).0,
            primary_chars,
            "the failed engine synthesised after the takeover (SC-051)"
        );
        prop_assert_eq!(
            pipeline.ledger_of(FALLBACK_ENGINE_ID).0,
            fallback_chars + characters_for(pre_roll_after_takeover()),
            "the takeover engine did not serve the post-takeover pre-roll (SC-051)"
        );
    }
}

/// The pre-roll duration the post-takeover Play press
/// begins: a fixed sub-second request, inside the FR-027
/// window, so the post-takeover billing is a single
/// countable request.
const fn pre_roll_after_takeover() -> f64 {
    0.5
}

/// The sharp form of I1 (SC-050): a session in Paused-idle
/// with no progress — no Play press has opened preparation
/// and no audible progress has been fed — serves no
/// synthesis at all across a full one-hour idle window:
/// zero requests, zero characters, zero bytes, and an
/// empty pre-roll buffer at every checkpoint.
// Exact-value contract: the floor position is the exact f64 literal 0.0 and an idle session must
// not move it by a single ulp (FR-002 / FR-005); == is the correct assertion
#[test]
fn a_paused_session_with_no_progress_serves_nothing_across_an_idle_hour() {
    let mut pipeline = IdlePipeline::single_engine();
    // The session's initial state is Paused at 0.0 with no
    // progress — Paused-idle as FR-015c defines it.
    prop_assert_idle_shape(&pipeline);
    // One full hour of simulated idle, sampled at
    // checkpoints.
    let checkpoints = 8u64;
    let step_secs = step_of(MIN_IDLE_SECS, checkpoints);
    for _ in 0..checkpoints {
        pipeline.simulate_idle(step_secs);
        prop_assert_idle_shape(&pipeline);
    }
    assert_eq!(
        pipeline.billing_record().len(),
        0,
        "a session that never played served a synthesis request across a one-hour idle window (SC-050)"
    );
    assert_eq!(
        pipeline.ledger_of(PRIMARY_ENGINE_ID),
        (0, 0),
        "a session that never played billed characters or bytes across a one-hour idle window (SC-050 / SC-050b)"
    );
    assert!(
        pipeline.pre_roll_is_empty(),
        "the pre-roll buffer was not empty across a one-hour idle window (SC-050)"
    );
    assert_eq!(
        pipeline.session.snapshot().position_secs,
        0.0,
        "an idle session advanced the position without audible progress (FR-002 / FR-005)"
    );
}

/// The idle shape every checkpoint of a no-progress idle
/// window must hold: nothing served, nothing billed, an
/// empty pre-roll buffer, and the transport still Paused.
fn prop_assert_idle_shape(pipeline: &IdlePipeline) {
    assert_eq!(
        pipeline.billing_record().len(),
        0,
        "the engine-spy served a synthesis request while the session sat Paused-idle with no progress (SC-050)"
    );
    assert_eq!(
        pipeline.ledger_of(PRIMARY_ENGINE_ID),
        (0, 0),
        "the engine-spy billed characters or bytes while the session sat Paused-idle with no progress (SC-050 / SC-050b)"
    );
    assert!(
        pipeline.pre_roll_is_empty(),
        "the pre-roll buffer was not empty while the session sat Paused-idle with no progress (FR-015c)"
    );
    assert_eq!(
        pipeline.session.snapshot().transport,
        TransportState::Paused,
        "the transport left Paused while the session sat idle with no progress (FR-015c)"
    );
}

/// The boundary the pre-roll bound and the byte budget
/// share (FR-015a / T007c): a full-capacity 60 s pre-roll
/// request bills `CHARS_PER_SEC × 60 × BYTES_PER_CHAR` =
/// 15 × 60 × 3 = 2 700 bytes — inside the 5 000-byte
/// documented limit, so the 60 s pre-roll bound and the
/// byte budget can never disagree. Only an amendment to
/// `docs/byte-budget.md` (or the constitution) may change
/// either number.
// Exact-value contract: MAX_PREROLL_SECS, the capacity views, and the 60.0 boundary are exact
// binary fractions; == is exact and a tolerance would admit an off-boundary pre-roll request
#[test]
fn the_full_capacity_pre_roll_request_bills_within_the_documented_budget() {
    // The unified cap: the buffer view and the ring view
    // read the same single constant (FR-044 — no second
    // cap value that can drift).
    assert_eq!(
        PreRollBuffer::capacity_secs(),
        PreRollRing::MAX_SECS,
        "the Play-gated buffer and the sample ring must read the same single pre-roll cap (FR-044)"
    );
    assert_eq!(
        MAX_PREROLL_SECS, 60.0,
        "the engine-side pre-roll bound must be the 60 s of FR-015a"
    );
    // The boundary arithmetic: a full 60 s request bills
    // 2 700 bytes, inside the 5 000-byte documented
    // per-request limit (docs/byte-budget.md / T007c).
    assert_eq!(
        characters_for(MAX_PREROLL_SECS),
        CHARS_PER_SEC * 60,
        "a full-capacity request must bill exactly CHARS_PER_SEC × 60 characters"
    );
    assert!(
        characters_for(MAX_PREROLL_SECS) * BYTES_PER_CHAR <= BYTE_BUDGET,
        "a full-capacity pre-roll request bills over the documented {BYTE_BUDGET}-byte per-request budget (SC-050b / T007c)"
    );
    // The per-character arithmetic itself: 3 bytes per
    // character, 15 characters per second.
    assert_eq!(
        BYTE_BUDGET, 5_000,
        "the documented per-request byte budget is the 5 000 bytes of docs/byte-budget.md (T007c) — only an amendment may change it"
    );
}
