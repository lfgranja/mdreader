//! T007 / D2 / D5 — RED-phase property audit for C-H3 census + banned-color invariant.
//! Blocking: spec C-H3 definition and banned-color reconciliation (D2/D5) must
//! complete before this passes. Uses proptest (workspace 1.12.0) per D12.
//!
//! Invariant (to be defined in spec.md / checklists by D2):
//! C-H3 = audit of rate-blind + notification-purity in surface HTML.
//!   - rate-blind: no numeric rate text (`1.25x`, `2.0`, etc.) painted in `#progressLabel`.
//!   - notification-purity: notification accent-only, no body/text/rate/voice/mark.
//! Banned colors (reconciled by D5): `#5AD6A6` (jade), `#5FD0A0` (jade-variant), `#B9A6F5` (celeste) absent from hex census C-H1.
//! Property: for any valid StatusView HTML snapshot, hex values ⊆ allowed token set && banned set ∩ hex = ∅.

use proptest::prelude::*;

#[test]
fn test_c_h3_rate_blind_invariant() {
    // RED: fails until spec defines rate-blind assertion clearly.
    // After D2: verify no `1.25×` / `2.0×` / numeric rate pattern in rendered HTML.
    assert!(false, "RED: C-H3 rate-blind definition pending (D2)");
}

#[test]
fn test_banned_colors_absent_invariant() {
    // RED: fails until D5 reconciles `#5AD6A8` vs `#5FD0A0` in token audit.
    let banned: &[&str] = &["#5AD6A8", "#5FD0A0", "#B9A6F5"];
    // Property: for any census, banned ∩ actual = ∅.
    assert!(false, "RED: banned-color set unreconciled; D5 required");
}

proptest! {
    #[test]
    fn prop_c_h1_hex_census_invariant(hexes in prop::collection::vec("[0-9A-F]{6}", 1..20)) {
        // After D1 (SC-001 reformatted) + D5: assert hexes match SonaToken ledger.
        // RED: does not yet enforce specific allowed set (definition pending in spec).
        assert!(!hexes.is_empty(), "census non-empty invariant");
    }
}
