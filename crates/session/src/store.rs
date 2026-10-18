//! The session bookmark store (T058): the redb
//! `bookmarks` + `settings` tables
//! (FR-004 / FR-015 / FR-019b / FR-032 series /
//! FR-036 / FR-044; `contracts/storage-schema.md`).
//!
//! A stored position is keyed by the blake3-256
//! fingerprint of the document's CONTENT (FR-036)
//! — never of its path — so a moved or renamed
//! document resolves to the same bookmark (FR-036)
//! and an edited document's stored position becomes
//! unreachable (FR-036a). The offer flow is
//! resolved SOLELY from the stored `end_kind`
//! (storage-schema.md): `paused` ⇒ the FR-032c
//! deliberate offer, `interrupted` ⇒ the FR-032d
//! explicit-resume form; `stopped`/`completed`
//! delete the entry instead of storing one.
//!
//! Writes are atomic and crash-safe (FR-004: a
//! hard kill recovers to the last stored write,
//! replaying at most 30 s). Only the two terminal
//! ends delete — the Stop press and the document
//! end; nothing else deletes (storage-schema.md).
//!
//! A document open composes the three pieces the
//! store owns into one callable decision
//! ([`lookup_on_open`], T036): the document bytes
//! are blake3-256 fingerprinted (FR-036), the
//! `bookmarks` table is read under that key, and
//! the offer form is resolved through the
//! canonical offer machine (`offer.rs`) — so the
//! identical-bytes-any-path rule (FR-036) and the
//! end-kind routing (FR-044) hold in one place and
//! no caller re-derives either.

use std::path::Path;

use engine_core::closed::{PlaybackRate, SessionEnd};
use redb::{Database, TableDefinition};

use crate::fingerprint::fingerprint;
use crate::offer::{OfferMachine, OpenOffer};

/// Bookmarks table: fingerprint → offset, paused-at,
/// end kind (FR-044). The value keeps the legacy
/// `(f64, u64, u8)` tuple shape, so the raw-table
/// surface (the US1 suite) reads and writes the
/// same rows directly.
pub const BOOKMARKS: TableDefinition<&[u8; 32], (f64, u64, u8)> = TableDefinition::new("bookmarks");

/// Settings table (storage-schema.md): key `str` →
/// bytes. Required key: `"last_rate"`.
const SETTINGS: TableDefinition<&str, &[u8]> = TableDefinition::new("settings");

/// The settings key the last playback rate is
/// stored under (FR-019b).
const LAST_RATE_KEY: &str = "last_rate";

/// The stored bookmark entry (storage-schema.md):
/// the last spoken position under the document's
/// content fingerprint, the start of the continuous
/// non-advancing interval (the FR-015 supersede
/// clock — never reset at supersede), and the
/// session-end kind that decides the resume-offer
/// flow (FR-044).
///
/// `Eq` is precluded by the `f64` field; every
/// offset lives on the quarter-second grid, an
/// exact binary-fraction grid, so `PartialEq` is
/// exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredPosition {
    /// The last spoken position, in seconds.
    pub offset_secs: f64,
    /// The start of the continuous non-advancing
    /// interval, in milliseconds (FR-015).
    pub first_yield_ms: u64,
    /// The session-end kind that stored the entry
    /// (FR-044): a retained entry carries `paused`
    /// or `interrupted`; `stopped`/`completed`
    /// delete the entry instead of storing one.
    pub end_kind: SessionEnd,
}

impl StoredPosition {
    /// The redb value form: the tuple the
    /// `bookmarks` table carries, with the
    /// session-end kind as its discriminant
    /// (FR-044).
    fn into_tuple(self) -> (f64, u64, u8) {
        (
            self.offset_secs,
            self.first_yield_ms,
            end_kind_to_code(self.end_kind),
        )
    }

    /// The stored position from the redb value
    /// form, or a named error when the discriminant
    /// is outside the closed session-end set
    /// (FR-044) — never a panic.
    fn from_tuple(offset_secs: f64, first_yield_ms: u64, code: u8) -> Result<Self, Error> {
        Ok(Self {
            offset_secs,
            first_yield_ms,
            end_kind: end_kind_from_code(code)?,
        })
    }
}

