//! Session crate — redb bookmarks + blake3 fingerprint (FR-004 / FR-015 / FR-036 / FR-039).
//! Single source of truth for position (FR-002 / FR-003).

use blake3::Hasher;
use redb::{Database, TableDefinition};
use std::path::Path;

/// Bookmarks table: fingerprint → offset, paused-at, sentence_idx, end_kind (FR-044).
pub const BOOKMARKS: TableDefinition<&[u8; 32], (f64, u64, u32, u8)> = TableDefinition::new("bookmarks");

/// Fingerprint: blake3-256 of document bytes (FR-036 / FR-038).
/// Same bytes → same 32-byte hash regardless of path; 1-bit change → different hash.
pub fn fingerprint(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Resume offer logic (FR-031 / FR-032 / FR-032c / FR-032d):
/// — Deliberate-pause (stopped/paused): visible 3 s offer, continue-default + start-over.
/// — Unexpected-end (interrupted): paused state + explicit resume control; never auto-starts.
/// — Start-over ⇒ discard stored position (FR-023 / FR-032f).
pub fn resume_offer(stored_offset: Option<f64>, session_end_kind: Option<u8>) -> ResumeChoice {
    match session_end_kind {
        Some(2) => ResumeChoice::ExplicitResume, // interrupted
        Some(0) | Some(1) => ResumeChoice::DeliberatePause, // paused/stopped
        _ => ResumeChoice::StartOver,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    DeliberatePause,
    ExplicitResume,
    StartOver,
}

/// Session-end reason closed set (FR-044 / data-model.md).
/// Never `playing` on session_end; never `Other/Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionEnd {
    Paused,
    Stopped,
    Completed,
    Interrupted,
}

/// Stored bookmark entry cleared only by Stop or document-end (FR-032f).
/// Super-session is a sequence invariant (two ordered entries), not per-entry (see T035/T067).
pub fn clear_bookmark(db: &Database, fp: &[u8; 32]) -> Result<(), redb::Error> {
    let mut write_tx = db.begin_write()?;
    {
        let mut table = write_tx.open_table(BOOKMARKS)?;
        table.remove(fp)?;
    }
    write_tx.commit()?;
    Ok(())
}
