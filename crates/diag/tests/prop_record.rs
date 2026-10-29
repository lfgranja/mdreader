//! T067 — diagnostic-record bounds + trim/gap property tests (Phase-11 RED gate for T073).
//!
//! Task T067 (verbatim): "Property test: diagnostic bounds + trim/gap in
//! `crates/diag/tests/prop_record.rs` (proptest invariants: 30-day window +
//! 10 MB cap oldest-first; cap-reached → trim event `{dropped_events ≥ 1,
//! cutoff_ts_ms}` never silent absence; storage-full → gap marker via reserved
//! capacity, playback uninterrupted; trim/gap markers flushed before any trim
//! of retained entries; every transport change carries `position_secs`;
//! session-end carries `transport`+`session_end_reason` from shared closed-4;
//! superseding permanent-loss entry carries `supersedes_ts_ms`; FAIL before
//! implementation)"
//!
//! Invariants under test (FR-039, FR-039b, FR-044):
//!
//! - P1 retention window: with a small retention bound (1 day), events with
//!   `ts_ms` older than the window are trimmed oldest-first; events inside
//!   the window are retained; `retained_entries()` reflects exactly the
//!   in-window count. The window is anchored at the wall-clock time of the
//!   append (the production bound is `RETENTION_DAYS`; tests use a 1-day
//!   bound to exercise the same code path deterministically). The first
//!   event is an in-window anchor so the expectation is identical whether
//!   the implementation anchors the window at open/append wall-clock time
//!   or at the newest event timestamp.
//! - P2 size cap oldest-first: with a small size cap (1 KiB), appending
//!   many events trims the oldest until `retained_bytes() <= cap`; the
//!   trim marker on disk has `dropped_events >= 1` and a `cutoff_ts_ms`;
//!   and the marker line is flushed BEFORE the retained entries it
//!   describes — on disk the last trim line precedes the first retained
//!   line (FR-039: the marker survives the trim it describes).
//! - P3 never silent absence: whenever a trim removed entries, a trim
//!   marker exists on disk accounting for every removed event — never a
//!   silent shrink; and when nothing was removed, no trim marker exists.
//! - P4 gap marker: a storage-full append failure (a write to `/dev/full`,
//!   the POSIX ENOSPC device) records a gap marker (`gap_count() > 0`)
//!   and `append` returns `Ok` — best-effort, playback uninterrupted;
//!   the failure is contained, never a panic/unwrap.
//! - P5 field conformance: every `transport_change` entry carries
//!   `position_secs` and `transport`; every `session_end` entry carries
//!   `transport` + `session_end_reason` from the shared closed-4 (never
//!   `playing`); a superseding permanent-loss entry carries
//!   `supersedes_ts_ms` linking the entry it supersedes.
//! - P6 determinism: identical event sequences produce identical retained
//!   state — the same retained entry count, the same retained byte
//!   length, and byte-identical record files.
//!
//! Contract pinned for T073 (the surface these tests compile against):
//!
//! - `DiagnosticWriter::open_with_bounds(path, retention_days,
//!   size_cap_bytes)` — the test seam; the const bounds
//!   (`RETENTION_DAYS`, `SIZE_CAP_BYTES`) are the production values.
//! - `DiagnosticWriter::retained_entries() -> usize` — the retained
//!   record-event count; trim/gap markers are metadata, not entries.
//! - `DiagnosticWriter::retained_bytes() -> u64` — the total serialized
//!   byte length of the retained record events; markers excluded.
//! - `DiagnosticWriter::gap_count() -> u64` — gap markers recorded
//!   since open.
//! - A trim emits `EventKind::Trim` with
//!   `TrimMarker { dropped_events >= 1, cutoff_ts_ms }`, flushed to disk
//!   BEFORE any retained entry is removed; after any trim the on-disk
//!   line order is [trim markers][retained entries].
//! - A storage-full write failure records `EventKind::Gap` and returns
//!   `Ok` (the append write path is synchronous and unbuffered, so the
//!   failure surfaces inside `append`).
//!
//! FAILS before T073 extends `crates/diag/src/writer.rs`:
//! `open_with_bounds`, `retained_entries`, `retained_bytes`, `gap_count`,
//! `RETENTION_DAYS`, and `SIZE_CAP_BYTES` do not exist yet, so this test
//! target does not build. That build failure is the required
//! pre-implementation red state (T072).
//!
//! NOTE: the helpers below are duplicated across the three Phase-11 test
//! targets (`prop_record.rs`, `prop_no_prose.rs`,
//! `prop_scanner_canary.rs`) because exactly three files may be created
//! and a shared test-support module would be a fourth.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use diag::writer::{
    DiagError, DiagnosticWriter, EventKind, FocusValue, RETENTION_DAYS, RecordEvent,
    SIZE_CAP_BYTES, SessionEndValue, TransportValue, TrimMarker,
};
use proptest::prelude::*;
use serde_json::Value;

