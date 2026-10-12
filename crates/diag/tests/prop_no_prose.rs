//! T068 — diagnostic-record no-prose scanner property tests (Phase-11 RED gate for T074).
//!
//! Task T068 (verbatim): "Property test: no-prose scanner in
//! `crates/diag/tests/prop_no_prose.rs` (proptest + scanner per FR-045,
//! SC-056: zero entries contain document text; `detail` absent-or-`≤200
//! chars` matching non-prose pattern where retained; EVERY `failure` AND
//! `refusal` carries `error`; SC-005 playing-advance rule scan; zero
//! `transport:"playing"` without greater-`position_secs` follow-up; FAIL
//! before implementation)"
//!
//! Invariants under test (FR-039a, FR-045, SC-005, SC-056), all evaluated
//! by `diag::scan::scan_record` in one pass over the JSONL record:
//!
//! - S1 zero prose: for arbitrary conforming event sequences (built only
//!   from the closed sets), the scanner reports ZERO violations — no
//!   document text ever enters the record (FR-039a/FR-045/SC-056).
//! - S2 detail bound: every `detail` is absent or at most 200 characters
//!   and matches the non-prose `key=value` identifier pattern — the
//!   pattern quoted from
//!   `specs/001-android-media-shell/contracts/diagnostic-record.schema.json`:
//!   `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`
//!   (identifier-style `key=value`; reimplemented as a predicate below
//!   because this test target has no regex dependency). The scanner
//!   reports zero `ProseInDetail` and zero `DetailTooLong` violations.
//! - S3 error required: every `failure` AND every `refusal` carries an
//!   `error`; the scanner reports zero `FailureWithoutError` and zero
//!   `RefusalWithoutError` violations.
//! - S4 SC-005 playing-advance rule: every `transport:"playing"` entry is
//!   followed, in the same session, by an entry with strictly greater
//!   `position_secs` (or an intervening non-playing transport,
//!   permanent-loss focus transition, or failure); the scanner reports
//!   zero `PlayingWithoutAdvance` violations.
//! - S5 session-end: no `session_end` entry carries `transport:"playing"`
//!   (the closed-4 never includes playing); the scanner reports zero
//!   `SessionEndWithPlaying` violations.
//! - S6 engine implies tier: every event carrying an `engine` also
//!   carries an `engine_tier`; the scanner reports zero
//!   `EngineWithoutTier` violations.
//!
//! Contract pinned for T074 (the scanner surface these tests compile
//! against — `crates/diag/src/scan.rs`, a new module):
//!
//! - `pub enum ScanViolation` — the closed violation set, exactly seven
//!   kinds: `ProseInDetail { ts_ms }`, `DetailTooLong { ts_ms, len }`,
//!   `FailureWithoutError { ts_ms }`, `RefusalWithoutError { ts_ms }`,
//!   `PlayingWithoutAdvance { ts_ms }`, `SessionEndWithPlaying { ts_ms }`,
//!   `EngineWithoutTier { ts_ms }`.
//! - `pub struct ScanReport { pub violations: Vec<ScanViolation> }` —
//!   every violation found, oldest-first.
//! - `pub fn scan_record(path: &Path) -> Result<ScanReport, DiagError>` —
//!   every check in one pass.
//! - The seven checks, per entry: (1) `ProseInDetail` — `detail` is
//!   present and does not match the non-prose pattern; (2)
//!   `DetailTooLong` — `detail` is present and longer than 200
//!   characters; (3) `FailureWithoutError` — kind is `failure` and `error`
//!   is absent; (4) `RefusalWithoutError` — kind is `refusal` and `error`
//!   is absent; (5) `PlayingWithoutAdvance` — kind is `transport_change`
//!   with `transport == "playing"` and there is no later entry in the same
//!   session with strictly greater `position_secs` and no later
//!   intervening entry (a non-playing `transport`, a `permanent_loss`
//!   focus transition, or a `failure`); (6) `SessionEndWithPlaying` —
//!   kind is `session_end` and `transport == "playing"`; (7)
//!   `EngineWithoutTier` — `engine` is present and `engine_tier` is
//!   absent.
//!
//! FAILS before T074 creates `crates/diag/src/scan.rs`: the
//! `diag::scan` module is unresolved, so this test target does not build.
//! That build failure is the required pre-implementation red state (T072).
//!
//! NOTE: the helpers below are duplicated across the three Phase-11 test
//! targets (`prop_record.rs`, `prop_no_prose.rs`,
//! `prop_scanner_canary.rs`) because exactly three files may be created
//! and a shared test-support module would be a fourth.

