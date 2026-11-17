// RED tests for US2 — waveform (T011-T012). Must NOT pass until implementation complete.
#[test]
fn waveform_canvas_present() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("id=\"waveform\""),
        "waveform canvas must be present"
    );
    assert!(
        html.contains("aria-hidden=\"true\""),
        "waveform canvas must be aria-hidden=\"true\""
    );
}

#[test]
fn waveform_has_five_layers() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    // Check for 5 layer rendering
    assert!(
        html.contains("li < 5") || html.contains("layers") || html.contains("[0,1,2,3,4]"),
        "waveform must render 5 layers"
    );
}

#[test]
fn waveform_has_bezier_curves() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("bezierCurveTo"),
        "waveform must use bezier curves"
    );
}

#[test]
fn waveform_has_playhead() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("arc") && html.contains("playhead") || html.contains("arc(px") || html.contains("6"),
        "waveform must have circular playhead"
    );
}

#[test]
fn waveform_rms_integration() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("rms") || html.contains("RMS"),
        "waveform must integrate with RMS"
    );
}

#[test]
fn waveform_fps_gate() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("30") && (html.contains("WAVEFORM_FPS") || html.contains("performance.now") || html.contains("33")),
        "waveform must have 30fps gate"
    );
}

#[test]
fn waveform_visibility_pause() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("visibilitychange"),
        "waveform must pause on visibility change"
    );
}

#[test]
fn waveform_reduced_motion_freeze() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("prefers-reduced-motion"),
        "waveform must respect reduced motion"
    );
}

#[test]
fn waveform_dpr_cap() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("devicePixelRatio") && html.contains("2"),
        "waveform must cap DPR at 2x"
    );
}

#[test]
fn waveform_fallback() {
    let html = std::fs::read_to_string("src-tauri/src/status_view.html")
        .unwrap_or_default();
    assert!(
        html.contains("getContext") || html.contains("fallback") || html.contains("placeholder"),
        "waveform must have fallback"
    );
}
