//! Test contract.

#[test]
fn glassmorphism_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(
        html.contains("backdrop-filter") && html.contains("blur(24px)"),
        "backdrop-filter blur(24px)"
    );
    assert!(
        html.contains("rgba(20,27,45,0.65)"),
        "overlay rgba(20,27,45,0.65)"
    );
    assert!(
        html.contains("border") && html.contains("doppel") || html.contains("Doppel"),
        "Doppelrand borders present"
    );
    assert!(html.contains("--sona-stage"), "solid fallback present");
}
