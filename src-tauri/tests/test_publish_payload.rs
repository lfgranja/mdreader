//! T132 / T079 / T099 — the publish-payload contract: the Rust→shell
//! STATE MIRROR and the Rust→Kotlin LOCK-SCREEN payload.
//!
//! TWO payloads leave the Rust bridge, and this file pins BOTH:
//!
//! 1. the STATE MIRROR ([`publish_mirror`]) — the 001 shell-ipc wire
//!    form (`specs/001-android-media-shell/contracts/shell-ipc.md`):
//!    `transport`, `position_secs`, `section_title`, `progress` and
//!    the optional `transient_reason`. It carries NO label: the
//!    progress label is a display projection, and the state mirror
//!    carries state.
//!
//! 2. the LOCK-SCREEN payload ([`build_kotlin_mirror_json`]) — the
//!    camelCase wire form the Kotlin surface parses: it carries the
//!    pt-BR `progressLabel` for determinate progress and omits it for
//!    an indeterminate one (FR-011 / FR-013; T083 / T089 / T090 /
//!    T094 — `contracts/surface.md`).
//!
//! T099 RECONCILES the two definitions. The earlier Phase 11/12
//! fixture (T079) asserted a LABEL-FREE LOCK-SCREEN payload, which
//! contradicted the T090/T094 contract the production Kotlin payload
//! implements. One definition now names the lock-screen payload —
//! [`build_kotlin_mirror_json`], the one the Kotlin surface parses —
//! and the state mirror keeps the wire form it always had.

use std::error::Error;

use player::surface::{Dismissal, Progress, SurfaceSnapshot};
use player::transport::TransportState;
use serde_json::Value;
use src_tauri::publish::{build_kotlin_mirror_json, publish_mirror};

/// The exact contract key set for the state mirror with a section
/// title present (transport, `position_secs`, `section_title`,
/// progress).
const CONTRACT_KEYS_WITH_SECTION: [&str; 4] =
    ["transport", "position_secs", "section_title", "progress"];

/// The exact contract key set for the state mirror with no section
/// title (transport, `position_secs`, progress).
const CONTRACT_KEYS_NO_SECTION: [&str; 3] = ["transport", "position_secs", "progress"];

/// Assert that the state mirror obeys the exact contract key set:
/// every expected key is present, no unexpected key exists, and the
/// progress label is always omitted — the label rides the lock-screen
/// payload, never the state mirror (T099).
fn assert_contract_keys(
    payload: &Value,
    expected_keys: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let obj = payload.as_object().ok_or("payload must be a JSON object")?;

    // T079/T099: the progress label MUST be omitted from the state
    // mirror — it is a display projection, and the state mirror is
    // state.
    assert!(
        obj.get("progress_label").is_none(),
        "progress_label MUST be omitted from the state mirror (T079/T099)"
    );

    // Exact key set: no more, no fewer.
    let mut actual_keys: Vec<&str> = obj.keys().map(std::string::String::as_str).collect();
    actual_keys.sort_unstable();
    let mut expected_sorted: Vec<&str> = expected_keys.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        actual_keys, expected_sorted,
        "the state mirror must carry exactly the contract keys"
    );
    Ok(())
}

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

