//! T011 — diagnostic-record JSON-schema conformance test (Phase 2, BLOCKING).
//!
//! Invariant under test (tasks.md T011, verbatim): "validates sample events
//! against `specs/001-android-media-shell/contracts/diagnostic-record.schema.json`
//! per FR-039, FR-044, FR-045: required `v/ts_ms/kind/position_secs`,
//! `detail` bound `maxLength 200` + pattern
//! `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`,
//! `error` required on `failure` AND `refusal`, `engine` implies `engine_tier`
//! (C1), `session_end` requires `transport`∈closed-4 + `session_end_reason`∈
//! closed-4 (never `playing` on `session_end`), `trim` requires
//! `dropped_events ≥ 1` + `cutoff_ts_ms`; supersession is NOT asserted here —
//! it is a two-entry sequence invariant owned by T035 and T067, because a
//! per-entry schema cannot express it without a stateful discriminator (see
//! the 2026-10-05 clarification)"
//!
//! The committed schema is the artifact under test: every valid sample event
//! the writer emits must validate against it with the draft 2020-12 validator
//! (jsonschema, the diag dev-dependency).
//!
//! FAILS before `crates/diag/src/writer.rs` exists (T073): the
//! `diag::writer` module is unresolved, so this test target does not build.
//! That build failure is the required pre-implementation red state.
//!
//! Expected API contract for T073 (`crates/diag/src/writer.rs`):
//! - `pub struct RecordEvent` — one JSONL record; `Serialize` + `Deserialize`;
//!   serde field names exactly the schema property names (`v`, `ts_ms`,
//!   `kind`, `position_secs`, `transport`, `session_end_reason`,
//!   `supersedes_ts_ms`, `focus`, `device`, `error`, `engine`,
//!   `engine_tier`, `detail`, `trim`), with absent options omitted on the
//!   wire (`skip_serializing_if = "Option::is_none"`). Every field is `pub`
//!   so a record can be constructed directly:
//!   `v: u8` (const 1), `ts_ms: u64`, `kind: EventKind`, `position_secs:
//!   f64`, `transport: Option<TransportValue>`, `session_end_reason:
//!   Option<SessionEndValue>`, `supersedes_ts_ms: Option<u64>`, `focus:
//!   Option<FocusValue>`, `device: Option<DeviceEvent>`, `error:
//!   Option<ErrorValue>`, `engine: Option<String>`, `engine_tier:
//!   Option<EngineTierValue>`, `detail: Option<String>`, `trim:
//!   Option<TrimMarker>`
//! - `pub enum EventKind { SessionStart, SessionEnd, TransportChange,
//!   FocusTransition, OutputDevice, Failure, Refusal, Trim, Gap }` — wire
//!   `session_start | session_end | transport_change | focus_transition |
//!   output_device | failure | refusal | trim | gap`
//! - `pub enum TransportValue { Playing, Paused, Stopped, Completed,
//!   Interrupted }`; `pub enum SessionEndValue { Paused, Stopped, Completed,
//!   Interrupted }`; `pub enum FocusValue { DuckableTransient,
//!   SilenceDemanding, PermanentLoss, Regained }`
//! - `pub enum ErrorValue { EngineUnreachable, EngineInitFailed, RateMismatch,
//!   ChannelMismatch, DeviceRefused, DeviceLost, StartRefused }`
//! - `pub enum EngineTierValue { OfflineDefault, CloudDefault, CloudEscalation,
//!   TestDouble, EvaluatedFallback }`
//! - `pub struct DeviceEvent { pub direction: DeviceDirection, pub device_id:
//!   String }` with `pub enum DeviceDirection { Appeared, Disappeared }`
//! - `pub struct TrimMarker { pub dropped_events: u64, pub cutoff_ts_ms: u64 }`
//! - `pub struct DiagnosticWriter` with `pub fn open(path: &Path) ->
//!   Result<Self, DiagError>` and `pub fn append(&mut self, event: &RecordEvent)
//!   -> Result<(), DiagError>` (append-only JSONL, one record per line)
//!
//! NOTE for T073/T014 coordination: the wire enums above MUST be the same
//! closed sets as `engine_core::closed` (FR-044: one list, never a second
//! drifting list) — re-export or share the definitions; the names here are
//! the wire-level view the record carries (contracts/shell-ipc.md case
//! mapping). The `diag` manifest needs the matching dependency or re-export
//! for this test to compile.

