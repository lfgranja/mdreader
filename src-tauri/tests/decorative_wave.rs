//! Test contract.

#[test]
fn decorative_wave_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(html.contains("wave-subtle"), "wave-subtle present");
    assert!(html.contains(".card::before"), "decorative wave present");
}
