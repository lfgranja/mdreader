// RED test — earcon burst / SHIPPED 004 grammar (T012/T013).
// Must FAIL before GREEN; stub enqueue_earcon always false (see src/earcon/mod.rs).

use std::time::{Duration, Instant};

#[test]
fn prop_earcon_burst_latest_wins() {
    // Burst: enqueue multiple; only latest survives (truncate / substitute).
    let mut results = Vec::new();
    for _ in 0..3 {
        results.push(earcon::enqueue_earcon(earcon::Confirmation::Adagio));
    }
    // RED: stub returns false for all → failure.
    assert!(results.iter().all(|r| *r), "burst latest-wins: all emissions succeed (GREEN T012)");
}

#[test]
fn test_earcon_latency_oracle() {
    let ms = earcon::first_consumed_sample_ms();
    assert!(
        ms <= 100,
        "first-consumed-sample latency {}ms exceeds 100ms bound (FR-010 / SHIPPED-004)",
        ms
    );
}
