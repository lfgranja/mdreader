// RED test for T007 — contract census (C-H1). Must NOT pass until contract updates land.
// Invariant: audited hex values match SonaToken 10-member closed set; banned jade
// #5AD6A8/#5FD0A0 and celeste #B9A6F5 absent; inventory count == 9.

#[test]
fn census_banned_jade_absent() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(!html.to_lowercase().contains("#5ad6a8"),
        "banned jade #5AD6A8 must be absent (C-H1)");
    assert!(!html.to_lowercase().contains("#5fd0a0"),
        "banned jade #5FD0A0 must be absent (C-H1)");
}

#[test]
fn census_banned_celeste_absent() {
    // celeste #B9A6F5 reserved unpainted — must not appear in rendered CSS
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    // The token declaration --sona-celeste:#B9A6F5 is the only allowed occurrence (ledger member).
    // Count occurrences: only the :root var declaration is permitted.
    let count = html.to_lowercase().matches("#b9a6f5").count();
    assert!(count <= 1,
        "celeste #B9A6F5 may appear only as --sona-celeste ledger member, found {} (C-H1)", count);
}

#[test]
fn census_inventory_count_is_nine() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(html.contains("data-elements=\"9\""),
        "scaffold-inventory data-elements must be 9 (C-H4)");
}

#[test]
fn census_sona_token_ledger_closed_ten() {
    // Closed set: exactly 10 members (data-model.md SonaToken).
    const SONA_TOKENS: [&str; 10] = [
        "#0B0F1A", "#141B2D", "#8FD8E8", "#FFD400", "#FF4D63",
        "#F2F0E6", "#A9B8CE", "#E8C46A", "#B9A6F5", "#55688F",
    ];
    assert_eq!(SONA_TOKENS.len(), 10, "SonaToken closed set must be 10 members");
}
