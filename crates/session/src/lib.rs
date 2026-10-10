//! Session crate — redb bookmarks + blake3 fingerprint (FR-004 / FR-015 / FR-036 / FR-039).
//! Single source of truth for position (FR-002 / FR-003).

use blake3::Hasher;
use redb::{Database, TableDefinition};

/// Bookmarks table: fingerprint → offset, paused-at, end_kind (FR-044).
pub const BOOKMARKS: TableDefinition<&[u8; 32], (f64, u64, u8)> = TableDefinition::new("bookmarks");

/// Fingerprint: blake3-256 of document bytes (FR-036 / FR-038).
/// Same bytes → same 32-byte hash regardless of path; 1-bit change → different hash.
pub fn fingerprint(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    // `update` returns `&mut Hasher` for chaining; the
    // chained form is not used here (unused_results deny).
    let _ = hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Resume offer logic (FR-031 / FR-032 / FR-032c / FR-032d):
/// — Deliberate-pause (stopped/paused): visible 3 s offer, continue-default + start-over.
/// — Unexpected-end (interrupted): paused state + explicit resume control; never auto-starts.
/// — Start-over ⇒ discard stored position (FR-023 / FR-032f).
pub fn resume_offer(_stored_offset: Option<f64>, session_end_kind: Option<u8>) -> ResumeChoice {
    match session_end_kind {
        Some(2) => ResumeChoice::ExplicitResume, // interrupted
        Some(0) | Some(1) => ResumeChoice::DeliberatePause, // paused/stopped
        _ => ResumeChoice::StartOver,
    }
}

/// The choice a resume offer presents — the closed
/// three-member set (FR-031 / FR-032c / SC-041).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    /// Deliberate pause (paused/stopped): continue from
    /// the stored offset, the 3 s offer's default.
    DeliberatePause,
    /// Unexpected end (interrupted): an explicit resume
    /// control; never auto-starts (FR-014).
    ExplicitResume,
    /// Start over from 0.0 s; the stored position is
    /// discarded (FR-023 / FR-032f).
    StartOver,
}

/// Session-end reason closed set (FR-044 / data-model.md).
/// Never `playing` on session_end; never `Other/Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionEnd {
    /// Session ended by a user pause; position stored.
    Paused,
    /// Session ended by Stop; position discarded.
    Stopped,
    /// Session ended by document completion.
    Completed,
    /// Session ended by interruption; resume offer applies.
    Interrupted,
}

/// Stored bookmark entry cleared only by Stop or document-end (FR-032f).
/// Super-session is a sequence invariant (two ordered entries), not per-entry (see T035/T067).
pub fn clear_bookmark(db: &Database, fp: &[u8; 32]) -> Result<(), redb::Error> {
    let write_tx = db.begin_write()?;
    {
        let mut table = write_tx.open_table(BOOKMARKS)?;
        // `remove` returns the previous value (an `Option`
        // guard); no caller consumes it (unused_results deny).
        let _ = table.remove(fp)?;
    }
    write_tx.commit()?;
    Ok(())
}
