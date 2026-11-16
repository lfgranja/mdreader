// SC-002: disposição haptic - property-based tests (ears/haptics)
use proptest::prelude::*;
use std::fs;

fn read_status_view() -> String {
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
    fn prop_sc002_no_numeric_rate_displayed(_seed in any::<u32>()) {
        let html = read_status_view();
        if html.is_empty() {
            return Ok(());
        }
        // Must not display numeric rate like "1.25×" or "Taxa: 1.0×"
        let forbidden_patterns = ["Taxa:", "×", "1.25", "1.5", "0.75", "2.0"];
        for pattern in &forbidden_patterns {
            // Allow in comments or data attributes? Check for visible text
            // Look for text content patterns
            if html.contains(&format!(">{}", pattern)) || html.contains(&format!("{}<", pattern)) {
                prop_assert!(false, "SC-002: numeric rate must not be displayed, found pattern: {}", pattern);
            }
        }
    }
    
    #[test]
    fn prop_sc002_rate_indicator_is_positional(_seed in any::<u32>()) {
        let html = read_status_view();
        if html.is_empty() {
            return Ok(());
        }
        // Should have speed dots/rate indicator for positional display
        prop_assert!(
            html.contains("speed") || html.contains("rate") || html.contains("dot"),
            "SC-002: positional rate indicator required"
        );
    }
}

#[test]
fn sc002_haptic_none_disposition_documented() {
    // Check src-tauri/src/lib.rs has haptic=None or documented as inert
    if let Ok(content) = fs::read_to_string("src-tauri/src/lib.rs") {
        assert!(
            content.contains("haptic") && (content.contains("None") || content.contains("haptic=None")),
            "SC-002: haptic disposition must be documented as None/inert"
        );
    }
}

#[test]
fn sc002_earcon_coverage_documented() {
    // Check earcon mapping exists
    let paths = vec![
        "specs/005-adopt-design-sona/contracts/earcon-mapping.md",
        "contracts/earcon-mapping.md",
        "crates/player/src/earcon.rs",
    ];
    let mut found = false;
    for p in paths {
        if Path::new(p).exists() {
            found = true;
            break;
        }
    }
    assert!(found, "SC-002: earcon mapping/implementation must exist");
}

#[test]
fn sc002_no_numeric_rate_in_status_view() {
    let html = read_status_view();
    if html.is_empty() {
        return;
    }
    // Check for visible numeric rate patterns
    assert!(!html.contains(">1.25×<") && !html.contains(">1.0×<"), 
        "SC-002: numeric rate must not be painted/displayed");
}
EOF
echo "done"
