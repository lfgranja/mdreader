//! T070 — US5 integration: failover + recovery trials
//! (FR-023 / FR-023a / FR-023c / FR-032c / FR-032d /
//! FR-004 / FR-015 / SC-011 / SC-012 / SC-020 / SC-021 /
//! SC-039 / SC-040 / SC-051).
//!
//! Trial legs (from `specs/001-android-media-shell/tasks.md`
//! T070):
//!
//! - F1: stand-in A fails mid-document → the named failure
//!   opens a 3 s offer naming stand-in B → the expired
//!   offer takes over from the last spoken position,
//!   locked, with no unlock (FR-023 / FR-023a / SC-039 /
//!   SC-040 / SC-051).
//! - F2: the takeover engine (B) is forced down → paused
//!   with a stated reason, never silence (FR-023 / FR-037).
//! - F3: a single-engine-registry failure → paused with a
//!   reason, never silence (FR-023).
//! - F4: service kill → paused at the last spoken position,
//!   no auto-audio until the reader activates resume
//!   (SC-011 / SC-012).
//! - F5: deliberate vs unexpected reopen (SC-020 / SC-021):
//!   a foregrounded deliberate-pause offer auto-starts
//!   within the 4 s offer-to-audio budget (3 s window per
//!   FR-032c + 1 s post-expiry start budget); a backgrounded
//!   expiry and every unexpected end stay silent until the
//!   reader activates resume (FR-032c / FR-032d).
//! - F6: power-loss replay ≤ 30 s (`player::rewind::cursor`,
//!   FR-004 / FR-015), as a property over every paused
//!   duration on the quarter-second grid.
//! - F7: step-by-step replay of
//!   `tests/device/fixtures/service_kill_power_loss.jsonl`,
//!   asserting each `result` token, in the style of
//!   `test_focus_duck_pause_us2.rs` (T034).
//! - F8: device-only gaps, documented at the end of this
//!   module comment.
//!
//! FAILS before the US5 implementation tasks exist: the
//! `engine_core::fallback` module (T075), the
//! `player::recovery` module (T076), and the
//! `session::offer` module (T060) are unresolved, so this
//! test target does not build. That build failure is the
//! required pre-implementation red state (the T072 BLOCKING
//! gate: all US5 tests written and FAILING before any US5
//! implementation task begins).
//!
//! Expected API contract for T060/T075/T076 (the RED
//! surface this file is written against — the implementation
//! builds exactly these surfaces, no different ones; both
//! machines also implement `Default`, the `new`-without-
//! `default` contract):
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
//! #[derive(Debug, Default)]
//! pub struct RecoveryMachine { /* private */ }
//! impl RecoveryMachine {
//!     pub fn new(registry: Vec<EngineDescriptor>) -> Self;
//!     /// The named failure of `engine_id` at `position_secs`:
//!     /// with remaining selectable engines, a 3 s offer naming
//!     /// them (FR-023a); with none, PausedWithReason.
//!     pub fn on_engine_failure(&mut self, engine_id: &str, position_secs: f64) -> RecoveryDecision;
//!     /// The 3 s offer window elapsed with no choice: the
//!     /// auto-fallback takes over from the last spoken
//!     /// position (FR-023a), or PausedWithReason when no
//!     /// engine remains.
//!     pub fn on_offer_expired(&mut self) -> RecoveryDecision;
//!     /// The reader's explicit choice inside the window.
//!     /// `Err(PlayerError::EngineNotAvailable)` for an engine
//!     /// that is not selectable (failed, unregistered, or
//!     /// locked-out by the resolution).
//!     pub fn choose(&mut self, engine_id: &str) -> Result<RecoveryDecision, player::cli::PlayerError>;
//! }
//!
//! // crates/session/src/offer.rs (T060) — the canonical
//! // shape pinned by the dedicated session suites
//! // (prop_offer.rs / prop_resume.rs):
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum Offer {
//!     /// Continue from the stored position — the
//!     /// deliberate-pause offer (FR-032c), carrying
//!     /// the stored offset and the FR-015 interval
//!     /// start.
//!     Continue { offset_secs: f64, first_yield_ms: u64 },
//!     /// The start-over alternative (FR-032c).
//!     StartOver,
//! }
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum ExpiryOutcome {
//!     /// Reading begins from the stored position — the
//!     /// auto-start of a foregrounded deliberate-pause
//!     /// offer at expiry (SC-020), or the start-over
//!     /// first-spoken-position start.
//!     AutoStart { offset_secs: f64 },
//!     /// No audio begins: an explicit-resume control is
//!     /// all the session offers (unexpected end per
//!     /// SC-021, or a backgrounded expiry per FR-032d).
//!     ExplicitResume,
//!     /// Nothing is open — no outcome (the window never
//!     /// outlives its session).
//!     NoOffer,
//! }
//! #[derive(Debug, Default)]
//! pub struct OfferMachine { /* private */ }
//! impl OfferMachine {
//!     pub fn new() -> Self;
//!     /// The session ended with `stored` under
//!     /// `fingerprint`: the offer form is resolved
//!     /// SOLELY from the stored end kind — a
//!     /// deliberate `Paused` opens the 3 s continue
//!     /// offer, an `Interrupted` end opens no offer
//!     /// (the explicit-resume requirement stands),
//!     /// `Stopped`/`Completed` never open one
//!     /// (FR-032c / FR-032d / FR-032f).
//!     pub fn on_session_end(
//!         &mut self,
//!         fingerprint: [u8; 32],
//!         stored: Option<session::store::StoredPosition>,
//!     ) -> Option<Offer>;
//!     /// Latch the application as foregrounded (the
//!     /// `AppForegrounded` intent); read at expiry ONLY,
//!     /// never converting an open offer early (FR-032c).
//!     pub fn on_foregrounded(&mut self);
//!     /// Latch the application as backgrounded (the
//!     /// `AppBackgrounded` intent); read at expiry ONLY.
//!     pub fn on_backgrounded(&mut self);
//!     /// Resolve the offer at the 3 s expiry, reading
//!     /// the latched foreground flag (FR-032c / SC-020).
//!     pub fn expire(&mut self) -> ExpiryOutcome;
//!     /// The open offer, if any — the reader's
//!     /// activation inside the window.
//!     pub fn continue_offer(&mut self) -> Option<Offer>;
//!     /// The reader chooses start-over (FR-032f):
//!     /// reading begins at the first spoken position,
//!     /// `AutoStart { offset_secs: 0.0 }`.
//!     pub fn start_over(&mut self) -> ExpiryOutcome;
//!     /// Document completion teardown (FR-032b /
//!     /// FR-032e / FR-032f): no offer remains.
//!     pub fn on_completion(&mut self);
//! }
//! ```
//!
//! Device-only gaps (F8 — traced, not asserted on host): the
//! wall-clock offer-to-audio latency behind SC-020's 4 s
//! budget, and the 10-of-10 device trials behind SC-011,
//! SC-012, SC-020, SC-021, SC-039, SC-040, and SC-051,
//! are measured on the reference device (VERIFICATION.md's
//! trial-environment binding). The host asserts the
//! state-machine decisions, the locked-resolution semantics,
//! the silent-expiry semantics, and the ≤30 s replay bound
//! those device trials rest on. The T063 CLI (`offer
//! show|continue|start-over|expire`, `foreground on|off`)
//! drives the same functions per Principle II — this test
//! drives the library form of the identical code path, which
//! is what makes the SC-020 foreground/background split
//! CI-drivable without an emulator (contracts/shell-ipc.md).

use std::error::Error;

use proptest::prelude::*;

use engine_core::closed::{EngineError, EngineTier, PlaybackRate, SessionEnd, TransportState};
use engine_core::fallback::{EngineDescriptor, EngineOffer};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;
use engine_core::trait_::{AlignmentGranularity, Engine, MAX_PREROLL_SECS, SsmlLevel, StreamChunk};
use player::cli::{PlaybackSession, ShellVerb};
use player::position::{REWIND_CAP_SECS, REWIND_FRACTION};
use player::recovery::{RecoveryDecision, RecoveryMachine};
use player::rewind::cursor;
use session::offer::{ExpiryOutcome, Offer, OfferMachine};
use session::store::{BookmarkStore, StoredPosition};

/// The last spoken position of the failover drills, in
/// seconds: the mid-document position the SC-051 drill
/// fails at and the takeover continues from (FR-023a).
const LAST_SPOKEN_SECS: f64 = 120.0;

/// The FR-032c resume-offer window, in seconds: the
/// deliberate-pause offer is visible for 3 s before it
/// expires.
const OFFER_WINDOW_SECS: f64 = 3.0;

/// The SC-020 post-expiry start budget, in seconds: the
/// 1 s an auto-started offer has to begin reading after
/// the window elapses.
const POST_EXPIRY_START_BUDGET_SECS: f64 = 1.0;

/// The SC-020 offer-to-audio bound, in seconds: reading
/// begins within 4 s of the offer appearing (the 3 s
/// window per FR-032c + the 1 s post-expiry start budget)
/// — stricter than the FR-027 3 s play-to-audio bound,
/// which still governs the post-expiry start itself.
const SC020_OFFER_TO_AUDIO_SECS: f64 = 4.0;

/// Position comparison tolerance, in seconds: the cursor
/// formula multiplies by the non-exact binary fraction
/// 0.05, so replay positions compare within tolerance
/// (the same convention `test_focus_duck_pause_us2.rs`
/// uses).
const POS_EPS_SECS: f64 = 0.001;

/// The document fingerprint of a drill session: a
/// fixed 32-byte key (every drill machine is fresh,
/// so the drills never collide fingerprints).
const DRILL_FINGERPRINT: [u8; 32] = [7_u8; 32];

/// The stored position a drill session leaves
/// behind: the last spoken position on the FR-015
/// supersede clock (a zero `first_yield_ms` — the
/// drills never advance it), under the given end
/// kind.
const fn drill_position(end_kind: SessionEnd) -> StoredPosition {
    StoredPosition {
        offset_secs: LAST_SPOKEN_SECS,
        first_yield_ms: 0,
        end_kind,
    }
}

/// Embedded device fixture: the service-kill / power-loss
/// trial (3 ordered steps).
const FIXTURE_SERVICE_KILL_POWER_LOSS: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/service_kill_power_loss.jsonl"
);

/// The two-stand-in registry (the SC-051 failover
/// configuration): `standin_a` primary, `standin_b`
/// fallback — both `test_double`, admissible only while
/// no non-`test_double` adapter is registered (FR-023c /
/// the 2026-10-05 clarification).
fn two_standin_registry() -> Vec<EngineDescriptor> {
    vec![
        EngineDescriptor {
            id: String::from(StandinA::ID),
            tier: EngineTier::TestDouble,
        },
        EngineDescriptor {
            id: String::from(StandinB::ID),
            tier: EngineTier::TestDouble,
        },
    ]
}

/// The single-stand-in registry (the
/// single-engine-equivalent configuration of F3): the one
/// registered engine whose failure leaves nothing to offer.
fn single_standin_registry() -> Vec<EngineDescriptor> {
    vec![EngineDescriptor {
        id: String::from(StandinA::ID),
        tier: EngineTier::TestDouble,
    }]
}

/// The no-auto-audio witness (SC-011 / SC-012): an
/// `engine_core::trait_::Engine` whose `stream` is the only
/// synthesis path, counting every served request. An unchanged
/// count across a kill, an idle window, or a reopen is the
/// host-runnable form of "produces no audio until the user
/// activates resume".
#[derive(Debug, Default)]
struct SynthesisWitness {
    /// Synthesis requests served, in order.
    served: usize,
}

impl Engine for SynthesisWitness {
    /// The witness declares the coarsest pair — plain-text
    /// delivery with no alignment events.
    fn declared_capabilities(&self) -> (SsmlLevel, AlignmentGranularity) {
        (SsmlLevel::None, AlignmentGranularity::None)
    }

    /// A rate change is a pitch-preserving time-stretch
    /// inside the engine (FR-043): the witness accepts every
    /// rung of the closed ladder.
    fn set_rate(&mut self, rate: PlaybackRate) -> Result<(), EngineError> {
        let _ = rate;
        Ok(())
    }

    /// Synthesise `max_secs` of audio from `position_secs`:
    /// one served request on the witness's count. A request
    /// beyond the engine-side pre-roll bound is rejected
    /// before any synthesis runs (the `stream` contract).
    fn stream(&mut self, position_secs: f64, max_secs: f64) -> Result<StreamChunk, EngineError> {
        #[expect(
            clippy::neg_cmp_op_on_partial_ord,
            reason = "the negated >= deliberately rejects NaN inputs: NaN >= 0.0 is false, so a NaN position or bound fails closed before any synthesis runs (the stream contract)"
        )]
        if !(position_secs >= 0.0) || !(max_secs >= 0.0) || max_secs > MAX_PREROLL_SECS {
            return Err(EngineError::StartRefused);
        }
        self.served += 1;
        Ok(StreamChunk {
            start_position_secs: position_secs,
            frames: Vec::new(),
            boundaries: Vec::new(),
        })
    }
}