use std::time::{SystemTime, UNIX_EPOCH};

use diag::scan::{ScanReport, ScanViolation, scan_record};
use diag::writer::{
    DeviceDirection, DeviceEvent, DiagnosticWriter, EngineTierValue, ErrorValue, EventKind,
    FocusValue, RecordEvent, SessionEndValue, TransportValue,
};
use proptest::prelude::*;

/// The closed-4 (FR-044): the transport states that may appear on a
/// `session_end` entry and the non-playing transports. `playing` is
/// excluded — it is a transport state, never a session-end state and
/// never an SC-005 intervention.
const CLOSED_FOUR: [TransportValue; 4] = [
    TransportValue::Paused,
    TransportValue::Stopped,
    TransportValue::Completed,
    TransportValue::Interrupted,
];

/// The closed session-end reasons, aligned with [`CLOSED_FOUR`] so a
/// session-end entry's transport and reason describe the same ending.
const SESSION_END_REASONS: [SessionEndValue; 4] = [
    SessionEndValue::Paused,
    SessionEndValue::Stopped,
    SessionEndValue::Completed,
    SessionEndValue::Interrupted,
];

/// The closed focus set (three outcomes plus the regained direction).
const FOCUS_VALUES: [FocusValue; 4] = [
    FocusValue::DuckableTransient,
    FocusValue::SilenceDemanding,
    FocusValue::PermanentLoss,
    FocusValue::Regained,
];

/// The closed engine-error set (seven named errors).
const ERROR_VALUES: [ErrorValue; 7] = [
    ErrorValue::EngineUnreachable,
    ErrorValue::EngineInitFailed,
    ErrorValue::RateMismatch,
    ErrorValue::ChannelMismatch,
    ErrorValue::DeviceRefused,
    ErrorValue::DeviceLost,
    ErrorValue::StartRefused,
];

/// The closed engine-tier set (five tiers).
const ENGINE_TIER_VALUES: [EngineTierValue; 5] = [
    EngineTierValue::OfflineDefault,
    EngineTierValue::CloudDefault,
    EngineTierValue::CloudEscalation,
    EngineTierValue::TestDouble,
    EngineTierValue::EvaluatedFallback,
];

/// Device identifiers for output-device events.
const DEVICE_IDS: [&str; 2] = ["bt-car-audio", "usb-speaker"];

/// An engine identifier carried together with its tier.
const ENGINE_ID: &str = "sherpa-onnx+faber";

/// The record version carried by every event (schema const 1).
const RECORD_VERSION: u8 = 1;

/// Test-local error type. Proptest turns any `std::error::Error`
/// returned via `?` from a property body into a test failure
/// (`TestCaseError: From<E: Error>`), so property bodies can use `?`
/// for fallible setup (temp dirs, writer open/append, scanning).
#[derive(Debug, thiserror::Error)]
enum PropError {
    /// A temporary-directory or file operation failed.
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Opening or appending to the diagnostic record failed.
    #[error("diagnostic writer failed: {0}")]
    Diag(#[from] diag::writer::DiagError),
    /// Serializing a record event failed.
    #[error("serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

/// Current wall-clock time in milliseconds since the Unix epoch.
/// Returns 0 when the system clock precedes the epoch (never on a normal
/// system); the value is only ever used as a timestamp anchor.
fn now_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(0),
        Err(_) => 0,
    }
}

