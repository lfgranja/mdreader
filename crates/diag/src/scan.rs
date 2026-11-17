//! The diagnostic-record scanners (T074).
//!
//! One pass over a JSONL diagnostic record (the same record
//! [`crate::writer::DiagnosticWriter`] produces) reports every
//! conformance violation the record schema cannot express
//! per-entry (SC-005, SC-056; FR-039a, FR-044, FR-045):
//!
//! - [`ScanViolation::ProseInDetail`] — a `detail` that does not
//!   match the contract's non-prose `key=value` pattern
//!   (`^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=
//!   [A-Za-z0-9_.+\-]+)*$`): document text never enters the
//!   record (FR-039a, FR-045, SC-056).
//! - [`ScanViolation::DetailTooLong`] — a `detail` longer than
//!   the 200-character bound (FR-045).
//! - [`ScanViolation::FailureWithoutError`] — a `failure` entry
//!   with no `error` (FR-045).
//! - [`ScanViolation::RefusalWithoutError`] — a `refusal` entry
//!   with no `error` (FR-045).
//! - [`ScanViolation::PlayingWithoutAdvance`] — a
//!   `transport_change` to `playing` with no later entry in the
//!   same session at a strictly greater `position_secs` and no
//!   later intervening entry (SC-005).
//! - [`ScanViolation::SessionEndWithPlaying`] — a `session_end`
//!   entry carrying `transport:"playing"` (FR-044: the closed-4
//!   session-end set never includes `playing`).
//! - [`ScanViolation::EngineWithoutTier`] — an entry carrying an
//!   `engine` with no `engine_tier` (the C1 pairing invariant).
//!
//! The scanner is a read-only, tooling-facing check (SC-056):
//! it consumes the record, never writes it, and never
//! transmits it (FR-040). There is no in-app surface (FR-039a);
//! the record is read by the CLI and by CI, nothing else.

use std::path::Path;

use serde::Serialize;

use crate::writer::{DiagError, EventKind, FocusValue, RecordEvent, TransportValue};

/// The maximum `detail` length in characters (the schema's
/// `maxLength 200`, FR-045): a `detail` at or under this bound
/// is length-conforming; one over it is not.
const MAX_DETAIL_CHARS: usize = 200;

