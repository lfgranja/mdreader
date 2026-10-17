//! T064 — BLOCKING US4 contract test: pt-BR progress label.
//!
//! A determinate mirror publishes the translated content-offset label;
//! an indeterminate mirror has no label. The in-app surface consumes the
//! same field next to its progress indication.

use player::surface::{Dismissal, Progress, SurfaceSnapshot};
use player::transport::TransportState;
use serde_json::Value;
use src_tauri::publish::{build_kotlin_mirror_json, frontend_mirror};

const STATUS_VIEW: &str = include_str!("../src/status_view.html");

#[test]
fn determinate_mirror_exposes_pt_br_progress_label_and_view_renders_it() {
    let surface = SurfaceSnapshot {
        title: "Documento".to_string(),
        section_title: Some("Capítulo".to_string()),
        progress: Progress::Determinate {
            consumed_units: 25,
            total_units: 120,
            k: 1,
            n: 1,
        },
    };
    let state = frontend_mirror(TransportState::Playing, 25.0, &surface);
    let encoded = serde_json::to_value(state).expect("frontend mirror serializes");
    assert_eq!(
        encoded.get("progress_label"),
        Some(&Value::String(
            "Seção 1 de 1 — 20% (content-offset)".to_string()
        )),
        "serialized frontend mirror must carry the pt-BR label"
    );
    let kotlin_json = build_kotlin_mirror_json(
        TransportState::Playing,
        25.0,
        &surface,
        None,
        None,
        &Dismissal::NonDismissable,
    );
    let kotlin: Value = serde_json::from_str(&kotlin_json).expect("Kotlin mirror is valid JSON");
    assert!(
        kotlin.get("progressLabel").is_none(),
        "lock-screen Kotlin payload must not carry the in-app label"
    );
    assert!(STATUS_VIEW.contains("progressLabel"));
    assert!(STATUS_VIEW.contains("mirror.progress_label"));
    assert!(STATUS_VIEW.contains("state.progressLabel"));
}

#[test]
fn indeterminate_mirror_has_no_progress_label() {
    let surface = SurfaceSnapshot {
        title: "Documento".to_string(),
        section_title: None,
        progress: Progress::Indeterminate,
    };
    let state = frontend_mirror(TransportState::Playing, 25.0, &surface);
    let encoded = serde_json::to_value(state).expect("frontend mirror serializes");
    assert!(
        encoded.get("progress_label").is_none(),
        "indeterminate frontend mirror must omit the label"
    );
}
