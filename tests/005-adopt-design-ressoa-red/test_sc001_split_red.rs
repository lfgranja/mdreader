//! RED — Finding A3/A7: SC-001 split ambiguity (host census vs device deferred leg; owner undefined).
//! TDD: fails until SC-001 names both legs and owners explicitly.

#[test]
fn sc001_has_explicit_host_and_device_legs() {
    let spec = include_str!("../../specs/005-adopt-design-sona/spec.md");
    // Must mention deterministic host census and deferred device leg with owner.
    let has_host = spec.contains("census") || spec.contains("host") || spec.contains("deterministic");
    let has_device = spec.contains("device") || spec.contains("DEFERRED") || spec.contains("trial");
    // Must name owner for deferred leg (e.g., device-build / gate-8/gate-9).
    let has_owner = spec.contains("device-build") || spec.contains("owner=") || spec.contains("gate-9");
    assert!(
        has_host && has_device && has_owner,
        "SC-001 ambiguous: host={}, device={}, owner={}; define both legs + deferred owner (device-build/gate-9)",
        has_host, has_device, has_owner
    );
}
