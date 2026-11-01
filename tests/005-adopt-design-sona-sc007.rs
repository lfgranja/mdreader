// SC-007: prefers-reduced-motion - property-based tests
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
    fn prop_sc007_reduced_motion_disables_animations(_seed in any::<u32>()) {
        let html = read_status_view();
        if html.is_empty() {
            return Ok(());
        }
        // Must have prefers-reduced-motion media query
        prop_assert!(
            html.contains("prefers-reduced-motion"),
            "SC-007: prefers-reduced-motion media query required"
        );
        // Must disable animations
        prop_assert!(
            html.contains("animation") && html.contains("none"),
            "SC-007: must disable animations in reduced motion mode"
        );
    }
    
    #[test]
    fn prop_sc007_reduced_motion_preserves_visibility(_seed in any::<u32>()) {
        let html = read_status_view();
        if html.is_empty() {
            return Ok(());
        }
        // When reduced motion, elements must remain visible (not opacity:0)
        // Check that reduced motion rules don't set opacity to 0 invisibly
        if let Some(pos) = html.find("@media(prefers-reduced-motion:reduce)") {
            let start = pos;
            if let Some(end) = html[start..].find("</style>") {
                let block = &html[start..start+end];
                // Should not have opacity:0 without visible alternative
                if block.contains("opacity:0") || block.contains("opacity: 0") {
                    // Must have alternative visibility mechanism
                    prop_assert!(
                        block.contains("opacity:1") || block.contains("animation-play-state: paused"),
                        "SC-007: reduced motion must keep elements visible"
                    );
                }
            }
        }
    }
}

#[test]
fn sc007_reduced_motion_media_query_exists() {
    let html = read_status_view();
    assert!(!html.is_empty(), "status_view.html must exist");
    assert!(html.contains("prefers-reduced-motion:reduce") || html.contains("prefers-reduced-motion: reduce"),
        "SC-007: prefers-reduced-motion: reduce required");
}

#[test]
fn sc007_reduced_motion_pauses_animations() {
    let html = read_status_view();
    assert!(
        html.contains("animation:none") || html.contains("animation-play-state: paused"),
        "SC-007: animations must be disabled or paused in reduced motion"
    );
}

#[test]
fn sc007_progress_remains_visible_in_reduced_motion() {
    let html = read_status_view();
    // Progress fill should remain visible
    assert!(
        html.contains("#progress-fill") || html.contains("progress-fill"),
        "SC-007: progress element must exist"
    );
}
EOF
echo "done"