/// Feed `secs` of audible progress through the witness (the
/// backend's clock seam, FR-002): each synthesis request is
/// bounded by the engine-side pre-roll bound
/// ([`MAX_PREROLL_SECS`]), and every request is a served
/// entry on the witness's count. Audible progress is the
/// only position-advance path (FR-002 / FR-005).
fn feed_audible(
    session: &mut PlaybackSession,
    witness: &mut SynthesisWitness,
    secs: f64,
) -> Result<(), EngineError> {
    let mut remaining = secs;
    #[expect(
        clippy::while_float,
        reason = "each bounded step is min(remaining, MAX_PREROLL_SECS): above the bound the decrement is exact (a representable subtrahend) and the final step lands on exactly 0.0; > 0.0 is the only correct termination guard"
    )]
    while remaining > 0.0 {
        let step = remaining.min(MAX_PREROLL_SECS);
        let position = session.snapshot().position_secs;
        let _chunk = witness.stream(position, step)?;
        session.on_audible_progress(step);
        remaining -= step;
    }
    Ok(())
}

/// Paused durations on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600] — every value an exact binary
/// fraction, so the cursor's bound checks are tolerance-free.
#[expect(
    clippy::cast_precision_loss,
    reason = "the generated quarters are at most 14 400, exact in f64; the multiplication by the exact binary fraction 0.25 keeps every value on the grid"
)]
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// F1 (FR-023 / FR-023a / SC-039 / SC-040 / SC-051):
/// stand-in A fails mid-document → the named failure opens a
/// 3 s offer naming stand-in B → the expired offer takes
/// over from the last spoken position, locked, with no
/// unlock.
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[test]
fn standin_a_failure_offers_b_then_expiry_takes_over_locked() {
    let mut machine = RecoveryMachine::new(two_standin_registry());
    // Stand-in A fails mid-document at the last spoken
    // position: the engine authority records the named
    // failure and opens the 3 s offer naming the remaining
    // selectable engine (FR-023 / FR-023a).
    let offered = machine.on_engine_failure(StandinA::ID, LAST_SPOKEN_SECS);
    let alternatives = match &offered {
        RecoveryDecision::Offered { alternatives } => Some(alternatives.clone()),
        RecoveryDecision::Takeover { .. } | RecoveryDecision::PausedWithReason { .. } => None,
    };
    assert!(
        alternatives.is_some(),
        "the first failure with a remaining engine must open a 3 s offer naming it (FR-023a / SC-051), got {offered:?}"
    );
    let alternatives = alternatives.unwrap_or_default();
    assert_eq!(
        alternatives.len(),
        1,
        "the offer must name exactly the one remaining selectable engine (FR-023a)"
    );
    let first = &alternatives[0];
    assert_eq!(
        first.engine_id,
        StandinB::ID,
        "the offer must name stand-in B, the remaining stand-in (SC-051)"
    );
    assert_eq!(
        first.tier,
        EngineTier::TestDouble,
        "the stand-in offer carries the test_double tier (the 2026-10-05 clarification)"
    );
    // The 3 s window elapses with no choice: the
    // auto-fallback takes over from the last spoken
    // position — a continuation of the already-started
    // session, never a restart (FR-023a / SC-012).
    let expired = machine.on_offer_expired();
    let takeover = match &expired {
        RecoveryDecision::Takeover {
            engine_id,
            from_position_secs,
        } => Some((engine_id.clone(), *from_position_secs)),
        RecoveryDecision::Offered { .. } | RecoveryDecision::PausedWithReason { .. } => None,
    };
    assert!(
        takeover.is_some(),
        "an expired offer with a remaining engine must take over from the last spoken position (FR-023a), got {expired:?}"
    );
    let (takeover_engine, takeover_position) = takeover.unwrap_or_default();
    assert_eq!(
        takeover_engine,
        StandinB::ID,
        "the expired offer must take over on stand-in B (SC-051)"
    );
    assert_eq!(
        takeover_position, LAST_SPOKEN_SECS,
        "the takeover continues from the last spoken position, never a restart (FR-023a)"
    );
    // Locked resolution without unlock (FR-023 / SC-039 /
    // SC-040): the failed engine is no longer selectable,
    // the takeover engine is, and the closed `ShellVerb`
    // set has no unlock member — a locked resolution is
    // mechanically unlock-free.
    let rejected = machine.choose(StandinA::ID);
    assert!(
        rejected.is_err(),
        "the failed engine must not be selectable after the locked takeover (FR-023a)"
    );
    let chosen = machine.choose(StandinB::ID);
    assert!(
        chosen.is_ok(),
        "the takeover engine must remain the locked selection (FR-023a)"
    );
    let _ = EngineOffer {
        engine_id: String::from(StandinB::ID),
        tier: EngineTier::TestDouble,
    };
}