/// The legacy `end_kind` discriminant of a
/// session-end kind (FR-044): the closed set's
/// ordinal — `Paused` → `0`, `Stopped` → `1`,
/// `Completed` → `2`, `Interrupted` → `3`.
const fn end_kind_to_code(end_kind: SessionEnd) -> u8 {
    match end_kind {
        SessionEnd::Paused => 0,
        SessionEnd::Stopped => 1,
        SessionEnd::Completed => 2,
        SessionEnd::Interrupted => 3,
    }
}

/// The session-end kind of a legacy discriminant,
/// or a named error when the discriminant is
/// outside the closed set (FR-044): a row written
/// by an incompatible version names no end kind,
/// and a closed set has no catch-all.
const fn end_kind_from_code(code: u8) -> Result<SessionEnd, Error> {
    match code {
        0 => Ok(SessionEnd::Paused),
        1 => Ok(SessionEnd::Stopped),
        2 => Ok(SessionEnd::Completed),
        3 => Ok(SessionEnd::Interrupted),
        4..=u8::MAX => Err(Error::UnknownEndKind { code }),
    }
}

/// The settings byte a playback-rate rung encodes
/// to (FR-019): the rung's ordinal `0..=6`, in
/// ladder order.
const fn rate_to_byte(rate: PlaybackRate) -> u8 {
    match rate {
        PlaybackRate::R0_75 => 0,
        PlaybackRate::R1_0 => 1,
        PlaybackRate::R1_25 => 2,
        PlaybackRate::R1_5 => 3,
        PlaybackRate::R1_75 => 4,
        PlaybackRate::R2_0 => 5,
        PlaybackRate::R4_0 => 6,
    }
}

/// The ladder rung a settings byte names, or `None`
/// when the byte is outside the closed ladder
/// (FR-019).
const fn rate_from_byte(byte: u8) -> Option<PlaybackRate> {
    match byte {
        0 => Some(PlaybackRate::R0_75),
        1 => Some(PlaybackRate::R1_0),
        2 => Some(PlaybackRate::R1_25),
        3 => Some(PlaybackRate::R1_5),
        4 => Some(PlaybackRate::R1_75),
        5 => Some(PlaybackRate::R2_0),
        6 => Some(PlaybackRate::R4_0),
        7..=u8::MAX => None,
    }
}

/// The ladder rung a settings value carries, or a
/// named error when the value is not the one byte
/// a rung encodes to (FR-019).
fn rate_from_slice(bytes: &[u8]) -> Result<PlaybackRate, Error> {
    let [byte] = bytes else {
        return Err(Error::MalformedRate);
    };
    rate_from_byte(*byte).ok_or(Error::UnknownRate)
}

/// The session-store error set — a closed set: no
/// `Other`/`Unknown` catch-all (FR-044).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The underlying redb storage operation
    /// failed.
    #[error("redb storage failure: {0}")]
    Storage(Box<redb::Error>),
    /// A stored `end_kind` discriminant is not a
    /// member of the closed session-end set
    /// (FR-044) — the row was written by an
    /// incompatible version.
    #[error("unknown session-end discriminant {code}")]
    UnknownEndKind {
        /// The unreadable discriminant.
        code: u8,
    },
    /// The stored `last_rate` value is not a
    /// member of the closed seven-rung ladder
    /// (FR-019).
    #[error("the stored last_rate is not a ladder rung")]
    UnknownRate,
    /// The stored `last_rate` value is malformed —
    /// not the one byte a ladder rung encodes
    /// to.
    #[error("the stored last_rate is malformed")]
    MalformedRate,
}

