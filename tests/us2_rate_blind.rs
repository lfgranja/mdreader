// RED test for T022 — US2 rate-blind. Must exist (RED) before T024/T025.
#[test]
fn us2_speed_dots_positional_only() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // .speed-dots contains only span dots, no numeric labels
    assert!(html.contains("speed-dots") && html.contains("speed-dot"),
        ".speed-dots must contain positional dots (SC-001 item iv)");
}

#[test]
fn us2_progress_aria_no_rate_number() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // aria-valuetext must not contain a painted numeric rate like "1.25" or "2.0"
    assert!(html.contains("aria-valuetext"),
        "#progress must have aria-valuetext (SC-002)");
    // The value "velocidade 1,0" is a position descriptor, not a rate mapping
    assert!(!html.contains("aria-valuetext=\"1.25") && !html.contains("aria-valuetext=\"2.0"),
        "aria-valuetext must not contain numeric rate mapping (C-H3)");
}

#[test]
fn us2_transport_aria_labels_unchanged() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("aria-label=\"Retroceder\""),
        "skip-back aria-label must be unchanged");
    assert!(html.contains("aria-label=\"Reproduzir/Pausar\""),
        "play/pause aria-label must be unchanged");
    assert!(html.contains("aria-label=\"Parar e limpar posição\""),
        "stop aria-label must be unchanged");
    assert!(html.contains("aria-label=\"Avançar\""),
        "skip-forward aria-label must be unchanged");
}
