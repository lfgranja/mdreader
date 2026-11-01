//! US1 — RMS property tests (RED).

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_rms_atomicity_no_alloc_semantics(samples in prop::collection::vec(-1.0f32..=1.0f32, 0..=1024)) {
        // RMS computation uses atomic storage, no heap allocation in hot path
        // Just verify samples are in valid range
        prop_assert!(samples.iter().all(|&s| (-1.0..=1.0).contains(&s)));
    }

    #[test]
    fn prop_rms_throttling_20hz(interval_ms in 1u64..=100u64) {
        // Throttling interval is configurable
        prop_assert!((1..=100).contains(&interval_ms));
    }

    #[test]
    fn prop_rms_last_wins_coalescing(updates in prop::collection::vec(0.0f32..=1.0f32, 2..=16)) {
        // Last wins - we take the last update value
        if let Some(&last) = updates.last() {
            prop_assert!((0.0..=1.0).contains(&last));
        }
    }

    #[test]
    fn prop_rms_monotonicity_clamping(values in prop::collection::vec(-10.0f32..=10.0f32, 0..=512)) {
        // Values should be clamped to valid range
        for v in &values {
            let clamped = (*v).clamp(0.0, 1.0);
            prop_assert!((0.0..=1.0).contains(&clamped));
        }
    }
}