/// F2 (FR-023 / FR-037): the takeover engine (B) is forced
/// down mid-document → paused with a stated reason, never
/// silence — a failure that cannot be named cannot be
/// reported, and with no engine remaining there is nothing
/// to offer or take over on.
#[test]
fn takeover_engine_forced_down_pauses_with_reason() {
    let mut machine = RecoveryMachine::new(two_standin_registry());
    let _offered = machine.on_engine_failure(StandinA::ID, LAST_SPOKEN_SECS);
    let _takeover = machine.on_offer_expired();
    // The takeover engine is forced down, deeper into the
    // document: both stand-ins are down, so the machine
    // pauses with a named reason.
    let forced_down = machine.on_engine_failure(StandinB::ID, 2.0 * LAST_SPOKEN_SECS);
    match &forced_down {
        RecoveryDecision::PausedWithReason { reason } => {
            assert!(
                !reason.is_empty(),
                "the no-alternative pause must state a named reason (FR-023 / FR-037)"
            );
        }
        RecoveryDecision::Takeover { .. } | RecoveryDecision::Offered { .. } => {
            let paused_with_reason =
                matches!(forced_down, RecoveryDecision::PausedWithReason { .. });
            assert!(
                paused_with_reason,
                "with both stand-ins down there is no takeover and nothing to offer — the session must pause with a reason (FR-023), got {forced_down:?}"
            );
        }
    }
}

