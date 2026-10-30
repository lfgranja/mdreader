//! Test contract.

#[test]
fn waveform_performance_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(html.contains("visibilitychange"), "visibilitychange");
    assert!(
        html.contains("matchMedia") && html.contains("prefers-reduced-motion"),
        "reduced-motion"
    );
    assert!(
        html.contains("devicePixelRatio") || html.contains("DPR"),
        "DPR"
    );
}