/// A record event with every optional field absent.
fn base_event(kind: EventKind, ts_ms: u64, position_secs: f64) -> RecordEvent {
    RecordEvent {
        v: RECORD_VERSION,
        ts_ms,
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

fn session_start(ts_ms: u64, position_secs: f64) -> RecordEvent {
    base_event(EventKind::SessionStart, ts_ms, position_secs)
}

fn transport_change(ts_ms: u64, position_secs: f64, transport: TransportValue) -> RecordEvent {
    let mut event = base_event(EventKind::TransportChange, ts_ms, position_secs);
    event.transport = Some(transport);
    event
}

fn session_end(
    ts_ms: u64,
    position_secs: f64,
    transport: TransportValue,
    reason: SessionEndValue,
) -> RecordEvent {
    let mut event = base_event(EventKind::SessionEnd, ts_ms, position_secs);
    event.transport = Some(transport);
    event.session_end_reason = Some(reason);
    event
}

fn focus_transition(
    ts_ms: u64,
    position_secs: f64,
    focus: FocusValue,
    supersedes_ts_ms: Option<u64>,
) -> RecordEvent {
    let mut event = base_event(EventKind::FocusTransition, ts_ms, position_secs);
    event.focus = Some(focus);
    event.supersedes_ts_ms = supersedes_ts_ms;
    event
}

fn output_device(
    ts_ms: u64,
    position_secs: f64,
    direction: DeviceDirection,
    device_id: &str,
) -> RecordEvent {
    let mut event = base_event(EventKind::OutputDevice, ts_ms, position_secs);
    event.device = Some(DeviceEvent {
        direction,
        device_id: device_id.to_string(),
    });
    event
}

fn failure(
    ts_ms: u64,
    position_secs: f64,
    error: ErrorValue,
    detail: Option<String>,
) -> RecordEvent {
    let mut event = base_event(EventKind::Failure, ts_ms, position_secs);
    event.error = Some(error);
    event.detail = detail;
    event
}

fn refusal(ts_ms: u64, position_secs: f64, error: ErrorValue) -> RecordEvent {
    let mut event = base_event(EventKind::Refusal, ts_ms, position_secs);
    event.error = Some(error);
    event
}

/// The contract's non-prose `detail` pattern, quoted from
/// `specs/001-android-media-shell/contracts/diagnostic-record.schema.json`:
/// `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`
/// — one or more space-separated `key=value` tokens, a non-empty
/// `[A-Za-z0-9_]+` key, a non-empty `[A-Za-z0-9_.+\-]+` value.
/// Reimplemented as a predicate because this test target has no regex
/// dependency; the scanner (T074) is contracted to accept exactly this
/// language.
fn is_conforming_detail(detail: &str) -> bool {
    !detail.is_empty() && detail.split(' ').all(is_conforming_detail_token)
}

/// One `key=value` token of the non-prose detail pattern.
fn is_conforming_detail_token(token: &str) -> bool {
    let Some((key, value)) = token.split_once('=') else {
        return false;
    };
    !key.is_empty()
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_'))
}

/// The closed violation kind of a scanner finding, for counting by kind.
fn violation_kind(violation: &ScanViolation) -> &'static str {
    match violation {
        ScanViolation::ProseInDetail { .. } => "prose_in_detail",
        ScanViolation::DetailTooLong { .. } => "detail_too_long",
        ScanViolation::FailureWithoutError { .. } => "failure_without_error",
        ScanViolation::RefusalWithoutError { .. } => "refusal_without_error",
        ScanViolation::PlayingWithoutAdvance { .. } => "playing_without_advance",
        ScanViolation::SessionEndWithPlaying { .. } => "session_end_with_playing",
        ScanViolation::EngineWithoutTier { .. } => "engine_without_tier",
    }
}

/// The number of violations of `kind` in the report.
fn count_violations(report: &ScanReport, kind: &str) -> usize {
    report
        .violations
        .iter()
        .filter(|violation| violation_kind(violation) == kind)
        .count()
}

/// Writes `events` to a fresh record and scans it.
fn write_and_scan(events: &[RecordEvent]) -> Result<ScanReport, PropError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("record.jsonl");
    let mut writer = DiagnosticWriter::open(&path)?;
    for event in events {
        writer.append(event)?;
    }
    drop(writer);
    Ok(scan_record(&path)?)
}

