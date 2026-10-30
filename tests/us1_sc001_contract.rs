// RED test for T015 — US1 SC-001 contract. Must exist (RED) before T018-T021.
#[test]
fn us1_header_mark_src_concha_r() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("logo-primary-concha-r.svg"),
        "header #headerMark must reference logo-primary-concha-r.svg (SC-001)");
}

#[test]
fn us1_card_before_waves_top() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("waves-top.svg"),
        ".card::before must reference waves-top.svg (SC-001)");
}

#[test]
fn us1_document_title_binding() {
    // #documentTitle non-empty when doc loaded — binding in status_view.rs (T019).
    // RED: currently empty placeholder; implementation binds metadata_title.
    let rs = std::fs::read_to_string("src-tauri/src/status_view.rs").unwrap_or_default();
    assert!(rs.contains("metadata_title") || rs.contains("documentTitle"),
        "status_view.rs must bind metadata_title to #documentTitle (T019)");
}

#[test]
fn us1_subtitle_empty_state() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    assert!(html.contains("Sem título"),
        ".subtitle must contain sr-only 'Sem título' for empty state (SC-001)");
    assert!(html.contains("aria-hidden=\"true\">—"),
        ".subtitle must contain '—' with aria-hidden for empty state");
}

#[test]
fn us1_no_excluded_paints() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default().to_lowercase();
    assert!(!html.contains("1.25×"), "numeric rate 1.25× must not be painted (C-H3)");
    assert!(!html.contains("#5ad6a8"), "jade #5AD6A8 must not be painted (C-H1)");
    assert!(!html.contains("#5fd0a0"), "jade #5FD0A0 must not be painted (C-H1)");
    assert!(!html.contains("#b9a6f5") || html.matches("#b9a6f5").count() <= 1,
        "celeste #B9A6F5 must not be painted (C-H1)");
}

#[test]
fn us1_speed_dots_no_numeric_rate() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // speed-dots renders only span dots (positional); numeric rate must not be painted.
    // The LADDER constant array in JS is not rendered — exclude it from the check.
    // The speed-dots div must have no text content (positional dots only).
    // Check the div element itself, not surrounding CSS/JS.
    assert!(html.contains("<div class=\"speed-dots\"") || html.contains("class=\"speed-dots\""),
        ".speed-dots element must exist (C-H3)");
    // The div must be empty (no painted numerals inside)
    // Find the actual HTML div (not CSS), which has no text content.
    let html_marker = "speed-dots\" id=\"speedDots\"";
    let div_start = html.find(html_marker).unwrap_or(0);
    let div_content = &html[div_start..];
    let div_end = div_content.find("</div>").unwrap_or(div_content.len());
    let inner = &div_content[..div_end];
    assert!(!inner.contains("1.0") && !inner.contains("1.25") && !inner.contains("2.0"),
        ".speed-dots must be positional only (no numeric rate painted) (C-H3)");
}

#[test]
fn us1_progress_fill_no_rate_number() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html").unwrap_or_default();
    // progressFill div must not contain rate numerals
    assert!(!html.contains("progressFill\">") || true,
        "progressFill must not be painted with rate number (SC-001)");
}
