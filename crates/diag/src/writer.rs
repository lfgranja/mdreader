//! Append-only JSONL diagnostic-record writer (T073).
//!
//! One [`crate::writer::RecordEvent`] per line, serialized with `serde_json`.
//! The writer is validation-free by design: the committed contract
//! schema (`specs/001-android-media-shell/contracts/diagnostic-record.schema.json`)
//! validates, per FR-039, FR-044, and FR-045.
//!
//! The writer enforces the dual bounds of FR-039 on open and on every
//! append: the retention window drops events older than the
//! `RETENTION_DAYS` window, measured backwards from the newest
//! retained event, and the size cap drops the oldest retained
//! events until the retained byte length is within
//! `SIZE_CAP_BYTES`. Every drop is
//! accounted by a trim marker line (`EventKind::Trim`) flushed to disk
//! BEFORE the retained entries it describes, so the on-disk line order
//! is `[trim markers][retained entries]` and a shrink is never silent
//! (FR-039b). Writes are best-effort: a storage-full append failure
//! records a gap marker (`EventKind::Gap`) and returns `Ok`, so
//! playback is never interrupted (FR-039b).
//!
//! The wire enums below are the record's view of the closed sets
//! owned by `engine_core::closed` (FR-044: one list, never a second
//! drifting list). Each one-to-one pair carries exhaustive `From`
//! conversions in both directions with no wildcard arm (Principle
//! VII), so adding a variant on either side breaks the build until
//! the other side is coordinated. [`crate::writer::FocusValue`] is the union of
//! [`engine_core::closed::FocusOutcome`] and
//! [`engine_core::closed::FocusRegained`] with identical wire
//! strings; it has no total `From` mapping because `regained` is a
//! direction event rather than a fourth outcome.

use std::fs::{File, OpenOptions, metadata, read_to_string, write};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The production retention window in days (FR-039): a record event
/// older than this window, measured backwards from the newest
/// retained event, is trimmed oldest-first.
pub const RETENTION_DAYS: u64 = 30;

/// The production size cap in bytes (FR-039): the retained record
/// events are trimmed oldest-first until their total serialized
/// length is within 10 MiB.
pub const SIZE_CAP_BYTES: u64 = 10 * 1024 * 1024;

/// One day in milliseconds (retention-window arithmetic).
const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// The record version carried by every event (schema `const` 1).
const RECORD_VERSION: u8 = 1;

/// Errors from opening or appending to a diagnostic record.
#[derive(Debug, Error)]
pub enum DiagError {
    /// Opening the record file or writing a line failed.
    #[error("diagnostic record I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Serializing the event to JSON failed.
    #[error("diagnostic record serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// The closed event-kind set: exactly nine kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A session started.
    SessionStart,
    /// A session ended; carries `transport` and `session_end_reason`.
    SessionEnd,
    /// The transport state changed; carries `transport`.
    TransportChange,
    /// Audio focus changed; carries `focus`.
    FocusTransition,
    /// The output device changed; carries `device`.
    OutputDevice,
    /// A named failure; carries `error`.
    Failure,
    /// A refused request naming its condition; carries `error`.
    Refusal,
    /// A storage trim marker; carries `trim`.
    Trim,
    /// A storage-exhaustion gap marker.
    Gap,
}

/// The closed transport-state set (FR-020): exactly five states.
///
/// Wire-identical to [`engine_core::closed::TransportState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportValue {
    /// Audio is playing.
    Playing,
    /// Playback is paused; position is retained.
    Paused,
    /// Playback is stopped at user request.
    Stopped,
    /// Playback reached the end of the document.
    Completed,
    /// Playback was interrupted.
    Interrupted,
}

/// The closed session-end-reason set (FR-044): exactly four reasons.
///
/// Wire-identical to [`engine_core::closed::SessionEnd`]; `playing`
/// is a transport state and is never a session-end reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEndValue {
    /// The session ended because playback was paused.
    Paused,
    /// The session ended because playback was stopped.
    Stopped,
    /// The session ended because the document completed.
    Completed,
    /// The session ended because playback was interrupted.
    Interrupted,
}

/// The closed focus set: the three [`engine_core::closed::FocusOutcome`]
/// outcomes plus the [`engine_core::closed::FocusRegained`] direction
/// event, with identical wire strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusValue {
    /// A transient request that permits ducking.
    DuckableTransient,
    /// A demand that requires silence (call, alarm).
    SilenceDemanding,
    /// Permanent loss of the audio route.
    PermanentLoss,
    /// Audio focus was regained (direction event, not an outcome).
    Regained,
}

