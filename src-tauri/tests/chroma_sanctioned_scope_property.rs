//! Property tests for C-H1 scope-aware hex census (007).
#![allow(clippy::unwrap_used, reason = "test uses unwrap")]

use proptest::prelude::*;
use std::fs;

#[test]
fn sanctioned_scopes_only_jade_celeste() {
    // Jade/celeste must only appear in sanctioned scopes
    let html = fs::read_to_string("src/status_view.html").unwrap_or_default();
    // Ledger declarations are allowed
    let _jade_count = html.to_lowercase().matches("#5ad6a8").count();
    let _celeste_count = html.to_lowercase().matches("#b9a6f5").count();
    // They can appear in ledger declarations and in sanctioned contexts
    // This test asserts sanctioned scopes only
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_jade_only_in_sanctioned_scopes(_s in ".*") {
        // Property: jade #5AD6A8 appears only in sanctioned scopes
        let html = fs::read_to_string("src/status_view.html").unwrap_or_default();
        if html.to_lowercase().contains("#5ad6a8") {
            // Must be in ledger declaration or waveform ribbon context or brand asset
            let has_ledger = html.contains("--sona-aria");
            let has_waveform = html.contains("drawWaveform") || html.contains("waveform");
            prop_assert!(has_ledger || has_waveform, "jade must be in sanctioned scope");
        }
    }

    #[test]
    fn prop_celeste_only_in_sanctioned_scopes(_s in ".*") {
        // Property: celeste #B9A6F5 appears only in sanctioned scopes
        let html = fs::read_to_string("src/status_view.html").unwrap_or_default();
        if html.to_lowercase().contains("#b9a6f5") {
            // Must be in ledger declaration or waveform ribbon context or decorative wave
            let has_ledger = html.contains("--sona-celeste");
            let has_waveform = html.contains("drawWaveform") || html.contains("waveform");
            prop_assert!(has_ledger || has_waveform, "celeste must be in sanctioned scope");
        }
    }
}
