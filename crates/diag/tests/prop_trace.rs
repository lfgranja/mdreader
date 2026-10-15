//! T040 — BLOCKING US4 diag no-prose + bounds property (Feature
//! 004-ressoa-dogfood-build, task T040).
//!
//! Requirement (tasks.md T040, verbatim): "Diag no-prose + bounds
//! property in `crates/diag/tests/prop_trace.rs` (30-day + 10 MB
//! oldest-first with `Trim` before retained, `Gap` on full with `Ok`;
//! `ProseInDetail`/`DetailTooLong`/`FailureWithoutError`/
//! `RefusalWithoutError` scan violations trip; zero egress — no network
//! call sites (diag-crate scope; app-wide covered by T047))"
//!
//! What these properties pin (FR-039, FR-039a, FR-039b, FR-040,
//! FR-044, FR-045, SC-005, SC-056; contracts/diagnostic-trace.md):
//!
//! - **The production bounds**: the 30-day retention window and the
//!   10 MiB size cap are the declared constants.
//! - **Oldest-first, never silent**: both bounds trim oldest-first,
//!   and every drop is accounted by a `trim` marker flushed to disk
//!   BEFORE the retained entries it describes (the on-disk line order
//!   is `[trim markers][retained entries]`), so a shrink is never a
//!   silent absence (FR-039b). Verified under small test bounds that
//!   exercise the same code paths deterministically (T067 pins the
//!   complementary determinism/field-conformance halves).
//! - **Gap on full**: a storage-full append write failure (a write to
//!   `/dev/full`, the POSIX ENOSPC device) records a `gap` marker and
//!   `append` returns `Ok` — best-effort, so playback is never
//!   interrupted (FR-039b).
//! - **Scan violations trip**: `ProseInDetail` (document text in
//!   `detail`), `DetailTooLong` (over the 200-character bound),
//!   `FailureWithoutError`, and `RefusalWithoutError` each fire on
//!   their violating entry and only on it — a conforming record scans
//!   clean. The writer is validation-free by design; the scanner is
//!   the read-only conformance check (SC-056) and the record is never
//!   transmitted (FR-040).
//! - **Zero egress (diag-crate scope)**: the diag crate's sources and
//!   manifest are embedded at test time and asserted to carry no
//!   network call-site pattern — the record leaves the device only by
//!   a maintainer's adb pull, never by a call site (app-wide egress is
//!   T047's scope).
//!
//! BLOCKING for the T041–T044 implementation wave: the bounds, the
//! trim/gap accounting, and the no-prose scan must hold across it.
//!
//! NOTE: the helpers below are duplicated across the diag test
//! targets (`prop_record.rs`, `prop_no_prose.rs`, `prop_scanner_canary.rs`)
//! by the same convention — a shared test-support module would be a
//! fourth file.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use diag::scan::{ScanReport, ScanViolation, scan_record};
use diag::writer::{
    DetailError, DiagError, DiagnosticWriter, EventKind, RETENTION_DAYS, RecordEvent,
    SIZE_CAP_BYTES, SessionEndValue, TransportValue, TrimMarker, detail_from_pairs,
};
use proptest::prelude::*;

/// One day in milliseconds (retention-window arithmetic).
const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// The small retention bound (1 day) used to exercise the window path
/// deterministically — the production bound is [`RETENTION_DAYS`].
const TEST_RETENTION_DAYS: u64 = 1;

/// The small size cap (1 KiB) used to exercise the cap path: sixteen
/// minimal events already exceed it, so every generated case trims,
/// and a single event always fits, so the cap trim always keeps the
/// newest entry. The production bound is [`SIZE_CAP_BYTES`].
const TEST_SIZE_CAP_BYTES: u64 = 1024;

/// The record version carried by every event (schema const 1).
const RECORD_VERSION: u8 = 1;

/// The network call-site pattern set: a diagnostic record is NEVER
/// transmitted (FR-040), so the diag crate carries no HTTP/TCP client
/// of any family. Diag-crate scope; app-wide egress is T047.
const NETWORK_PATTERNS: [&str; 11] = [
    "reqwest",
    "hyper",
    "ureq",
    "tonic",
    "isahc",
    "minreq",
    "curl",
    "tokio::net",
    "std::net",
    "TcpStream",
    "UdpSocket",
];

