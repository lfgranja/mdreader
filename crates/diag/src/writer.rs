//! Append-only JSONL diagnostic-record writer (T073).
//!
//! One [`crate::writer::RecordEvent`] per line, serialized with `serde_json`.
//! The writer is validation-free by design: the committed contract
//! schema (`specs/001-android-media-shell/contracts/diagnostic-record.schema.json`)
//! validates, per FR-039, FR-044, and FR-045.
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

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[derive(Debug)]
pub struct DiagnosticWriter {
    /// The record file, opened for appending.
    handle: File,
}

impl DiagnosticWriter {
    /// Opens (creating if absent) the record at `path` for appending.
    ///
    /// # Errors
    ///
    /// Returns [`DiagError::Io`] when the file cannot be opened.
    pub fn open(path: &Path) -> Result<Self, DiagError> {
        let handle = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { handle })
    }

    /// Appends `event` as a single JSONL line.
    ///
    /// # Errors
    ///
    /// Returns [`DiagError::Serialize`] when the event cannot be
    /// serialized, or [`DiagError::Io`] when the line cannot be
    /// written.
    pub fn append(&mut self, event: &RecordEvent) -> Result<(), DiagError> {
        let line = serde_json::to_string(event)?;
        writeln!(self.handle, "{line}")?;
        Ok(())
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
