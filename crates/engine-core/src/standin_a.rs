//! Stand-in engine A (T017b): the offline-like fixture.
//!
//! One of the two stand-in engines that honour the
//! engine-handoff contract (contracts/engine-handoff.md,
//! "Conformance") so the mid-document failover and
//! forced-double-failure drills (SC-051) have working
//! fixtures before any real adapter exists. Stand-in A
//! plays the offline-like role in the drill; stand-in B
//! (`crate::standin_b`) plays the cloud-like role.
//!
//! A stand-in is a `test_double` (the 2026-10-05
//! clarification): it is never selectable while any
//! non-`test_double` adapter is registered, and it
//! exists only as a test fixture — no fallback or
//! selection logic lives here (that is
//! `crate::fallback`, T075).
//!
//! The stand-in implements [`crate::trait_::Engine`]
//! (T075): it declares its capabilities honestly,
//! accepts every rung of the closed rate ladder with
//! `Ok(())`, and streams the deterministic
//! document-locked signal inside the pre-roll bound.
//! The pitch-preserving time-stretch (FR-043) holds by
//! construction — the delivered PCM is the same on
//! every rung, which is exactly the pitch invariance
//! `tests/prop_pitch.rs` asserts. The inherent
//! [`StandinA::set_rate`] stays the method-call surface
//! (the T055/T071 red-root-cause fix): it wins in
//! method-call syntax, and the
//! [`crate::trait_::Engine::set_rate`] trait method
//! routes through it, so both surfaces are the single
//! accepted-rate path.

use std::f64::consts::PI;

use crate::closed::{EngineError, EngineTier, PlaybackRate};
use crate::format::DeclaredFormat;
use crate::trait_::{
    AlignmentGranularity, BoundaryEvent, BoundaryKind, Engine, MAX_PREROLL_SECS, SsmlLevel,
    StreamChunk,
};

/// Stand-in engine A: the offline-like engine of the
/// SC-051 failover drill.
///
/// The stand-in honours the handoff contract exactly:
/// it delivers mono `f32` PCM at the declared 22050 Hz
/// on every path, including every rung of the rate
/// ladder — a rate change is a pitch-preserving
/// time-stretch inside the engine (FR-043, SC-034) and
/// never alters the delivered format.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandinA;

impl StandinA {
    /// The engine id: `standin_a`. One of the two
    /// non-registry ids permitted by the 2026-10-05
    /// clarification, valid only with
    /// [`EngineTier::TestDouble`] and only as a
    /// stand-in fixture.
    pub const ID: &'static str = "standin_a";

    /// The declared tier: [`EngineTier::TestDouble`] —
    /// never selectable while any non-`test_double`
    /// adapter is registered.
    pub const TIER: EngineTier = EngineTier::TestDouble;

    /// Set the playback rate (FR-019).
    ///
    /// The contract is a pitch-preserving time-stretch
    /// **inside the engine**: the delivered sample rate
    /// stays the declared 22050 Hz on all seven rungs
    /// (FR-043, SC-034), and the pitch shift stays
    /// within the shared SC-015 bound
    /// ([`crate::format::PITCH_TOLERANCE_CENTS`]) — a
    /// deviation beyond that bound is an [`crate::closed::EngineError::RateMismatch`]
    /// (contracts/engine-handoff.md). The stand-in
    /// satisfies the contract by construction: it
    /// accepts the rate and keeps the delivered format
    /// invariant, so every rung delivers the declared
    /// shape.
    pub const fn set_rate(&mut self, rate: PlaybackRate) {
        // The requested rate is accepted per the engine
        // contract; the delivered format below stays
        // invariant across all seven rungs (SC-034).
        let _ = rate;
    }

    /// The delivered handoff shape as
    /// `(rate_hz, channels, width_bits)`: always the
    /// declared mono-`f32`-at-22050 Hz shape, on every
    /// path and at every rate rung.
    #[must_use]
    pub const fn delivered_format(&self) -> (u32, u16, u16) {
        (
            DeclaredFormat::SAMPLE_RATE,
            DeclaredFormat::CHANNELS,
            DeclaredFormat::SAMPLE_WIDTH_BITS,
        )
    }
}

