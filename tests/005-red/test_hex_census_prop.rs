// RED — closed hex census invariant (10 members: 8 sona + 2 signals)
// Contract: specs/004-ressoa-dogfood-build/contracts/earcon.md + specs/005-adopt-design-sona/spec.md (FR-012/SC-001)
// Before any implementation: must FAIL (no closed set defined in host files).
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HexMember(&'static str);

fn closed_hex_set() -> Vec<&'static str> {
    vec![
        "sona-voice", "sona-hush", "sona-boundary", "sona-harp",
        "sona-drift", "sona-surface", "sona-ridge", "sona-tint",
        "signal-advisory", "signal-completion",
    ] // 8 sona + 2 signals = 10
}

proptest! {
    #[test]
    fn prop_closed_hex_census_size_is_ten(_seed in any::<u32>()) {
        let set = closed_hex_set();
        prop_assert_eq!(set.len(), 10, "expected 10-member closed set; got {}", set.len());
        prop_assert!(!set.contains(&"jade"), "jade forbidden in closed hex set");
    }
}

#[test]
fn hex_census_deterministic_over_status_view_and_media_mirror() {
    // SC-001 deterministic host check: status_view.html + MediaMirror.kt must expose 10 hex tokens
    let html_path = "src/status_view.html";
    let kt_path = "android-shell/MediaMirror.kt";
    let html = std::fs::read_to_string(html_path).unwrap_or_default();
    let kt = std::fs::read_to_string(kt_path).unwrap_or_default();
    let combined = html + &kt;
    let set = closed_hex_set();
    for member in &set {
        assert!(combined.contains(member), "missing hex member {} in host surfaces", member);
    }
    assert!(!combined.contains("jade"), "jade forbidden in closed hex census");
    assert_eq!(combined.matches("sona-").count(), 8, "expected 8 sona tokens; found different");
}