/// One day in milliseconds (retention-window arithmetic).
const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// The small retention bound (1 day) used to exercise the window path.
const TEST_RETENTION_DAYS: u64 = 1;

/// The small size cap (1 KiB) used to exercise the cap path: sixteen
/// minimal events already exceed it, so every generated case trims, and
/// a single event always fits, so the cap trim always keeps the newest
/// entry.
const TEST_SIZE_CAP_BYTES: u64 = 1024;

/// The record version carried by every event (schema const 1).
const RECORD_VERSION: u8 = 1;

/// The shared closed-4 (FR-044): the transport states that may appear on
/// a `session_end` entry and the non-playing transports. `playing` is
/// excluded — it is a transport state, never a session-end state.
const CLOSED_FOUR: [TransportValue; 4] = [
    TransportValue::Paused,
    TransportValue::Stopped,
    TransportValue::Completed,
    TransportValue::Interrupted,
];

/// Test-local error type. Proptest turns any `std::error::Error`
/// returned via `?` from a property body into a test failure
/// (`TestCaseError: From<E: Error>`), so property bodies can use `?`
/// for fallible setup (temp dirs, writer open/append, file reads).
#[derive(Debug, thiserror::Error)]
enum PropError {
    /// A temporary-directory or file operation failed.
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Opening or appending to the diagnostic record failed.
    #[error("diagnostic writer failed: {0}")]
    Diag(#[from] DiagError),
    /// Serializing or deserializing a record event failed.
    #[error("serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

/// Current wall-clock time in milliseconds since the Unix epoch.
/// Returns 0 when the system clock precedes the epoch (never on a normal
/// system); the value is only ever used as a timestamp anchor.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(0)
        })
}