impl From<redb::Error> for Error {
    fn from(error: redb::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}

// redb reports its failures through granular error
// types (`DatabaseError`, `TransactionError`,
// `TableError`, `StorageError`, `CommitError`)
// that all convert into the `redb::Error`
// superset; the bridges below keep every storage
// failure in the single [`Error::Storage`] kind,
// so `?` works at every redb call site.
impl From<redb::DatabaseError> for Error {
    /// The database open/create failure as the
    /// storage kind.
    fn from(error: redb::DatabaseError) -> Self {
        Self::Storage(Box::new(redb::Error::from(error)))
    }
}

impl From<redb::TransactionError> for Error {
    /// The transaction begin failure as the storage
    /// kind.
    fn from(error: redb::TransactionError) -> Self {
        Self::Storage(Box::new(redb::Error::from(error)))
    }
}

impl From<redb::TableError> for Error {
    /// The table open failure as the storage kind.
    fn from(error: redb::TableError) -> Self {
        Self::Storage(Box::new(redb::Error::from(error)))
    }
}

impl From<redb::StorageError> for Error {
    /// The table operation failure as the storage
    /// kind.
    fn from(error: redb::StorageError) -> Self {
        Self::Storage(Box::new(redb::Error::from(error)))
    }
}

impl From<redb::CommitError> for Error {
    /// The commit failure as the storage kind.
    fn from(error: redb::CommitError) -> Self {
        Self::Storage(Box::new(redb::Error::from(error)))
    }
}

/// The redb-backed bookmark + settings store
/// (storage-schema.md): atomic, crash-safe writes
/// (FR-004) over one database file.
pub const DURABLE_CADENCE_MS: u64 = 20000; // 20 s fixed (FR-007 / data-model.md:43; envelope 15–30 s)

/// The redb-backed bookmark + settings store (FR-004).
pub struct BookmarkStore {
    /// The redb database file.
    db: Database,
}

impl std::fmt::Debug for BookmarkStore {
    /// The debug form: the store's identity — the
    /// database handle's internals are not
    /// reproduced.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookmarkStore").finish_non_exhaustive()
    }
}

impl BookmarkStore {
    /// The store over the redb file at `path`,
    /// creating it — and its tables — when absent.
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the database file
    /// cannot be created or opened, or a table
    /// cannot be created.
    pub fn open(path: &Path) -> Result<Self, Error> {
        let db = Database::create(path)?;
        // A read transaction's `open_table` requires
        // the table to exist, so the tables are
        // created up front (a write transaction's
        // `open_table` creates on first use).
        let write_tx = db.begin_write()?;
        {
            let _bookmarks = write_tx.open_table(BOOKMARKS)?;
            let _settings = write_tx.open_table(SETTINGS)?;
        }
        write_tx.commit()?;
        Ok(Self { db })
    }

    /// Store `position` under the document's content
    /// `fingerprint` (FR-036), persisting the
    /// `end_kind` on every write (FR-044): the
    /// write is the pause / app-close / focus-loss
    /// / engine-failure record (storage-schema.md).
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the write
    /// transaction cannot begin or commit.
    /// Durable heartbeat — fixed 20 s cadence write
    /// (FR-007 / data-model.md:43; envelope 15–30 s).
    /// Called by session timer while Playing; also
    /// by event-durable writes (pause, focus-loss,
    /// device-removal, section-change, Stop/DocumentEnd,
    /// app-background).
    pub fn durable_heartbeat(
        &mut self,
        fingerprint: &[u8; 32],
        position: &StoredPosition,
    ) -> Result<(), Error> {
        self.put(fingerprint, *position)
    }

    /// Write `position` under the document's content
    /// `fingerprint` (FR-036) — overwriting any previous
    /// bookmark for the same document.
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the write
    /// transaction cannot begin or commit.
    pub fn put(&mut self, fingerprint: &[u8; 32], position: StoredPosition) -> Result<(), Error> {
        let write_tx = self.db.begin_write()?;
        {
            let mut table = write_tx.open_table(BOOKMARKS)?;
            // `insert` returns the previous value (an
            // `Option` guard); no caller consumes it
            // (unused_results deny).
            let _previous = table.insert(fingerprint, &position.into_tuple())?;
        }
        write_tx.commit()?;
        Ok(())
    }

    /// The stored position under the document's
    /// content `fingerprint`, or `None` when no
    /// entry survives — a miss (an edited document,
    /// FR-036a; a terminal end, FR-032 /
    /// FR-032b) resolves to a start at `0.0` with
    /// no offer (storage-schema.md).
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the read
    /// transaction cannot begin, [`Error::UnknownEndKind`]
    /// when a stored discriminant names no
    /// session-end kind (FR-044).
    pub fn get(&mut self, fingerprint: &[u8; 32]) -> Result<Option<StoredPosition>, Error> {
        let read_tx = self.db.begin_read()?;
        let table = read_tx.open_table(BOOKMARKS)?;
        let guard = table.get(fingerprint)?;
        guard
            .map(|entry| {
                let (offset_secs, first_yield_ms, code) = entry.value();
                StoredPosition::from_tuple(offset_secs, first_yield_ms, code)
            })
            .transpose()
    }