/// The timestamp and position of the last event. The background always
/// contains at least the `session_start`, so the `None` arm is
/// unreachable; it exists to keep the match total without `unwrap`.
fn last_timestamp_and_position(events: &[RecordEvent]) -> (u64, f64) {
    match events.last() {
        Some(last) => (last.ts_ms, last.position_secs),
        None => (now_ms(), 0.0),
    }
}

/// A generated middle event of a conforming sequence: its kind, whether
/// it carries an `engine`+`engine_tier` pair, and its `detail` choice.
#[derive(Debug, Clone)]
struct MidSpec {
    /// The middle-event kind.
    kind: MidKind,
    /// Whether the event carries `engine` and `engine_tier` together.
    with_engine: bool,
    /// The `detail` choice.
    detail: DetailSpec,
}

/// The middle-event kinds of a conforming sequence.
#[derive(Debug, Clone)]
enum MidKind {
    /// A transport change: `playing` or a generated non-playing state.
    Transport {
        /// Whether the transport is `playing`.
        playing: bool,
        /// The index into [`CLOSED_FOUR`] for the non-playing case.
        non_playing_index: usize,
    },
    /// A focus transition to a generated closed focus value.
    Focus {
        /// The index into [`FOCUS_VALUES`].
        focus_index: usize,
    },
    /// An output-device event.
    Device {
        /// Whether the device appeared.
        appeared: bool,
    },
    /// A failure carrying a generated closed error.
    Failure {
        /// The index into [`ERROR_VALUES`].
        error_index: usize,
    },
    /// A refusal carrying a generated closed error.
    Refusal {
        /// The index into [`ERROR_VALUES`].
        error_index: usize,
    },
}

/// The `detail` choice of a generated event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailSpec {
    /// No `detail` field.
    Absent,
    /// One pattern-conforming `key=value` pair.
    Single,
    /// Two pattern-conforming `key=value` pairs.
    Double,
}

/// The `detail` string for a generated choice.
fn detail_for(spec: DetailSpec) -> Option<String> {
    match spec {
        DetailSpec::Absent => None,
        DetailSpec::Single => Some("engine=sherpa_onnx".to_string()),
        DetailSpec::Double => Some("engine=sherpa_onnx tier=offline_default".to_string()),
    }
}

/// Builds a full conforming sequence: a `session_start`, the generated
/// middle events, and a closing `session_end` — with strictly increasing
/// timestamps (1 ms apart) and strictly increasing positions (1.0 s
/// apart). Timestamps are anchored at the current wall-clock time so
/// every event sits inside the production 30-day retention window. Every
/// middle event conforms: failures and refusals carry an error,
/// permanent-loss transitions carry `supersedes_ts_ms`, engine
/// identifiers carry their tier, and `detail` is absent or
/// pattern-conforming; the closing `session_end` carries a closed-4
/// transport and the matching closed-4 reason, so every `transport_change`
/// to `playing` is followed by strictly greater positions.
fn build_conforming_sequence(mids: &[MidSpec]) -> Vec<RecordEvent> {
    let start_ts = now_ms();
    let mut events = Vec::with_capacity(mids.len() + 2);
    let mut ts = start_ts;
    let mut position = 0.0_f64;
    events.push(session_start(ts, position));
    for (index, mid) in mids.iter().enumerate() {
        ts += 1;
        position += 1.0;
        let mut event = match &mid.kind {
            MidKind::Transport {
                playing,
                non_playing_index,
            } => {
                let transport = if *playing {
                    TransportValue::Playing
                } else {
                    CLOSED_FOUR[*non_playing_index]
                };
                transport_change(ts, position, transport)
            }
            MidKind::Focus { focus_index } => {
                let focus = FOCUS_VALUES[*focus_index];
                let supersedes_ts_ms = if focus == FocusValue::PermanentLoss {
                    Some(start_ts)
                } else {
                    None
                };
                focus_transition(ts, position, focus, supersedes_ts_ms)
            }
            MidKind::Device { appeared } => {
                let direction = if *appeared {
                    DeviceDirection::Appeared
                } else {
                    DeviceDirection::Disappeared
                };
                let device_id = DEVICE_IDS[index % DEVICE_IDS.len()];
                output_device(ts, position, direction, device_id)
            }
            MidKind::Failure { error_index } => {
                let error = ERROR_VALUES[*error_index];
                let detail = detail_for(mid.detail);
                failure(ts, position, error, detail)
            }
            MidKind::Refusal { error_index } => {
                let error = ERROR_VALUES[*error_index];
                refusal(ts, position, error)
            }
        };
        if mid.with_engine {
            event.engine = Some(ENGINE_ID.to_string());
            event.engine_tier = Some(ENGINE_TIER_VALUES[0]);
        }
        events.push(event);
    }
    let (last_ts, last_position) = last_timestamp_and_position(&events);
    let end_index = mids.len() % CLOSED_FOUR.len();
    events.push(session_end(
        last_ts + 1,
        last_position + 1.0,
        CLOSED_FOUR[end_index],
        SESSION_END_REASONS[end_index],
    ));
    events
}

