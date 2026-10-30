//! Test contract.

#[test]
fn numeric_rate_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(html.contains("id=\"rateValue\""), "rateValue present");
    assert!(
        html.contains("speed-control") || html.contains("speed"),
        "in speed-control area"
    );
}