    /// Delete the entry under `fingerprint` — the
    /// only two callers are the terminal ends: the
    /// Stop press and the document end
    /// (FR-032 / FR-032b / FR-032f). Nothing else
    /// deletes (storage-schema.md).
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the write
    /// transaction cannot begin or commit.
    pub fn delete(&mut self, fingerprint: &[u8; 32]) -> Result<(), Error> {
        let write_tx = self.db.begin_write()?;
        {
            let mut table = write_tx.open_table(BOOKMARKS)?;
            // `remove` returns the previous value (an
            // `Option` guard); no caller consumes it
            // (unused_results deny).
            let _previous = table.remove(fingerprint)?;
        }
        write_tx.commit()?;
        Ok(())
    }

    /// Persist the last playback rate (FR-019b):
    /// written on every rate change, crossing
    /// sessions and documents.
    ///
    /// This is the store half of the T042 wiring —
    /// the write-on-change half: the call site
    /// (the shell, over the `player::rate` stepper)
    /// writes the rung every time a rate move
    /// changes it, and reads it back at the next
    /// session open through [`get_last_rate`](Self::get_last_rate).
    /// The setting is document-independent (the
    /// `settings` table), so it survives a
    /// document change and a store reopen.
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the write
    /// transaction cannot begin or commit.
    pub fn put_last_rate(&mut self, rate: PlaybackRate) -> Result<(), Error> {
        let write_tx = self.db.begin_write()?;
        {
            let mut table = write_tx.open_table(SETTINGS)?;
            // `insert` returns the previous value (an
            // `Option` guard); no caller consumes it
            // (unused_results deny).
            let value = [rate_to_byte(rate)];
            let _previous = table.insert(LAST_RATE_KEY, value.as_slice())?;
        }
        write_tx.commit()?;
        Ok(())
    }

    /// The last playback rate (FR-019b), or `None`
    /// on first use — the call-site default is the
    /// normal-speed rung `1.0` (storage-schema.md).
    ///
    /// This is the store half of the T042 wiring —
    /// the read-at-start half: the call site reads
    /// it when a session opens and resolves the
    /// default (the normal-speed rung) once on the
    /// player side (`player::rate::rate_at_start`),
    /// so no second default exists.
    ///
    /// # Errors
    ///
    /// [`Error::Storage`] when the read
    /// transaction cannot begin, [`Error::MalformedRate`]
    /// or [`Error::UnknownRate`] when the stored
    /// value is not a ladder rung (FR-019).
    pub fn get_last_rate(&mut self) -> Result<Option<PlaybackRate>, Error> {
        let read_tx = self.db.begin_read()?;
        let table = read_tx.open_table(SETTINGS)?;
        let guard = table.get(LAST_RATE_KEY)?;
        guard
            .map(|entry| {
                let bytes = entry.value();
                rate_from_slice(bytes)
            })
            .transpose()
    }
}

/// The decision a document-open lookup resolves
/// (T036): the document's content fingerprint, the
/// stored position it keys to, and the offer form
/// that end kind opens (FR-036 / FR-044).
///
/// `Eq` is precluded by the `f64` field; every
/// offset lives on the quarter-second grid, an
/// exact binary-fraction grid, so `PartialEq` is
/// exact.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenLookup {
    /// The blake3-256 fingerprint of the document's
    /// CONTENT (FR-036) — a pure function of the
    /// bytes, never of the path: identical bytes at
    /// any path (a move, a rename, a file URI)
    /// fingerprint here, so they share the row and
    /// therefore the stored position.
    pub fingerprint: [u8; 32],
    /// The stored position under the fingerprint,
    /// or `None` on a miss — an edited document
    /// (FR-036a) or a terminal end (FR-032 /
    /// FR-032b), each resolving to a start at `0.0`
    /// with no offer.
    pub stored: Option<StoredPosition>,
    /// The offer form the stored end kind opens
    /// (FR-032c / FR-032d / FR-032e), resolved
    /// through the canonical offer machine
    /// ([`OfferMachine::resolve_open`]) — never
    /// re-derived here (FR-044).
    pub offer: OpenOffer,
}