use jsonschema::Validator;
use serde_json::{Value, json};

use diag::writer::{DiagnosticWriter, RecordEvent};

/// Path of the committed contract schema, relative to this crate
/// (`crates/diag`), i.e. the workspace-root `specs/` tree.
fn schema_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("specs")
        .join("001-android-media-shell")
        .join("contracts")
        .join("diagnostic-record.schema.json")
}

fn load_validator() -> Result<Validator, Box<dyn std::error::Error>> {
    let schema_text = std::fs::read_to_string(schema_path())?;
    let schema: Value = serde_json::from_str(&schema_text)?;
    Validator::new(&schema)
        .map_err(|e| format!("committed contract schema failed to compile: {e}").into())
}

/// A minimal valid event of `kind`; every optional field is left out.
fn base_event(kind: &str, position_secs: f64) -> Value {
    json!({
        "v": 1,
        "ts_ms": 1_759_000_000_000_u64,
        "kind": kind,
        "position_secs": position_secs,
    })
}

/// Valid sample events covering every `kind`, every closed enum value the
/// schema admits, and every optional field — including the boundary cases
/// (a 200-char pattern-valid `detail`, the supersede link field, an engine
/// with its tier). Each must (a) deserialize into the crate's `RecordEvent`
/// and (b) validate against the committed schema.
fn valid_samples() -> Vec<Value> {
    let mut samples = Vec::new();

    // Every transport state appears on a transport_change entry (FR-020:
    // the surface reflects the Rust module's transport state).
    for transport in ["playing", "paused", "stopped", "completed", "interrupted"] {
        let mut sample = base_event("transport_change", 10.0);
        sample["transport"] = json!(transport);
        samples.push(sample);
    }

    // Every session-end reason appears on a session_end entry, each paired
    // with a transport from the closed-4 — never "playing" (FR-044: the
    // session-end reason and the transport state are one closed set of four
    // values shared with the record).
    for reason in ["paused", "stopped", "completed", "interrupted"] {
        let mut sample = base_event("session_end", 12.5);
        sample["transport"] = json!(reason);
        sample["session_end_reason"] = json!(reason);
        samples.push(sample);
    }

    // Every focus outcome plus the regained direction event. The supersede
    // link field is carried where it applies; per-entry supersession
    // semantics are deliberately NOT asserted here (owned by T035/T067).
    for focus in [
        "duckable_transient",
        "silence_demanding",
        "permanent_loss",
        "regained",
    ] {
        let mut sample = base_event("focus_transition", 10.0);
        sample["focus"] = json!(focus);
        if focus == "permanent_loss" {
            sample["supersedes_ts_ms"] = json!(1_758_999_999_000_u64);
        }
        samples.push(sample);
    }

    // Every named engine error appears on a failure entry and on a refusal
    // entry — `error` is required on both (FR-045, per the 2026-10-02
    // clarification: `error` becomes required for a `refusal` as well as a
    // `failure`, since both name a condition).
    for (kind, error) in [
        ("failure", "engine_unreachable"),
        ("failure", "engine_init_failed"),
        ("failure", "rate_mismatch"),
        ("failure", "channel_mismatch"),
        ("failure", "device_refused"),
        ("failure", "device_lost"),
        ("failure", "start_refused"),
        ("refusal", "engine_unreachable"),
        ("refusal", "engine_init_failed"),
        ("refusal", "rate_mismatch"),
        ("refusal", "channel_mismatch"),
        ("refusal", "device_refused"),
        ("refusal", "device_lost"),
        ("refusal", "start_refused"),
    ] {
        let mut sample = base_event(kind, 10.0);
        sample["error"] = json!(error);
        samples.push(sample);
    }

    // An output-device event (FR-017a: every switch is recorded as an
    // output-device event).
    let mut device_sample = base_event("output_device", 10.0);
    device_sample["device"] = json!({
        "direction": "appeared",
        "device_id": "bt-car-audio",
    });
    samples.push(device_sample);

    // A trim marker: `dropped_events >= 1` and `cutoff_ts_ms` are both
    // required (FR-039b: a trim is visible as a trim, never as an absence).
    let mut trim_sample = base_event("trim", 10.0);
    trim_sample["trim"] = json!({
        "dropped_events": 3_u64,
        "cutoff_ts_ms": 1_758_999_000_000_u64,
    });
    samples.push(trim_sample);

    // A gap marker (storage exhaustion; the reserved-capacity case).
    samples.push(base_event("gap", 10.0));

    // `detail` at its 200-character bound with a pattern-valid
    // identifier=value string (FR-045: `detail` is either typed fields or a
    // bound, non-prose pattern — never free text).
    let mut detail_sample = base_event("failure", 10.0);
    detail_sample["error"] = json!("engine_unreachable");
    detail_sample["detail"] = json!(format!("k={}", "a".repeat(198)));
    samples.push(detail_sample);

    // `engine` implies `engine_tier` (C1): an engine identifier always
    // carries its closed tier, so a test_double identifier is distinguishable
    // from a real engine without a second registry list (2026-10-05
    // clarification).
    let mut engine_sample = base_event("session_start", 0.0);
    engine_sample["engine"] = json!("sherpa-onnx+faber");
    engine_sample["engine_tier"] = json!("offline_default");
    samples.push(engine_sample);

    samples
}