/// One conformance violation found by the scanner.
///
/// The set is closed — exactly the seven kinds the record's
/// invariants admit; a new kind is a coordinated edit (a new
/// invariant, a semver bump, and documentation), never an
/// `Other` catch-all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanViolation {
    /// A `detail` is present and does not match the non-prose
    /// `key=value` pattern — document text in the record.
    ProseInDetail {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
    /// A `detail` is present and longer than 200 characters.
    DetailTooLong {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
        /// The `detail` length in characters.
        len: usize,
    },
    /// A `failure` entry with no `error`.
    FailureWithoutError {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
    /// A `refusal` entry with no `error`.
    RefusalWithoutError {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
    /// A `transport_change` to `playing` with no later entry in
    /// the same session at a strictly greater `position_secs`
    /// and no later intervening entry.
    PlayingWithoutAdvance {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
    /// A `session_end` entry carrying `transport:"playing"`.
    SessionEndWithPlaying {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
    /// An entry carrying an `engine` with no `engine_tier`.
    EngineWithoutTier {
        /// The violating entry's timestamp, in milliseconds
        /// since the Unix epoch.
        ts_ms: u64,
    },
}

/// The scan report: every violation found, oldest-first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanReport {
    /// Every violation found, oldest-first (the record's on-disk
    /// line order, which the writer keeps in chronological
    /// order).
    pub violations: Vec<ScanViolation>,
}

/// Scan the JSONL diagnostic record at `path` in one pass and
/// report every conformance violation it contains.
///
/// The record is read as the writer left it — one JSON event
/// per line, in line order — and every check is evaluated over
/// the whole record in a single pass, so the report is
/// oldest-first. Empty lines are ignored (the record is
/// line-oriented); a non-empty line that is not a valid record
/// event is a corrupt record, not a conformance question, and
/// fails the scan.
///
/// # Errors
///
/// * [`DiagError::Io`] when the record cannot be read.
/// * [`DiagError::Serialize`] when a line is not a valid
///   [`crate::writer::RecordEvent`].
pub fn scan_record(path: &Path) -> Result<ScanReport, DiagError> {
    let text = std::fs::read_to_string(path)?;
    let mut events = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let event: RecordEvent = serde_json::from_str(line)?;
        events.push(event);
    }
    Ok(scan_events(&events))
}

/// Run every check over the parsed record events, oldest-first.
///
/// The SC-005 verdicts are precomputed per entry (they need
/// later entries), so a single chronological pass can emit all
/// seven checks per entry in the contract's order — keeping the
/// report oldest-first.
#[must_use]
fn scan_events(events: &[RecordEvent]) -> ScanReport {
    let advance = playing_advance_flags(events);
    let mut violations = Vec::new();
    for (index, event) in events.iter().enumerate() {
        check_detail(event, &mut violations);
        check_error_required(event, &mut violations);
        if is_playing_entry(event) && !advance[index] {
            violations.push(ScanViolation::PlayingWithoutAdvance { ts_ms: event.ts_ms });
        }
        check_session_end_transport(event, &mut violations);
        check_engine_tier(event, &mut violations);
    }
    ScanReport { violations }
}

/// The per-entry SC-005 verdict: `true` at index `i` when the
/// entry — if it is a `transport_change` to `playing` — is
/// followed, in the same session, by an entry at a strictly
/// greater `position_secs` or by an intervening entry (a
/// non-playing `transport`, a `permanent_loss` focus transition,
/// or a `failure`).
///
/// The record is segmented into sessions first — a `session_end`
/// closes its session inclusively, a `session_start` opens the
/// next — and each segment is scanned backwards once, carrying
/// the suffix position maximum and the suffix intervening flag,
/// so the verdict for every entry costs constant work
/// (the whole scan stays linear in the record length).
#[must_use]
fn playing_advance_flags(events: &[RecordEvent]) -> Vec<bool> {
    let mut advance = vec![false; events.len()];
    for (start, end) in session_segments(events) {
        // The suffix state strictly after the current entry:
        // the greatest position seen and whether any entry
        // intervenes. Both start empty, so the last entry of a
        // session has no later entry at all.
        let mut suffix_max_position = f64::NEG_INFINITY;
        let mut suffix_has_intervening = false;
        for (offset, event) in events[start..end].iter().enumerate().rev() {
            let index = start + offset;
            advance[index] = suffix_max_position > event.position_secs || suffix_has_intervening;
            if event.position_secs > suffix_max_position {
                suffix_max_position = event.position_secs;
            }
            if is_intervening_entry(event) {
                suffix_has_intervening = true;
            }
        }
    }
    advance
}

/// Segment the record into sessions: a session runs from after
/// the previous session's end (or the start of the record) to
/// the next `session_end` — inclusive, the session-end entry is
/// the session's last entry — or to the end of the record. A
/// `session_start` also opens a segment, so a session never
/// reads past a following session's start.
///
/// Returns the segments as half-open `(start, end)` index
/// pairs.
#[must_use]
fn session_segments(events: &[RecordEvent]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut start = 0;
    for (index, event) in events.iter().enumerate() {
        match event.kind {
            EventKind::SessionStart => {
                // A session_start opens a new session: anything
                // before it that has not been closed is its own
                // segment.
                if index > start {
                    segments.push((start, index));
                }
                start = index;
            }
            EventKind::SessionEnd => {
                // The session_end closes the session inclusively.
                segments.push((start, index + 1));
                start = index + 1;
            }
            EventKind::TransportChange
            | EventKind::FocusTransition
            | EventKind::OutputDevice
            | EventKind::Failure
            | EventKind::Refusal
            | EventKind::Trim
            | EventKind::Gap
            | EventKind::EarconTruncation
            | EventKind::EarconEmissionFailure => {}
        }
    }
    if start < events.len() {
        segments.push((start, events.len()));
    }
    segments
}

/// Whether the entry is a `transport_change` to `playing` —
/// the only kind the SC-005 playing-advance rule governs.
#[must_use]
fn is_playing_entry(event: &RecordEvent) -> bool {
    event.kind == EventKind::TransportChange && event.transport == Some(TransportValue::Playing)
}

/// Whether the entry intervenes in a stall: a non-playing
/// `transport` (any entry carrying a transport that is
/// not `playing`), a `permanent_loss` focus transition,
/// or a `failure`. An intervening entry explains why
/// playback stopped advancing, so the stall it follows
/// is not a violation.
#[must_use]
fn is_intervening_entry(event: &RecordEvent) -> bool {
    let non_playing_transport = event
        .transport
        .is_some_and(|transport| transport != TransportValue::Playing);
    let permanent_loss =
        event.kind == EventKind::FocusTransition && event.focus == Some(FocusValue::PermanentLoss);
    let failure = event.kind == EventKind::Failure;
    non_playing_transport || permanent_loss || failure
}

/// The per-entry `detail` checks: [`ScanViolation::ProseInDetail`]
/// when the `detail` does not match the non-prose pattern, and
/// [`ScanViolation::DetailTooLong`] when it exceeds the
/// 200-character bound. Both can fire on the same entry.
fn check_detail(event: &RecordEvent, violations: &mut Vec<ScanViolation>) {
    let Some(detail) = event.detail.as_deref() else {
        return;
    };
    if !is_conforming_detail(detail) {
        violations.push(ScanViolation::ProseInDetail { ts_ms: event.ts_ms });
    }
    let len = detail.chars().count();
    if len > MAX_DETAIL_CHARS {
        violations.push(ScanViolation::DetailTooLong {
            ts_ms: event.ts_ms,
            len,
        });
    }
}

/// The per-entry error-presence checks:
/// [`ScanViolation::FailureWithoutError`] on a `failure` with no
/// `error`, and [`ScanViolation::RefusalWithoutError`] on a
/// `refusal` with no `error` (FR-045).
fn check_error_required(event: &RecordEvent, violations: &mut Vec<ScanViolation>) {
    let missing_error = event.error.is_none();
    if event.kind == EventKind::Failure && missing_error {
        violations.push(ScanViolation::FailureWithoutError { ts_ms: event.ts_ms });
    }
    if event.kind == EventKind::Refusal && missing_error {
        violations.push(ScanViolation::RefusalWithoutError { ts_ms: event.ts_ms });
    }
}

/// The per-entry session-end check:
/// [`ScanViolation::SessionEndWithPlaying`] when a `session_end`
/// carries `transport:"playing"` — `playing` is a transport
/// state, never a session-end state (FR-044).
fn check_session_end_transport(event: &RecordEvent, violations: &mut Vec<ScanViolation>) {
    if event.kind == EventKind::SessionEnd && event.transport == Some(TransportValue::Playing) {
        violations.push(ScanViolation::SessionEndWithPlaying { ts_ms: event.ts_ms });
    }
}

/// The per-entry engine-pairing check:
/// [`ScanViolation::EngineWithoutTier`] when an entry carries
/// an `engine` with no `engine_tier`.
fn check_engine_tier(event: &RecordEvent, violations: &mut Vec<ScanViolation>) {
    if event.engine.is_some() && event.engine_tier.is_none() {
        violations.push(ScanViolation::EngineWithoutTier { ts_ms: event.ts_ms });
    }
}

/// The contract's non-prose `detail` pattern, quoted from
/// `specs/001-android-media-shell/contracts/
/// diagnostic-record.schema.json`:
/// `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=
/// [A-Za-z0-9_.+\-]+)*$` — one or more space-separated
/// `key=value` tokens, a non-empty `[A-Za-z0-9_]+` key, a
/// non-empty `[A-Za-z0-9_.+\-]+` value. Reimplemented as a
/// predicate so the scanner carries no regex dependency.
#[must_use]
fn is_conforming_detail(detail: &str) -> bool {
    !detail.is_empty() && detail.split(' ').all(is_conforming_token)
}

/// One `key=value` token of the non-prose `detail` pattern: a
/// non-empty alphanumeric-or-underscore key, `=`, and a non-empty
/// alphanumeric-or-`.`/`+`/`-`/`_` value.
#[must_use]
fn is_conforming_token(token: &str) -> bool {
    let Some((key, value)) = token.split_once('=') else {
        return false;
    };
    !key.is_empty()
        && key
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        && !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '+' | '-' | '_')
        })
}
