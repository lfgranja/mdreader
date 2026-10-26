// RED — FR-007 notification logo/brand exclusion from notification surfaces
// Contract: contracts/surface.md L71 (no brand logo / sound-wave in MediaStyle/lock-screen)
fn notification_surface_has_logo(path: &str) -> bool {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content.contains("logo") || content.contains("brand") || content.contains("sound-wave")
}

#[test]
fn fr007_status_view_has_no_logo_in_notification() {
    assert!(!notification_surface_has_logo("status_view.html"),
        "FR-007: status_view.html must exclude brand/logo from notification surface");
}

#[test]
fn fr007_media_mirror_has_no_logo_in_notification() {
    assert!(!notification_surface_has_logo("MediaMirror.kt"),
        "FR-007: MediaMirror.kt must exclude brand/logo from notification surface");
}