#[test]
fn every_valid_sample_event_conforms_to_the_contract_schema()
-> Result<(), Box<dyn std::error::Error>> {
    let validator = load_validator()?;
    let samples = valid_samples();
    assert!(!samples.is_empty(), "valid_samples() produced no events");

    // The samples are written through the real writer and validated as
    // emitted — the system under test is the writer's output, not a literal
    // copy of the test data.
    let dir = tempfile::tempdir()?;
    let record_path = dir.path().join("diagnostic.jsonl");
    let mut writer = DiagnosticWriter::open(&record_path)?;
    for sample in &samples {
        let event: RecordEvent = serde_json::from_value(sample.clone())
            .map_err(|e| format!("valid sample event is not representable as RecordEvent: {e}"))?;
        writer.append(&event)?;
    }
    drop(writer);

    let written = std::fs::read_to_string(&record_path)?;
    let lines: Vec<&str> = written.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(
        lines.len(),
        samples.len(),
        "the writer dropped or merged records (one JSONL line per event)"
    );

    for (i, line) in lines.iter().enumerate() {
        let record: Value = serde_json::from_str(line)
            .map_err(|e| format!("writer emitted invalid JSON on line {}: {e}", i + 1))?;
        let errors: Vec<_> = validator.iter_errors(&record).collect();
        assert!(
            errors.is_empty(),
            "record {} violated the contract schema: {errors:#?}",
            i + 1
        );
    }
    Ok(())
}

/// The four required root properties are required on every kind — a record
/// missing any of `v/ts_ms/kind/position_secs` is not a valid event. (The
/// negative-control suite in `contract_schema_negative.rs` asserts the planted
/// violations are rejected; this test pins the required-field floor that the
/// schema enforces on the samples above.)
#[test]
fn the_contract_schema_declares_the_required_root_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let validator = load_validator()?;
    let schema_text = std::fs::read_to_string(schema_path())?;
    let schema: Value = serde_json::from_str(&schema_text)?;

    // The schema itself declares the required set (reading the contract back
    // out of the committed artifact keeps this test honest against drift).
    let required = schema["required"]
        .as_array()
        .ok_or_else(|| "schema root has no required array".to_string())?;
    for field in ["v", "ts_ms", "kind", "position_secs"] {
        assert!(
            required.iter().any(|f| f == &json!(field)),
            "schema does not require the root field \"{field}\""
        );
    }

    // And the floor holds on a concrete record: dropping a required field
    // from a valid sample makes it invalid.
    let mut missing = base_event("session_start", 0.0);
    let _ = missing
        .as_object_mut()
        .ok_or_else(|| "base event is not an object".to_string())?
        .remove("position_secs");
    assert!(
        !validator.is_valid(&missing),
        "a record without position_secs was accepted"
    );
    Ok(())
}