/// The diag crate's sources and manifest, embedded at test time: the
/// zero-egress guard scans the crate's own scope — the five source
/// files plus its dependency manifest (a network dependency would
/// name itself here).
const DIAG_SOURCES: [(&str, &str); 6] = [
    ("crates/diag/src/lib.rs", include_str!("../src/lib.rs")),
    (
        "crates/diag/src/writer.rs",
        include_str!("../src/writer.rs"),
    ),
    ("crates/diag/src/scan.rs", include_str!("../src/scan.rs")),
    ("crates/diag/src/cli.rs", include_str!("../src/cli.rs")),
    ("crates/diag/src/main.rs", include_str!("../src/main.rs")),
    ("crates/diag/Cargo.toml", include_str!("../Cargo.toml")),
];

/// Test-local error type. Proptest turns any `std::error::Error`
/// returned via `?` from a property body into a test failure
/// (`TestCaseError: From<E: Error>`), so property bodies can use `?`
/// for fallible setup (temp dirs, writer open/append, scans).
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
/// Returns 0 when the system clock precedes the epoch (never on a
/// normal system); the value is only ever used as a timestamp anchor.
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

/// Whether the line is a retained record event (a trim or gap marker
/// is metadata, not a retained entry) — the closed nine-kind set,
/// enumerated (Principle VII / FR-044).
fn is_retained_event(line: &RecordEvent) -> bool {
    match line.kind {
        EventKind::Trim | EventKind::Gap => false,
        EventKind::SessionStart
        | EventKind::SessionEnd
        | EventKind::TransportChange
        | EventKind::FocusTransition
        | EventKind::OutputDevice
        | EventKind::Failure
        | EventKind::Refusal => true,
    }
}

/// Assert the FR-039b on-disk ordering: no trim or gap marker line
/// follows a retained entry line — the marker accounting for a trim
/// is flushed BEFORE the retained entries it describes.
fn assert_markers_precede_retained(lines: &[RecordEvent]) {
    let mut saw_retained = false;
    let mut marker_after_retained = false;
    for line in lines {
        if is_retained_event(line) {
            saw_retained = true;
        } else if saw_retained {
            marker_after_retained = true;
        }
    }
    assert!(
        !marker_after_retained,
        "a trim/gap marker line follows a retained entry line (FR-039b)"
    );
}

/// Ages in days: 0 (inside any 1-day window) or 2..=5 (outside it).
/// Age 1 is excluded so no generated event sits on the window
/// boundary, where millisecond-scale test execution could flip the
/// verdict.
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

/// A cap-scenario event sequence: sixteen to sixty-four in-window
/// events with strictly increasing timestamps and positions, so only
/// the size cap binds (the retention window never does).
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

/// The violating entry kinds this property builds: each trips exactly
/// its own documented scan violation, and `Clean` trips none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViolationKind {
    /// A `detail` carrying document prose (not the `key=value`
    /// identifier pattern) — trips `ProseInDetail`.
    Prose,
    /// A pattern-conforming `detail` over the 200-character bound —
    /// trips `DetailTooLong` only.
    TooLong,
    /// A `failure` entry with no `error` — trips `FailureWithoutError`.
    FailureMissingError,
    /// A `refusal` entry with no `error` — trips `RefusalWithoutError`.
    RefusalMissingError,
    /// A conforming entry — trips nothing.
    Clean,
}

/// A generated violating kind.
fn violation_strategy() -> impl Strategy<Value = ViolationKind> {
    prop_oneof![
        Just(ViolationKind::Prose),
        Just(ViolationKind::TooLong),
        Just(ViolationKind::FailureMissingError),
        Just(ViolationKind::RefusalMissingError),
        Just(ViolationKind::Clean),
    ]
}

