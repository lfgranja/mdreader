//! Test contract.

#[test]
fn overflow_and_empty_title() {
    let html = std::fs::read_to_string("src/status_view.html").unwrap_or_default();
    assert!(
        html.contains("Sem título") || html.contains("Sem t\u{00ed}tulo"),
        "empty title present"
    );
    assert!(
        html.contains("—") || html.contains("&mdash;"),
        "em dash present"
    );
}
