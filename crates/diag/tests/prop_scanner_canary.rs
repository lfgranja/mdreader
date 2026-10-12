//! T068b — diagnostic-record scanner negative controls (Phase-11 RED gate for T074).
//!
//! Task T068b (verbatim): "Property test: scanner negative controls in
//! `crates/diag/tests/prop_scanner_canary.rs` (proptest over planted
//! non-conforming records: assert the T068/T074 scanners report
//! **non-zero** for each canary — `transport:"playing"` with no
//! strictly-greater `position_secs` follow-up (SC-005), prose inside
//! `detail`, `detail` >200 chars, `failure` without `error`, `refusal`
//! without `error`, `session_end` with `transport:"playing"`, an
//! `engine` without `engine_tier`; and report zero only for conforming
//! records. Every scanner assertion elsewhere is one-directional - it
//! asserts a clean report on conforming data - so a scanner that can
//! never fail would pass all of them while proving nothing. This is the
//! only assertion that distinguishes a scanner from a no-op. FAILS
//! before the scanners exist)"
//!
//! Each property plants exactly one non-conforming canary record inside
//! an otherwise conforming, proptest-generated background (see
//! `prop_no_prose.rs` for the generator) and asserts the scanner reports
//! a NON-EMPTY report in which every violation is the canary's own
//! matching kind — the canary is detected, and detected as exactly its
//! own violation kind:
//!
//! - C1 SC-005: a `transport:"playing"` entry with no strictly-greater
//!   `position_secs` follow-up → `PlayingWithoutAdvance`. Two consecutive
//!   playing entries at the same position, at the end of the record: the
//!   first has no later entry with a greater position and no intervening
//!   non-playing transport/permanent-loss/failure, and the second has no
//!   follow-up at all — a violation under every reading of the rule.
//! - C2: prose inside `detail` (a full sentence of document text) →
//!   `ProseInDetail`.
//! - C3: `detail` longer than 200 characters (202 chars, still
//!   pattern-conforming so only the length bound is violated) →
//!   `DetailTooLong`.
//! - C4: a `failure` entry with no `error` → `FailureWithoutError`.
//! - C5: a `refusal` entry with no `error` → `RefusalWithoutError`.
//! - C6: a `session_end` entry with `transport:"playing"` →
//!   `SessionEndWithPlaying`.
//! - C7: an entry carrying an `engine` with no `engine_tier` →
//!   `EngineWithoutTier`.
//! - C8 (control): a fully conforming record set reports ZERO
//!   violations — the canaries above are the only reason the scanner's
//!   non-zero reports mean anything.
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
//! - The SC-005 rule: a `transport_change` entry with
//!   `transport == "playing"` violates `PlayingWithoutAdvance` iff there
//!   is no later entry in the same session with strictly greater
//!   `position_secs` and no later intervening entry (a non-playing
//!   `transport`, a `permanent_loss` focus transition, or a `failure`).
//!
//! FAILS before T074 creates `crates/diag/src/scan.rs`: the
//! `diag::scan` module is unresolved, so this test target does not
//! build. That build failure is the required pre-implementation red
//! state (T072).
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

/// Document text planted inside `detail`: a full sentence that cannot
/// match the non-prose `key=value` pattern (and is under the
/// 200-character bound, so only the prose rule fires).
const PROSE_CANARY: &str = "The quick brown fox jumps over the lazy dog while the screen stays off and playback continues uninterrupted.";

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
    error: Option<ErrorValue>,
    detail: Option<String>,
) -> RecordEvent {
    let mut event = base_event(EventKind::Failure, ts_ms, position_secs);
    event.error = error;
    event.detail = detail;
    event
}