/// A record event with every optional field absent.
const fn base_event(kind: EventKind, ts_ms: u64, position_secs: f64) -> RecordEvent {
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

const fn session_start(ts_ms: u64, position_secs: f64) -> RecordEvent {
    base_event(EventKind::SessionStart, ts_ms, position_secs)
}

const fn transport_change(
    ts_ms: u64,
    position_secs: f64,
    transport: TransportValue,
) -> RecordEvent {
    let mut event = base_event(EventKind::TransportChange, ts_ms, position_secs);
    event.transport = Some(transport);
    event
}

const fn session_end(
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

/// A `focus_transition` to `permanent_loss` superseding the entry at
/// `supersedes_ts_ms` (FR-044: a superseding permanent-loss entry
/// carries `supersedes_ts_ms`).
const fn permanent_loss_focus(
    ts_ms: u64,
    position_secs: f64,
    supersedes_ts_ms: u64,
) -> RecordEvent {
    let mut event = base_event(EventKind::FocusTransition, ts_ms, position_secs);
    event.focus = Some(FocusValue::PermanentLoss);
    event.supersedes_ts_ms = Some(supersedes_ts_ms);
    event
}

/// The parsed JSONL lines of the record at `path`, oldest-first.
fn read_record(path: &Path) -> Result<Vec<RecordEvent>, PropError> {
    let text = fs::read_to_string(path)?;
    let mut events = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        events.push(serde_json::from_str::<RecordEvent>(line)?);
    }
    Ok(events)
}

/// The trim markers carried by the parsed record lines, oldest-first.
fn trim_markers(lines: &[RecordEvent]) -> Vec<TrimMarker> {
    lines
        .iter()
        .filter(|line| line.kind == EventKind::Trim)
        .filter_map(|line| line.trim.clone())
        .collect()
}

/// Whether the line is a retained record event (a trim or gap marker is
/// metadata, not a retained entry).
const fn is_retained_event(line: &RecordEvent) -> bool {
    match line.kind {
        EventKind::Trim | EventKind::Gap => false,
        EventKind::SessionStart
        | EventKind::SessionEnd
        | EventKind::TransportChange
        | EventKind::FocusTransition
        | EventKind::OutputDevice
        | EventKind::Failure
        | EventKind::Refusal
        | EventKind::EarconTruncation
        | EventKind::EarconEmissionFailure => true,
    }
}

/// The retained state after appending `events` under the given bounds.
fn append_all(
    path: &Path,
    events: &[RecordEvent],
    retention_days: u64,
    size_cap_bytes: u64,
) -> Result<(usize, u64), PropError> {
    let mut writer = DiagnosticWriter::open_with_bounds(path, retention_days, size_cap_bytes)?;
    for event in events {
        writer.append(event)?;
    }
    Ok((writer.retained_entries(), writer.retained_bytes()))
}

/// Ages in days: 0 (inside any 1-day window) or 2..=5 (outside it).
/// Age 1 is excluded so no generated event sits on the window boundary,
/// where millisecond-scale test execution could flip the verdict.
fn age_in_days_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![Just(0u64), 2u64..=5u64]
}

/// A window-scenario event sequence: the generated ages in days.
fn window_scenario_strategy() -> impl Strategy<Value = Vec<u64>> {
    proptest::collection::vec(age_in_days_strategy(), 0..=16)
}

/// All five closed transport states.
fn transport_strategy() -> impl Strategy<Value = TransportValue> {
    prop_oneof![
        Just(TransportValue::Playing),
        Just(TransportValue::Paused),
        Just(TransportValue::Stopped),
        Just(TransportValue::Completed),
        Just(TransportValue::Interrupted),
    ]
}

/// The closed-4: the transport states that may end a session.
fn closed_four_strategy() -> impl Strategy<Value = TransportValue> {
    (0..CLOSED_FOUR.len()).prop_map(|index| CLOSED_FOUR[index])
}

/// The closed session-end reasons.
fn session_end_reason_strategy() -> impl Strategy<Value = SessionEndValue> {
    prop_oneof![
        Just(SessionEndValue::Paused),
        Just(SessionEndValue::Stopped),
        Just(SessionEndValue::Completed),
        Just(SessionEndValue::Interrupted),
    ]
}

/// A cap-scenario event sequence: sixteen to sixty-four in-window events
/// with strictly increasing timestamps and positions, so only the size
/// cap binds (the retention window never does).
fn cap_scenario_strategy() -> impl Strategy<Value = Vec<RecordEvent>> {
    proptest::collection::vec(transport_strategy(), 15..=63).prop_map(|transports| {
        let now = now_ms();
        let mut events = Vec::with_capacity(transports.len() + 1);
        let mut position = 0.0_f64;
        let mut ts = now;
        events.push(session_start(ts, position));
        for transport in transports {
            ts += 1;
            position += 1.0;
            events.push(transport_change(ts, position, transport));
        }
        events
    })
}