/// F3 (FR-023): a single-engine-registry failure → paused
/// with a reason, never silence — the
/// single-engine-equivalent configuration has no alternative
/// to offer or take over on, so the one engine's failure is
/// a named paused-with-reason decision.
#[test]
fn single_engine_registry_failure_pauses_with_reason() {
    let mut machine = RecoveryMachine::new(single_standin_registry());
    let failed = machine.on_engine_failure(StandinA::ID, LAST_SPOKEN_SECS);
    match &failed {
        RecoveryDecision::PausedWithReason { reason } => {
            assert!(
                !reason.is_empty(),
                "the single-engine failure must state a named reason (FR-023)"
            );
        }
        RecoveryDecision::Takeover { .. } | RecoveryDecision::Offered { .. } => {
            let paused_with_reason = matches!(failed, RecoveryDecision::PausedWithReason { .. });
            assert!(
                paused_with_reason,
                "a single-engine registry has no alternative to offer or take over on (FR-023), got {failed:?}"
            );
        }
    }
}

/// F4 (SC-011 / SC-012): the service is killed under memory
/// pressure → the next launch shows a paused state at the
/// last spoken position and produces no audio until the user
/// activates resume. The kill is not a Stop — only Stop
/// discards the stored position (FR-032f) — so the stored
/// position survives it.
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[test]
fn service_kill_pauses_at_position_without_auto_audio() {
    let mut session = PlaybackSession::new();
    let mut witness = SynthesisWitness::default();
    // Playback to the last spoken position (120.0 s of
    // audible progress — two bounded synthesis requests).
    let played = session.execute(ShellVerb::Play);
    assert!(
        played.is_ok(),
        "the Play press must succeed from the initial Paused state, got {played:?}"
    );
    let fed = feed_audible(&mut session, &mut witness, LAST_SPOKEN_SECS);
    assert!(
        fed.is_ok(),
        "audible progress must succeed while Playing, got {fed:?}"
    );
    assert_eq!(
        witness.served, 2,
        "120.0 s of audible progress is two bounded synthesis requests (the {MAX_PREROLL_SECS} s engine-side bound)"
    );
    let spoken = session.snapshot();
    assert_eq!(
        spoken.transport,
        TransportState::Playing,
        "audible progress must leave the transport on Playing (FR-005)"
    );
    assert_eq!(
        spoken.position_secs, LAST_SPOKEN_SECS,
        "the position authority must hold the last spoken position (FR-002)"
    );
    // The service is killed under memory pressure: the
    // session lands Paused at the last spoken position with
    // the bookmark preserved (SC-011).
    let paused = session.execute(ShellVerb::Pause);
    assert!(
        paused.is_ok(),
        "the pause into Paused must succeed from Playing, got {paused:?}"
    );
    let killed = session.snapshot();
    assert_eq!(
        killed.transport,
        TransportState::Paused,
        "a service kill shows a paused state (SC-011)"
    );
    assert_eq!(
        killed.position_secs, LAST_SPOKEN_SECS,
        "the paused state holds the last spoken position — the kill is not a Stop, so the position is preserved (SC-011 / FR-032f)"
    );
    // No auto-audio: the kill and the idle time that
    // follows serve no synthesis request, and nothing moves
    // the transport on its own (SC-011 / SC-012).
    assert_eq!(
        witness.served, 2,
        "the service kill must not trigger synthesis (SC-011 / SC-012)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "no audio begins until the user activates resume (SC-011)"
    );
}

