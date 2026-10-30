// RED test for T017 — US1 a11y inventory. Must exist (RED) before T018-T021.
#[test]
fn us1_progress_aria_valuetext_positional() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("aria-valuetext"),
        "#progress must have aria-valuetext (positional, not numeric rate) (SC-002)");
}

#[test]
fn us1_subtitle_sr_only_sem_titulo() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("sr-only") && html.contains("Sem título"),
        ".subtitle .sr-only must announce 'Sem título' when empty (SC-002)");
}

#[test]
fn us1_no_alert_assertive_misuse() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default().to_lowercase();
    // No assertive role on interactive elements; reason-banner uses role="status" (polite).
    assert!(!html.contains("role=\"alert\""),
        "no role=alert misuse on interactive elements (SC-002)");
    assert!(!html.contains("aria-live=\"assertive\""),
        "no aria-live=assertive misuse (SC-002)");
}

#[test]
fn us1_focus_ring_visible() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("--sona-etwahl") && html.contains(":focus-visible"),
        "focus ring --sona-etwahl must be visible via :focus-visible (SC-002)");
}
