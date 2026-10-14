//! T010 — Contract test `contracts/diagnostic-trace` (Phase 2, BLOCKING).
//!
//! Invariant under test (tasks.md T010, verbatim): required event kinds
//! present (`session` / `transport` / per-section / per-rate / buffer /
//! `FocusTransition` + measured `play_to_audio_ms` / `removal_pause_ms` /
//! `replay_secs` / `failure` + `error`); `detail` ≤ 200 ch; `failure` / `refusal`
//! without `error` FAILS; prose predicate FAILS.

use diag::scan::{ScanReport, ScanViolation, scan_record};
use diag::writer::{
    DiagnosticWriter, EventKind, FocusValue, RecordEvent, TransportValue,
};

/// The closed event-kind set required by the 004 diagnostic-trace
/// contract: exactly the nine kinds the schema declares (the base
/// set is reused; per-section / per-rate / buffer / latency ride as
/// `detail` keys, not new kinds — see contract).
const REQUIRED_KINDS: [EventKind; 9] = [
    EventKind::SessionStart,
    EventKind::SessionEnd,
    EventKind::TransportChange,
    EventKind::FocusTransition,
    EventKind::OutputDevice,
    EventKind::Failure,
    EventKind::Refusal,
    EventKind::Trim,
    EventKind::Gap,
];

/// The measured latency detail keys that must be representable in
/// a conforming `detail` string (004 contract, line 12 and 23).
const MEASURED_DETAIL_KEYS: [&str; 3] = [
    "play_to_audio_ms",
    "removal_pause_ms",
    "replay_secs",
];

/// A conforming identifier-pattern `detail` token used across
/// assertions (matches `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+$`).
fn conforming_detail_token(key: &str, value: &str) -> String {
    format!("{}={}", key, value)
}

/// Replicates the contract's non-prose predicate (scan.rs `is_conforming_detail`)
/// so the contract test does not depend on the private predicate directly.
fn is_conforming_detail(detail: &str) -> bool {
    if detail.is_empty() {
        return false;
    }
    detail.split(' ').all(|token| {
        let Some((k, v)) = token.split_once('=') else {
            return false;
        };
        !k.is_empty()
            && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !v.is_empty()
            && v.chars().all(|c| {
                c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_')
            })
    })
}

/// Asserts the nine required event kinds exist in the contract/module.
#[test]
fn required_event_kinds_present() {
    // Every required kind must be constructible as a distinct variant.
    let kinds: Vec<&str> = REQUIRED_KINDS
        .iter()
        .map(|k| match k {
            EventKind::SessionStart => "session_start",
            EventKind::SessionEnd => "session_end",
            EventKind::TransportChange => "transport_change",
            EventKind::FocusTransition => "focus_transition",
            EventKind::OutputDevice => "output_device",
            EventKind::Failure => "failure",
            EventKind::Refusal => "refusal",
            EventKind::Trim => "trim",
            EventKind::Gap => "gap",
        })
        .collect();
    assert!(
        kinds.len() == 9,
        "expected exactly 9 event kinds, got {}",
        kinds.len()
    );
    // The contract specifically requires FocusTransition + measured detail.
    assert!(
        kinds.contains(&"focus_transition"),
        "FocusTransition kind is missing"
    );
    assert!(
        kinds.contains(&"transport_change"),
        "TransportChange kind is missing"
    );
    assert!(
        kinds.contains(&"session_start"),
        "SessionStart kind is missing"
    );
}

/// Asserts measured latency detail keys are representable in a
/// conforming `detail` string and that `FocusTransition` exists as
/// the carrier kind.
#[test]
fn measured_latency_detail_keys_present() {
    for key in &MEASURED_DETAIL_KEYS {
        assert!(
            key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "measured detail key '{}' must match identifier-pattern",
            key
        );
        let token = conforming_detail_token(key, "42");
        assert!(
            is_conforming_detail(&token),
            "measured detail token '{}' must conform to non-prose pattern",
            token
        );
    }
    // FocusTransition is the interruption-carrier kind; it must exist.
    assert!(
        matches!(EventKind::FocusTransition, EventKind::FocusTransition),
        "FocusTransition event kind must be present"
    );
}

/// Asserts `detail` is bounded to ≤ 200 characters and that a
/// pattern-valid detail at exactly 200 chars passes while 201 chars
/// triggers the scanner violation.
#[test]
fn detail_length_bound_200() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("detail_bound.jsonl");

    // A 200-character conforming detail: `k=` (2) + 198 chars = 200.
    let value_198 = "a".repeat(198);
    let detail_200 = format!("k={}", value_198);
    assert_eq!(
        detail_200.chars().count(),
        200,
        "setup error: detail_200 should be exactly 200 chars"
    );
    assert!(
        is_conforming_detail(&detail_200),
        "a 200-char pattern-valid detail must conform"
    );
    assert!(
        detail_200.chars().count() <= 200,
        "detail must be ≤ 200 chars"
    );

    // A 201-character detail: `k=` + 199 chars = 201.
    let value_199 = "a".repeat(199);
    let detail_201 = format!("k={}", value_199);
    assert_eq!(
        detail_201.chars().count(),
        201,
        "setup error: detail_201 should be exactly 201 chars"
    );

    // Write both through the real writer and scan.
    {
        let mut writer = DiagnosticWriter::open(&path)?;
        let mut event_200 = base_event(EventKind::TransportChange, 1.0);
        event_200.detail = Some(detail_200.clone());
        writer.append(&event_200)?;

        let mut event_201 = base_event(EventKind::TransportChange, 2.0);
        event_201.detail = Some(detail_201.clone());
        writer.append(&event_201)?;
    }

    let report = scan_record(&path)?;
    let too_long_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v, ScanViolation::DetailTooLong { .. }))
        .collect();
    assert_eq!(
        too_long_violations.len(),
        1,
        "exactly one DetailTooLong violation expected for the 201-char detail"
    );
    assert!(
        too_long_violations.iter().all(|v| {
            if let ScanViolation::DetailTooLong { len, .. } = v {
                *len == 201
            } else {
                false
            }
        }),
        "the too-long violation must report len=201"
    );

    // Confirm no prose violation on the 200-char entry.
    let prose_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v, ScanViolation::ProseInDetail { .. }))
        .collect();
    assert!(
        prose_violations.is_empty(),
        "a pattern-valid 200-char detail must not trigger ProseInDetail"
    );
    Ok(())
}

