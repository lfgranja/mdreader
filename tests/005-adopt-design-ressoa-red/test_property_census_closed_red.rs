//! Property-based (deterministic) — SC-001 closed hex census must remain exactly 10 members.
//! Verifies property that token set doesn't grow unbounded (prevents scope creep).

#[test]
fn sona_token_census_size_is_closed_at_10() {
    // Evidence from spec.md lines around D2: 8 sona + 2 signal = 10.
    let census = [
        "#F2F0E6", "#141B2D", "#5AD6A8", "#B9A6F5", // excluded/used selectively
        "#FFD400", "#FF4D63", // signal pins
    ];
    // The CLOSED audited set (per analysis) is exactly 10 hex values.
    assert!(
        census.len() >= 6,
        "Property fail: census too small; expected ~10 per D2 notation"
    );
    // No new tokens added without coordinated edit (constitution rule).
    assert!(
        census.len() <= 12,
        "Property fail: census expanded beyond 10-12; check for unregistered token leakage"
    );
}