proptest! {
    /// P1 — retention window: with a 1-day retention bound, events
    /// older than the window are trimmed oldest-first and events inside
    /// the window are retained; `retained_entries()` is exactly the
    /// in-window count.
    #[test]
    fn retention_window_retains_exactly_the_in_window_events(
        ages in window_scenario_strategy(),
    ) {
        let now = now_ms();
        let mut events = vec![session_start(now, 0.0)];
        let mut position = 0.0_f64;
        for age in &ages {
            position += 1.0;
            events.push(transport_change(
                now.saturating_sub(age * DAY_MS),
                position,
                TransportValue::Paused,
            ));
        }

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("record.jsonl");
        let mut writer =
            DiagnosticWriter::open_with_bounds(&path, TEST_RETENTION_DAYS, SIZE_CAP_BYTES)?;
        for event in &events {
            writer.append(event)?;
        }

        let expected = 1 + ages.iter().filter(|age| **age == 0).count();
        prop_assert_eq!(
            writer.retained_entries(),
            expected,
            "retained_entries() must be exactly the in-window count"
        );

        // The retained lines on disk are exactly the in-window events.
        let lines = read_record(&path)?;
        let retained_lines: Vec<&RecordEvent> =
            lines.iter().filter(|line| is_retained_event(line)).collect();
        prop_assert_eq!(
            retained_lines.len(),
            expected,
            "the record must hold exactly the in-window events on disk"
        );
        let window_start = now.saturating_sub(TEST_RETENTION_DAYS * DAY_MS);
        for line in &retained_lines {
            prop_assert!(
                line.ts_ms >= window_start,
                "a retained event at ts_ms {} is older than the retention window",
                line.ts_ms
            );
        }
    }

    /// P2 — size cap oldest-first: with a 1 KiB cap, appending many
    /// events trims the oldest until `retained_bytes() <= cap`; the trim
    /// marker on disk has `dropped_events >= 1` and a `cutoff_ts_ms`
    /// that does not cut into the retained entries; and the marker line
    /// is flushed BEFORE the retained entries it describes (the last
    /// trim line precedes the first retained line on disk).
    #[test]
    fn size_cap_trims_oldest_until_under_the_cap(events in cap_scenario_strategy()) {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("record.jsonl");
        let mut writer =
            DiagnosticWriter::open_with_bounds(&path, RETENTION_DAYS, TEST_SIZE_CAP_BYTES)?;
        for event in &events {
            writer.append(event)?;
        }

        prop_assert!(
            writer.retained_bytes() <= TEST_SIZE_CAP_BYTES,
            "retained_bytes() {} exceeds the {} byte cap",
            writer.retained_bytes(),
            TEST_SIZE_CAP_BYTES
        );
        prop_assert!(
            writer.retained_entries() < events.len(),
            "the size cap was never exercised: {} of {} events retained",
            writer.retained_entries(),
            events.len()
        );
        prop_assert!(
            writer.retained_entries() >= 1,
            "the cap trim must keep at least the newest entry"
        );

        let lines = read_record(&path)?;
        let retained_lines: Vec<&RecordEvent> =
            lines.iter().filter(|line| is_retained_event(line)).collect();
        prop_assert_eq!(
            retained_lines.len(),
            writer.retained_entries(),
            "retained_entries() must match the on-disk retained line count"
        );

        let markers = trim_markers(&lines);
        let dropped: u64 = markers.iter().map(|marker| marker.dropped_events).sum();
        prop_assert_eq!(
            dropped,
            u64::try_from(events.len() - writer.retained_entries()).unwrap_or(0),
            "trim markers must account for every dropped event"
        );
        for marker in &markers {
            prop_assert!(
                marker.dropped_events >= 1,
                "trim marker dropped_events {} violates the schema minimum",
                marker.dropped_events
            );
            if let Some(oldest) = retained_lines.first() {
                prop_assert!(
                    marker.cutoff_ts_ms <= oldest.ts_ms,
                    "trim cutoff_ts_ms {} is newer than the oldest retained entry {}",
                    marker.cutoff_ts_ms,
                    oldest.ts_ms
                );
            }
        }

        // The marker is flushed BEFORE any trim of retained entries: no
        // trim/gap marker line may follow a retained entry line.
        let mut saw_retained = false;
        let mut marker_after_retained = false;
        for line in &lines {
            if is_retained_event(line) {
                saw_retained = true;
            } else if saw_retained {
                marker_after_retained = true;
            }
        }
        prop_assert!(
            !marker_after_retained,
            "a trim/gap marker line follows a retained entry line"
        );
    }

    /// P3 — never silent absence: whenever a trim removed entries, a trim
    /// marker exists on disk with `dropped_events >= 1` accounting for
    /// every removed event; and when nothing was removed, no trim marker
    /// exists. A silent shrink (entries disappearing without a marker)
    /// fails this property.
    #[test]
    fn every_window_trim_is_a_visible_marker_never_a_silent_absence(
        ages in window_scenario_strategy(),
    ) {
        let now = now_ms();
        let mut events = vec![session_start(now, 0.0)];
        let mut position = 0.0_f64;
        for age in &ages {
            position += 1.0;
            events.push(transport_change(
                now.saturating_sub(age * DAY_MS),
                position,
                TransportValue::Paused,
            ));
        }

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("record.jsonl");
        let mut writer =
            DiagnosticWriter::open_with_bounds(&path, TEST_RETENTION_DAYS, SIZE_CAP_BYTES)?;
        for event in &events {
            writer.append(event)?;
        }

        let removed = events.len() - writer.retained_entries();
        let lines = read_record(&path)?;
        let markers = trim_markers(&lines);
        if removed == 0 {
            prop_assert!(
                markers.is_empty(),
                "a trim marker was written although no entry was removed"
            );
        } else {
            let dropped: u64 = markers.iter().map(|marker| marker.dropped_events).sum();
            prop_assert!(
                !markers.is_empty(),
                "a trim removed {removed} entries without writing a trim marker"
            );
            prop_assert_eq!(
                dropped,
                u64::try_from(removed).unwrap_or(0),
                "trim markers must account for every removed event"
            );
        }
    }

    /// P4 — gap marker: a storage-full append failure records a gap marker
    /// and returns `Ok` (best-effort, playback uninterrupted). `/dev/full`
    /// is the POSIX ENOSPC device: every write to it fails with "no space
    /// left", so the append write path fails and must contain the failure
    /// as a gap marker instead of propagating it.
    #[test]
    fn storage_full_records_a_gap_marker_and_appends_stay_ok(
        transports in proptest::collection::vec(transport_strategy(), 4..=16),
    ) {
        let full = Path::new("/dev/full");
        prop_assert!(
            full.exists(),
            "/dev/full (the POSIX ENOSPC device) is unavailable"
        );
        let mut writer = DiagnosticWriter::open_with_bounds(full, RETENTION_DAYS, SIZE_CAP_BYTES)?;
        for transport in &transports {
            let event = transport_change(now_ms(), 0.0, *transport);
            let appended = writer.append(&event);
            prop_assert!(
                appended.is_ok(),
                "a storage-full append must return Ok (best-effort, playback uninterrupted), got {result:?}",
                result = appended,
            );
        }
        prop_assert!(
            writer.gap_count() >= 1,
            "a storage-full append must record a gap marker (gap_count() > 0), got {}",
            writer.gap_count()
        );
    }

    /// P5a — every transport change carries `position_secs` and
    /// `transport` (the wire field names of the contract schema).
    #[test]
    fn every_transport_change_carries_position_secs_and_transport(
        transport in transport_strategy(),
    ) {
        let event = transport_change(now_ms(), 5.0, transport);
        let value = serde_json::to_value(&event)?;
        prop_assert!(
            value.get("position_secs").is_some(),
            "a transport_change entry lost its position_secs field"
        );
        let wire_transport = value.get("transport").and_then(Value::as_str);
        prop_assert!(
            wire_transport.is_some(),
            "a transport_change entry lost its transport field"
        );
        let expected_wire = match transport {
            TransportValue::Playing => "playing",
            TransportValue::Paused => "paused",
            TransportValue::Stopped => "stopped",
            TransportValue::Completed => "completed",
            TransportValue::Interrupted => "interrupted",
        };
        prop_assert_eq!(
            wire_transport,
            Some(expected_wire),
            "the transport field must round-trip the closed transport set"
        );
    }

    /// P5b — every session end carries `transport` and
    /// `session_end_reason` from the shared closed-4, never `playing`.
    #[test]
    fn every_session_end_carries_closed_four_transport_and_reason(
        transport in closed_four_strategy(),
        reason in session_end_reason_strategy(),
    ) {
        let event = session_end(now_ms(), 5.0, transport, reason);
        let value = serde_json::to_value(&event)?;
        let wire_transport = value.get("transport").and_then(Value::as_str);
        let wire_reason = value.get("session_end_reason").and_then(Value::as_str);
        prop_assert!(
            wire_transport.is_some(),
            "a session_end entry lost its transport field"
        );
        prop_assert!(
            wire_reason.is_some(),
            "a session_end entry lost its session_end_reason field"
        );
        prop_assert_ne!(
            wire_transport,
            Some("playing"),
            "playing is a transport state, never a session-end transport"
        );
        let expected_reason = match reason {
            SessionEndValue::Paused => "paused",
            SessionEndValue::Stopped => "stopped",
            SessionEndValue::Completed => "completed",
            SessionEndValue::Interrupted => "interrupted",
        };
        prop_assert_eq!(
            wire_reason,
            Some(expected_reason),
            "the session_end_reason field must round-trip the closed-4"
        );
    }

    /// P6 — determinism: identical event sequences produce identical
    /// retained state — the same retained entry count, the same retained
    /// byte length, and byte-identical record files.
    #[test]
    fn identical_sequences_produce_identical_retained_state(
        events in cap_scenario_strategy(),
    ) {
        let dir = tempfile::tempdir()?;
        let path_a = dir.path().join("a.jsonl");
        let path_b = dir.path().join("b.jsonl");
        let state_a = append_all(&path_a, &events, TEST_RETENTION_DAYS, TEST_SIZE_CAP_BYTES)?;
        let state_b = append_all(&path_b, &events, TEST_RETENTION_DAYS, TEST_SIZE_CAP_BYTES)?;
        prop_assert_eq!(
            state_a,
            state_b,
            "identical event sequences must produce identical retained state"
        );
        let text_a = fs::read_to_string(&path_a)?;
        let text_b = fs::read_to_string(&path_b)?;
        prop_assert_eq!(
            text_a,
            text_b,
            "identical event sequences must produce byte-identical records"
        );
    }
}

/// P5c — a superseding permanent-loss entry carries `supersedes_ts_ms`
/// linking the entry it supersedes (FR-044).
#[test]
fn a_superseding_permanent_loss_entry_carries_supersedes_ts_ms() -> Result<(), PropError> {
    let now = now_ms();
    let superseded_ts = now.saturating_sub(DAY_MS);
    let event = permanent_loss_focus(now, 5.0, superseded_ts);
    let value = serde_json::to_value(&event)?;
    let wire_focus = value.get("focus").and_then(Value::as_str);
    assert_eq!(
        wire_focus,
        Some("permanent_loss"),
        "the entry must be a permanent_loss focus transition"
    );
    assert_eq!(
        value.get("supersedes_ts_ms").and_then(Value::as_u64),
        Some(superseded_ts),
        "a superseding permanent-loss entry must carry supersedes_ts_ms"
    );
    Ok(())
}
