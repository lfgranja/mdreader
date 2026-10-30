// RED test for T016 — US1 reduced-motion. Must exist (RED) before T018-T021.
#[test]
fn us1_reduced_motion_wave_frozen() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains(".card::before") && html.contains("animation:none !important"),
        "decorative wave must freeze under reduce (SC-004)");
}

#[test]
fn us1_reduced_motion_progress_static() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("#progress-fill") && html.contains("animation:none !important"),
        "progress #progress-fill must be static under reduce (SC-004)");
}

#[test]
fn us1_reduced_motion_banner_static() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains(".reason-banner.warning") && html.contains("animation:none !important"),
        ".reason-banner.warning must be static under reduce (SC-004)");
}

#[test]
fn us1_no_new_infinite_loops() {
    // The indeterminate progress animation uses 'infinite' (line 425) —
    // this is the ONE allowed continuous motion (pill). No NEW infinite
    // loops may be added by 006 (decorative wave must not loop).
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // Count 'infinite' occurrences — should be exactly 1 (indeterminate progress)
    let count = html.matches("infinite").count();
    assert!(count <= 1,
        "at most 1 'infinite' (indeterminate progress) allowed; found {} (SC-004/S5)", count);
}

#[test]
fn us1_decorative_aria_hidden() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("aria-hidden=\"true\""),
        "decorative wave must be aria-hidden=\"true\" (C-H5)");
}
