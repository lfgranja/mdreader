// RED — SC-001 census + contrast + aria inventory over status_view.html + MediaMirror.kt
// Deterministic host check; contracts/earcon.md + contracts/surface.md (SC-001 split)
fn census_inventory(path: &str) -> usize {
    std::fs::read_to_string(path).unwrap_or_default().matches("aria-").count()
}

#[test]
fn sc001_census_aria_inventory_present() {
    assert!(census_inventory("status_view.html") >= 2,
        "SC-001: status_view.html aria inventory < 2 (RED — missing census)");
    assert!(census_inventory("MediaMirror.kt") >= 2,
        "SC-001: MediaMirror.kt aria inventory < 2 (RED — missing census)");
}

#[test]
fn sc001_contrast_token_census_match() {
    // Token census must match closed hex count (10) in surfaces
    let html = std::fs::read_to_string("status_view.html").unwrap_or_default();
    let count = html.matches("sona-").count();
    assert_eq!(count, 10, "SC-001 contrast census: expected 10 sona tokens, got {}", count);
}