fn refusal(ts_ms: u64, position_secs: f64, error: Option<ErrorValue>) -> RecordEvent {
    let mut event = base_event(EventKind::Refusal, ts_ms, position_secs);
    event.error = error;
    event
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

/// Whether the report is non-empty and every violation is of `kind` —
/// the canary was detected, and detected as exactly its own violation
/// kind.
fn is_exactly_kind(report: &ScanReport, kind: &str) -> bool {
    !report.violations.is_empty()
        && report
            .violations
            .iter()
            .all(|violation| violation_kind(violation) == kind)
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

/// The timestamp and position of the last event, for appending a canary
/// after it. The background always contains at least the `session_start`,
/// so the `None` arm is unreachable; it exists to keep the match total
/// without `unwrap`.
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

/// Builds the conforming background: a `session_start` plus the
/// generated middle events, with strictly increasing timestamps (1 ms
/// apart) and strictly increasing positions (1.0 s apart). Timestamps
/// are anchored at the current wall-clock time so every event sits
/// inside the production 30-day retention window, and the background
/// deliberately ends without a `session_end` so a planted canary (or a
/// conforming session end) can be appended after it.
fn build_conforming_background(mids: &[MidSpec]) -> Vec<RecordEvent> {
    let start_ts = now_ms();
    let mut events = Vec::with_capacity(mids.len() + 1);
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
                failure(ts, position, Some(error), detail)
            }
            MidKind::Refusal { error_index } => {
                let error = ERROR_VALUES[*error_index];
                refusal(ts, position, Some(error))
            }
        };
        if mid.with_engine {
            event.engine = Some(ENGINE_ID.to_string());
            event.engine_tier = Some(ENGINE_TIER_VALUES[0]);
        }
        events.push(event);
    }
    events
}