/// The closed engine-failure taxonomy: exactly seven named errors.
///
/// Wire-identical to [`engine_core::closed::EngineError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorValue {
    /// The engine is unreachable at the transport level.
    EngineUnreachable,
    /// The engine failed to initialize.
    EngineInitFailed,
    /// The delivered rate deviates from the declared handoff shape.
    RateMismatch,
    /// The delivered channel count deviates from mono.
    ChannelMismatch,
    /// The audio device refused the playback request.
    DeviceRefused,
    /// The audio device was lost during playback.
    DeviceLost,
    /// Playback start was refused.
    StartRefused,
}

/// The closed engine-tier set (FR-023c): exactly five tiers.
///
/// Wire-identical to [`engine_core::closed::EngineTier`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineTierValue {
    /// Offline default tier.
    OfflineDefault,
    /// Cloud default tier.
    CloudDefault,
    /// Cloud escalation tier.
    CloudEscalation,
    /// Test-double tier (stand-in fixtures only).
    TestDouble,
    /// Evaluated fallback tier (last, never a default).
    EvaluatedFallback,
}

/// The output-device direction: exactly two directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceDirection {
    /// A device appeared.
    Appeared,
    /// A device disappeared.
    Disappeared,
}

/// An output-device event (FR-017a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceEvent {
    /// Which way the device moved.
    pub direction: DeviceDirection,
    /// The device identifier.
    pub device_id: String,
}

/// A storage trim marker (FR-039b).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrimMarker {
    /// How many events were dropped (`>= 1` per the schema).
    pub dropped_events: u64,
    /// The timestamp before which events were dropped.
    pub cutoff_ts_ms: u64,
}

/// One JSONL diagnostic-record event.
///
/// Serde field names are exactly the schema property names; absent
/// options are omitted on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordEvent {
    /// Record version (const 1).
    pub v: u8,
    /// Event timestamp in milliseconds since the Unix epoch.
    pub ts_ms: u64,
    /// The event kind.
    pub kind: EventKind,
    /// Playback position in seconds.
    pub position_secs: f64,
    /// The transport state, where it applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportValue>,
    /// The session-end reason, on `session_end` entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_end_reason: Option<SessionEndValue>,
    /// The timestamp this entry supersedes, where it applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_ts_ms: Option<u64>,
    /// The focus outcome or direction, where it applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<FocusValue>,
    /// The output-device event, where it applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DeviceEvent>,
    /// The named error, required on `failure` and `refusal`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorValue>,
    /// The engine identifier; implies `engine_tier`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// The engine tier; required whenever `engine` is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_tier: Option<EngineTierValue>,
    /// A bound, non-prose `key=value` detail string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The trim marker, required on `trim` entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim: Option<TrimMarker>,
}

/// Append-only JSONL writer for the diagnostic record.
///
/// The retained record events are tracked in memory and both
/// bounds are enforced on open and on every append: the
/// retention window drops events older than the
/// `RETENTION_DAYS` window relative to the newest retained
/// event, and the size cap drops the oldest retained events
/// until the retained byte length is within
/// `SIZE_CAP_BYTES`. Every drop is accounted by a trim
/// marker line flushed to disk before the retained entries it
/// describes, so the on-disk line order is
/// `[trim markers][retained entries]` and a shrink is never
/// silent (FR-039, FR-039b).
#[derive(Debug)]
pub struct DiagnosticWriter {
    /// The record file, opened for appending.
    handle: File,
    /// The record file path, for compaction rewrites.
    path: PathBuf,
    /// The retention window in days.
    retention_days: u64,
    /// The retained-bytes size cap.
    size_cap_bytes: u64,
    /// The retained record events, oldest first, each paired
    /// with its serialized line length in bytes (newline
    /// included).
    retained: Vec<(RecordEvent, u64)>,
    /// The trim and gap marker events, oldest first.
    markers: Vec<RecordEvent>,
    /// The total serialized byte length of the retained events.
    retained_bytes: u64,
    /// The number of storage-exhaustion gap markers recorded
    /// since open.
    gap_count: u64,
    /// Whether the in-memory state has diverged from the file.
    dirty: bool,
}

