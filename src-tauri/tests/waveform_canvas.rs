//! Test contract.

#[test]
fn waveform_canvas_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(
        html.contains("<canvas") && html.contains("id=\"waveform\""),
        "canvas present"
    );
    assert!(html.contains("aria-hidden=\"true\""), "aria-hidden");
    assert!(html.contains("drawWaveform"), "drawWaveform present");
}
