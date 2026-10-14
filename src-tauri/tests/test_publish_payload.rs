//! T132 — Phase 14 BLOCKING TEST (RED): publish-payload contract
//! (contracts/shell-ipc.md State mirror — Rust→shell streaming).
//!
//! Pins EXACTLY the mirror wire form the sanctioned
//! `tauri::ipc::Channel` publish path MUST emit on every
//! transport/position/focus event (FR-003, FR-028 series).
//!
//! The wire form is a JSON object with exactly these keys
//! (snake_case, per the case-mapping table in
//! `contracts/shell-ipc.md`):
//!
//! - `transport` — externally-tagged closed enum:
//!   `playing` | `paused` | `stopped` | `completed` | `interrupted`
//! - `position_secs` — f64, the session's authoritative position
//! - `section_title` — string, present iff a heading has been
//!   passed (absent, not null, when before first heading or
//!   heading-less document)
//! - `progress` — externally-tagged closed enum:
//!   `{"determinate": {"position_secs": f64, "duration_secs": f64}}`
//!   | `"indeterminate"`
//! - `transient_reason` — string, present ONLY when the pause
//!   carries a stated transient reason (focus-loss pause under
//!   FR-041 or failure reason under FR-037); absent otherwise
//!
//! Never: body text, elapsed/remaining time, rate, voice, or
//! any key outside the enumerated set (FR-028 series).
//!
//! This test FAILS before T133 because the publish path
//! (`tauri::ipc::Channel` streaming from Rust to shell) does
//! not exist yet — no `Channel` creation, no `emit` call sites,
//! no Kotlin `@Command` listeners in `src-tauri/`.

use serde_json::Value;

/// The publish function that T133 will implement.
/// This import will fail until T133 creates the publish module.
use src_tauri::publish::publish_mirror;

#[test]
fn publish_payload_carries_exactly_the_five_contract_keys_for_each_transport() {
    // Test every transport state with a known position and section.
    let transports = ["playing", "paused", "stopped", "completed", "interrupted"];

    for transport in transports {
        let position_secs = 30.0;
        let _duration_secs = 120.0;
        let section_title = Some("Chapter 1".to_string());

        // This call will fail until T133 implements publish_mirror.
        let payload = publish_mirror(
            transport,
            position_secs,
            section_title.clone(),
            true, // determinate
            None, // no transient reason
        );

        // Assert the wire form has exactly the 4 keys (section_title present).
        let obj = payload.as_object().expect("payload must be a JSON object");
        assert_eq!(
            obj.len(),
            4,
            "publish payload for {transport} must have exactly 4 keys (transport, position_secs, section_title, progress)"
        );

        // Verify each key and its shape.
        assert_eq!(
            obj.get("transport"),
            Some(&Value::String(transport.to_string())),
            "transport key must be the snake_case wire form"
        );
        assert_eq!(
            obj.get("position_secs"),
            Some(&Value::Number(
                serde_json::Number::from_f64(position_secs).unwrap()
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
        let progress = obj.get("progress").unwrap();
        assert!(
            progress.get("determinate").is_some(),
            "progress must be determinate for known-duration session"
        );
        let det = progress.get("determinate").unwrap();
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
        assert!(det.get("k").is_some(), "k (current section) must be present");
        assert!(det.get("n").is_some(), "n (total sections) must be present");
        assert!(
            det.get("position_secs").is_none() && det.get("duration_secs").is_none(),
            "the 001 time-based progress fields must not survive the 004 content-offset shape (T005)"
        );

        // Verify no extra keys (especially no body text, rate, voice, etc.)
        for key in obj.keys() {
            assert!(
                matches!(
                    key.as_str(),
                    "transport" | "position_secs" | "section_title" | "progress"
                ),
                "unexpected key in publish payload: {key}"
            );
        }
    }
}

#[test]
fn publish_payload_section_title_absent_before_first_heading() {
    let position_secs = 5.0;
    let _duration_secs = 120.0;

    // This call will fail until T133 implements publish_mirror.
    let payload = publish_mirror(
        "playing",
        position_secs,
        None, // no section title before first heading
        true, // determinate
        None, // no transient reason
    );

    let obj = payload.as_object().expect("payload must be a JSON object");
    assert_eq!(
        obj.len(),
        3,
        "publish payload before first heading must have exactly 3 keys (transport, position_secs, progress)"
    );
    assert!(
        obj.get("section_title").is_none(),
        "section_title must be absent (not null) before first heading"
    );
}

#[test]
fn publish_payload_indeterminate_progress_when_duration_unknown() {
    let position_secs = 30.0;

    // This call will fail until T133 implements publish_mirror.
    let payload = publish_mirror(
        "playing",
        position_secs,
        Some("Chapter 1".to_string()),
        false, // indeterminate
        None,  // no transient reason
    );

    let obj = payload.as_object().expect("payload must be a JSON object");
    let progress = obj.get("progress").unwrap();
    assert_eq!(
        progress,
        &Value::String("indeterminate".to_string()),
        "indeterminate progress must serialize as bare wire string"
    );
}

#[test]
fn publish_payload_transient_reason_present_only_for_transient_pause() {
    let position_secs = 30.0;
    let _duration_secs = 120.0;

    // With transient reason (focus loss, FR-041).
    let with_reason = publish_mirror(
        "paused",
        position_secs,
        Some("Chapter 1".to_string()),
        true, // determinate
        Some("outro app usando o áudio".to_string()),
    );
    let obj = with_reason.as_object().unwrap();
    assert_eq!(
        obj.len(),
        5,
        "payload with transient reason must have 5 keys"
    );
    assert!(
        obj.get("transient_reason").is_some(),
        "transient_reason must be present for focus-loss pause"
    );
    assert_eq!(
        obj.get("transient_reason"),
        Some(&Value::String("outro app usando o áudio".to_string()))
    );

    // Without transient reason (deliberate pause, resident).
    let without_reason = publish_mirror(
        "paused",
        position_secs,
        Some("Chapter 1".to_string()),
        true, // determinate
        None,
    );
    let obj = without_reason.as_object().unwrap();
    assert_eq!(
        obj.len(),
        4,
        "resident pause payload must have 4 keys (no transient_reason)"
    );
    assert!(
        obj.get("transient_reason").is_none(),
        "transient_reason must be absent for resident pause"
    );
}

#[test]
fn publish_payload_never_carries_body_text_or_extra_fields() {
    let position_secs = 30.0;
    let _duration_secs = 120.0;

    // This call will fail until T133 implements publish_mirror.
    let payload = publish_mirror(
        "playing",
        position_secs,
        Some("Chapter 1".to_string()),
        true, // determinate
        None, // no transient reason
    );

    let obj = payload.as_object().unwrap();

    // Ensure no body text, no elapsed/remaining, no rate, no voice.
    let forbidden_keys = [
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
            "publish payload must never carry forbidden key: {key}"
        );
    }
}

#[test]
fn publish_payload_transport_enum_matches_contract_case_mapping() {
    let transports = [
        ("playing", "playing"),
        ("paused", "paused"),
        ("stopped", "stopped"),
        ("completed", "completed"),
        ("interrupted", "interrupted"),
    ];

    for (input_transport, expected_wire) in transports {
        // This call will fail until T133 implements publish_mirror.
        let payload = publish_mirror(input_transport, 30.0, None, true, None);
        let transport_value = payload.get("transport").unwrap();
        assert_eq!(
            transport_value,
            &Value::String(expected_wire.to_string()),
            "transport {input_transport} must serialize to {expected_wire} (contracts/shell-ipc.md case-mapping)"
        );
    }
}
