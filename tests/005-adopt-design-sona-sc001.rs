// SC-001: papel de foco - property-based tests
use proptest::prelude::*;
use std::fs;
use std::path::Path;

fn read_status_view() -> String {
    // Try different possible locations
    if let Ok(s) = fs::read_to_string("src-tauri/src/status_view.html") {
        return s;
    }
    if let Ok(s) = fs::read_to_string("../src-tauri/src/status_view.html") {
        return s;
    }
    String::new()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    
    #[test]
    fn prop_sc001_focus_ring_only_uses_sona_etwahl(_seed in any::<u32>()) {
        let html = read_status_view();
        if html.is_empty() {
            return Ok(());
        }
        // Find :focus-visible blocks
        let mut violations = Vec::new();
        let mut idx = 0;
        while let Some(pos) = html[idx..].find(":focus-visible") {
            let start = idx + pos;
            // Find the block
            if let Some(brace) = html[start..].find('{') {
                let block_start = start + brace;
                if let Some(end_brace) = html[block_start..].find('}') {
                    let block = &html[block_start..block_start + end_brace + 1];
                    // Extract hex colors
                    let bytes = block.as_bytes();
                    let mut i = 0;
                    while i < bytes.len() {
                        if bytes[i] == b'#' && i + 7 <= bytes.len() {
                            // Check if 6 hex chars follow
                            let mut is_hex = true;
                            for j in 1..7 {
                                if !bytes[i+j].is_ascii_hexdigit() {
                                    is_hex = false;
                                    break;
                                }
                            }
                            if is_hex {
                                let hex = &block[i..i+7];
                                if hex != "#8FD8E8" {
                                    violations.push(hex.to_string());
                                }
                            }
                            i += 7;
                            continue;
                        }
                        i += 1;
                    }
                }
            }
            idx = start + 1;
        }
        prop_assert!(violations.is_empty(), "focus-visible must only use #8FD8E8 (--sona-etwahl), found: {:?}", violations);
    }
}

#[test]
fn sc001_focus_visible_rule_exists() {
    let html = read_status_view();
    assert!(!html.is_empty(), "status_view.html must exist");
    assert!(html.contains(":focus-visible"), "SC-001: :focus-visible rule must exist for keyboard focus");
}

#[test]
fn sc001_focus_uses_etwahl_token() {
    let html = read_status_view();
    assert!(
        html.contains("#8FD8E8") || html.contains("var(--sona-etwahl)"),
        "SC-001: focus ring must use --sona-etwahl (#8FD8E8)"
    );
}