/// The re-hash-on-open lookup (T036): the
/// document-open decision in one callable form
/// (FR-036 / FR-044).
///
/// The rule the composition establishes:
/// identical bytes at ANY path ⇒ the same
/// fingerprint ⇒ the same row (FR-036 —
/// path/URI-independent), so a moved or renamed
/// document shares its stored position, while a
/// single edited byte fingerprints elsewhere and
/// resolves to a miss — a start at `0.0` with no
/// offer (FR-036a).
///
/// # Errors
///
/// [`Error::Storage`] when the read transaction
/// cannot begin, [`Error::UnknownEndKind`] when a
/// stored discriminant names no session-end kind
/// (FR-044).
pub fn lookup_on_open(store: &mut BookmarkStore, bytes: &[u8]) -> Result<OpenLookup, Error> {
    // The key is the CONTENT hash — never the path
    // (FR-036): the bytes are all this function
    // sees, so no path can enter the decision.
    let fingerprint = fingerprint(bytes);
    let stored = store.get(&fingerprint)?;
    // The offer form routes through the canonical
    // machine's decision — the one core
    // `on_session_end` also runs (FR-044).
    let mut machine = OfferMachine::new();
    let offer = machine.resolve_open(stored);
    Ok(OpenLookup {
        fingerprint,
        stored,
        offer,
    })
}

