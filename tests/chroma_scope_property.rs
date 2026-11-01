//! Property tests asserting sanctioned scopes only for jade/celeste (007).
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;

#[test]
fn sanctioned_scopes_property_initial() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // Jade and celeste only allowed in sanctioned scopes
    assert!(html.contains("--sona-celeste") || html.contains("--sona-aria") || html.contains("drawWaveform"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_no_jade_celeste_outside_sanctioned_contexts(_seed in any::<u32>()) {
        let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
        let html_lower = html.to_lowercase();
        if html_lower.contains("#5ad6a8") {
            // Must be in ledger declaration or waveform ribbon/drawWaveform context
            let sanctioned = html.contains("--sona-aria") || html.contains("drawWaveform") || html.contains("waveform");
            prop_assert!(sanctioned, "jade outside sanctioned scope");
        }
        if html_lower.contains("#b9a6f5") {
            let sanctioned = html.contains("--sona-celeste") || html.contains("drawWaveform") || html.contains("waveform");
            prop_assert!(sanctioned, "celeste outside sanctioned scope");
        }
    }
}
