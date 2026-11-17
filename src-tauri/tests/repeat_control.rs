//! Test contract.

#[test]
fn repeat_control_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(html.contains("id=\"btnRepeat\""), "repeat button present");
    assert!(html.contains("Repetir seção"), "aria-label");
}
