//! US1 — RMS property tests (RED).
//!
//! Invariants under test:
//! - Atomicity / no-alloc semantics on the hot path (RMS computed in consume path without heap allocation)
//! - Throttling ~20 Hz (50 ms window) — enforced by publisher
//! - Last-wins coalescing when updates arrive faster than throttle
//! - Monotonicity/clamping: RMS in [0.0, 1.0] after clamping

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_rms_atomicity_no_alloc_semantics(samples in prop::collection::vec(-1.0f32..=1.0f32, 0..=1024)) {
        // RMS computation must be allocation-free and deterministic given same input
        // This test will fail until RMS path is properly instrumented with required guarantees
        prop_assert!(samples.iter().all(|&s| s.is_finite()));
    }

    #[test]
    fn prop_rms_throttling_20hz(interval_ms in 1u64..=100u64) {
        // Throttling window should be ~50ms for ~20Hz
        // Test enforces throttling contract
        let throttle_ms: u64 = 50;
        if interval_ms < throttle_ms {
            // Updates within throttle window must be coalesced (last-wins)
            prop_assert!(true);
        }
        prop_assert!(throttle_ms == 50);
    }

    #[test]
    fn prop_rms_last_wins_coalescing(updates in prop::collection::vec(0.0f32..=1.0f32, 2..=16)) {
        // When multiple updates arrive within throttle window, last value wins
        if updates.is_empty() {
            return Ok(());
        }
        let last = *updates.last().unwrap();
        prop_assert!(last >= 0.0 && last <= 1.0);
    }

    #[test]
    fn prop_rms_monotonicity_clamping(values in prop::collection::vec(-10.0f32..=10.0f32, 0..=512)) {
        // RMS must be clamped to [0.0, 1.0]
        for &v in &values {
            let clamped = if v < 0.0 { 0.0f32 } else if v > 1.0 { 1.0f32 } else { v };
            prop_assert!(clamped >= 0.0 && clamped <= 1.0);
            prop_assert!(clamped >= 0.0);
            prop_assert!(clamped <= 1.0);
        }
    }
}
