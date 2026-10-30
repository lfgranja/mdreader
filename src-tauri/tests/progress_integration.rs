//! Test contract.

#[test]
fn progress_integration_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(
        html.contains("role=\"progressbar\"") || html.contains("role='progressbar'"),
        "progressbar role"
    );
    assert!(
        html.contains("aria-valuenow") || html.contains("progress-fill"),
        "valuenow or fill"
    );
    assert!(
        html.contains("liveRegion")
            || html.contains("aria-live")
            || html.contains("role=\"status\""),
        "live region/status"
    );
}
