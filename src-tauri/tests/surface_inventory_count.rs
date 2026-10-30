//! Test contract.

#[test]
fn surface_inventory_count_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(
        html.contains("data-elements=\"10\"") || html.contains("data-elements='10'"),
        "data-elements=10"
    );
}
