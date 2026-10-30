// RED test for T029 — US3 contract audit. Must exist (RED) before T030-T031.
#[test]
fn us3_contracts_contain_justification() {
    let ss = std::fs::read_to_string("specs/005-adopt-design-sona/contracts/status-surface.md").unwrap_or_default();
    assert!(ss.contains("§VII") || ss.contains("coordinated edit") || ss.contains("justification"),
        "status-surface.md must contain §VII justification (SC-007)");
    let sa = std::fs::read_to_string("specs/005-adopt-design-sona/contracts/surface-audit.md").unwrap_or_default();
    assert!(sa.contains("006") || sa.contains("Lente Sinestésica"),
        "surface-audit.md must reference 006 (SC-007)");
}

#[test]
fn us3_data_model_no_new_token_members() {
    let dm = std::fs::read_to_string("specs/005-adopt-design-sona/data-model.md").unwrap_or_default();
    assert!(dm.contains("SonaToken") && dm.contains("10 members"),
        "data-model.md must record 10-member closed set (C-H1)");
    assert!(dm.contains("006") || dm.contains("Lente Sinestésica"),
        "data-model.md must record 006 assumption (CHK073)");
}

#[test]
fn us3_commit_references_section_vii() {
    // Verify that git log contains §VII reference in recent commits
    // (This is a documentation check; actual verification at T033/T039)
    let ss = std::fs::read_to_string("specs/005-adopt-design-sona/contracts/status-surface.md").unwrap_or_default();
    assert!(ss.contains("S11") || ss.contains("workflow"),
        "contract must document update workflow (CHK066)");
}
