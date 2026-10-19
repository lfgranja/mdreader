//! T132 / T079 — Phase 14 BLOCKING TEST (RED): publish-payload
//! contract (contracts/shell-ipc.md State mirror — Rust→shell
//! streaming) AND the T079 lock-screen payload invariant:
//! the public Kotlin/lock-screen payload helper MUST omit
//! `progress_label` and obey the exact contract key set.
//!
//! Pins EXACTLY the mirror wire form the sanctioned
//! `tauri::ipc::Channel` publish path MUST emit on every
//! transport/position/focus event (FR-003, FR-028 series).
//!
//! The wire form is a JSON object with exactly these keys
//! (`snake_case`, per the case-mapping table in
//! `contracts/shell-ipc.md`):
//!
//! - `transport` — externally-tagged closed enum:
//!   `playing` | `paused` | `stopped` | `completed` | `interrupted`
//! - `position_secs` — f64, the session's authoritative position
//! - `section_title` — string, present iff a heading has been
//!   passed (absent, not null, when before first heading or
//!   heading-less document)
//! - `progress` — externally-tagged closed enum:
//!   `{"determinate": {"consumed_units": u64, "total_units": u64,
//!   "k": u32, "n": u32}}` | `"indeterminate"`
//! - `transient_reason` — string, present ONLY when the pause
//!   carries a stated transient reason (focus-loss pause under
//!   FR-041 or failure reason under FR-037); absent otherwise
//!
//! Never: `progress_label`, body text, elapsed/remaining time,
//! rate, voice, or any key outside the enumerated set
//! (FR-028 series, T079).
//!
//! T079 (RED): the public Kotlin/lock-screen payload helper
//! (`publish_mirror`) currently emits a `progress_label` key
//! when progress is determinate — a 5th key outside the
//! contract set. This test asserts the exact contract keys
//! and the absence of `progress_label`, so it FAILS until
//! the implementation removes that key.

use serde_json::Value;

/// The publish function that T133 will implement.
/// This import will fail until T133 creates the publish module.
use src_tauri::publish::publish_mirror;

/// The exact contract key set for a publish payload with
/// a section title present (transport, `position_secs`,
/// `section_title`, progress).
const CONTRACT_KEYS_WITH_SECTION: [&str; 4] =
    ["transport", "position_secs", "section_title", "progress"];

/// The exact contract key set for a publish payload with
/// no section title (transport, `position_secs`, progress).
const CONTRACT_KEYS_NO_SECTION: [&str; 3] = ["transport", "position_secs", "progress"];

/// Assert that the payload obeys the exact contract key set:
/// every expected key is present, no unexpected key exists,
/// and `progress_label` is always omitted (T079).
fn assert_contract_keys(
    payload: &Value,
    expected_keys: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let obj = payload.as_object().ok_or("payload must be a JSON object")?;

    // T079: progress_label MUST be omitted from the public
    // Kotlin/lock-screen payload — it is an app-only field
    // that never crosses the wire.
    assert!(
        obj.get("progress_label").is_none(),
        "progress_label MUST be omitted from the publish payload (T079)"
    );

    // Exact key set: no more, no fewer.
    let mut actual_keys: Vec<&str> = obj.keys().map(std::string::String::as_str).collect();
    actual_keys.sort_unstable();
    let mut expected_sorted: Vec<&str> = expected_keys.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        actual_keys, expected_sorted,
        "publish payload must carry exactly the contract keys"
    );
    Ok(())
}

#[test]
fn publish_payload_carries_exactly_the_contract_keys_for_each_transport()
-> Result<(), Box<dyn std::error::Error>> {
    // Test every transport state with a known position and section.
    let transports = ["playing", "paused", "stopped", "completed", "interrupted"];

    for transport in transports {
        let position_secs = 30.0;
        let section_title = Some("Chapter 1".to_string());

        // This call will fail until T133 implements publish_mirror.
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

    // This call will fail until T133 implements publish_mirror.
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

    // This call will fail until T133 implements publish_mirror.
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
        "progress_label MUST be omitted from the publish payload (T079)"
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
        true, // determinate
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

    // This call will fail until T133 implements publish_mirror.
    let payload = publish_mirror(
        "playing",
        position_secs,
        Some("Chapter 1".to_string()),
        true, // determinate
        None, // no transient reason
    );

    let obj = payload.as_object().ok_or("payload must be a JSON object")?;

    // T079: progress_label is explicitly forbidden.
    assert!(
        obj.get("progress_label").is_none(),
        "publish payload must never carry progress_label (T079)"
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
            "publish payload must never carry forbidden key: {key}"
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
        // This call will fail until T133 implements publish_mirror.
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