/// The lock-screen payload for one surface, parsed to a JSON value.
fn lock_screen_payload(surface: &SurfaceSnapshot) -> Result<Value, Box<dyn Error>> {
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

/// T132 / T079: the state mirror carries exactly the shell-ipc
/// contract keys for every transport state.
#[test]
fn publish_payload_carries_exactly_the_contract_keys_for_each_transport()
-> Result<(), Box<dyn std::error::Error>> {
    // Test every transport state with a known position and section.
    let transports = ["playing", "paused", "stopped", "completed", "interrupted"];

    for transport in transports {
        let position_secs = 30.0;
        let section_title = Some("Chapter 1".to_string());

        let payload = publish_mirror(
            transport,
            position_secs,
            section_title.clone(),
            true, // determinate
            None, // no transient reason
        );

        // T079: assert exact contract keys + progress_label omission.
        assert_contract_keys(&payload, &CONTRACT_KEYS_WITH_SECTION)?;

        // Verify each key and its shape.
        let obj = payload.as_object().ok_or("payload must be a JSON object")?;
        assert_eq!(
            obj.get("transport"),
            Some(&Value::String(transport.to_string())),
            "transport key must be the snake_case wire form"
        );
        assert_eq!(
            obj.get("position_secs"),
            Some(&Value::Number(
                serde_json::Number::from_f64(position_secs)
                    .ok_or("position_secs must be a finite f64")?
            )),
            "position_secs key must be the authoritative position"
        );
        assert!(
            obj.get("section_title").is_some(),
            "section_title must be present when a heading has been passed"
        );
        assert!(
            obj.get("progress").is_some(),
            "progress key must be present"
        );

        // Verify progress is externally tagged determinate.
        let progress = obj.get("progress").ok_or("progress key must be present")?;
        assert!(
            progress.get("determinate").is_some(),
            "progress must be determinate for known-duration session"
        );
        let det = progress
            .get("determinate")
            .ok_or("determinate progress must be a JSON object")?;
        // 004 content-offset shape (contracts/surface.md / T005):
        // the determinate payload carries parsed-content units and the
        // section counts — never the time-based position/duration the
        // 001 shell-ipc wire form carried (audio-duration estimates never
        // feed progress).
        assert_eq!(
            det.get("consumed_units"),
            Some(&Value::Number(30_u64.into())),
            "consumed_units must be the content-offset numerator (T005 shape)"
        );
        assert_eq!(
            det.get("total_units"),
            Some(&Value::Number(120_u64.into())),
            "total_units must be the session-start content-offset total (T005 shape)"
        );
        assert!(
            det.get("k").is_some(),
            "k (current section) must be present"
        );
        assert!(det.get("n").is_some(), "n (total sections) must be present");
        assert!(
            det.get("position_secs").is_none() && det.get("duration_secs").is_none(),
            "the 001 time-based progress fields must not survive the 004 content-offset shape (T005)"
        );
    }
    Ok(())
}

#[test]
fn publish_payload_section_title_absent_before_first_heading()
-> Result<(), Box<dyn std::error::Error>> {
    let position_secs = 5.0;

    let payload = publish_mirror(
        "playing",
        position_secs,
        None, // no section title before first heading
        true, // determinate
        None, // no transient reason
    );

    // T079: exact contract keys + progress_label omission.
    assert_contract_keys(&payload, &CONTRACT_KEYS_NO_SECTION)?;

    let obj = payload.as_object().ok_or("payload must be a JSON object")?;
    assert!(
        obj.get("section_title").is_none(),
        "section_title must be absent (not null) before first heading"
    );
    Ok(())
}

#[test]
fn publish_payload_indeterminate_progress_when_duration_unknown()
-> Result<(), Box<dyn std::error::Error>> {
    let position_secs = 30.0;

    let payload = publish_mirror(
        "playing",
        position_secs,
        Some("Chapter 1".to_string()),
        false, // indeterminate
        None,  // no transient reason
    );

    // T079: exact contract keys + progress_label omission.
    assert_contract_keys(&payload, &CONTRACT_KEYS_WITH_SECTION)?;

    let obj = payload.as_object().ok_or("payload must be a JSON object")?;
    let progress = obj.get("progress").ok_or("progress key must be present")?;
    assert_eq!(
        progress,
        &Value::String("indeterminate".to_string()),
        "indeterminate progress must serialize as bare wire string"
    );
    Ok(())
}

#[test]
fn publish_payload_transient_reason_present_only_for_transient_pause()
-> Result<(), Box<dyn std::error::Error>> {
    let position_secs = 30.0;

    // With transient reason (focus loss, FR-041).
    let with_reason = publish_mirror(
        "paused",
        position_secs,
        Some("Chapter 1".to_string()),
        true, // determinate
        Some("outro app usando o audio".to_string()),
    );

    // T079: with transient reason, the contract keys are
    // transport, position_secs, section_title, progress,
    // transient_reason — 5 keys, no progress_label.
    let obj = with_reason
        .as_object()
        .ok_or("payload must be a JSON object")?;
    assert!(
        obj.get("progress_label").is_none(),
        "progress_label MUST be omitted from the state mirror (T079/T099)"
    );
    let mut actual_keys: Vec<&str> = obj.keys().map(std::string::String::as_str).collect();
    actual_keys.sort_unstable();
    let mut expected_sorted = [
        "transport",
        "position_secs",
        "section_title",
        "progress",
        "transient_reason",
    ];
    expected_sorted.sort_unstable();
    assert_eq!(
        actual_keys, expected_sorted,
        "payload with transient reason must have exactly 5 contract keys"
    );
    assert!(
        obj.get("transient_reason").is_some(),
        "transient_reason must be present for focus-loss pause"
    );
    assert_eq!(
        obj.get("transient_reason"),
        Some(&Value::String("outro app usando o audio".to_string()))
    );

    // Without transient reason (deliberate pause, resident).
    let without_reason = publish_mirror(
        "paused",
        position_secs,
        Some("Chapter 1".to_string()),
        true,
        None,
    );

    // T079: without transient reason, back to the 4-key contract.
    assert_contract_keys(&without_reason, &CONTRACT_KEYS_WITH_SECTION)?;
    Ok(())
}

#[test]
fn publish_payload_never_carries_body_text_or_extra_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let position_secs = 30.0;

    let payload = publish_mirror(
        "playing",
        position_secs,
        Some("Chapter 1".to_string()),
        true,
        None,
    );

    let obj = payload.as_object().ok_or("payload must be a JSON object")?;

    // T079: progress_label is explicitly forbidden.
    assert!(
        obj.get("progress_label").is_none(),
        "the state mirror must never carry progress_label (T079/T099)"
    );

    // Ensure no body text, no elapsed/remaining, no rate, no voice.
    let forbidden_keys = [
        "progress_label",
        "body",
        "text",
        "sentence",
        "paragraph",
        "elapsed",
        "remaining",
        "elapsed_secs",
        "remaining_secs",
        "rate",
        "playback_rate",
        "voice",
        "engine",
        "engine_id",
        "rate_rung",
    ];

    for key in forbidden_keys {
        assert!(
            obj.get(key).is_none(),
            "the state mirror must never carry forbidden key: {key}"
        );
    }
    Ok(())
}

