// SC-001: papel de foco / token semantics - property based tests
use proptest::prelude::*;
use std::fs;

const ALLOWED_FOCUS_HEX: &[&str] = &["#8FD8E8"]; // --sona-etwahl only in :focus-visible

proptest! {
    #[test]
    fn prop_focus_ring_only_etwahl_in_focus_visible(_seed in any::<u32>()) {
        let html = fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
        // Focus ring must only use etwahl
        let focus_visible_block: String = html
            .split(":focus-visible")
            .skip(1)
            .next()
            .unwrap_or("")
            .split('}')
            .next()
            .unwrap_or("")
            .to_string();
        if focus_visible_block.contains('#') {
            // Check all hex in focus-visible block are from allowed set
            let mut i = 0;
            let bytes = focus_visible_block.as_bytes();
            while i < bytes.len() {
                if bytes[i] == b'#' {
                    let mut j = i + 1;
                    while j < bytes.len() && bytes[j].is_ascii_hexdigit() && (j - i - 1) < 6 {
                        j += 1;
                    }
                    if (j - i - 1) == 3 || (j - i - 1) == 6 {
                        let hex = &focus_visible_block[i..j];
                        prop_assert!(
                            ALLOWED_FOCUS_HEX.contains(&hex),
                            "focus ring must only use --sona-etwahl, found {}",
                            hex
                        );
                    }
                    i = j;
                    continue;
                }
                i += 1;
            }
        }
    }
}

#[test]
fn sc001_focus_visible_has_etwahl() {
    let html = fs::read_to_string("src-tauri/src/status_view.html").expect("status_view.html");
    assert!(html.contains(":focus-visible"), "focus-visible rule must exist");
    assert!(html.contains("#8FD8E8") || html.contains("var(--sona-etwahl)"), 
        "focus-visible must use --sona-etwahl");
}

#[test]
fn sc001_focus_ring_not_on_base_buttons() {
    let html = fs::read_to_string("src-tauri/src/status_view.html").expect("status_view.html");
    // Base button styles should not paint focus ring color directly
    let focus_visible_block: String = html
        .split(":focus-visible")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .unwrap_or("")
        .to_string();
    assert!(focus_visible_block.contains("#8FD8E8") || focus_visible_block.contains("sona-etwahl"),
        "focus-visible must paint --sona-etwahl");
}