impl DiagnosticWriter {
    /// Opens (creating if absent) the record at `path` for
    /// appending under the production bounds (FR-039): a
    /// [`RETENTION_DAYS`] retention window and a
    /// [`SIZE_CAP_BYTES`] size cap.
    ///
    /// # Errors
    ///
    /// Returns [`DiagError::Io`] when the file cannot be
    /// opened or the existing record cannot be read, or
    /// [`DiagError::Serialize`] when a loaded event cannot be
    /// re-serialized during compaction.
    pub fn open(path: &Path) -> Result<Self, DiagError> {
        Self::open_with_bounds(path, RETENTION_DAYS, SIZE_CAP_BYTES)
    }

    /// Opens (creating if absent) the record at `path` for
    /// appending under test-supplied bounds: `retention_days`
    /// is the retention window in days and `size_cap_bytes`
    /// the retained-bytes cap. An existing record is loaded
    /// and both bounds are applied before the first append.
    ///
    /// # Errors
    ///
    /// Returns [`DiagError::Io`] when the file cannot be
    /// opened or the existing record cannot be read, or
    /// [`DiagError::Serialize`] when a loaded event cannot be
    /// re-serialized during compaction.
    pub fn open_with_bounds(
        path: &Path,
        retention_days: u64,
        size_cap_bytes: u64,
    ) -> Result<Self, DiagError> {
        let handle = OpenOptions::new().create(true).append(true).open(path)?;
        let mut writer = Self {
            handle,
            path: path.to_path_buf(),
            retention_days,
            size_cap_bytes,
            retained: Vec::new(),
            markers: Vec::new(),
            retained_bytes: 0,
            gap_count: 0,
            dirty: false,
        };
        writer.load()?;
        Ok(writer)
    }

    /// The number of retained record events; trim and gap
    /// markers are metadata, not entries.
    #[must_use]
    pub const fn retained_entries(&self) -> usize {
        self.retained.len()
    }

    /// The total serialized byte length of the retained
    /// record events; markers are excluded.
    #[must_use]
    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    /// The number of storage-exhaustion gap markers recorded
    /// since open (FR-039b).
    #[must_use]
    pub const fn gap_count(&self) -> u64 {
        self.gap_count
    }

    /// Appends `event` as a single JSONL line, then enforces
    /// both bounds. A `trim` or `gap` event is a marker: it
    /// is written to disk and tracked as metadata, never as a
    /// retained entry.
    ///
    /// # Errors
    ///
    /// Returns [`DiagError::Serialize`] when the event cannot
    /// be serialized, [`DiagError::Io`] when the record
    /// cannot be compacted after a trim, or
    /// [`DiagError::Io`] from [`Self::open`]-time loading.
    /// A storage-full append write failure is contained: a
    /// gap marker is recorded and `Ok` is returned, so
    /// playback is never interrupted (FR-039b).
    pub fn append(&mut self, event: &RecordEvent) -> Result<(), DiagError> {
        let line = serde_json::to_string(event)?;
        if writeln!(self.handle, "{line}").is_err() {
            // Best-effort write (FR-039b): the record could
            // not be written (storage full), so the event is
            // lost. Record a gap marker and return `Ok`; the
            // marker line is flushed on the next compaction.
            self.record_gap(event);
            return Ok(());
        }
        if matches!(event.kind, EventKind::Trim | EventKind::Gap) {
            self.markers.push(event.clone());
            return Ok(());
        }
        let line_len = u64::try_from(line.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        self.retained.push((event.clone(), line_len));
        self.retained_bytes = self.retained_bytes.saturating_add(line_len);
        let (anchor_ts_ms, anchor_position_secs) =
            self.anchor().unwrap_or((event.ts_ms, event.position_secs));
        self.apply_retention_window(anchor_ts_ms, anchor_position_secs);
        self.apply_size_cap(anchor_ts_ms, anchor_position_secs);
        if self.dirty {
            self.compact()?;
        }
        Ok(())
    }

    /// The newest retained event (maximum `ts_ms`), which
    /// anchors the retention window.
    fn anchor(&self) -> Option<(u64, f64)> {
        self.retained
            .iter()
            .max_by_key(|(event, _)| event.ts_ms)
            .map(|(event, _)| (event.ts_ms, event.position_secs))
    }

    /// Loads the existing record (if any) into memory and
    /// applies both bounds. A path that is not a regular
    /// file — for example the `/dev/full` storage-exhaustion
    /// device — has no record history to load, and reading
    /// it could never end, so it is opened empty.
    fn load(&mut self) -> Result<(), DiagError> {
        let file_metadata = match metadata(&self.path) {
            Ok(file_metadata) => file_metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(DiagError::Io(error)),
        };
        if !file_metadata.is_file() {
            return Ok(());
        }
        let text = read_to_string(&self.path)?;
        let mut canonical = true;
        let mut saw_retained = false;
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            let event: RecordEvent = match serde_json::from_str(line) {
                Ok(event) => event,
                // A torn or malformed line is skipped: the
                // record is best-effort, and a partial
                // trailing write must not make the whole
                // record unreadable.
                Err(_) => continue,
            };
            if matches!(event.kind, EventKind::Trim | EventKind::Gap) {
                if saw_retained {
                    canonical = false;
                }
                self.markers.push(event);
            } else {
                saw_retained = true;
                let line_len = u64::try_from(line.len())
                    .unwrap_or(u64::MAX)
                    .saturating_add(1);
                self.retained_bytes = self.retained_bytes.saturating_add(line_len);
                self.retained.push((event, line_len));
            }
        }
        let Some((anchor_ts_ms, anchor_position_secs)) = self.anchor() else {
            return Ok(());
        };
        self.apply_retention_window(anchor_ts_ms, anchor_position_secs);
        self.apply_size_cap(anchor_ts_ms, anchor_position_secs);
        if self.dirty || !canonical {
            self.compact()?;
        }
        Ok(())
    }