/// A conforming background: zero to fourteen generated middle events.
fn conforming_background_strategy() -> impl Strategy<Value = Vec<RecordEvent>> {
    proptest::collection::vec(mid_spec_strategy(), 0..=14)
        .prop_map(|mids| build_conforming_background(&mids))
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

/// A conforming sequence: the background plus a closed-4 `session_end`.
fn conforming_sequence_strategy() -> impl Strategy<Value = Vec<RecordEvent>> {
    conforming_background_strategy().prop_map(|background| {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        let end_index = mids_len_of(&events) % CLOSED_FOUR.len();
        events.push(session_end(
            last_ts + 1,
            last_position + 1.0,
            CLOSED_FOUR[end_index],
            SESSION_END_REASONS[end_index],
        ));
        events
    })
}

/// The number of middle events in a built background: the event count
/// minus the leading `session_start`.
fn mids_len_of(events: &[RecordEvent]) -> usize {
    events.len().saturating_sub(1)
}

proptest! {
    /// C1 — SC-005 canary: a `transport:"playing"` entry with no
    /// strictly-greater `position_secs` follow-up. Two consecutive
    /// playing entries at the same position, at the end of the record:
    /// the first has no later entry with a greater position and no
    /// intervening non-playing transport/permanent-loss/failure, and
    /// the second has no follow-up at all — a violation under every
    /// reading of the rule. The scanner must report non-zero, and every
    /// reported violation must be `PlayingWithoutAdvance`.
    #[test]
    fn canary_playing_without_advance_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        let canary_position = last_position + 1.0;
        events.push(transport_change(
            last_ts + 1,
            canary_position,
            TransportValue::Playing,
        ));
        events.push(transport_change(
            last_ts + 2,
            canary_position,
            TransportValue::Playing,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "playing_without_advance"),
            "the playing_without_advance canary must be reported as exactly PlayingWithoutAdvance, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C2 — prose canary: document text inside `detail` — a full
    /// sentence, which cannot match the non-prose `key=value` pattern
    /// (and is under the 200-character bound, so only the prose rule
    /// fires). The scanner must report non-zero, and every reported
    /// violation must be `ProseInDetail`.
    #[test]
    fn canary_prose_in_detail_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        let mut canary = transport_change(
            last_ts + 1,
            last_position + 1.0,
            TransportValue::Paused,
        );
        canary.detail = Some(PROSE_CANARY.to_string());
        events.push(canary);
        let (end_ts, end_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            end_ts + 1,
            end_position + 1.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "prose_in_detail"),
            "the prose_in_detail canary must be reported as exactly ProseInDetail, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C3 — detail-too-long canary: a `detail` over 200 characters that
    /// still matches the non-prose pattern (a 1-character key, `=`, and a
    /// 200-character value — 202 characters), so only the length bound is
    /// violated. The scanner must report non-zero, and every reported
    /// violation must be `DetailTooLong`.
    #[test]
    fn canary_detail_too_long_is_detected(
        background in conforming_background_strategy(),
    ) {
        let long_detail = format!("k={}", "a".repeat(200));
        prop_assert!(
            long_detail.chars().count() > 200,
            "the too-long detail canary must exceed the 200-character bound, got {len}",
            len = long_detail.chars().count(),
        );
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        let mut canary = transport_change(
            last_ts + 1,
            last_position + 1.0,
            TransportValue::Paused,
        );
        canary.detail = Some(long_detail);
        events.push(canary);
        let (end_ts, end_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            end_ts + 1,
            end_position + 1.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "detail_too_long"),
            "the detail_too_long canary must be reported as exactly DetailTooLong, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C4 — failure-without-error canary: a `failure` entry with no
    /// `error` field. The scanner must report non-zero, and every
    /// reported violation must be `FailureWithoutError`.
    #[test]
    fn canary_failure_without_error_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        // The error field is deliberately absent.
        events.push(failure(last_ts + 1, last_position + 1.0, None, None));
        let (end_ts, end_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            end_ts + 1,
            end_position + 1.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "failure_without_error"),
            "the failure_without_error canary must be reported as exactly FailureWithoutError, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C5 — refusal-without-error canary: a `refusal` entry with no
    /// `error` field. The scanner must report non-zero, and every
    /// reported violation must be `RefusalWithoutError`.
    #[test]
    fn canary_refusal_without_error_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        // The error field is deliberately absent.
        events.push(refusal(last_ts + 1, last_position + 1.0, None));
        let (end_ts, end_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            end_ts + 1,
            end_position + 1.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "refusal_without_error"),
            "the refusal_without_error canary must be reported as exactly RefusalWithoutError, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C6 — session-end-with-playing canary: a `session_end` entry
    /// carrying `transport:"playing"` — playing is a transport state,
    /// never a session-end state (the closed-4 never includes it). The
    /// scanner must report non-zero, and every reported violation must be
    /// `SessionEndWithPlaying`.
    #[test]
    fn canary_session_end_with_playing_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            last_ts + 1,
            last_position + 1.0,
            TransportValue::Playing,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "session_end_with_playing"),
            "the session_end_with_playing canary must be reported as exactly SessionEndWithPlaying, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C7 — engine-without-tier canary: an entry carrying an `engine`
    /// identifier with no `engine_tier`. The scanner must report
    /// non-zero, and every reported violation must be
    /// `EngineWithoutTier`.
    #[test]
    fn canary_engine_without_tier_is_detected(
        background in conforming_background_strategy(),
    ) {
        let mut events = background;
        let (last_ts, last_position) = last_timestamp_and_position(&events);
        let mut canary = transport_change(
            last_ts + 1,
            last_position + 1.0,
            TransportValue::Paused,
        );
        canary.engine = Some(ENGINE_ID.to_string());
        // The engine_tier field is deliberately absent.
        events.push(canary);
        let (end_ts, end_position) = last_timestamp_and_position(&events);
        events.push(session_end(
            end_ts + 1,
            end_position + 1.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ));
        let report = write_and_scan(&events)?;
        prop_assert!(
            is_exactly_kind(&report, "engine_without_tier"),
            "the engine_without_tier canary must be reported as exactly EngineWithoutTier, got {violations:?}",
            violations = &report.violations,
        );
    }

    /// C8 — conforming control: a fully conforming record set reports
    /// ZERO violations. The canaries above are the only reason the
    /// scanner's non-zero reports mean anything; this control proves the
    /// conforming background alone never trips them.
    #[test]
    fn conforming_records_report_zero_violations(
        events in conforming_sequence_strategy(),
    ) {
        let report = write_and_scan(&events)?;
        prop_assert!(
            report.violations.is_empty(),
            "a conforming record must scan with zero violations, got {violations:?}",
            violations = &report.violations,
        );
    }
}
