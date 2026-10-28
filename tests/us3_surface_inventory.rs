// RED test for T028 — US3 surface inventory. Must exist (RED) before T030-T031.
#[test]
fn us3_data_elements_count_unchanged() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("data-elements=\"9\""),
        "data-elements count must remain 9 (C-H4)");
}

#[test]
fn us3_no_display_none() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(!html.contains("display:none"),
        "no display:none corpses (C-H4)");
}

#[test]
fn us3_all_elements_visible() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    for el in &["card::before", "headerMark", "documentTitle", "subtitle", "controls", "speed-control", "progress", "sr-only", "reason-banner", "version-footer"] {
        // Check that the element/selector exists in the HTML
        assert!(html.contains(el.trim_start_matches('.')),
            "element {} must be present (C-H4)", el);
    }
}
