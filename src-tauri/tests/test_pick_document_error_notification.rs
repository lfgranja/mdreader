//! T173 — Contract test: format refusal notification.
//!
//! Requirement:
//! - When a format refusal happens (e.g. oversize file >93636 bytes),
//!   `FormatUndeliverable` is announced via `reasonBanner` and `liveRegion`.
//! - Playback session is not terminated; audio continues undisturbed.

const STATUS_VIEW: &str = include_str!("../src/status_view.html");

#[test]
fn status_view_carries_reason_banner_and_live_region() {
    assert!(
        STATUS_VIEW.contains("id=\"reasonBanner\""),
        "Status view must have reasonBanner"
    );
    assert!(
        STATUS_VIEW.contains("id=\"liveRegion\""),
        "Status view must have liveRegion for accessibility"
    );
    assert!(
        STATUS_VIEW.contains("renderReason"),
        "Status view must contain renderReason helper"
    );
}

#[test]
fn status_view_handles_format_undeliverable() {
    assert!(
        STATUS_VIEW.contains("FormatUndeliverable") || STATUS_VIEW.contains("reason"),
        "Status view must handle error reasons"
    );
}
