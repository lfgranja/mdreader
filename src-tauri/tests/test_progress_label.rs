//! T094 — Phase 14 RED-FIRST contract test: production mirror
//! builders and the pt-BR progress label (feature
//! `004-ressoa-dogfood-build`).
//!
//! Pins the two production mirror builders in `src_tauri::publish` —
//! the `MirrorState` the Rust→frontend channel carries, and the
//! Kotlin lock-screen payload:
//!
//! * [`src_tauri::publish::frontend_mirror`] — the `MirrorState`
//!   sent to the frontend — carries `progress_label` for a
//!   determinate surface and omits it for an indeterminate one
//!   (FR-011 / FR-013). The in-app status view renders the same
//!   field.
//! * [`src_tauri::publish::build_kotlin_mirror_json`] — the Kotlin
//!   lock-screen payload — carries the same pt-BR `progressLabel`
//!   for a determinate surface and omits it for an indeterminate
//!   one, matching the frontend mirror (FR-011 / FR-013; T083/T089).
//!
//! RED-FIRST (T094): the original RED run asserted the pre-T083
//! contract (label-free Kotlin payload). T090 reconciled the
//! implementation to the T083/T089 contract — the Kotlin payload
//! now carries `progressLabel` for determinate progress — so the
//! label-free assertion is replaced by the green one below.

use std::error::Error;

use player::surface::{Dismissal, Progress, SurfaceSnapshot};
use player::transport::TransportState;
use serde_json::Value;
use src_tauri::publish::{build_kotlin_mirror_json, frontend_mirror};

/// The status-view HTML the in-app surface renders through.
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// A determinate surface snapshot: the totals are known at session
/// start, so the progress carries the section indices and the
/// content-offset units.
fn determinate(k: u32, n: u32, consumed_units: u64, total_units: u64) -> SurfaceSnapshot {
    SurfaceSnapshot {
        title: String::from("Falando com estranhos"),
        section_title: Some(String::from("Capítulo 2 — A chegada")),
        progress: Progress::Determinate {
            consumed_units,
            total_units,
            k,
            n,
        },
    }
}

/// An indeterminate surface snapshot: the totals were unknown at
/// session start, so the whole session stays indeterminate.
fn indeterminate() -> SurfaceSnapshot {
    SurfaceSnapshot {
        title: String::from("notas.md"),
        section_title: None,
        progress: Progress::Indeterminate,
    }
}

/// Build the Kotlin lock-screen payload for one surface and parse it
/// to a JSON value.
fn kotlin_payload(surface: &SurfaceSnapshot) -> Result<Value, Box<dyn Error>> {
    let json = build_kotlin_mirror_json(
        TransportState::Playing,
        12.0,
        surface,
        None,
        None,
        &Dismissal::NonDismissable,
    );
    Ok(serde_json::from_str::<Value>(&json)?)
}

/// T094: the production frontend mirror carries the pt-BR progress
/// label for every determinate surface and omits it for an
/// indeterminate one; the in-app status view renders the same field.
#[test]
fn frontend_mirror_carries_progress_label_only_when_determinate() -> Result<(), Box<dyn Error>> {
    for k in 1..=3 {
        for n in (k + 1)..=5 {
            let surface = determinate(k, n, u64::from(100 * k / n), 100);
            let mirror = frontend_mirror(TransportState::Playing, 12.0, &surface);
            let encoded = serde_json::to_value(mirror)?;
            let label = encoded.get("progress_label").ok_or(
                "T094: the frontend MirrorState must carry progress_label for determinate",
            )?;
            let text = label.as_str().ok_or("progress_label is a string")?;
            assert!(
                text.contains("Seção") && text.contains(" de ") && text.contains('%'),
                "T094: the determinate frontend label is the pt-BR identifier: {text}"
            );
        }
    }
    let mirror = frontend_mirror(TransportState::Paused, 0.0, &indeterminate());
    let encoded = serde_json::to_value(mirror)?;
    assert!(
        encoded.get("progress_label").is_none(),
        "T094: an indeterminate frontend mirror omits progress_label"
    );
    assert!(STATUS_VIEW.contains("progressLabel"));
    assert!(STATUS_VIEW.contains("mirror.progress_label"));
    assert!(STATUS_VIEW.contains("state.progressLabel"));
    Ok(())
}

/// T100 RED-FIRST: the pt-BR label is the FULL translated string —
/// the `(content-offset)` fragment is translated too
/// (`contracts/surface.md:9`: `"Seção {k} de {N} — {p}% (deslocamento
/// de conteúdo)"`). An English identifier must never render
/// user-visible (FR-011), and the label IS user-visible: the
/// lock-screen projection and the in-app status view render it
/// verbatim.
#[test]
fn the_pt_br_progress_label_translates_the_content_offset_fragment() -> Result<(), Box<dyn Error>> {
    let surface = determinate(2, 5, 40, 100);
    let mirror = frontend_mirror(TransportState::Playing, 12.0, &surface);
    let encoded = serde_json::to_value(mirror)?;
    let label = encoded
        .get("progress_label")
        .ok_or("T100: the frontend mirror must carry progress_label for determinate")?;
    let text = label.as_str().ok_or("progress_label is a string")?;
    assert_eq!(
        text, "Seção 2 de 5 — 40% (deslocamento de conteúdo)",
        "T100: the pt-BR label must be the fully translated string (contracts/surface.md:9)"
    );
    assert!(
        !text.contains("content-offset"),
        "T100: the English `(content-offset)` fragment must never render user-visible (FR-011)"
    );
    Ok(())
}

/// T100: the Kotlin lock-screen payload carries the same fully
/// translated label — the two payloads never disagree about the
/// user-visible string (T090 / T094).
#[test]
fn the_kotlin_lock_screen_label_translates_the_content_offset_fragment()
-> Result<(), Box<dyn Error>> {
    let payload = kotlin_payload(&determinate(2, 5, 40, 100))?;
    let label = payload
        .get("progressLabel")
        .ok_or("T100: the Kotlin lock-screen payload must carry progressLabel")?;
    let text = label.as_str().ok_or("progressLabel is a string")?;
    assert_eq!(
        text, "Seção 2 de 5 — 40% (deslocamento de conteúdo)",
        "T100: the pt-BR label must be the fully translated string (contracts/surface.md:9)"
    );
    Ok(())
}

/// T094 GREEN: the production Kotlin lock-screen payload carries the
/// same pt-BR `progressLabel` as the frontend mirror for every
/// determinate surface and omits it for an indeterminate one
/// (FR-011 / FR-013; T083/T089). The T090 implementation emits the
/// key for determinate progress, reconciling the contract.
#[test]
fn kotlin_lock_screen_payload_carries_progress_label_only_when_determinate()
-> Result<(), Box<dyn Error>> {
    for k in 1..=3 {
        for n in (k + 1)..=5 {
            let surface = determinate(k, n, u64::from(100 * k / n), 100);
            let payload = kotlin_payload(&surface)?;
            let label = payload.get("progressLabel").ok_or(
                "T094: the Kotlin lock-screen payload must carry progressLabel for determinate",
            )?;
            let text = label.as_str().ok_or("progressLabel is a string")?;
            assert!(
                text.contains("Seção") && text.contains(" de ") && text.contains('%'),
                "T094: the determinate Kotlin label is the pt-BR identifier: {text}"
            );
        }
    }
    let payload = kotlin_payload(&indeterminate())?;
    assert!(
        payload.get("progressLabel").is_none(),
        "T094: an indeterminate Kotlin payload must not carry progressLabel"
    );
    Ok(())
}