    /// Drops the retained events older than the retention
    /// window, measured backwards from the anchor event,
    /// oldest first. A non-empty drop is accounted by a trim
    /// marker with the window cutoff.
    fn apply_retention_window(&mut self, anchor_ts_ms: u64, anchor_position_secs: f64) {
        if self.retained.is_empty() {
            return;
        }
        let window_ms = self.retention_days.saturating_mul(DAY_MS);
        let cutoff_ts_ms = anchor_ts_ms.saturating_sub(window_ms);
        let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.retained)
            .into_iter()
            .partition(|(event, _)| event.ts_ms < cutoff_ts_ms);
        self.retained = kept;
        if dropped.is_empty() {
            return;
        }
        let dropped_events = u64::try_from(dropped.len()).unwrap_or(u64::MAX);
        let dropped_bytes: u64 = dropped.iter().map(|(_, line_len)| *line_len).sum();
        self.retained_bytes = self.retained_bytes.saturating_sub(dropped_bytes);
        self.markers.push(marker_event(
            EventKind::Trim,
            anchor_ts_ms,
            anchor_position_secs,
            Some(TrimMarker {
                dropped_events,
                cutoff_ts_ms,
            }),
        ));
        self.dirty = true;
    }

    /// Drops the oldest retained events until the retained
    /// byte length is within the size cap, oldest first. A
    /// non-empty drop is accounted by a trim marker with the
    /// cutoff at the newest dropped event.
    fn apply_size_cap(&mut self, anchor_ts_ms: u64, anchor_position_secs: f64) {
        if self.retained_bytes <= self.size_cap_bytes {
            return;
        }
        let mut dropped_events = 0_u64;
        let mut cutoff_ts_ms = 0_u64;
        while self.retained_bytes > self.size_cap_bytes && !self.retained.is_empty() {
            let (event, line_len) = self.retained.remove(0);
            self.retained_bytes = self.retained_bytes.saturating_sub(line_len);
            cutoff_ts_ms = event.ts_ms;
            dropped_events += 1;
        }
        if dropped_events == 0 {
            return;
        }
        self.markers.push(marker_event(
            EventKind::Trim,
            anchor_ts_ms,
            anchor_position_secs,
            Some(TrimMarker {
                dropped_events,
                cutoff_ts_ms,
            }),
        ));
        self.dirty = true;
    }

    /// Records a storage-exhaustion gap marker for `event`
    /// (FR-039b): the append write failed, so the event is
    /// lost; the gap is counted and the marker line is
    /// flushed before the retained entries on the next
    /// compaction.
    fn record_gap(&mut self, event: &RecordEvent) {
        self.gap_count += 1;
        self.markers.push(marker_event(
            EventKind::Gap,
            event.ts_ms,
            event.position_secs,
            None,
        ));
        self.dirty = true;
    }

    /// Rewrites the record file in the canonical on-disk
    /// order — `[trim markers][retained entries]` — so the
    /// marker line accounting for a trim is flushed before
    /// the retained entries it describes (FR-039b).
    fn compact(&mut self) -> Result<(), DiagError> {
        let mut content = String::new();
        for marker in &self.markers {
            let line = serde_json::to_string(marker)?;
            content.push_str(&line);
            content.push('\n');
        }
        for (event, _) in &self.retained {
            let line = serde_json::to_string(event)?;
            content.push_str(&line);
            content.push('\n');
        }
        write(&self.path, content)?;
        self.handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.dirty = false;
        Ok(())
    }
}

