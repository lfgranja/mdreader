//! T011b — negative-control schema test for the diagnostic record
//! (Phase 2, BLOCKING).
//!
//! Invariant under test (tasks.md T011b, verbatim): "asserts the
//! committed schema **REJECTS** each planted violation: `failure`
//! without `error`; `refusal` without `error`; `session_end` with
//! `transport:"playing"`; `detail` of 201 chars; `detail` containing
//! prose; `engine` without `engine_tier`; unknown `kind`; unknown
//! property. A validator that accepts a planted violation is a broken
//! validator, not a passing test — this is the only assertion that can
//! tell a real schema check from a no-op."
//!
//! Why the negative control exists: every other schema assertion
//! (T011) is one-directional — it asserts a clean report on conforming
//! data. A validator that accepts everything would pass all of them
//! while proving nothing. This test plants each violation the schema
//! is required to catch and asserts rejection, which is the only way
//! to distinguish a real schema check from a no-op.
//!
//! FAILS before `crates/diag/src/writer.rs` exists (T073): the
//! `diag::writer` module is unresolved, so this test target does not
//! build. That build failure is the required pre-implementation red
//! state. (The positive control writes one valid record through the
//! real writer so the rejections below are measured against a
//! validator that is demonstrably able to accept.)
//!
//! Expected API contract for T073 (`crates/diag/src/writer.rs`):
//! - `pub struct RecordEvent` — `Serialize` + `Deserialize`, serde
//!   field names exactly the schema property names, absent options
//!   omitted on the wire
//! - `pub struct DiagnosticWriter` with `pub fn open(path: &Path) ->
//!   Result<Self, DiagError>` and `pub fn append(&mut self, event:
//!   &RecordEvent) -> Result<(), DiagError>` (append-only JSONL, one
//!   record per line)
//!
//! (Full field list documented in `contract_schema.rs`, T011.)

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
fn base_event(kind: &str) -> Value {
    json!({
        "v": 1,
        "ts_ms": 1_759_000_000_000_u64,
        "kind": kind,
        "position_secs": 10.0,
    })
}

/// One valid record, emitted through the real writer, used as the
/// positive control: if the validator rejects this, every rejection
/// below would be vacuous (a reject-everything no-op passes any
/// negative test).
fn valid_control_event() -> Value {
    let mut control = base_event("failure");
    control["error"] = json!("engine_unreachable");
    control["engine"] = json!("sherpa-onnx+faber");
    control["engine_tier"] = json!("offline_default");
    control["detail"] = json!("engine=sherpa-onnx+faber");
    control
}

/// The eight planted violations, each with a name for the failure
/// message. Each is a minimal mutation of a valid record that
/// violates exactly one rule of the committed schema.
fn planted_violations() -> Vec<(&'static str, Value)> {
    let mut violations = Vec::new();

    // 1. `failure` without `error` — `error` is required on a
    //    failure (FR-045, per the 2026-10-02 clarification).
    violations.push(("failure without error", base_event("failure")));

    // 2. `refusal` without `error` — `error` is required on a
    //    refusal as well as a failure (both name a condition).
    violations.push(("refusal without error", base_event("refusal")));

    // 3. `session_end` with `transport: "playing"` — a session-end
    //    reason is one of the closed-4 (paused|stopped|completed|
    //    interrupted); `playing` is a transport state and is never a
    //    session-end transport (FR-044).
    let mut session_end_playing = base_event("session_end");
    session_end_playing["transport"] = json!("playing");
    session_end_playing["session_end_reason"] = json!("paused");
    violations.push(("session_end with transport playing", session_end_playing));

    // 4. `detail` of 201 chars — maxLength is 200. The string is
    //    pattern-valid (a single identifier=value pair), so the
    //    rejection isolates the length bound rather than the pattern.
    let mut long_detail = base_event("failure");
    long_detail["error"] = json!("engine_unreachable");
    long_detail["detail"] = json!(format!("k={}", "a".repeat(199)));
    violations.push(("detail of 201 chars", long_detail));

    // 5. `detail` containing prose — the pattern admits only
    //    identifier=value pairs (FR-045: no entry may contain
    //    document text; an unbounded field is exactly where a
    //    document sentence would end up by accident).
    let mut prose_detail = base_event("failure");
    prose_detail["error"] = json!("engine_unreachable");
    prose_detail["detail"] = json!("engine=sherpa-onnx+faber the reader reported a crash");
    violations.push(("detail containing prose", prose_detail));

    // 6. `engine` without `engine_tier` — an engine identifier
    //    implies its tier (C1), so a stand-in is distinguishable
    //    from a real engine without a second registry list
    //    (2026-10-05 clarification).
    let mut engine_no_tier = base_event("session_start");
    engine_no_tier["engine"] = json!("sherpa-onnx+faber");
    violations.push(("engine without engine_tier", engine_no_tier));

    // 7. unknown `kind` — the event-kind set is closed
    //    (session_start|session_end|transport_change|
    //    focus_transition|output_device|failure|refusal|trim|gap).
    let mut unknown_kind = base_event("exploded");
    unknown_kind["error"] = json!("engine_unreachable");
    violations.push(("unknown kind", unknown_kind));

    // 8. unknown property — additionalProperties is false: the
    //    record carries exactly the declared fields and nothing
    //    else (no free-form extension point where prose could
    //    hide).
    let mut unknown_property = base_event("session_start");
    unknown_property["voice"] = json!("cadu");
    violations.push(("unknown property", unknown_property));

    violations
}

#[test]
fn the_schema_rejects_every_planted_violation() -> Result<(), Box<dyn std::error::Error>> {
    let validator = load_validator()?;

    // Positive control: the validator accepts a valid writer-emitted
    // record, so a rejection below is a real rejection, not a
    // reject-everything no-op.
    let dir = tempfile::tempdir()?;
    let record_path = dir.path().join("control.jsonl");
    let control_event: RecordEvent = serde_json::from_value(valid_control_event())
        .map_err(|e| format!("control event not representable: {e}"))?;
    {
        let mut writer = DiagnosticWriter::open(&record_path)?;
        writer.append(&control_event)?;
    }
    let written = std::fs::read_to_string(&record_path)?;
    let line = written.trim();
    let control: Value = serde_json::from_str(line)?;
    assert!(
        validator.is_valid(&control),
        "positive control was rejected — the validator rejects valid records, so its rejections below prove nothing"
    );

    for (name, violation) in planted_violations() {
        assert!(
            validator.iter_errors(&violation).next().is_some(),
            "planted violation ACCEPTED by the schema (broken validator, not a passing test): {name}\nplanted record: {violation}"
        );
    }
    Ok(())
}

/// The 200-character bound is inclusive: a pattern-valid detail of
/// exactly 200 characters is accepted, so the 201-char rejection in
/// the suite above is the length bound and nothing else.
#[test]
fn the_detail_bound_is_inclusive_at_200_characters() -> Result<(), Box<dyn std::error::Error>> {
    let validator = load_validator()?;

    let mut at_bound = base_event("failure");
    at_bound["error"] = json!("engine_unreachable");
    at_bound["detail"] = json!(format!("k={}", "a".repeat(198)));

    assert!(
        validator.is_valid(&at_bound),
        "a pattern-valid detail of exactly 200 characters was rejected — the maxLength 200 bound is not inclusive"
    );
    Ok(())
}
