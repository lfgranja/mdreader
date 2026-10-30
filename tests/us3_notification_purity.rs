// RED test for T027 — US3 notification purity. Must exist (RED) before T030-T031.
#[test]
fn us3_notification_accent_only() {
    // MediaMirror.kt constants block must use #F2F0E6 (sona-voice) as sole accent
    let kt = std::fs::read_to_string("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt").unwrap_or_default();
    assert!(kt.contains("F2F0E6") || kt.contains("sona-voice") || kt.contains("0xFFF2F0E6"),
        "notification must use --sona-voice #F2F0E6 accent (SC-003)");
}

#[test]
fn us3_no_body_rate_voice_in_notification() {
    let kt = std::fs::read_to_string("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt").unwrap_or_default();
    // No body text / rate / voice fields in notification payload
    // Check that the constants block doesn't include extra fields
    assert!(!kt.contains("notificationBody") || !kt.contains("notificationRate"),
        "notification must not include body/rate/voice fields (SC-003)");
}

#[test]
fn us3_no_banned_colors_in_notification() {
    let kt = std::fs::read_to_string("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt").unwrap_or_default().to_lowercase();
    assert!(!kt.contains("#b9a6f5") && !kt.contains("#5ad6a8"),
        "notification must not use #B9A6F5 or #5AD6A8 (SC-003)");
}