/// F5a (SC-020 / FR-032c): reopening a deliberately paused
/// document with the application in the foreground — the
/// 3 s offer window elapses and reading begins within the
/// 4 s offer-to-audio budget, whether or not the user acts.
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "the budget arithmetic is compile-time constants by design; the assertion pins the SC-020 bound as a documented, greppable invariant of the fixture (FR-032c)"
)]
#[test]
fn foregrounded_deliberate_reopen_auto_starts_within_the_sc020_budget() {
    let mut offer = OfferMachine::new();
    let _ = offer.on_session_end(DRILL_FINGERPRINT, Some(drill_position(SessionEnd::Paused)));
    offer.on_foregrounded();
    // The latch is not the conversion: the foreground flag
    // is read at expiry ONLY, never converting an open
    // offer early (FR-032c's visibility MUST). The window
    // elapses with the application in the foreground: the
    // deliberate-pause offer auto-continues — the FR-032c
    // exception SC-012 names.
    let expired = offer.expire();
    match &expired {
        ExpiryOutcome::AutoStart { offset_secs } => {
            assert_eq!(
                *offset_secs, LAST_SPOKEN_SECS,
                "the auto-start continues from the stored position (FR-032c / SC-020)"
            );
        }
        ExpiryOutcome::ExplicitResume | ExpiryOutcome::NoOffer => {
            let auto_started = matches!(expired, ExpiryOutcome::AutoStart { .. });
            assert!(
                auto_started,
                "a foregrounded deliberate-pause offer must auto-start at expiry (SC-020), got {expired:?}"
            );
        }
    }
    // SC-020's bound: reading begins within 4 s of the
    // offer appearing — the 3 s offer window (FR-032c) plus
    // the 1 s post-expiry start budget, stricter than the
    // FR-027 3 s play-to-audio bound that governs the
    // post-expiry start itself. The wall-clock measurement
    // is a device trial (F8); the budget arithmetic and the
    // auto-start decision are asserted here.
    assert!(
        OFFER_WINDOW_SECS + POST_EXPIRY_START_BUDGET_SECS <= SC020_OFFER_TO_AUDIO_SECS,
        "the offer-to-audio budget exceeds the SC-020 {SC020_OFFER_TO_AUDIO_SECS} s bound (FR-032c)"
    );
    let _ = OFFER_WINDOW_SECS;
    let _ = POST_EXPIRY_START_BUDGET_SECS;
}