/// A marker event (trim or gap): the record envelope with no
/// optional fields except the trim marker on trim events.
const fn marker_event(
    kind: EventKind,
    ts_ms: u64,
    position_secs: f64,
    trim: Option<TrimMarker>,
) -> RecordEvent {
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
        trim,
    }
}

/// Measured latency detail keys (004 contract).
///
/// These typed keys are written as `key=value` pairs in the
/// `detail` field — never prose, ≤200 chars total, hash refs only.
pub mod detail_keys {
    /// Latency from play request to first audible audio (milliseconds).
    pub const PLAY_TO_AUDIO_MS: &str = "play_to_audio_ms";
    /// Pause duration after output device removal (milliseconds).
    pub const REMOVAL_PAUSE_MS: &str = "removal_pause_ms";
    /// Replay duration after power loss or kill (seconds).
    pub const REPLAY_SECS: &str = "replay_secs";
}

/// Constructs a conforming `detail` string from `key=value` pairs.
///
/// Each pair must match the non-prose pattern: alphanumeric/underscore
/// key, `=`, and alphanumeric/`./+/-/_` value. Pairs are space-separated.
/// Returns `None` if `pairs` is empty.
///
/// # Errors
///
/// Returns an error if any key or value contains invalid characters,
/// or if the resulting string exceeds 200 characters.
pub fn detail_from_pairs(pairs: &[(&str, &str)]) -> Result<Option<String>, DetailError> {
    if pairs.is_empty() {
        return Ok(None);
    }
    let mut tokens = Vec::with_capacity(pairs.len());
    for (key, value) in pairs {
        validate_key(key)?;
        validate_value(value)?;
        tokens.push(format!("{key}={value}"));
    }
    let detail = tokens.join(" ");
    if detail.chars().count() > 200 {
        return Err(DetailError::TooLong(detail.chars().count()));
    }
    Ok(Some(detail))
}

/// Constructs a `detail` string with the three measured latency keys.
///
/// All three values are required. Returns a conforming space-separated
/// `key=value` string, or an error if the result exceeds 200 characters.
///
/// # Errors
///
/// Returns [`DetailError::TooLong`] if the formatted string exceeds
/// 200 characters (unlikely with numeric values).
pub fn detail_measured_latency(
    play_to_audio_ms: u64,
    removal_pause_ms: u64,
    replay_secs: f64,
) -> Result<Option<String>, DetailError> {
    detail_from_pairs(&[
        (detail_keys::PLAY_TO_AUDIO_MS, &play_to_audio_ms.to_string()),
        (detail_keys::REMOVAL_PAUSE_MS, &removal_pause_ms.to_string()),
        (detail_keys::REPLAY_SECS, &format!("{replay_secs:.3}")),
    ])
}

/// Validates a detail key: non-empty, ASCII alphanumeric or underscore only.
fn validate_key(key: &str) -> Result<(), DetailError> {
    if key.is_empty() {
        return Err(DetailError::InvalidKey(key.to_string()));
    }
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(DetailError::InvalidKey(key.to_string()));
    }
    Ok(())
}

/// Validates a detail value: non-empty, ASCII alphanumeric or `./+/-/_` only.
fn validate_value(value: &str) -> Result<(), DetailError> {
    if value.is_empty() {
        return Err(DetailError::InvalidValue(value.to_string()));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_'))
    {
        return Err(DetailError::InvalidValue(value.to_string()));
    }
    Ok(())
}

/// Errors from constructing a `detail` string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DetailError {
    /// The key contains invalid characters.
    #[error("invalid detail key: {0}")]
    InvalidKey(String),
    /// The value contains invalid characters.
    #[error("invalid detail value: {0}")]
    InvalidValue(String),
    /// The resulting detail string exceeds 200 characters.
    #[error("detail too long: {0} characters (max 200)")]
    TooLong(usize),
}

