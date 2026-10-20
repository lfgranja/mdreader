//! The append-only diagnostic record (FR-017 / T097).
//!
//! The record is a PRIVATE, on-device trace: one JSONL line per
//! event, appended and never rewritten, living in the app's own
//! storage. It is never transmitted — no network call site, no
//! upload, no share surface — and it has no in-app surface: the
//! only access path is a maintainer pulling the file off the
//! device (FR-017 / SC-008).
//!
//! The writer itself lives in the `diag` crate (the record writer,
//! the retention bounds and the closed scanners); this module is the
//! shell-side seam. It owns the ONE writer instance the app has,
//! opens it in the app's private storage, and appends the events
//! the session lifecycle produces.
//!
//! Every append is best-effort by construction: a closed record, a
//! full device or a poisoned lock drops the event with a tracing
//! record and returns — playback is never interrupted by the record
//! (FR-039b).

use std::path::Path;
use std::sync::Mutex;

use diag::writer::{DiagnosticWriter, RecordEvent};

/// The record file name inside the app's private storage.
///
/// The name is fixed so the maintainer's pull path is stable: the
/// file lives beside the bookmark store in the app's own data
/// directory.
pub const RECORD_FILE_NAME: &str = "diagnostic-record.jsonl";

/// The app's ONE diagnostic record.
///
/// The recorder is created with the application state and opened
/// once the app's private data directory is known. Until then every
/// append is a no-op: the record degrades to "no record", never to
/// a panic.
#[derive(Debug)]
pub struct Recorder {
    /// The record writer; `None` until the record is opened.
    writer: Mutex<Option<DiagnosticWriter>>,
}

impl Recorder {
    /// A recorder with no record open yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            writer: Mutex::new(None),
        }
    }

    /// Open (creating it when absent) the record at `path`.
    ///
    /// # Errors
    ///
    /// Returns the open failure's message when the file cannot be
    /// opened or read, or the lock's message when the recorder's own
    /// lock is poisoned. The caller records the failure and keeps
    /// running: a missing record degrades the diagnostics, never the
    /// playback.
    pub fn open(&self, path: &Path) -> Result<(), String> {
        let writer = DiagnosticWriter::open(path).map_err(|error| error.to_string())?;
        let Ok(mut guard) = self.writer.lock() else {
            return Err("the diagnostic record lock is poisoned".to_string());
        };
        *guard = Some(writer);
        Ok(())
    }

    /// Whether the record is open and can accept entries.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.writer.lock().is_ok_and(|guard| guard.is_some())
    }

    /// Append one event to the record.
    ///
    /// A closed record, a poisoned lock or a storage failure drops
    /// the event with a tracing record — the record is a diagnostic
    /// aid, never a reason to interrupt the session (FR-039b).
    pub fn append(&self, event: &RecordEvent) {
        let Ok(mut guard) = self.writer.lock() else {
            tracing::warn!("the diagnostic record was not written: the lock is poisoned");
            return;
        };
        let Some(writer) = guard.as_mut() else {
            return; // no record opened — the event is dropped
        };
        if let Err(error) = writer.append(event) {
            tracing::warn!(%error, "the diagnostic record was not written");
        }
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}
