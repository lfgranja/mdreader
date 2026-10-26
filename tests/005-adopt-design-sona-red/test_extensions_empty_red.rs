//! RED — Finding A8: extensions.yml empty (no registered extensions / prerequisites documented).
//! TDD: fails until file has content or explicit "none registered" declaration.

#[test]
fn extensions_yml_not_empty_or_documented() {
    let path = ".specify/extensions/extensions.yml";
    let content = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        !content.trim().is_empty(),
        "extensions.yml is empty / missing; document prerequisites (host/device/gate) or declare 'none registered' per constitution"
    );
}
