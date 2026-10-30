// RED test for T009 — surface inventory / dead CSS (C-H4). Must NOT pass until audit.
#[test]
fn no_display_none_corpses() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        !html.contains("display:none"),
        "no dead display:none corpses allowed (C-H4)"
    );
}

#[test]
fn decorative_wave_aria_hidden() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("aria-hidden=\"true\""),
        "decorative wave must be aria-hidden=\"true\" (C-H5)"
    );
}

#[test]
fn progressbar_role_intact() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("role=\"progressbar\""),
        "#progress must retain role=\"progressbar\" (SC-002)"
    );
}

#[test]
fn inventory_count_unchanged() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("data-elements=\"9\""),
        "scaffold-inventory data-elements must stay 9 (C-H4)"
    );
}
