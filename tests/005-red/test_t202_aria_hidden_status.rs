// T202 RED — hidden role="status" over-count invariant (rule 1)
// Contract S4: exactly 1 hidden live region (#liveRegion) + #reasonBanner;
// #progressLabel (visible) must NOT carry role="status".
#[test]
fn t202_hidden_status_overcount() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap();
    // Count occurrences of role="status"
    let count = html.matches(r#"role="status""#).count();
    // Excluding #reasonBanner — hidden status must be exactly 2 total (liveRegion + reasonBanner)
    // progressLabel (line 254) must NOT contribute.
    assert_eq!(count, 2,
        "hidden status over-count: {} occurrences, expected 2 (liveRegion + reasonBanner); \
         progressLabel (id=\"progressLabel\") must not be role=\"status\"", count);
}
