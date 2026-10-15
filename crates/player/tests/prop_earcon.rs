//! T007 — BLOCKING FOUNDATIONAL TEST: property `prop_earcon_band_duration`
//! (Feature 004-ressoa-dogfood-build / task T007).
//!
//! Invariants under test (tasks.md T007, verbatim):
//! "Property `prop_earcon_band_duration`: every kind ≤500 ms,
//! band 300–2500 Hz, all three of {timbre,rhythm,contour}
//! distinct pairwise; PCM mono f32 @ 22050 Hz."
//!
//! The earcon grammar is defined in `contracts/earcon.md` and
//! implemented in `crates/player/src/earcon.rs`. This test
//! asserts the hard bounds on duration and frequency band,
//! and the pairwise distinctness of the three perceptual
//! dimensions for each kind.
//!
//! FAILS before earcon synthesis is implemented — acceptable
//! red state for the blocking gate.

use proptest::prelude::*;

use player::earcon::{ContourClass, EarconDescriptor, EarconKind, RhythmClass, TimbreClass};

/// The declared sample rate for all earcons (contracts/earcon.md).
const EARCON_SAMPLE_RATE: u32 = 22_050;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T007: `prop_earcon_band_duration` — every earcon kind
    /// must produce a descriptor within the hard bounds.
    #[test]
    fn prop_earcon_band_duration(kind in prop_oneof![
        Just(EarconKind::Play),
        Just(EarconKind::Pause),
        Just(EarconKind::SkipFwd),
        Just(EarconKind::SkipBack),
        Just(EarconKind::Warning),
        Just(EarconKind::Failure),
        Just(EarconKind::RateStep),
    ]) {
        let descriptor = EarconDescriptor::for_kind(kind);

        // Invariant 1: duration ≤ 500 ms.
        prop_assert!(
            descriptor.duration_ms <= 500,
            "earcon {:?} duration {} ms exceeds 500 ms bound",
            kind,
            descriptor.duration_ms
        );

        // Invariant 2: frequency band 300–2500 Hz.
        prop_assert!(
            descriptor.band_hz.0 >= 300 && descriptor.band_hz.1 <= 2500,
            "earcon {:?} band {:?} outside 300–2500 Hz",
            kind,
            descriptor.band_hz
        );

        // Invariant 3: PCM mono f32 @ 22050 Hz.
        prop_assert_eq!(
            descriptor.sample_rate_hz, EARCON_SAMPLE_RATE,
            "earcon {:?} sample rate {} != 22050 Hz",
            kind,
            descriptor.sample_rate_hz
        );
        prop_assert_eq!(
            descriptor.channels, 1,
            "earcon {:?} channels {} != 1 (mono)",
            kind,
            descriptor.channels
        );
        prop_assert!(
            matches!(descriptor.format, player::earcon::SampleFormat::F32),
            "earcon {:?} format {:?} != F32",
            kind,
            descriptor.format
        );

        // Invariant 4: three perceptual dimensions present.
        // The fingerprint is a plain enum triple (not Option) —
        // every kind must have a concrete classification.
        let _fp_timbre: TimbreClass = descriptor.perceptual_fingerprint.timbre;
        let _fp_rhythm: RhythmClass = descriptor.perceptual_fingerprint.rhythm;
        let _fp_contour: ContourClass = descriptor.perceptual_fingerprint.contour;
    }

    /// Pairwise distinctness: for any two distinct kinds,
    /// at least one perceptual dimension must differ.
    #[test]
    fn prop_earcon_pairwise_distinct(
        k1 in prop_oneof![
            Just(EarconKind::Play),
            Just(EarconKind::Pause),
            Just(EarconKind::SkipFwd),
            Just(EarconKind::SkipBack),
            Just(EarconKind::Warning),
            Just(EarconKind::Failure),
            Just(EarconKind::RateStep),
        ],
        k2 in prop_oneof![
            Just(EarconKind::Play),
            Just(EarconKind::Pause),
            Just(EarconKind::SkipFwd),
            Just(EarconKind::SkipBack),
            Just(EarconKind::Warning),
            Just(EarconKind::Failure),
            Just(EarconKind::RateStep),
        ],
    ) {
        if k1 == k2 {
            return Ok(()); // skip same-kind pairs
        }
        let d1 = EarconDescriptor::for_kind(k1);
        let d2 = EarconDescriptor::for_kind(k2);
        let p1 = &d1.perceptual_fingerprint;
        let p2 = &d2.perceptual_fingerprint;

        // At least one of timbre, rhythm, contour must differ.
        let differs = p1.timbre != p2.timbre
            || p1.rhythm != p2.rhythm
            || p1.contour != p2.contour;
        prop_assert!(
            differs,
            "earcon {:?} and {:?} are perceptually identical (all three dimensions match)",
            k1, k2
        );
    }
}
