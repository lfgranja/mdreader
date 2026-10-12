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
//! The crate root re-exports the shared surface:
//! the raw `bookmarks` table definition, the
//! raw-table `clear_bookmark`, and the content
//! `fingerprint` (re-exported from the
//! [`fingerprint`] module — the module lives in
//! the type namespace, the function in the value
//! namespace). The closed session-end vocabulary
//! lives in `engine_core::closed` (shared via
//! [`store`] and [`offer`]); the resume offer
//! itself is the canonical [`offer::OfferMachine`].

/// The session text-I/O protocol (T063): the CLI
/// surface every driver shares (Principle II).
pub mod cli;
pub mod fingerprint;
pub mod offer;
pub mod store;

pub use fingerprint::fingerprint;
pub use store::{BOOKMARKS, clear_bookmark};