/// The stand-in's sentence-boundary cadence: one
/// sentence boundary per second of delivered audio.
///
/// The stand-in declares sentence alignment (its
/// [`Engine::declared_capabilities`]), so every
/// boundary event it delivers is a
/// [`BoundaryKind::Sentence`] at a cadence multiple.
const SENTENCE_BOUNDARY_SECS: f64 = 1.0;

/// The fundamental of the deterministic stand-in
/// signal: 220 Hz (A3), with two phase-locked harmonics
/// at 440 Hz and 660 Hz — the same fixture waveform
/// the pitch-invariance property (`tests/prop_pitch.rs`)
/// calibrates its estimators against.
const SOURCE_F0_HZ: f64 = 220.0;

/// Whether a pre-roll request sits outside the contract
/// bounds: a negative or non-finite position, a
/// negative or non-finite budget, or a budget above the
/// inclusive [`MAX_PREROLL_SECS`] bound.
///
/// Each bound is checked explicitly — `NaN` is
/// caught by the `is_nan` check and every other
/// value is compared against zero directly, which is
/// exactly the contract's rejection condition
/// (`!(value >= 0.0)`) restated without a negated
/// partially-ordered comparison. The budget bound is
/// inclusive (exactly 60.0 s is the largest
/// outstanding pre-roll the contract permits, H6).
#[must_use]
fn is_out_of_bounds(position_secs: f64, max_secs: f64) -> bool {
    position_secs < 0.0
        || position_secs.is_nan()
        || max_secs < 0.0
        || max_secs.is_nan()
        || max_secs > MAX_PREROLL_SECS
}

/// The number of frames a pre-roll budget spans at the
/// declared rate: `max_secs × 22050`.
///
/// `max_secs` is validated to `[0, MAX_PREROLL_SECS]`
/// before this runs, so the product is at most
/// `1_323_000` — non-negative and far below any `usize`
/// truncation boundary.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "max_secs is validated to [0, MAX_PREROLL_SECS] before this runs, so the product is at most 1_323_000 — far below any usize truncation boundary and non-negative"
)]
#[must_use]
fn frames_for_budget(max_secs: f64) -> usize {
    (max_secs * f64::from(DeclaredFormat::SAMPLE_RATE)) as usize
}

/// Synthesise `max_secs` of the deterministic signal
/// starting at document `position_secs`: a 220 Hz
/// fundamental with two phase-locked harmonics —
/// `sin(2π·220·t) + 0.5·sin(2π·440·t) +
/// 0.25·sin(2π·660·t)` — at the declared 22050 Hz,
/// phase-locked from document t = 0.
///
/// The waveform at a document time `t` is the same on
/// every call and at every rate rung, so the delivered
/// PCM is pitch-invariant by construction: the
/// pitch-preserving time-stretch (FR-043) is an
/// engine-internal operation the fixture honours by
/// delivering the document-locked signal, and the
/// delivered sample rate stays the declared 22050 Hz
/// on every path (SC-034).
#[expect(
    clippy::cast_possible_truncation,
    reason = "the source signal is a deterministic fixture waveform bounded to ±1.75 (1 + 0.5 + 0.25), far inside f32's range; the f64→f32 narrowing is the declared handoff width (f32 PCM) and the contour estimators absorb the rounding"
)]
#[must_use]
fn synthesize_frames(position_secs: f64, max_secs: f64) -> Vec<f32> {
    let frame_count = frames_for_budget(max_secs);
    let mut frames = Vec::with_capacity(frame_count);
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    let phase_step = 2.0 * PI * SOURCE_F0_HZ / sample_rate_hz;
    // The phase starts at the document position, so the
    // waveform is locked to the document — the signal at
    // document time `t` never depends on which chunk
    // requested it.
    let mut phase = 2.0 * PI * SOURCE_F0_HZ * position_secs;
    for _ in 0..frame_count {
        let fundamental = phase.sin();
        let second_harmonic = (2.0 * phase).sin();
        let third_harmonic = (3.0 * phase).sin();
        frames.push(0.25_f64.mul_add(
            third_harmonic,
            0.5_f64.mul_add(second_harmonic, fundamental),
        ) as f32);
        phase += phase_step;
    }
    frames
}