/// A record holding one subject entry of `kind` between a
/// `session_start` and a closing `session_end`: the subject's
/// timestamp is returned alongside, so the assertions name it.
fn violating_record(now: u64, kind: ViolationKind) -> (Vec<RecordEvent>, u64) {
    let subject_ts = now + 1;
    let subject = match kind {
        ViolationKind::Prose => {
            let mut event = transport_change(subject_ts, 1.0, TransportValue::Paused);
            event.detail = Some("document prose sentence".to_string());
            event
        }
        ViolationKind::TooLong => {
            let mut event = transport_change(subject_ts, 1.0, TransportValue::Paused);
            event.detail = Some(format!("k={}", "a".repeat(260)));
            event
        }
        ViolationKind::FailureMissingError => base_event(EventKind::Failure, subject_ts, 1.0),
        ViolationKind::RefusalMissingError => base_event(EventKind::Refusal, subject_ts, 1.0),
        ViolationKind::Clean => transport_change(subject_ts, 1.0, TransportValue::Paused),
    };
    let events = vec![
        session_start(now, 0.0),
        subject,
        session_end(
            subject_ts + 1,
            2.0,
            TransportValue::Paused,
            SessionEndValue::Paused,
        ),
    ];
    (events, subject_ts)
}

#[test]
fn the_production_bounds_are_thirty_days_and_ten_mib() {
    // FR-039: the declared retention window (30 days) and size cap
    // (10 MiB) are the production constants both bounds enforce.
    assert_eq!(RETENTION_DAYS, 30);
    assert_eq!(SIZE_CAP_BYTES, 10 * 1024 * 1024);
}

