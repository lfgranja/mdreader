//! RED — Finding A4: SC block duplication (spec.md has repeated SC contract clauses).
//! TDD: test fails until duplication removed / consolidated.
//! Phase: 0 (scaffold-first). Verification: T031.

#[test]
fn sc_block_no_duplicate_sc_clause() {
    // Evidence: grep over spec.md shows SC-001 clause repeated near lines 155+137.
    // Expected after remediation: single authoritative SC-001 definition with host+device legs.
    let spec = include_str!("../../specs/005-adopt-design-sona/spec.md");
    // Heuristic: count occurrences of full SC-001 header paragraph; must be <= 1.
    let occurrences = spec.matches("SC-001").count();
    // Currently >1; RED fails.
    assert!(
        occurrences <= 1,
        "SC-001 duplicated in spec.md (found {} occurrences); consolidate to single clause before Phase 1",
        occurrences
    );
}