/// The number of [`SENTENCE_BOUNDARY_SECS`] multiples
/// in the half-open pre-roll window `[position_secs,
/// position_secs + max_secs)`: the integer cadence
/// indices at or after the window start and before the
/// window end.
///
/// The count is at most `max_secs / cadence + 1` — the
/// validated pre-roll budget bounds it — so the
/// `f64`→`usize` conversion below is exact,
/// non-negative, and far below any truncation boundary.
/// Computing the count first (rather than accumulating
/// the boundary position) also keeps the generator
/// advancing past the `f64` precision limit (`2^53` s):
/// an accumulated `+= cadence` would stop advancing
/// there and never terminate.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the count is the number of cadence multiples inside the validated window — at most MAX_PREROLL_SECS / SENTENCE_BOUNDARY_SECS + 1 = 61 — so the f64→usize conversion is exact and non-negative"
)]
#[must_use]
fn boundary_count(position_secs: f64, max_secs: f64) -> usize {
    let first_multiple = (position_secs / SENTENCE_BOUNDARY_SECS).ceil();
    let window_end_index = (position_secs + max_secs) / SENTENCE_BOUNDARY_SECS;
    ((window_end_index - first_multiple).ceil().max(0.0)) as usize
}

/// The sentence boundaries inside a pre-roll window:
/// one [`BoundaryKind::Sentence`] event at every
/// [`SENTENCE_BOUNDARY_SECS`] multiple in the half-open
/// window `[position_secs, position_secs + max_secs)`,
/// in ascending position order.
#[expect(
    clippy::cast_precision_loss,
    reason = "the offset is a small non-negative count — at most MAX_PREROLL_SECS / SENTENCE_BOUNDARY_SECS = 60 (see `boundary_count`) — so the usize→f64 conversion is exact"
)]
#[must_use]
fn sentence_boundaries(position_secs: f64, max_secs: f64) -> Vec<BoundaryEvent> {
    let first_multiple = (position_secs / SENTENCE_BOUNDARY_SECS).ceil();
    let count = boundary_count(position_secs, max_secs);
    (0..count)
        .map(|offset| BoundaryEvent {
            kind: BoundaryKind::Sentence,
            position_secs: (first_multiple + offset as f64) * SENTENCE_BOUNDARY_SECS,
        })
        .collect()
}

impl Engine for StandinA {
    /// Declare the stand-in's capabilities honestly
    /// (contract "Conformance": "granularity honestly
    /// declared"): the offline-like role of the SC-051
    /// drill declares the partial SSML subset with
    /// sentence alignment — the FR-015 amendment's
    /// delivered-engine granularity — and the delivered
    /// boundary events honour the declaration (every
    /// [`BoundaryEvent`] is a [`BoundaryKind::Sentence`]).
    fn declared_capabilities(&self) -> (SsmlLevel, AlignmentGranularity) {
        (SsmlLevel::Partial, AlignmentGranularity::Sentence)
    }

    /// Accept the rate rung (FR-019, FR-043, SC-034).
    ///
    /// The pitch-preserving time-stretch happens inside
    /// the engine and never alters the delivered sample
    /// rate: the stand-in accepts every rung of the
    /// closed ladder and keeps the delivered format
    /// invariant, so the stretch holds the pitch
    /// tolerance by construction — the contract's
    /// failure side ([`crate::closed::EngineError::RateMismatch`])
    /// never fires for the fixture.
    ///
    /// # Errors
    ///
    /// Never fails: the fixture honours the stretch
    /// contract by construction, so every rung is
    /// accepted with `Ok(())`.
    fn set_rate(&mut self, rate: PlaybackRate) -> Result<(), EngineError> {
        // The inherent `set_rate` — the T055/T071
        // red-root-cause fix — wins in method-call
        // syntax, so the trait method routes through
        // the single accepted-rate path.
        self.set_rate(rate);
        Ok(())
    }