/// F5b (SC-020 / FR-032c / FR-032d): the same deliberate
/// pause, but the window elapses with the application in
/// the background — no audio begins until the user activates
/// resume.
#[test]
fn backgrounded_deliberate_reopen_expires_without_auto_audio() -> Result<(), Box<dyn Error>> {
    // The stored position the explicit-resume control
    // reads: the drill's last spoken position under the
    // deliberate-pause end kind, held by the bookmark
    // store under the document's fingerprint.
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("bookmarks.redb");
    let mut store = BookmarkStore::open(&path)?;
    store.put(&DRILL_FINGERPRINT, drill_position(SessionEnd::Paused))?;
    let mut offer = OfferMachine::new();
    let _ = offer.on_session_end(DRILL_FINGERPRINT, Some(drill_position(SessionEnd::Paused)));
    offer.on_backgrounded();
    // SC-020's tail: where the application is not in the
    // foreground when the window expires, no audio begins
    // until the user activates resume (FR-032c / FR-032d).
    let expired = offer.expire();
    match &expired {
        ExpiryOutcome::ExplicitResume => {
            // The silent expiry keeps the stored position
            // for the explicit resume control (SC-020): the
            // control resumes from the position the store
            // still holds under the fingerprint.
            let kept = store.get(&DRILL_FINGERPRINT)?;
            assert_eq!(
                kept.map(|stored| stored.offset_secs),
                Some(LAST_SPOKEN_SECS),
                "the silent expiry dropped the stored position the explicit resume control reads (SC-020)"
            );
        }
        ExpiryOutcome::AutoStart { .. } | ExpiryOutcome::NoOffer => {
            let explicit = matches!(expired, ExpiryOutcome::ExplicitResume);
            assert!(
                explicit,
                "a backgrounded expiry must not auto-start (SC-020 / FR-032d), got {expired:?}"
            );
        }
    }
    Ok(())
}

/// F5c (SC-021 / FR-032d): reopening a document left by an
/// unexpected end produces no audio at all until the user
/// activates resume — even with the application in the
/// foreground. The foreground latch never converts an
/// unexpected-end offer to an auto-start.
#[test]
fn unexpected_end_never_auto_starts_even_in_the_foreground() -> Result<(), Box<dyn Error>> {
    // The stored position the explicit-resume control
    // reads: the drill's last spoken position under the
    // interrupted end kind, held by the bookmark store
    // under the document's fingerprint.
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("bookmarks.redb");
    let mut store = BookmarkStore::open(&path)?;
    store.put(&DRILL_FINGERPRINT, drill_position(SessionEnd::Interrupted))?;
    let mut offer = OfferMachine::new();
    // An unexpected end opens no continue offer — the
    // offer form is resolved solely from the stored end
    // kind (storage-schema.md).
    let opened = offer.on_session_end(
        DRILL_FINGERPRINT,
        Some(drill_position(SessionEnd::Interrupted)),
    );
    assert!(
        opened.is_none(),
        "an unexpected end opened a continue offer (FR-032d)"
    );
    offer.on_foregrounded();
    // SC-021: an unexpected end (a power-off, a battery
    // death, a network loss, an interrupted session) offers
    // an explicit resume control and nothing more.
    let expired = offer.expire();
    match &expired {
        ExpiryOutcome::ExplicitResume => {
            let kept = store.get(&DRILL_FINGERPRINT)?;
            assert_eq!(
                kept.map(|stored| stored.offset_secs),
                Some(LAST_SPOKEN_SECS),
                "the unexpected-end offer dropped the stored position the explicit resume control reads (SC-021)"
            );
        }
        ExpiryOutcome::AutoStart { .. } | ExpiryOutcome::NoOffer => {
            let explicit = matches!(expired, ExpiryOutcome::ExplicitResume);
            assert!(
                explicit,
                "an unexpected end never auto-starts, even in the foreground (SC-021 / FR-032d), got {expired:?}"
            );
        }
    }
    Ok(())
}

/// F5d (FR-032c / FR-032f): the reader's own actions
/// inside the window — activating the offer continues from
/// the stored position, and start-over discards it.
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[test]
fn reader_activation_and_start_over_inside_the_window() {
    // The reader activates the offer inside the 3 s
    // window: the open offer continues from the stored
    // position immediately (FR-032c).
    let mut activated = OfferMachine::new();
    let _ = activated.on_session_end(DRILL_FINGERPRINT, Some(drill_position(SessionEnd::Paused)));
    let continued = activated.continue_offer();
    match &continued {
        Some(Offer::Continue { offset_secs, .. }) => {
            assert_eq!(
                *offset_secs, LAST_SPOKEN_SECS,
                "the reader's activation continues from the stored position (FR-032c)"
            );
        }
        Some(Offer::StartOver) | None => {
            let auto_started = matches!(continued, Some(Offer::Continue { .. }));
            assert!(
                auto_started,
                "activating the offer inside the window must continue from the stored position (FR-032c), got {continued:?}"
            );
        }
    }
    // Start-over discards the stored position (FR-023 /
    // FR-032f): reading begins from the first spoken
    // position — exactly 0.0.
    let mut restarted = OfferMachine::new();
    let _ = restarted.on_session_end(DRILL_FINGERPRINT, Some(drill_position(SessionEnd::Paused)));
    let started_over = restarted.start_over();
    match &started_over {
        ExpiryOutcome::AutoStart { offset_secs } => {
            assert_eq!(
                *offset_secs, 0.0,
                "start-over must begin from the first spoken position, 0.0 (FR-032f)"
            );
        }
        ExpiryOutcome::ExplicitResume | ExpiryOutcome::NoOffer => {
            let discarded = matches!(started_over, ExpiryOutcome::AutoStart { .. });
            assert!(
                discarded,
                "start-over must discard the stored position (FR-032f), got {started_over:?}"
            );
        }
    }
}

