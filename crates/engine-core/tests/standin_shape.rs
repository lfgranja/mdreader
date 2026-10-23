//! T013b — stand-in handoff-shape test (Phase 2, BLOCKING).
//!
//! Invariant under test (tasks.md T013b, verbatim): "stand-ins
//! deliver mono f32 @22050 Hz on all paths; rate/channel/width
//! deviation → named `EngineError`, never silent resample per
//! FR-033, FR-035, SC-034"
//!
//! Spec anchors: FR-033 ("Engine emits mono f32 already resampled
//! to a fixed rate the shell declares; the shell only converts to
//! the device format"), FR-035 (a device that refuses the declared
//! rate is a named failure — never playback at a substituted
//! format, never a silent resample), FR-043/SC-034 (a rate change
//! never alters the delivered sample rate, so the single fixed
//! rate holds across every rung of the ladder), and
//! contracts/engine-handoff.md ("Any deviation (rate, channel
//! count, sample width) ⇒ named `EngineError` (`RateMismatch |
//! ChannelMismatch`), never silent resample/downmix").
//!
//! FAILS before `crates/engine-core/src/format.rs` (T015) and
//! `crates/engine-core/src/standin_a.rs` / `standin_b.rs` (T017b)
//! exist: the `engine_core::format`, `engine_core::standin_a` and
//! `engine_core::standin_b` modules are unresolved, so this test
//! target does not build. That build failure is the required
//! pre-implementation red state.
//!
//! Expected API contract for T015/T017b:
//! - `engine_core::format::DeclaredFormat` with associated
//!   constants `SAMPLE_RATE: u32 = 22050`, `CHANNELS: u16 = 1`,
//!   `SAMPLE_WIDTH_BITS: u16 = 32` (f32), and
//!   `pub fn check_handoff(rate_hz: u32, channels: u16,
//!   width_bits: u16) -> Result<(), EngineError>` — returns
//!   `RateMismatch` when the rate is not the declared 22050 Hz,
//!   `ChannelMismatch` when the channel count is not mono, and a
//!   named error (never `Ok`) for any width other than 32-bit
//!   f32. It never adapts silently: there is no resample or
//!   downmix path through it.
//! - `engine_core::standin_a::StandinA` and
//!   `engine_core::standin_b::StandinB` (T017b), each a unit
//!   struct with `pub fn set_rate(&mut self, rate: PlaybackRate)`
//!   (pitch-preserving; never alters the delivered format) and
//!   `pub fn delivered_format(&self) -> (u32, u16, u16)`
//!   returning `(rate_hz, channels, width_bits)` — always
//!   `(22050, 1, 32)`.

use proptest::prelude::*;

use engine_core::closed::{EngineError, PlaybackRate};
use engine_core::format::{DeclaredFormat, check_handoff};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;

/// Every rung of the closed rate ladder (FR-019).
fn rate_strategy() -> impl Strategy<Value = PlaybackRate> {
    prop_oneof![
        Just(PlaybackRate::R0_75),
        Just(PlaybackRate::R1_0),
        Just(PlaybackRate::R1_25),
        Just(PlaybackRate::R1_5),
        Just(PlaybackRate::R1_75),
        Just(PlaybackRate::R2_0),
        Just(PlaybackRate::R4_0),
    ]
}

/// The declared handoff shape: mono f32 at exactly 22050 Hz,
/// as `(rate_hz, channels, width_bits)`.
const DECLARED: (u32, u16, u16) = (
    DeclaredFormat::SAMPLE_RATE,
    DeclaredFormat::CHANNELS,
    DeclaredFormat::SAMPLE_WIDTH_BITS,
);

