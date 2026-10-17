//! T064 — BLOCKING US4 contract test: pt-BR progress label.
//!
//! A determinate mirror publishes the translated content-offset label;
//! an indeterminate mirror has no label. The in-app surface consumes the
//! same field next to its progress indication.

use player::surface::{Dismissal, Progress, SurfaceSnapshot, progress_label};
use player::transport::TransportState;
use serde_json::Value;
use src_tauri::publish::{build_kotlin_mirror_json, publish_mirror};

const STATUS_VIEW: &str = include_str!("../src/status_view.html");

#[test]
fn determinate_mirror_exposes_pt_br_progress_label_and_view_renders_it() {
    let payload = publish_mirror(
        "playing",
        25.0,
        Some("Capítulo".to_string()),
        true,
        None,
    );
    assert_eq!(
        payload.get("progress_label"),
        Some(&Value::String(
            "Seção 1 de 1 — 20% (content-offset)".to_string()
        )),
        "determinate mirror must expose the translated content-offset label"
    );
    assert!(
        STATUS_VIEW.contains("progressLabel")
            && STATUS_VIEW.contains("progress_label"),
        "the in-app view must render the mirrored progress_label"
    );

    // The actual Rust→Kotlin builder is also required to carry the
    // display label while preserving its existing closed payload keys.
    let surface = SurfaceSnapshot {
        title: "Documento".to_string(),
        section_title: Some("Capítulo".to_string()),
        progress: Progress::Determinate {
            consumed_units: 25,
            total_units: 125,
            k: 1,
            n: 1,
        },
    };
    let kotlin = build_kotlin_mirror_json(
        TransportState::Playing,
        25.0,
        &surface,
        None,
        None,
        &Dismissal::NonDismissable,
    );
    let kotlin_payload: Value = serde_json::from_str(&kotlin).expect("valid mirror JSON");
    assert_eq!(
        kotlin_payload.get("progressLabel"),
        Some(&Value::String("Seção 1 de 1 — 20% (content-offset)".to_string()))
    );
}

#[test]
fn indeterminate_mirror_has_no_progress_label() {
    let payload = publish_mirror("playing", 25.0, Some("Capítulo".to_string()), false, None);
    assert!(
        payload.get("progress_label").is_none_or(Value::is_null),
        "indeterminate progress must not expose a label"
    );
    assert_eq!(progress_label(&Progress::Indeterminate), None);
}