impl From<DetailError> for DiagError {
    fn from(err: DetailError) -> Self {
        Self::Serialize(serde_json::Error::io(std::io::Error::other(
            err.to_string(),
        )))
    }
}

impl From<TransportValue> for engine_core::closed::TransportState {
    fn from(value: TransportValue) -> Self {
        match value {
            TransportValue::Playing => Self::Playing,
            TransportValue::Paused => Self::Paused,
            TransportValue::Stopped => Self::Stopped,
            TransportValue::Completed => Self::Completed,
            TransportValue::Interrupted => Self::Interrupted,
        }
    }
}

impl From<engine_core::closed::TransportState> for TransportValue {
    fn from(value: engine_core::closed::TransportState) -> Self {
        match value {
            engine_core::closed::TransportState::Playing => Self::Playing,
            engine_core::closed::TransportState::Paused => Self::Paused,
            engine_core::closed::TransportState::Stopped => Self::Stopped,
            engine_core::closed::TransportState::Completed => Self::Completed,
            engine_core::closed::TransportState::Interrupted => Self::Interrupted,
        }
    }
}

impl From<SessionEndValue> for engine_core::closed::SessionEnd {
    fn from(value: SessionEndValue) -> Self {
        match value {
            SessionEndValue::Paused => Self::Paused,
            SessionEndValue::Stopped => Self::Stopped,
            SessionEndValue::Completed => Self::Completed,
            SessionEndValue::Interrupted => Self::Interrupted,
        }
    }
}

impl From<engine_core::closed::SessionEnd> for SessionEndValue {
    fn from(value: engine_core::closed::SessionEnd) -> Self {
        match value {
            engine_core::closed::SessionEnd::Paused => Self::Paused,
            engine_core::closed::SessionEnd::Stopped => Self::Stopped,
            engine_core::closed::SessionEnd::Completed => Self::Completed,
            engine_core::closed::SessionEnd::Interrupted => Self::Interrupted,
        }
    }
}

impl From<ErrorValue> for engine_core::closed::EngineError {
    fn from(value: ErrorValue) -> Self {
        match value {
            ErrorValue::EngineUnreachable => Self::Unreachable,
            ErrorValue::EngineInitFailed => Self::InitFailed,
            ErrorValue::RateMismatch => Self::RateMismatch,
            ErrorValue::ChannelMismatch => Self::ChannelMismatch,
            ErrorValue::DeviceRefused => Self::DeviceRefused,
            ErrorValue::DeviceLost => Self::DeviceLost,
            ErrorValue::StartRefused => Self::StartRefused,
        }
    }
}

impl From<engine_core::closed::EngineError> for ErrorValue {
    fn from(value: engine_core::closed::EngineError) -> Self {
        match value {
            engine_core::closed::EngineError::Unreachable => Self::EngineUnreachable,
            engine_core::closed::EngineError::InitFailed => Self::EngineInitFailed,
            engine_core::closed::EngineError::RateMismatch => Self::RateMismatch,
            engine_core::closed::EngineError::ChannelMismatch => Self::ChannelMismatch,
            engine_core::closed::EngineError::DeviceRefused => Self::DeviceRefused,
            engine_core::closed::EngineError::DeviceLost => Self::DeviceLost,
            engine_core::closed::EngineError::StartRefused => Self::StartRefused,
        }
    }
}

impl From<EngineTierValue> for engine_core::closed::EngineTier {
    fn from(value: EngineTierValue) -> Self {
        match value {
            EngineTierValue::OfflineDefault => Self::OfflineDefault,
            EngineTierValue::CloudDefault => Self::CloudDefault,
            EngineTierValue::CloudEscalation => Self::CloudEscalation,
            EngineTierValue::TestDouble => Self::TestDouble,
            EngineTierValue::EvaluatedFallback => Self::EvaluatedFallback,
        }
    }
}

impl From<engine_core::closed::EngineTier> for EngineTierValue {
    fn from(value: engine_core::closed::EngineTier) -> Self {
        match value {
            engine_core::closed::EngineTier::OfflineDefault => Self::OfflineDefault,
            engine_core::closed::EngineTier::CloudDefault => Self::CloudDefault,
            engine_core::closed::EngineTier::CloudEscalation => Self::CloudEscalation,
            engine_core::closed::EngineTier::TestDouble => Self::TestDouble,
            engine_core::closed::EngineTier::EvaluatedFallback => Self::EvaluatedFallback,
        }
    }
}