/// The middle-event kinds of a conforming sequence.
fn mid_kind_strategy() -> impl Strategy<Value = MidKind> {
    prop_oneof![
        (any::<bool>(), 0usize..CLOSED_FOUR.len()).prop_map(|(playing, non_playing_index)| {
            MidKind::Transport {
                playing,
                non_playing_index,
            }
        }),
        (0usize..FOCUS_VALUES.len()).prop_map(|focus_index| MidKind::Focus { focus_index }),
        any::<bool>().prop_map(|appeared| MidKind::Device { appeared }),
        (0usize..ERROR_VALUES.len()).prop_map(|error_index| MidKind::Failure { error_index }),
        (0usize..ERROR_VALUES.len()).prop_map(|error_index| MidKind::Refusal { error_index }),
    ]
}

/// The `detail` choice of a generated event.
fn detail_spec_strategy() -> impl Strategy<Value = DetailSpec> {
    prop_oneof![
        Just(DetailSpec::Absent),
        Just(DetailSpec::Single),
        Just(DetailSpec::Double),
    ]
}

/// A generated middle event: its kind, engine pairing, and detail choice.
fn mid_spec_strategy() -> impl Strategy<Value = MidSpec> {
    (mid_kind_strategy(), any::<bool>(), detail_spec_strategy()).prop_map(
        |(kind, with_engine, detail)| MidSpec {
            kind,
            with_engine,
            detail,
        },
    )
}

/// A conforming sequence: zero to fourteen generated middle events
/// plus the leading `session_start` and the closing `session_end`.
fn conforming_sequence_strategy() -> impl Strategy<Value = Vec<RecordEvent>> {
    proptest::collection::vec(mid_spec_strategy(), 0..=14)
        .prop_map(|mids| build_conforming_sequence(&mids))
}

