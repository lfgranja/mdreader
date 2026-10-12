//! Session crate — the position authority's
//! storage, fingerprint, and resume-offer flow
//! (FR-002 / FR-003 / FR-004 / FR-015 / FR-019b /
//! FR-031 / FR-032 series / FR-036 / FR-036a /
//! FR-038 / FR-039 / FR-044).
//!
//! Single source of truth for position
//! (FR-002 / FR-003). The canonical layout:
//!
//! - [`store`] (T058) — the redb bookmark store:
//!   the `bookmarks` table (content fingerprint →
//!   `{offset_secs, first_yield_ms, end_kind}`,
//!   storage-schema.md) and the `settings`
//!   table's `last_rate` (FR-019b).
//! - [`fingerprint`] (T059) — the blake3-256
//!   content fingerprint (FR-036 / FR-036a) and
//!   the FR-038 re-anchor.
//! - [`offer`] (T060) — the resume-offer state
//!   machine (FR-032c / FR-032d / FR-032f), the
//!   offer form resolved solely from the stored
//!   end kind.
//!
//! The crate root keeps the legacy surface the US1
//! recovery suite depends on: the raw `bookmarks`
//! table definition, the raw-table
//! `clear_bookmark`, the content `fingerprint`
//! (re-exported from the [`fingerprint`] module —
//! the module lives in the type namespace, the
//! function in the value namespace), and the
//! closed `resume_offer` / `ResumeChoice` /
//! `SessionEnd` vocabulary.

/// The session text-I/O protocol (T063): the CLI
/// surface every driver shares (Principle II).
pub mod cli;
pub mod fingerprint;
pub mod offer;
pub mod store;

pub use fingerprint::fingerprint;
pub use store::{BOOKMARKS, clear_bookmark};

/// Resume offer logic (FR-031 / FR-032 / FR-032c / FR-032d):
/// — Deliberate-pause (stopped/paused): visible 3 s offer, continue-default + start-over.
/// — Unexpected-end (interrupted): paused state + explicit resume control; never auto-starts.
/// — Start-over ⇒ discard stored position (FR-023 / FR-032f).
pub fn resume_offer(_stored_offset: Option<f64>, session_end_kind: Option<u8>) -> ResumeChoice {
    match session_end_kind {
        Some(2) => ResumeChoice::ExplicitResume, // interrupted
        Some(0) | Some(1) => ResumeChoice::DeliberatePause, // paused/stopped
        // Unknown codes (3..=255) and no stored kind both fall back
        // to start-over (FR-032d); arms stay exhaustive with no wildcard.
        Some(3..=u8::MAX) => ResumeChoice::StartOver,
        None => ResumeChoice::StartOver,
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
