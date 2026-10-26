// RED — latency oracle ≤100ms (first-consumed sample bound)
// Contract: specs/005-adopt-design-sona/spec.md FR-006/FR-007 + earcon latency oracle
use std::time::{Instant, Duration};

fn first_sample_latency_ms() -> u64 {
    let start = Instant::now();
    // RED: implementation missing; simulated >100ms path
    std::thread::sleep(Duration::from_millis(150));
    start.elapsed().as_millis() as u64
}

#[test]
fn latency_oracle_first_consumed_bound_100ms() {
    let lat = first_sample_latency_ms();
    assert!(lat <= 100, "latency oracle FAIL: first-consumed sample {} ms > 100 ms bound", lat);
}