proptest! {
    /// S1 — zero prose: any conforming event sequence (built only from
    /// the closed sets, with absent or pattern-conforming `detail`)
    /// scans with ZERO violations — no document text ever enters the
    /// record (FR-039a, FR-045, SC-056).
    #[test]
    fn conforming_sequences_scan_with_zero_violations(
        events in conforming_sequence_strategy(),
    ) {
        let report = write_and_scan(&events)?;
        prop_assert!(
            report.violations.is_empty(),
            "a conforming record must scan with zero violations, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// S2 — detail bound: every `detail` is absent or at most 200
    /// characters and matches the non-prose `key=value` pattern
    /// `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`;
    /// the scanner therefore reports zero `ProseInDetail` and zero
    /// `DetailTooLong` violations.
    #[test]
    fn every_detail_is_absent_or_bound_and_non_prose(
        events in conforming_sequence_strategy(),
    ) {
        for event in &events {
            let detail = event.detail.as_deref();
            let bound = detail.is_none_or(|text| {
                text.chars().count() <= 200 && is_conforming_detail(text)
            });
            prop_assert!(
                bound,
                "a generated detail violates the 200-character bound or the non-prose pattern: {detail:?}",
                detail = detail,
            );
        }
        let report = write_and_scan(&events)?;
        prop_assert_eq!(
            count_violations(&report, "prose_in_detail"),
            0,
            "a conforming record must scan with zero ProseInDetail violations"
        );
        prop_assert_eq!(
            count_violations(&report, "detail_too_long"),
            0,
            "a conforming record must scan with zero DetailTooLong violations"
        );
    }

    /// S3 — error required: every `failure` AND every `refusal` carries
    /// an `error`; the scanner reports zero `FailureWithoutError` and
    /// zero `RefusalWithoutError` violations (FR-045).
    #[test]
    fn every_failure_and_refusal_carries_an_error(
        events in conforming_sequence_strategy(),
    ) {
        for event in &events {
            let requires_error = matches!(event.kind, EventKind::Failure | EventKind::Refusal);
            if requires_error {
                prop_assert!(
                    event.error.is_some(),
                    "a {kind:?} entry must carry an error",
                    kind = event.kind,
                );
            }
        }
        let report = write_and_scan(&events)?;
        prop_assert_eq!(
            count_violations(&report, "failure_without_error"),
            0,
            "a conforming record must scan with zero FailureWithoutError violations"
        );
        prop_assert_eq!(
            count_violations(&report, "refusal_without_error"),
            0,
            "a conforming record must scan with zero RefusalWithoutError violations"
        );
    }

    /// S4 — SC-005 playing-advance rule: every `transport:"playing"`
    /// entry is followed, in the same session, by an entry with strictly
    /// greater `position_secs` (position advances while playing); the
    /// scanner reports zero `PlayingWithoutAdvance` violations.
    #[test]
    fn every_playing_entry_is_followed_by_strictly_greater_position(
        events in conforming_sequence_strategy(),
    ) {
        for (index, event) in events.iter().enumerate() {
            let is_playing = event.kind == EventKind::TransportChange
                && event.transport == Some(TransportValue::Playing);
            if is_playing {
                let has_advance = events[index + 1..]
                    .iter()
                    .any(|later| later.position_secs > event.position_secs);
                prop_assert!(
                    has_advance,
                    "a playing entry at position {position} has no strictly-greater follow-up",
                    position = event.position_secs,
                );
            }
        }
        let report = write_and_scan(&events)?;
        prop_assert_eq!(
            count_violations(&report, "playing_without_advance"),
            0,
            "a conforming record must scan with zero PlayingWithoutAdvance violations"
        );
    }

    /// S5 — session-end: no `session_end` entry carries
    /// `transport:"playing"` (the closed-4 never includes playing) and
    /// every `session_end` carries its closed-4 reason; the scanner
    /// reports zero `SessionEndWithPlaying` violations (FR-044).
    #[test]
    fn no_session_end_carries_playing(events in conforming_sequence_strategy()) {
        for event in &events {
            if event.kind == EventKind::SessionEnd {
                prop_assert_ne!(
                    event.transport,
                    Some(TransportValue::Playing),
                    "a session_end entry must never carry transport playing"
                );
                prop_assert!(
                    event.session_end_reason.is_some(),
                    "a session_end entry must carry a session_end_reason from the closed-4"
                );
            }
        }
        let report = write_and_scan(&events)?;
        prop_assert_eq!(
            count_violations(&report, "session_end_with_playing"),
            0,
            "a conforming record must scan with zero SessionEndWithPlaying violations"
        );
    }

    /// S6 — engine implies tier: every event carrying an `engine` also
    /// carries an `engine_tier`; the scanner reports zero
    /// `EngineWithoutTier` violations.
    #[test]
    fn every_engine_entry_carries_an_engine_tier(
        events in conforming_sequence_strategy(),
    ) {
        for event in &events {
            if event.engine.is_some() {
                prop_assert!(
                    event.engine_tier.is_some(),
                    "an entry carrying engine {engine:?} must also carry engine_tier",
                    engine = event.engine,
                );
            }
        }
        let report = write_and_scan(&events)?;
        prop_assert_eq!(
            count_violations(&report, "engine_without_tier"),
            0,
            "a conforming record must scan with zero EngineWithoutTier violations"
        );
    }
}