#[test]
fn publish_payload_transport_enum_matches_contract_case_mapping()
-> Result<(), Box<dyn std::error::Error>> {
    let transports = [
        ("playing", "playing"),
        ("paused", "paused"),
        ("stopped", "stopped"),
        ("completed", "completed"),
        ("interrupted", "interrupted"),
    ];

    for (input_transport, expected_wire) in transports {
        let payload = publish_mirror(input_transport, 30.0, None, true, None);

        // T079: exact contract keys + progress_label omission.
        assert_contract_keys(&payload, &CONTRACT_KEYS_NO_SECTION)?;

        let transport_value = payload
            .get("transport")
            .ok_or("payload must carry a transport key")?;
        assert_eq!(
            transport_value,
            &Value::String(expected_wire.to_string()),
            "transport {input_transport} must serialize to {expected_wire} (contracts/shell-ipc.md case-mapping)"
        );
    }
    Ok(())
}

/// T099 (RED→GREEN): the LOCK-SCREEN payload — the one the Kotlin
/// surface parses — carries the pt-BR progress label for determinate
/// progress and omits it for an indeterminate one (FR-011 / FR-013;
/// T089 / T090 / T094; `contracts/surface.md`).
///
/// The earlier Phase 11/12 fixture asserted a LABEL-FREE lock-screen
/// payload, which contradicted this contract: with the production
/// payload builder, the label-free claim FAILS (the RED run). The
/// reconciliation keeps the production builder's contract and drops
/// the stale claim.
#[test]
fn the_lock_screen_payload_carries_the_pt_br_label_only_when_determinate()
-> Result<(), Box<dyn std::error::Error>> {
    for k in 1..=3 {
        for n in (k + 1)..=5 {
            let surface = determinate(k, n, u64::from(100 * k / n), 100);
            let payload = lock_screen_payload(&surface)?;
            let label = payload.get("progressLabel").ok_or(
                "T099: the lock-screen payload must carry progressLabel for determinate progress",
            )?;
            let text = label.as_str().ok_or("progressLabel is a string")?;
            assert!(
                text.contains("Seção") && text.contains(" de ") && text.contains('%'),
                "T099: the determinate lock-screen label is the pt-BR identifier: {text}"
            );
        }
    }
    let payload = lock_screen_payload(&indeterminate())?;
    assert!(
        payload.get("progressLabel").is_none(),
        "T099: an indeterminate lock-screen payload must not carry progressLabel"
    );
    Ok(())
}

/// T099: the lock-screen payload carries no document prose, no
/// elapsed or remaining time, no rate value and no voice identity —
/// the lock screen's own purity rule (contracts/surface.md).
#[test]
fn the_lock_screen_payload_carries_no_prose_time_rate_or_voice()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = lock_screen_payload(&determinate(2, 5, 40, 100))?;
    let rendered = serde_json::to_string(&payload)?;
    for forbidden in [
        "Chapter",
        "paragraph",
        "sentence",
        "elapsed",
        "remaining",
        "0.75x",
        "1.0x",
        "1.25x",
        "1.5x",
        "1.75x",
        "2.0x",
        "4.0x",
        "voice",
        "engine",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "the lock-screen payload leaks {forbidden:?}: {rendered}"
        );
    }
    Ok(())
}
