//! Stand-in engine B (T017b): the cloud-like fixture.
//!
//! One of the two stand-in engines that honour the
//! engine-handoff contract (contracts/engine-handoff.md,
//! "Conformance") so the mid-document failover and
//! forced-double-failure drills (SC-051) have working
//! fixtures before any real adapter exists. Stand-in B
//! plays the cloud-like role in the drill; stand-in A
//! (`crate::standin_a`) plays the offline-like role.
//!
//! A stand-in is a `test_double` (the 2026-10-05
//! clarification): it is never selectable while any
//! non-`test_double` adapter is registered, and it
//! exists only as a test fixture — no fallback or
//! selection logic lives here (that is T075/T076).

use crate::closed::{EngineTier, PlaybackRate};
use crate::format::DeclaredFormat;

/// Stand-in engine B: the cloud-like engine of the
/// SC-051 failover drill.
///
/// The stand-in honours the same handoff contract
/// as stand-in A: it delivers mono `f32` PCM at
/// the declared 22050 Hz on every path, including
/// every rung of the rate ladder — a rate change
/// is a pitch-preserving time-stretch inside the
/// engine (FR-043, SC-034) and never alters the
/// delivered format.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandinB;

impl StandinB {
    /// The engine id: `standin_b`. One of the two
    /// non-registry ids permitted by the 2026-10-05
    /// clarification, valid only with
    /// [`EngineTier::TestDouble`] and only as a
    /// stand-in fixture.
    pub const ID: &'static str = "standin_b";

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
    /// within ±3 cents at 2.0× — a deviation beyond
    /// that tolerance is an [`crate::closed::EngineError::RateMismatch`]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_and_tier_declare_the_test_double_contract() {
        assert_eq!(StandinB::ID, "standin_b");
        assert_eq!(StandinB::TIER, EngineTier::TestDouble);
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
            let mut engine = StandinB;
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
}