proptest! {
    /// The 30-day window (exercised under the 1-day test bound) trims
    /// oldest-first: events older than the window are dropped, events
    /// inside it are retained, every drop is accounted by a trim
    /// marker, and the markers are flushed before the retained
    /// entries (never a silent absence, FR-039 / FR-039b).
    #[test]
    fn the_retention_window_trims_oldest_first_with_trim_markers_before_retained(
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
            "retained_entries() must be exactly the in-window count (oldest-first trim)",
        );

        let lines = read_record(&path)?;
        let retained: Vec<&RecordEvent> =
            lines.iter().filter(|line| is_retained_event(line)).collect();
        prop_assert_eq!(
            retained.len(),
            expected,
            "the record must hold exactly the in-window events on disk",
        );
        // Oldest-first: every retained event sits inside the window,
        // and every dropped event was older than the window cutoff.
        let cutoff = now.saturating_sub(TEST_RETENTION_DAYS * DAY_MS);
        for line in &retained {
            prop_assert!(
                line.ts_ms >= cutoff,
                "a retained event at ts_ms {} is older than the window cutoff {}",
                line.ts_ms,
                cutoff,
            );
        }
        // Every drop is accounted, the markers carry at least one
        // dropped event, and the markers precede the retained lines.
        let dropped: u64 = lines
            .iter()
            .filter(|line| line.kind == EventKind::Trim)
            .filter_map(|line| line.trim.clone())
            .map(|marker| marker.dropped_events)
            .sum();
        prop_assert_eq!(
            dropped,
            u64::try_from(events.len() - expected).unwrap_or(0),
            "trim markers must account for every dropped event (never a silent absence)",
        );
        assert_markers_precede_retained(&lines);
    }

    /// The 10 MiB cap (exercised under the 1 KiB test bound) trims
    /// oldest-first until the retained bytes are within the cap, keeps
    /// at least the newest entry, accounts every drop with a trim
    /// marker whose cutoff does not cut into the retained entries, and
    /// flushes the markers before the retained entries.
    #[test]
    fn the_size_cap_trims_oldest_first_with_trim_markers_before_retained(
        events in cap_scenario_strategy(),
    ) {
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
            TEST_SIZE_CAP_BYTES,
        );
        prop_assert!(
            writer.retained_entries() < events.len(),
            "the size cap was never exercised: {} of {} events retained",
            writer.retained_entries(),
            events.len(),
        );
        prop_assert!(
            writer.retained_entries() >= 1,
            "the cap trim must keep at least the newest entry",
        );

        let lines = read_record(&path)?;
        let retained: Vec<&RecordEvent> =
            lines.iter().filter(|line| is_retained_event(line)).collect();
        prop_assert_eq!(
            retained.len(),
            writer.retained_entries(),
            "retained_entries() must match the on-disk retained line count",
        );
        let markers = trim_markers(&lines);
        let dropped: u64 = markers.iter().map(|marker| marker.dropped_events).sum();
        prop_assert_eq!(
            dropped,
            u64::try_from(events.len() - writer.retained_entries()).unwrap_or(0),
            "trim markers must account for every dropped event (never a silent absence)",
        );
        for marker in &markers {
            prop_assert!(
                marker.dropped_events >= 1,
                "trim marker dropped_events {} violates the schema minimum",
                marker.dropped_events,
            );
            if let Some(oldest) = retained.first() {
                prop_assert!(
                    marker.cutoff_ts_ms <= oldest.ts_ms,
                    "trim cutoff_ts_ms {} is newer than the oldest retained entry {}",
                    marker.cutoff_ts_ms,
                    oldest.ts_ms,
                );
            }
        }
        // The markers are flushed BEFORE the retained entries they
        // describe.
        assert_markers_precede_retained(&lines);
    }

    /// Every documented scan violation trips on its violating entry
    /// (and only that one): prose in `detail`, an over-long `detail`, a
    /// `failure` with no `error`, a `refusal` with no `error` — and a
    /// conforming record scans clean (FR-045 / SC-056).
    #[test]
    fn every_documented_scan_violation_trips(
        kind in violation_strategy(),
        ts_seed in 0u64..1_000_000,
    ) {
        let now = now_ms().saturating_add(ts_seed);
        let (events, subject_ts) = violating_record(now, kind);
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("record.jsonl");
        let mut writer = DiagnosticWriter::open_with_bounds(&path, RETENTION_DAYS, SIZE_CAP_BYTES)?;
        for event in &events {
            writer.append(event)?;
        }
        drop(writer);
        let report: ScanReport = scan_record(&path)?;
        match kind {
            ViolationKind::Prose => {
                prop_assert!(
                    report.violations.contains(&ScanViolation::ProseInDetail { ts_ms: subject_ts }),
                    "a prose detail must trip ProseInDetail, got {violations:?}",
                    violations = &report.violations,
                );
                prop_assert!(
                    !report.violations.iter().any(|violation| matches!(
                        violation,
                        ScanViolation::DetailTooLong { .. }
                    )),
                    "a short prose detail must not trip DetailTooLong, got {violations:?}",
                    violations = &report.violations,
                );
            }
            ViolationKind::TooLong => {
                let long = format!("k={}", "a".repeat(260));
                let expected_len = long.chars().count();
                prop_assert!(
                    report.violations.contains(&ScanViolation::DetailTooLong {
                        ts_ms: subject_ts,
                        len: expected_len,
                    }),
                    "an over-long conforming detail must trip DetailTooLong (len {expected_len}), got {violations:?}",
                    violations = &report.violations,
                );
                prop_assert!(
                    !report.violations.iter().any(|violation| matches!(
                        violation,
                        ScanViolation::ProseInDetail { .. }
                    )),
                    "a pattern-conforming detail must not trip ProseInDetail, got {violations:?}",
                    violations = &report.violations,
                );
            }
            ViolationKind::FailureMissingError => {
                prop_assert!(
                    report.violations.contains(&ScanViolation::FailureWithoutError {
                        ts_ms: subject_ts,
                    }),
                    "a failure with no error must trip FailureWithoutError, got {violations:?}",
                    violations = &report.violations,
                );
            }
            ViolationKind::RefusalMissingError => {
                prop_assert!(
                    report.violations.contains(&ScanViolation::RefusalWithoutError {
                        ts_ms: subject_ts,
                    }),
                    "a refusal with no error must trip RefusalWithoutError, got {violations:?}",
                    violations = &report.violations,
                );
            }
            ViolationKind::Clean => {
                prop_assert!(
                    report.violations.is_empty(),
                    "a conforming record must scan with zero violations, got {violations:?}",
                    violations = &report.violations,
                );
            }
        }
    }
}