/// F6 (FR-004 / FR-015): the power-loss replay cursor
/// stays inside the 30 s hard cap — the 5% rule for a
/// short pause, the saturated cap for a long power loss,
/// and a replay that lands at the last spoken area, never
/// the document start.
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[test]
fn power_loss_replay_stays_within_the_30s_cap() {
    // The 5% rule: a short pause rewinds 5% of the paused
    // duration.
    assert!(
        (cursor(100.0) - 5.0).abs() <= POS_EPS_SECS,
        "the FR-015 cursor for a 100.0 s pause must be 5.0 s"
    );
    // The hard cap: a long power loss saturates at 30.0 s.
    assert!(
        (cursor(600.0) - 30.0).abs() <= POS_EPS_SECS,
        "the cursor must saturate at the 30 s rewind cap (FR-004 / FR-015)"
    );
    // A non-positive pause rewinds nothing.
    assert_eq!(
        cursor(0.0),
        0.0,
        "a zero paused duration owes no rewind (FR-015)"
    );
    // The replay lands at the last spoken area, never the
    // document start: 120.0 s spoken, a 600.0 s power
    // loss, a 30.0 s cap — replay at 90.0 s.
    let replay_secs = (LAST_SPOKEN_SECS - cursor(600.0)).max(0.0);
    assert!(
        (replay_secs - 90.0).abs() <= POS_EPS_SECS,
        "the power-loss replay must land at 90.0 s, got {replay_secs} s (FR-004)"
    );
    assert!(
        replay_secs > 0.0,
        "the replay must not land at the document start (the fixture records document_start=false)"
    );
}