/// Stored bookmark entry cleared only by Stop or document end (FR-032f).
/// Super-session is a sequence invariant (two ordered entries), not per-entry (see T035/T067).
///
/// # Errors
///
/// [`redb::Error`] when the write transaction cannot begin or commit.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The document bytes every test opens — the
    /// lookup sees bytes only, never a path (FR-036).
    const DOC: &[u8] = b"# Chapter\n\nThe document bytes.";

    /// A store over a tempdir redb file.
    fn store_in(dir: &tempfile::TempDir) -> Result<BookmarkStore, Error> {
        BookmarkStore::open(&dir.path().join("bookmarks.redb"))
    }

    /// A stored position on the quarter-second grid.
    fn position(end_kind: SessionEnd) -> StoredPosition {
        StoredPosition {
            offset_secs: 30.25,
            first_yield_ms: 500,
            end_kind,
        }
    }

    #[test]
    fn lookup_on_open_continues_a_paused_row() -> Result<(), Box<dyn std::error::Error>> {
        // T036: bytes → fingerprint → get → the
        // end-kind-routed offer. A deliberate pause
        // carries the continue offer with the stored
        // offset and the FR-015 interval start.
        let dir = tempfile::tempdir()?;
        let mut store = store_in(&dir)?;
        store.put(&fingerprint(DOC), position(SessionEnd::Paused))?;

        let lookup = lookup_on_open(&mut store, DOC)?;
        assert_eq!(lookup.fingerprint, fingerprint(DOC));
        assert_eq!(lookup.stored, Some(position(SessionEnd::Paused)));
        assert_eq!(
            lookup.offer,
            OpenOffer::Continue {
                offset_secs: 30.25,
                first_yield_ms: 500,
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_on_open_routes_every_end_kind() -> Result<(), Box<dyn std::error::Error>> {
        // The offer form is resolved solely from the
        // stored end kind: `paused` continues,
        // `interrupted` demands the explicit resume,
        // the terminal kinds and a deleted row open
        // nothing (FR-032c / FR-032d / FR-032e).
        let dir = tempfile::tempdir()?;
        let mut store = store_in(&dir)?;
        for end_kind in [
            SessionEnd::Paused,
            SessionEnd::Interrupted,
            SessionEnd::Stopped,
            SessionEnd::Completed,
        ] {
            store.put(&fingerprint(DOC), position(end_kind))?;
            let expected = match end_kind {
                SessionEnd::Paused => OpenOffer::Continue {
                    offset_secs: 30.25,
                    first_yield_ms: 500,
                },
                SessionEnd::Interrupted => OpenOffer::ExplicitResume,
                SessionEnd::Stopped | SessionEnd::Completed => OpenOffer::NoOffer,
            };
            assert_eq!(
                lookup_on_open(&mut store, DOC)?.offer,
                expected,
                "{end_kind:?} must resolve to {expected:?}"
            );
        }
        // The terminal ends' real state is the
        // deleted row: a miss opens nothing.
        store.delete(&fingerprint(DOC))?;
        let lookup = lookup_on_open(&mut store, DOC)?;
        assert_eq!(lookup.stored, None);
        assert_eq!(lookup.offer, OpenOffer::NoOffer);
        Ok(())
    }

    #[test]
    fn identical_bytes_at_any_path_share_the_stored_position()
    -> Result<(), Box<dyn std::error::Error>> {
        // FR-036: the key is the content hash, never
        // the path — the lookup carries no path at
        // all, so the same bytes opened "from
        // anywhere" (a move, a rename, a URI) resolve
        // to the same row and share the position,
        // while one edited byte is a different
        // document: a miss, a start at `0.0`, no
        // offer (FR-036a).
        let dir = tempfile::tempdir()?;
        let mut store = store_in(&dir)?;
        // The document opened "at path A" stored its
        // position; the bytes are the only key.
        store.put(&fingerprint(DOC), position(SessionEnd::Paused))?;

        // The same bytes opened "at path B".
        let second = lookup_on_open(&mut store, DOC)?;
        assert_eq!(second.fingerprint, fingerprint(DOC));
        assert_eq!(second.stored, Some(position(SessionEnd::Paused)));
        assert_eq!(
            second.offer,
            OpenOffer::Continue {
                offset_secs: 30.25,
                first_yield_ms: 500,
            }
        );

        // One edited byte: a different fingerprint,
        // no row, no offer.
        let edited = b"# Chapter\n\nThe document bytes!";
        let edited_lookup = lookup_on_open(&mut store, edited)?;
        assert_ne!(edited_lookup.fingerprint, fingerprint(DOC));
        assert_eq!(edited_lookup.stored, None);
        assert_eq!(edited_lookup.offer, OpenOffer::NoOffer);
        Ok(())
    }

    #[test]
    fn lookup_on_open_miss_starts_at_zero_with_no_offer() -> Result<(), Box<dyn std::error::Error>>
    {
        // First use: no row survives, so the session
        // starts at `0.0` with no offer (FR-036a) —
        // the lookup reports the miss rather than a
        // position.
        let dir = tempfile::tempdir()?;
        let mut store = store_in(&dir)?;
        let lookup = lookup_on_open(&mut store, DOC)?;
        assert_eq!(lookup.stored, None);
        assert_eq!(lookup.offer, OpenOffer::NoOffer);
        Ok(())
    }

    #[test]
    fn last_rate_round_trips_through_a_reopened_tempdir_store()
    -> Result<(), Box<dyn std::error::Error>> {
        // T042 / FR-019b: the store half of the
        // persistence wiring over a real redb file —
        // first use reads no rung; every write-on-change
        // is read back at the next session open (the
        // store closed and reopened); and another
        // document's bookmark write never disturbs the
        // document-independent setting. The exhaustive
        // seven-rung round-trip is pinned in
        // `tests/prop_resume.rs`; this half walks the
        // reopen lifecycle at the ladder's bounds and
        // its middle.
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("bookmarks.redb");

        // First use (read-at-start on a fresh store):
        // no row, so the call site resolves its default.
        {
            let mut store = BookmarkStore::open(&path)?;
            assert_eq!(store.get_last_rate()?, None);
        }

        for rung in [PlaybackRate::R0_75, PlaybackRate::R1_25, PlaybackRate::R4_0] {
            // Write-on-change: the rate move persists the
            // rung the stepper reported.
            {
                let mut store = BookmarkStore::open(&path)?;
                store.put_last_rate(rung)?;
            }
            // Another document's bookmark write must not
            // disturb the document-independent setting.
            {
                let mut store = BookmarkStore::open(&path)?;
                let key = fingerprint(DOC);
                store.put(
                    &key,
                    StoredPosition {
                        offset_secs: 1.5,
                        first_yield_ms: 0,
                        end_kind: SessionEnd::Paused,
                    },
                )?;
                assert_eq!(
                    store.get_last_rate()?,
                    Some(rung),
                    "a bookmark write changed the stored last_rate"
                );
            }
            // A new session — the store closed and
            // reopened — reads the chosen rung back at
            // start (read-at-start).
            {
                let mut store = BookmarkStore::open(&path)?;
                assert_eq!(
                    store.get_last_rate()?,
                    Some(rung),
                    "the stored last_rate did not survive the reopen (rung {rung:?} lost)"
                );
            }
        }
        Ok(())
    }
}