    /// Stream pre-roll audio starting at `position_secs`,
    /// up to `max_secs` of the deterministic signal.
    ///
    /// The request is rejected before any synthesis
    /// runs: a negative or non-finite position, a
    /// negative or non-finite budget, or a budget above
    /// the inclusive [`MAX_PREROLL_SECS`] bound is a
    /// named [`crate::closed::EngineError::StartRefused`]
    /// — the same rejection the failover contract's
    /// synthesis witness names. The delivered chunk is
    /// buffered-only source: the stand-in holds no
    /// playback position, so reading the chunk never
    /// advances anything.
    ///
    /// # Errors
    ///
    /// Returns [`crate::closed::EngineError::StartRefused`]
    /// for a request outside the pre-roll bounds; the
    /// deterministic signal itself never fails.
    fn stream(&mut self, position_secs: f64, max_secs: f64) -> Result<StreamChunk, EngineError> {
        if is_out_of_bounds(position_secs, max_secs) {
            return Err(EngineError::StartRefused);
        }
        Ok(StreamChunk {
            start_position_secs: position_secs,
            frames: synthesize_frames(position_secs, max_secs),
            boundaries: sentence_boundaries(position_secs, max_secs),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_and_tier_declare_the_test_double_contract() {
        assert_eq!(StandinA::ID, "standin_a");
        assert_eq!(StandinA::TIER, EngineTier::TestDouble);
    }

    #[test]
    fn every_rate_rung_delivers_the_declared_shape() {
        for rate in [
            PlaybackRate::R0_75,
            PlaybackRate::R1_0,
            PlaybackRate::R1_25,
            PlaybackRate::R1_5,
            PlaybackRate::R1_75,
            PlaybackRate::R2_0,
            PlaybackRate::R4_0,
        ] {
            let mut engine = StandinA;
            engine.set_rate(rate);
            assert_eq!(
                engine.delivered_format(),
                (
                    DeclaredFormat::SAMPLE_RATE,
                    DeclaredFormat::CHANNELS,
                    DeclaredFormat::SAMPLE_WIDTH_BITS
                ),
                "rate {rate:?}"
            );
        }
    }

    #[test]
    fn the_trait_set_rate_accepts_every_rung() {
        for rate in [
            PlaybackRate::R0_75,
            PlaybackRate::R1_0,
            PlaybackRate::R1_25,
            PlaybackRate::R1_5,
            PlaybackRate::R1_75,
            PlaybackRate::R2_0,
            PlaybackRate::R4_0,
        ] {
            let mut engine = StandinA;
            assert_eq!(
                Engine::set_rate(&mut engine, rate),
                Ok(()),
                "the offline-like stand-in refused rung {rate:?}"
            );
            assert_eq!(
                engine.delivered_format(),
                (
                    DeclaredFormat::SAMPLE_RATE,
                    DeclaredFormat::CHANNELS,
                    DeclaredFormat::SAMPLE_WIDTH_BITS
                ),
                "rate {rate:?}"
            );
        }
    }

    #[test]
    fn the_declared_capabilities_are_stable_and_honest() {
        let (first_level, first_granularity) = Engine::declared_capabilities(&StandinA);
        let (second_level, second_granularity) = Engine::declared_capabilities(&StandinA);
        assert_eq!(
            (first_level, first_granularity),
            (second_level, second_granularity),
            "the declared capabilities are not stable across calls"
        );
        // The offline-like role declares the partial SSML
        // subset with sentence alignment (the FR-015
        // amendment: the delivered engines declare
        // sentence granularity), and both halves
        // round-trip their wire forms.
        assert_eq!(first_level, SsmlLevel::Partial);
        assert_eq!(first_level.as_wire_str(), "partial");
        assert_eq!(first_granularity, AlignmentGranularity::Sentence);
        assert_eq!(first_granularity.as_wire_str(), "sentence");
    }

    #[test]
    fn a_mid_document_stream_starts_at_the_requested_position() {
        let mut engine = StandinA;
        let Ok(chunk) = Engine::stream(&mut engine, 12.5, 1.0) else {
            // The stream is inside the pre-roll bound: a
            // conforming stand-in never refuses it.
            panic!("the mid-document stream was refused");
        };
        assert!((chunk.start_position_secs - 12.5).abs() < f64::EPSILON);
        assert_eq!(chunk.frame_count(), 22_050);
        // Sentence alignment: the one-second cadence puts
        // the single boundary of the half-open [12.5,
        // 13.5) window at 13.0 s.
        assert_eq!(
            chunk.boundaries,
            vec![BoundaryEvent {
                kind: BoundaryKind::Sentence,
                position_secs: 13.0,
            }]
        );
        // Every delivered frame is finite — the
        // deterministic signal is bounded.
        assert!(
            chunk.frames.iter().all(|frame| frame.is_finite()),
            "the mid-document stream delivered a non-finite frame"
        );
    }

    #[test]
    fn an_over_bound_budget_is_refused_before_any_synthesis() {
        let mut engine = StandinA;
        let over_bound = MAX_PREROLL_SECS + 0.5;
        assert_eq!(
            Engine::stream(&mut engine, 0.0, over_bound),
            Err(EngineError::StartRefused),
            "a {over_bound} s budget was not refused"
        );
        // The rejection never substitutes the delivered
        // format (FR-035).
        assert_eq!(
            engine.delivered_format(),
            (
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS
            )
        );
        // The bound is inclusive — exactly 60.0 s is the
        // largest outstanding pre-roll the contract
        // permits (H6).
        let at_bound = Engine::stream(&mut engine, 0.0, MAX_PREROLL_SECS);
        assert!(
            at_bound.is_ok(),
            "a pre-roll budget of exactly {MAX_PREROLL_SECS} s was refused: {at_bound:?}"
        );
    }

    #[test]
    fn a_negative_or_non_finite_request_is_refused() {
        let mut engine = StandinA;
        for (position_secs, max_secs) in [
            (-1.0, 1.0),
            (0.0, -1.0),
            (f64::NAN, 1.0),
            (0.0, f64::NAN),
            (f64::NAN, f64::NAN),
        ] {
            assert_eq!(
                Engine::stream(&mut engine, position_secs, max_secs),
                Err(EngineError::StartRefused),
                "the request ({position_secs}, {max_secs}) was not refused"
            );
        }
    }

    #[test]
    fn the_streamed_signal_is_document_locked_and_pitch_invariant() {
        let mut engine = StandinA;
        // The waveform at a document position is the same
        // on every call...
        let first = Engine::stream(&mut engine, 12.5, 1.0);
        let repeated = Engine::stream(&mut engine, 12.5, 1.0);
        assert_eq!(first, repeated);
        // ...and a rate change — a pitch-preserving
        // time-stretch inside the engine (FR-043) —
        // never alters the delivered PCM (SC-034): the
        // same PCM at every rung is the pitch invariance
        // the property suite asserts.
        for rung in [PlaybackRate::R0_75, PlaybackRate::R2_0, PlaybackRate::R4_0] {
            assert_eq!(
                Engine::set_rate(&mut engine, rung),
                Ok(()),
                "the offline-like stand-in refused rung {rung:?}"
            );
            assert_eq!(
                Engine::stream(&mut engine, 12.5, 1.0),
                first,
                "the delivered PCM changed at rung {rung:?}"
            );
        }
    }
}