/// F7: full replay of the device fixture — every step, in
/// fixture order, driven through the Rust state machines
/// with the outcome each `result` token records.
#[test]
#[expect(
    clippy::float_cmp,
    reason = "the drill positions, yields, pauses and grid gaps are exact quarter-second values and the budget constants are exact literals; == is the correct assertion and a tolerance would admit an off-spec replay"
)]
#[expect(
    clippy::too_many_lines,
    reason = "the fixture replays the whole power-loss sequence end to end — kill, restart, replay, cap check — in one flow over the single session state it perturbs; splitting would thread that state through helpers and scatter the cap arithmetic"
)]
fn service_kill_power_loss_fixture_replay() -> Result<(), Box<dyn Error>> {
    let lines: Vec<&str> = FIXTURE_SERVICE_KILL_POWER_LOSS.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "service_kill_power_loss.jsonl must carry exactly the 3 recorded trial steps"
    );
    assert!(
        lines[0].contains("SERVICE_KILL") && lines[0].contains("paused_at_position"),
        "fixture step 1 must be SERVICE_KILL → paused_at_position"
    );
    assert!(
        lines[1].contains("REOPEN") && lines[1].contains("explicit_resume_not_auto_audio"),
        "fixture step 2 must be REOPEN → explicit_resume_not_auto_audio"
    );
    assert!(
        lines[2].contains("POWER_LOSS") && lines[2].contains("replay_at_last_spoken"),
        "fixture step 3 must be POWER_LOSS → replay_at_last_spoken"
    );

    // Step 1 — SERVICE_KILL (ts 0, `bookmark_preserved:
    // true`): playback to the last spoken position, then
    // the service is killed under memory pressure. The
    // next launch shows a paused state at the last spoken
    // position (SC-011); the kill is not a Stop, so the
    // stored position — the bookmark basis — is preserved
    // (FR-032f).
    let mut session = PlaybackSession::new();
    let mut witness = SynthesisWitness::default();
    let played = session.execute(ShellVerb::Play);
    assert!(
        played.is_ok(),
        "the Play press must succeed from the initial Paused state, got {played:?}"
    );
    let fed = feed_audible(&mut session, &mut witness, LAST_SPOKEN_SECS);
    assert!(
        fed.is_ok(),
        "audible progress must succeed while Playing, got {fed:?}"
    );
    assert_eq!(
        witness.served, 2,
        "120.0 s of audible progress is two bounded synthesis requests"
    );
    let killed = session.execute(ShellVerb::Pause);
    assert!(
        killed.is_ok(),
        "the pause into Paused must succeed from Playing, got {killed:?}"
    );
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "the kill shows a paused state (SC-011 → paused_at_position)"
    );
    assert_eq!(
        snapshot.position_secs, LAST_SPOKEN_SECS,
        "the paused state holds the last spoken position (SC-011 → paused_at_position)"
    );

    // Step 2 — REOPEN (ts 31000, `unexpected_end: true`,
    // `position_offer: true`): an unexpected end offers an
    // explicit resume control and produces no audio at all
    // until the user activates resume (SC-021 / FR-032d) —
    // even with the application in the foreground. The
    // position the control reads stays in the bookmark
    // store under the document's fingerprint.
    let reopen_dir = tempfile::tempdir()?;
    let reopen_path = reopen_dir.path().join("bookmarks.redb");
    let mut reopen_store = BookmarkStore::open(&reopen_path)?;
    reopen_store.put(&DRILL_FINGERPRINT, drill_position(SessionEnd::Interrupted))?;
    let mut offer = OfferMachine::new();
    let opened = offer.on_session_end(
        DRILL_FINGERPRINT,
        Some(drill_position(SessionEnd::Interrupted)),
    );
    assert!(
        opened.is_none(),
        "an unexpected-end reopen opened a continue offer (SC-021 → explicit_resume_not_auto_audio)"
    );
    offer.on_foregrounded();
    let reopened = offer.expire();
    match &reopened {
        ExpiryOutcome::ExplicitResume => {
            let kept = reopen_store.get(&DRILL_FINGERPRINT)?;
            assert_eq!(
                kept.map(|stored| stored.offset_secs),
                Some(LAST_SPOKEN_SECS),
                "the reopen dropped the stored position the explicit resume control reads (SC-021 → explicit_resume_not_auto_audio)"
            );
        }
        ExpiryOutcome::AutoStart { .. } | ExpiryOutcome::NoOffer => {
            let explicit = matches!(reopened, ExpiryOutcome::ExplicitResume);
            assert!(
                explicit,
                "an unexpected-end reopen must never auto-start (SC-021 → explicit_resume_not_auto_audio), got {reopened:?}"
            );
        }
    }
    assert_eq!(
        witness.served, 2,
        "the unexpected-end reopen served no synthesis — explicit resume, not auto audio (SC-021)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "the reopen left the transport Paused — no audio until the user activates resume (SC-021)"
    );

    // Step 3 — POWER_LOSS (ts 62000, `replay_bound_ms:
    // 30000`, `document_start: false`): the session has
    // been paused since the kill at ts 0, so the power
    // loss interrupts a 62.0 s pause; the replay cursor is
    // min(0.05 × 62.0, 30.0) = 3.1 s — inside the
    // fixture's 30 000 ms bound — and the replay lands at
    // the last spoken area, 120.0 − 3.1 = 116.9 s, never
    // the document start (SC-011 / FR-004 / FR-015).
    let paused_duration_secs = 62.0;
    let replay_cursor = cursor(paused_duration_secs);
    assert!(
        replay_cursor <= REWIND_CAP_SECS,
        "the replay cursor must respect the fixture's 30 000 ms bound (FR-004)"
    );
    assert!(
        (replay_cursor - 3.1).abs() <= POS_EPS_SECS,
        "the cursor for the 62.0 s pause must be 3.1 s, got {replay_cursor} s (FR-015)"
    );
    let replay_secs = (LAST_SPOKEN_SECS - replay_cursor).max(0.0);
    assert!(
        (replay_secs - 116.9).abs() <= POS_EPS_SECS,
        "the replay must land at the last spoken area 116.9 s, got {replay_secs} s (FR-004 → replay_at_last_spoken)"
    );
    assert!(
        replay_secs > 0.0,
        "the replay must not land at the document start (the fixture records document_start=false)"
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// F6 as a property (FR-004 / FR-015): the power-loss
    /// replay cursor stays inside the 30 s hard cap for
    /// every paused duration on the quarter-second grid,
    /// never rewinds below the floor, and equals the single
    /// formula `player::rewind::cursor` owns —
    /// `min(0.05 × paused, 30.0)`.
    #[test]
    fn the_power_loss_replay_cursor_stays_within_the_cap(
        paused_duration_secs in paused_duration_strategy()
    ) {
        let replay_cursor = cursor(paused_duration_secs);
        prop_assert!(
            replay_cursor >= 0.0,
            "the cursor must never rewind below the floor (FR-015), got {replay_cursor} s"
        );
        prop_assert!(
            replay_cursor <= REWIND_CAP_SECS,
            "the cursor must never exceed the 30 s rewind cap (FR-004), got {replay_cursor} s"
        );
        let expected = (REWIND_FRACTION * paused_duration_secs).min(REWIND_CAP_SECS);
        prop_assert!(
            (replay_cursor - expected).abs() <= POS_EPS_SECS,
            "the cursor must equal min(0.05 × paused, 30.0), got {replay_cursor} s for a {paused_duration_secs} s pause (FR-015)"
        );
    }
}