/// The declared handoff shape is mono f32 at 22050 Hz (FR-033:
/// the engine emits mono f32 already resampled to the fixed
/// declared rate; the numeric value is fixed by the
/// specification, not by implementation).
#[test]
fn the_declared_format_is_mono_f32_at_22050_hz() {
    assert_eq!(DeclaredFormat::SAMPLE_RATE, 22_050);
    assert_eq!(DeclaredFormat::CHANNELS, 1);
    assert_eq!(DeclaredFormat::SAMPLE_WIDTH_BITS, 32);
    assert!(
        check_handoff(
            DeclaredFormat::SAMPLE_RATE,
            DeclaredFormat::CHANNELS,
            DeclaredFormat::SAMPLE_WIDTH_BITS
        )
        .is_ok(),
        "the declared handoff shape must pass its own handoff check"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn standin_a_delivers_mono_f32_at_22050_on_every_rate_path(rate in rate_strategy()) {
        // SC-034: the delivered rate is invariant at 22050 Hz
        // across all rungs — a rate change (the engine's
        // pitch-preserving time-stretch, FR-043) never alters
        // the delivered sample rate. "On all paths" means: after
        // any rate the stand-in still delivers the declared
        // shape, and that shape passes the handoff check.
        let mut engine = StandinA;
        engine.set_rate(rate);
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinA deviated from the declared handoff shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinA's delivered shape failed its own handoff check at rate {:?}",
            rate
        );
    }

    #[test]
    fn standin_b_delivers_mono_f32_at_22050_on_every_rate_path(rate in rate_strategy()) {
        // Same invariant for the second stand-in (SC-051 needs
        // two working engines for the mid-document failover
        // drill; both honour the same handoff contract).
        let mut engine = StandinB;
        engine.set_rate(rate);
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinB deviated from the declared handoff shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinB's delivered shape failed its own handoff check at rate {:?}",
            rate
        );
    }

    #[test]
    fn a_rate_deviation_is_a_named_error_never_a_silent_resample(
        candidate in any::<u32>().prop_filter(
            "not the declared rate",
            |rate| *rate != DeclaredFormat::SAMPLE_RATE,
        )
    ) {
        // FR-035: a rate other than the declared 22050 Hz is a
        // named RateMismatch — never playback at a substituted
        // format and never a silent resample outside the engine.
        let result = check_handoff(
            candidate,
            DeclaredFormat::CHANNELS,
            DeclaredFormat::SAMPLE_WIDTH_BITS,
        );
        prop_assert_eq!(
            result,
            Err(EngineError::RateMismatch),
            "rate {} Hz was not named as a RateMismatch",
            candidate
        );
    }

    #[test]
    fn a_channel_deviation_is_a_named_error_never_a_silent_downmix(candidate in 0u16..=u16::MAX) {
        prop_assume!(candidate != DeclaredFormat::CHANNELS);
        // FR-033: an engine delivering stereo (or any non-mono
        // count) where mono f32 was declared surfaces as a named
        // ChannelMismatch — silently downmixing would make an
        // adapter conform by accident rather than by contract.
        let result = check_handoff(
            DeclaredFormat::SAMPLE_RATE,
            candidate,
            DeclaredFormat::SAMPLE_WIDTH_BITS,
        );
        prop_assert_eq!(
            result,
            Err(EngineError::ChannelMismatch),
            "channel count {} was not named as a ChannelMismatch",
            candidate
        );
    }

    #[test]
    fn a_width_deviation_is_a_named_error_never_silent(
        candidate in any::<u16>().prop_filter(
            "not the declared f32 width",
            |width| *width != DeclaredFormat::SAMPLE_WIDTH_BITS,
        )
    ) {
        // contracts/engine-handoff.md: any sample-width deviation
        // is a named EngineError (RateMismatch | ChannelMismatch)
        // — never a silent reinterpretation of the buffer, which
        // would make an adapter conform by accident.
        let result = check_handoff(
            DeclaredFormat::SAMPLE_RATE,
            DeclaredFormat::CHANNELS,
            candidate,
        );
        let named = matches!(
            result,
            Err(EngineError::RateMismatch | EngineError::ChannelMismatch)
        );
        prop_assert!(
            named,
            "width {} bits was not surfaced as a named EngineError",
            candidate
        );
    }
}
