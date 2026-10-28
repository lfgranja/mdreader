// RED test for T008 — reduced-motion freeze (SC-004). Must NOT pass until T013 extends guard.
#[test]
fn reduced_motion_progress_fill_frozen() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("animation:none !important") && html.contains("#progress-fill"),
        "#progress-fill must freeze under @media(prefers-reduced-motion:reduce) (SC-004)"
    );
}

#[test]
fn reduced_motion_banner_warning_frozen() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains(".reason-banner.warning") && html.contains("animation:none !important"),
        ".reason-banner.warning must freeze under reduce (SC-004)"
    );
}

#[test]
fn reduced_motion_card_before_frozen() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    // T013: .card::before freeze (animation:none + transform:none) must be added.
    assert!(
        html.contains(".card::before") && html.contains("transform:none !important"),
        ".card::before must freeze animation+transform under reduce (T013)"
    );
}

#[test]
fn no_infinite_loops_in_css() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        !html.to_lowercase().contains("infinite"),
        "no infinite loops allowed in status_view.html CSS (S5)"
    );
}
