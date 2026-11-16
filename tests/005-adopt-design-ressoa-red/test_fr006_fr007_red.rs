//! RED — Finding A5: FR-006 / FR-007 missing RED (no pre/post conditions, no contract reference).
//! TDD: fails until each FR has RED + links to specs/004-ressoa-dogfood-build/contracts/surface.md / contracts/notification.md.

#[test]
fn fr006_fr007_have_red_and_contract_ref() {
    let spec = include_str!("../../specs/005-adopt-design-sona/spec.md");
    // Check both FR-006 and FR-007 present (they are — lines 137/138).
    assert!(spec.contains("FR-006"), "FR-006 missing");
    assert!(spec.contains("FR-007"), "FR-007 missing");

    // RED requires: MUST (precondition / invariant) + contract reference.
    // Heuristic: near each FR line, expect "MUST" and a contract file name.
    let near_006 = spec.lines()
        .skip_while(|l| !l.contains("FR-006")).take(5).collect::<Vec<_>>();
    let near_007 = spec.lines()
        .skip_while(|l| !l.contains("FR-007")).take(5).collect::<Vec<_>>();

    let has_must_006 = near_006.iter().any(|l| l.contains("MUST"));
    let has_contract_006 = near_006.iter().any(|l| l.contains("contracts/"));
    let has_must_007 = near_007.iter().any(|l| l.contains("MUST"));
    let has_contract_007 = near_007.iter().any(|l| l.contains("contracts/"));

    assert!(
        has_must_006 && has_contract_006,
        "FR-006 missing RED / contract ref (must={}, contract={})",
        has_must_006, has_contract_006
    );
    assert!(
        has_must_007 && has_contract_007,
        "FR-007 missing RED / contract ref (must={}, contract={})",
        has_must_007, has_contract_007
    );
}
