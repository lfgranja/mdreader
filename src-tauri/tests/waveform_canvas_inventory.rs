//! Test contract.

#[test]
fn canvas_present_aria_hidden_and_progress_retained() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(html.contains("<canvas"), "canvas element present");
    assert!(html.contains("id=\"waveform\""), "waveform canvas present");
    // aria-hidden
    assert!(
        html.contains("aria-hidden=\"true\"") || html.contains("aria-hidden='true'"),
        "canvas aria-hidden"
    );
    // progress retained
    assert!(html.contains("id=\"progress\""), "progress retained");
}
