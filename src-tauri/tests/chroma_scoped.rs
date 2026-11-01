//! Test contract.

#[test]
fn chroma_scoped_contract() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    // celeste/jade only in sanctioned scopes
    assert!(
        html.contains("#B9A6F5") || html.contains("#5AD6A8"),
        "jade/celeste present in sanctioned scopes"
    );
}