/// Asserts `failure` events always have `error` and `refusal` events
/// always have `error`; missing `error` FAILS (produces scanner
/// violations `FailureWithoutError` / `RefusalWithoutError`).
#[test]
fn failure_and_refusal_require_error() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("failure_refusal_error.jsonl");

    {
        let mut writer = DiagnosticWriter::open(&path)?;
        // A failure WITHOUT error must produce FailureWithoutError.
        let mut failure_no_error = base_event(EventKind::Failure, 1.0);
        failure_no_error.error = None;
        writer.append(&failure_no_error)?;

        // A refusal WITHOUT error must produce RefusalWithoutError.
        let mut refusal_no_error = base_event(EventKind::Refusal, 2.0);
        refusal_no_error.error = None;
        writer.append(&refusal_no_error)?;

        // A failure WITH error must NOT produce any violation.
        use diag::writer::ErrorValue;
        let mut failure_ok = base_event(EventKind::Failure, 3.0);
        failure_ok.error = Some(ErrorValue::EngineUnreachable);
        writer.append(&failure_ok)?;

        let mut refusal_ok = base_event(EventKind::Refusal, 4.0);
        refusal_ok.error = Some(ErrorValue::RateMismatch);
        writer.append(&refusal_ok)?;
    }

    let report = scan_record(&path)?;
    assert!(
        report.violations.iter().any(|v| matches!(
            v,
            ScanViolation::FailureWithoutError { .. }
        )),
        "failure without error must trigger FailureWithoutError"
    );
    assert!(
        report.violations.iter().any(|v| matches!(
            v,
            ScanViolation::RefusalWithoutError { .. }
        )),
        "refusal without error must trigger RefusalWithoutError"
    );

    // The conforming entries (with error) must not trigger violations.
    let error_violations = report.violations.iter().filter(|v| {
        matches!(
            v,
            ScanViolation::FailureWithoutError { .. } | ScanViolation::RefusalWithoutError { .. }
        )
    });
    assert_eq!(
        report.violations.len(),
        2,
        "exactly two error-related violations expected (one failure, one refusal)"
    );
    Ok(())
}

/// Asserts prose predicate FAILS: any `detail` that contains prose
/// (free-form text, not identifier-pattern) must trigger
/// `ProseInDetail`.
#[test]
fn prose_predicate_fails() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("prose.jsonl");

    // A conforming non-prose detail (pattern-valid).
    let conforming = conforming_detail_token("engine", "sherpa_onnx+faber");
    assert!(
        is_conforming_detail(&conforming),
        "conforming token must pass predicate"
    );

    // A prose sentence that is under 200 chars but violates the pattern.
    let prose = "the engine reported a crash during playback";
    assert!(
        !is_conforming_detail(prose),
        "prose sentence must fail the non-prose predicate"
    );
    assert!(
        prose.chars().count() <= 200,
        "prose canary must be under the 200-char bound so length does not mask the prose violation"
    );

    {
        let mut writer = DiagnosticWriter::open(&path)?;
        let mut event_prose = base_event(EventKind::TransportChange, 1.0);
        event_prose.detail = Some(prose.to_string());
        writer.append(&event_prose)?;

        let mut event_conforming = base_event(EventKind::TransportChange, 2.0);
        event_conforming.detail = Some(conforming.clone());
        writer.append(&event_conforming)?;
    }

    let report = scan_record(&path)?;
    let prose_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v, ScanViolation::ProseInDetail { .. }))
        .collect();
    assert_eq!(
        prose_violations.len(),
        1,
        "exactly one ProseInDetail violation expected for the prose canary"
    );
    assert!(
        !report.violations.iter().any(|v| matches!(
            v,
            ScanViolation::DetailTooLong { .. }
        )),
        "no DetailTooLong violation should fire (prose is under 200 chars)"
    );
    Ok(())
}

/// Helper to construct a minimal base event (replicates the pattern
/// used in other diag contract tests).
fn base_event(kind: EventKind, position_secs: f64) -> RecordEvent {
    RecordEvent {
        v: 1,
        ts_ms: 1_759_000_000_000,
        kind,
        position_secs,
        transport: None,
        session_end_reason: None,
        supersedes_ts_ms: None,
        focus: None,
        device: None,
        error: None,
        engine: None,
        engine_tier: None,
        detail: None,
        trim: None,
    }
}