#[test]
fn a_mixed_record_trips_exactly_the_injected_violations() -> Result<(), PropError> {
    // All four documented violations in one record trip exactly once
    // each, oldest-first — the scanner's report is the injected set
    // and nothing else.
    let now = now_ms();
    let long = format!("k={}", "a".repeat(260));
    let mut expected = Vec::new();
    let mut events = vec![session_start(now, 0.0)];

    let mut prose = transport_change(now + 1, 1.0, TransportValue::Paused);
    prose.detail = Some("document prose sentence".to_string());
    expected.push(ScanViolation::ProseInDetail { ts_ms: now + 1 });
    events.push(prose);

    let mut too_long = transport_change(now + 2, 2.0, TransportValue::Paused);
    too_long.detail = Some(long.clone());
    expected.push(ScanViolation::DetailTooLong {
        ts_ms: now + 2,
        len: long.chars().count(),
    });
    events.push(too_long);

    expected.push(ScanViolation::FailureWithoutError { ts_ms: now + 3 });
    events.push(base_event(EventKind::Failure, now + 3, 3.0));

    expected.push(ScanViolation::RefusalWithoutError { ts_ms: now + 4 });
    events.push(base_event(EventKind::Refusal, now + 4, 4.0));

    events.push(session_end(
        now + 5,
        5.0,
        TransportValue::Paused,
        SessionEndValue::Paused,
    ));

    let dir = tempfile::tempdir()?;
    let path = dir.path().join("record.jsonl");
    let mut writer = DiagnosticWriter::open_with_bounds(&path, RETENTION_DAYS, SIZE_CAP_BYTES)?;
    for event in &events {
        writer.append(event)?;
    }
    drop(writer);

    let report = scan_record(&path)?;
    assert_eq!(
        report.violations, expected,
        "the mixed record must trip exactly the injected violations, oldest-first"
    );
    Ok(())
}

#[test]
fn a_storage_full_append_records_a_gap_marker_and_returns_ok() -> Result<(), PropError> {
    // FR-039b: a storage-full append failure (a write to `/dev/full`,
    // the POSIX ENOSPC device) records a gap marker and `append`
    // returns `Ok` — best-effort, playback never interrupted.
    let full = Path::new("/dev/full");
    assert!(
        full.exists(),
        "/dev/full (the POSIX ENOSPC device) is unavailable"
    );
    let mut writer = DiagnosticWriter::open_with_bounds(full, RETENTION_DAYS, SIZE_CAP_BYTES)?;
    for offset in 0..4_u64 {
        let event = transport_change(now_ms().saturating_add(offset), 0.0, TransportValue::Paused);
        let appended = writer.append(&event);
        assert!(
            appended.is_ok(),
            "a storage-full append must return Ok (best-effort, playback uninterrupted), got {appended:?}"
        );
    }
    assert!(
        writer.gap_count() >= 1,
        "a storage-full append must record a gap marker, got gap_count() = {}",
        writer.gap_count()
    );
    Ok(())
}

#[test]
fn the_writer_refuses_prose_and_overlong_details_at_construction()
-> Result<(), Box<dyn std::error::Error>> {
    // The construction seam refuses what the scanner would flag: a
    // prose value (not the identifier pattern) and an over-long
    // detail (over 200 characters) — the two halves of the no-prose +
    // bounds contract meet at the writer (FR-045).
    let prose = detail_from_pairs(&[("note", "document prose sentence")]);
    assert!(
        prose.is_err(),
        "a prose detail must be refused at construction (the scanner would flag ProseInDetail)"
    );
    let long: String = "a".repeat(260);
    match detail_from_pairs(&[("k", long.as_str())]) {
        Err(DetailError::TooLong(len)) => assert!(
            len > 200,
            "the refusal must name the over-long length, got {len}"
        ),
        Err(other) => {
            return Err(format!("an over-long detail must refuse as TooLong, got {other}").into());
        }
        Ok(_) => {
            return Err("an over-long detail must be refused at construction".into());
        }
    }
    Ok(())
}

#[test]
fn the_diag_crate_has_zero_network_call_sites() {
    // FR-040: the record is never transmitted — retrieval is a
    // maintainer-only adb pull from app-private storage, so the diag
    // crate carries no network call site of any family. The sources
    // and the manifest are embedded at test time and scanned here
    // (diag-crate scope; app-wide egress is T047).
    for (name, source) in DIAG_SOURCES {
        for pattern in NETWORK_PATTERNS {
            assert!(
                !source.contains(pattern),
                "the diag crate source {name} carries the network call-site pattern {pattern:?} — the record is never transmitted (FR-040)"
            );
        }
    }
}
