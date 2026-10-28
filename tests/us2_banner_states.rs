// RED test for T023 — US2 banner states. Must exist (RED) before T024/T025.
#[test]
fn us2_banner_failure_static() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains(".reason-banner.failure") && html.contains("animation: none"),
        ".reason-banner.failure must be static (no keyframe) (SC-005)");
}

#[test]
fn us2_banner_warning_ttl() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("#FFD400") || html.contains("signal-caution"),
        ".reason-banner.warning must use #signal-caution #FFD400 (SC-005)");
    assert!(html.contains("3000") || html.contains("3 s"),
        "warning banner TTL must be ≤3s (SC-005)");
}

#[test]
fn us2_no_alert_assertive() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default().to_lowercase();
    assert!(!html.contains("role=\"alert\""),
        "no alert role on interactive elements (SC-005)");
    assert!(!html.contains("aria-live=\"assertive\""),
        "no assertive aria-live misuse (SC-005)");
}

#[test]
fn us2_banner_no_autodismiss_failure() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // failure banner must not auto-disappear (only warning has TTL)
    // Check that failure class does not have setTimeout
    let failure_section = html.split(".reason-banner.failure").nth(1).unwrap_or("");
    assert!(!failure_section.contains("setTimeout"),
        "failure banner must not auto-disappear (SC-005)");
}
