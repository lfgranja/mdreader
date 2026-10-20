//! Tauri bridge — IPC delegation to the Rust playback path.
//!
//! Every surface (notification, lock screen, CLI, IPC) executes the
//! same Rust code (Principle II): `run_mobile_plugin` splits the
//! document with the adapter, synthesizes it with the bound
//! offline engine, enqueues the PCM on the CPAL backend, and
//! advances the single playback session. No transport, rate, or
//! position logic lives in Kotlin or the frontend.
//!
//! Durability cadence (T035, FR-007 as single source): the
//! position a reader perceives frame-to-frame is the in-RAM
//! [`player::position::PositionClock`] — it advances only on
//! audible progress, never on its own — sampled at the ~1 s
//! in-RAM cadence the backend's audible-progress feed already
//! produces, and this module adds no timer of its own to watch
//! it. That clock is also the only source of the per-sentence
//! snap and the 5–10 s rewind cue, which are never durable
//! writes. The DURABLE row is written on exactly two cadences:
//! the 20 s heartbeat ([`HEARTBEAT_PERIOD_SECS`], T011) while
//! `Playing`, and the event-durable call sites — every pause
//! (deliberate, focus-loss, `BecomingNoisy`/
//! `DeviceDisappeared`), each section-change (a skip), the
//! pre-suspend app-background write, and the two DELETING ends
//! (a `Stop` press and `DocumentEnd`, FR-032 / FR-032b). A hard
//! kill therefore replays at most ~30 s (FR-004).
//!
//! Cold-start open (T019): [`commands::open_document`] takes the
//! snapshot BYTES — the sole synthesis input, never a URI or
//! path — fingerprints them as content (FR-036), resolves the
//! open lookup through [`session::store::lookup_on_open`] (T036),
//! publishes the resume-offer route (T037), and reuses the
//! `play` path for the one synthesis path (Principle II). It
//! owns the ≤3 s press-to-first-audio budget (SC-002) end-to-end.
//!
//! Rate persistence (T042 / FR-019b): the shell composes the two
//! halves the player crate and the store define — at session
//! open it seeds the session at the stored `last_rate` (read-at
//! start), and every rate dispatch persists the new rung
//! (write-on-change) through `persist_last_rate`, value-blind at
//! every step (FR-012).

#![expect(
    clippy::multiple_crate_versions,
    reason = "every duplicate is third-party-forced at newest-release pins: the Tauri 2 / Android / desktop dependency tree carries base64 0.21+0.22+0.23, bitflags 1+2, cpufeatures 0.2+0.3, hashbrown 0.12+0.17, heck 0.4+0.5, indexmap 1+2, jni 0.21+0.22, jni-sys 0.3+0.4, miniz_oxide 0.8+0.9, png 0.17+0.18, proc-macro-crate 1+2+3, r-efi 5+6, schemars 0.8+0.9+1, serde_spanned 0.6+1, syn 1+2+3, thiserror 1+2, thiserror-impl 1+2, toml_datetime 0.6+1, toml_edit 0.19+0.20+0.25 and winnow 0.5+1 — cargo cannot unify across semver majors, crates.io shows no newer unifying release, and supply-chain governance for these transitive duplicates belongs to cargo-deny, not this lint"
)]

use std::sync::Mutex;

use diag::writer::{
    DeviceEvent, ErrorValue, EventKind, FocusValue, RecordEvent, SessionEndValue, TransportValue,
};
use engine_core::closed::SessionEnd;
use player::backend::CpalBackend;
use player::cli::{PlaybackSession, SessionSnapshot, ShellVerb};
use player::focus::FocusOutcome;
use player::rate::RateStep;
use player::surface::MirrorState;
use player::transport::{PlaybackRate, TransportState};
use session::fingerprint;
use session::store::{BookmarkStore, StoredPosition};
use std::time::Duration;
use tauri::Manager;
use tokio::time;

/// The permission-state gate (T030b).
///
/// The pure, headless half of FR-008/FR-010/FR-021. The
/// Kotlin shell reads the platform permissions and pushes the
/// answer as a `PermissionSnapshot`; the startup and
/// pre-play gates consume it. No Android API calls live
/// here (Principle V).
pub mod permissions;

/// The Kotlin-to-Rust shell channel (T111).
///
/// The integer-code drain the `ShellHost` bootstrap posts
/// through. Installed once in [`run`]; consulted (never
/// constructed) by [`commands::register_android_plugin`].
pub mod shell_jni;

/// The shared reason shapes every surface renders (T065).
///
/// Short lock-screen strings, full in-app strings, and the
/// reason expiry (3 s focus-loss vs persistent failure).
/// T065b's status view and T078's failure surface render
/// through these getters — never a second definition.
pub mod surface;

/// The minimal in-app status view (T065b).
///
/// The read-only render of the stored `last_rate`, the
/// transport/position mirror, and the current failure/focus
/// reason with its expiry — through the T065 getters alone,
/// with no playback logic and no second string definition.
pub mod status_view;

/// The engine-failure surface (T078).
///
/// The named failure + alternatives rendering, conforming to
/// the T065 reason shapes.
pub mod failure;

/// The append-only diagnostic record (FR-017 / T097).
///
/// The private, on-device JSONL trace: the writer seam the
/// session lifecycle appends its events through. The record is
/// never transmitted and has no in-app surface — a maintainer
/// pulls it off the device.
pub mod diag_record;

/// Shared application state for the Tauri bridge.
///
/// One playback session, one lazily-opened audio backend,
/// the platform permission snapshot the Kotlin shell
/// pushes, the held focus/offer machines (T125, FR-014 /
/// FR-032c), and the Rust→shell publish channel (T133,
/// FR-003 / FR-028): streams `MirrorState` snapshots on
/// every transport/position/focus event. Does not derive
/// Debug because it contains `tauri::ipc::Channel` which
/// does not implement Debug.
pub struct AppState {
    /// The single playback session (sole position authority).
    session: Mutex<PlaybackSession>,
    /// The audio backend, opened on first play.
    backend: Mutex<Option<CpalBackend>>,
    /// The platform permission snapshot (T030b): pushed by
    /// the Kotlin shell through `update_permissions`, read
    /// by the startup and pre-play gates. The all-granted
    /// host baseline holds until the first push (FR-008,
    /// FR-010, FR-021).
    permissions: Mutex<permissions::PermissionSnapshot>,
    /// The held focus tracker (T125, FR-014): every focus
    /// intent in [`commands::dispatch_intent`] applies
    /// here, so latest-event-wins supersede and the
    /// duck/resume flags persist across IPC calls.
    pub(crate) focus_tracker: Mutex<player::focus::FocusTracker>,
    /// The held resume-offer machine (T125, FR-032c /
    /// FR-032d): the lifecycle latches write here and the
    /// foreground latch is read at expiry only — never
    /// converting an open offer early.
    pub(crate) offer: Mutex<session::offer::OfferMachine>,
    /// The parked cold-start open (T110): the document whose
    /// resume offer is open, waiting for the offer to resolve
    /// before ANY audio begins (FR-009 / FR-032c). `None` when
    /// no offer is open.
    pub(crate) pending_open: Mutex<Option<PendingOpen>>,
    /// The Rust→shell publish channel (T133): created on
    /// first frontend listener registration, used to
    /// stream `MirrorState` snapshots (sanctioned
    /// Rust→frontend direction via `tauri::ipc::Channel`).
    pub(crate) publish_channel: Mutex<Option<tauri::ipc::Channel<MirrorState>>>,
    /// The surface resolver for the current document (T133):
    /// holds the title, filename, and duration knowledge
    /// fixed at session start (FR-028a / FR-029). `None`
    /// when no document is loaded.
    pub(crate) surface_resolver: Mutex<Option<player::surface::SurfaceResolver>>,
    /// The document headings for the current session (T133):
    /// ordered list of (`position_secs`, title) for section
    /// title resolution (FR-028b). Empty when no document
    /// is loaded or document has no headings.
    pub(crate) headings: Mutex<Vec<player::surface::Heading>>,
    /// The 20 s durable heartbeat bookmark store (T011):
    /// periodic `put` while Playing for crash-safe
    /// position recovery (FR-004 / FR-007 / FR-032).
    /// Opened lazily on first session start with the
    /// app's data directory.
    bookmark_store: Mutex<Option<BookmarkStore>>,
    /// The blake3-256 content fingerprint of the currently
    /// loaded document (FR-036). `None` when no document
    /// is loaded. Used by the 20 s heartbeat (T011) to
    /// persist the position under the correct key.
    current_fingerprint: Mutex<Option<[u8; 32]>>,
    /// The wall-clock start of the current continuous
    /// playback interval, in unix milliseconds — the
    /// FR-015 supersede clock every bookmark write
    /// stamps as `first_yield_ms` (T011). `None` while
    /// no interval is open: the clock opens when
    /// playback starts, survives a deliberate pause and
    /// its resume (one continuous interval — the clock
    /// is never reset mid-session), and closes at the
    /// terminal ends (`Stopped` / `Completed`), which
    /// end the interval it anchors. A write with the
    /// clock closed carries `0` — the "no interval
    /// recorded" value the raw-table surface accepts
    /// (T063 / storage-schema.md).
    first_yield_ms: Mutex<Option<u64>>,
    /// The diagnostic record (T097, FR-017): the append-only
    /// JSONL trace in the app's private storage. Opened once the
    /// app's data directory is known; every append is
    /// best-effort, and the record is never transmitted.
    pub(crate) diag: diag_record::Recorder,
    /// The measured-latency origins the diagnostic record carries
    /// (T097): the play-to-audio clock, the device-removal pause
    /// clock, and the replay window the kill/power-loss recovery
    /// opens.
    pub(crate) diag_measures: Mutex<DiagMeasures>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("session", &self.session)
            .field("backend", &self.backend)
            .field("permissions", &self.permissions)
            .field("focus_tracker", &self.focus_tracker)
            .field("offer", &self.offer)
            .field("pending_open", &self.pending_open)
            .field("publish_channel", &"<tauri::ipc::Channel<MirrorState>>")
            .field("surface_resolver", &self.surface_resolver)
            .field("headings", &self.headings)
            .field("bookmark_store", &"<BookmarkStore>")
            .field("current_fingerprint", &self.current_fingerprint)
            .field("first_yield_ms", &self.first_yield_ms)
            .field("diag", &self.diag)
            .field("diag_measures", &self.diag_measures)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl AppState {
    /// Fresh state for the latch-persistence tests: a new
    /// session, no backend, the default (all-granted host
    /// baseline) permissions, and fresh focus/offer
    /// machines with the backgrounded latch.
    pub(crate) fn test_state() -> Self {
        Self {
            session: Mutex::new(PlaybackSession::new()),
            backend: Mutex::new(None),
            permissions: Mutex::new(permissions::PermissionSnapshot::default()),
            focus_tracker: Mutex::new(player::focus::FocusTracker::new()),
            offer: Mutex::new(session::offer::OfferMachine::new()),
            pending_open: Mutex::new(None),
            publish_channel: Mutex::new(None),
            surface_resolver: Mutex::new(None),
            headings: Mutex::new(Vec::new()),
            bookmark_store: Mutex::new(None),
            current_fingerprint: Mutex::new(None),
            first_yield_ms: Mutex::new(None),
            diag: diag_record::Recorder::new(),
            diag_measures: Mutex::new(DiagMeasures::default()),
        }
    }

    /// Test state over an already-opened bookmark store
    /// (T011): the persistence tests drive the durable
    /// writes into a real redb file and read the rows
    /// back through the store's own `get`, never a
    /// mock. Everything else matches [`Self::test_state`].
    pub(crate) fn test_state_with_bookmark_store(store: BookmarkStore) -> Self {
        Self {
            bookmark_store: Mutex::new(Some(store)),
            ..Self::test_state()
        }
    }
}

/// The durable heartbeat period, in seconds (T011):
/// the 20 s cadence the crash-safe recovery bound
/// names — a hard kill replays at most ~30 s of
/// position (FR-004), so a 20 s durable write keeps
/// the worst-case loss inside that bound.
const HEARTBEAT_PERIOD_SECS: u64 = 20;

/// The playback-watch period, in milliseconds (F1/F3):
/// the 500 ms audit cadence that feeds the session clock
/// from the backend's audible clock, republishes the
/// mirror when the consumed-unit count crosses, and
/// commands the document-end teardown when the stream
/// drains — fine enough that the end of a document is
/// detected within a bound no run can notice, coarse
/// enough to stay far off the audio path.
const PLAYBACK_WATCH_PERIOD_MS: u64 = 500;
/// The current unix time in milliseconds — the epoch
/// the FR-015 supersede clock stamps (T011). A clock
/// set before the unix epoch (an impossible
/// `duration_since`) saturates at `0` and a
/// millisecond count beyond `u64` at `u64::MAX`:
/// never a panic, never a wrap.
#[must_use]
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// The loaded document's blake3-256 content
/// fingerprint (FR-036), or `None` when no document
/// is loaded or the lock is poisoned — the key every
/// bookmark write persists the position under.
fn current_fingerprint(state: &AppState) -> Option<[u8; 32]> {
    state.current_fingerprint.lock().map_or_else(
        |_| {
            tracing::warn!("the fingerprint lock is poisoned; no bookmark can be keyed");
            None
        },
        |guard| *guard,
    )
}

/// The FR-015 supersede clock in unix milliseconds,
/// or `0` — the "no interval recorded" value — when
/// no interval is open or the lock is poisoned.
fn current_first_yield_ms(state: &AppState) -> u64 {
    state
        .first_yield_ms
        .lock()
        .map_or(0, |clock| clock.unwrap_or(0))
}

/// Record the loaded document's blake3-256 content
/// fingerprint in the shared state (FR-036): the one
/// call site every document-opening path routes
/// through — `open_document` and `play` both call it
/// with the document bytes — so the key the 20 s
/// heartbeat and the event-durable pause puts
/// persist under (T011 / T035) is set by the same
/// code everywhere (Principle II).
///
/// A poisoned fingerprint lock is recorded and
/// skipped: durability is best-effort, never a
/// panic, and a missed write only means the next
/// heartbeat finds no document.
pub(crate) fn set_document_fingerprint(state: &AppState, bytes: &[u8]) {
    let fp = fingerprint::fingerprint(bytes);
    if let Ok(mut guard) = state.current_fingerprint.lock() {
        *guard = Some(fp);
    } else {
        tracing::warn!("document fingerprint not recorded: the state lock is poisoned");
    }
}

/// Note the transport the session landed in for the
/// FR-015 supersede clock (T011): the clock opens
/// when playback starts, never resets while playback
/// continues — a deliberate pause and its resume are
/// one continuous interval — and closes at the
/// terminal ends (`Stopped` / `Completed`), which
/// end the interval it anchors.
fn note_transport(state: &AppState, transport: TransportState) {
    if let Ok(mut clock) = state.first_yield_ms.lock() {
        match transport {
            TransportState::Playing => {
                if clock.is_none() {
                    *clock = Some(now_ms());
                }
            }
            TransportState::Paused | TransportState::Interrupted => {}
            TransportState::Stopped | TransportState::Completed => *clock = None,
        }
    } else {
        tracing::warn!("the FR-015 clock was not updated: the state lock is poisoned");
    }
}

/// The session-end kind a landed pause records
/// (FR-044 / contracts/storage-schema.md): the offer
/// flow is resolved SOLELY from the stored
/// `end_kind`, so the mapping lives here exactly
/// once — a deliberate pause (and the residency
/// teardowns that pause first, `resident_stop` /
/// `swipe_away`) keeps the `paused` kind that opens
/// the FR-032c continue offer, while an unexpected
/// silence (focus loss, route removal) keeps the
/// `interrupted` kind that leaves the FR-032d
/// explicit-resume requirement. Any other transport
/// names no stored end and records nothing.
const fn end_kind_of_pause(transport: TransportState) -> Option<SessionEnd> {
    match transport {
        TransportState::Paused => Some(SessionEnd::Paused),
        TransportState::Interrupted => Some(SessionEnd::Interrupted),
        TransportState::Playing | TransportState::Stopped | TransportState::Completed => None,
    }
}

/// The end kind an event-durable write records for a landed
/// transport (T035): the one mapping every event-durable call
/// site that is NOT already a pause landing shares, so the
/// section-change (skip) and pre-suspend (app-background)
/// writes answer identically.
///
/// A session still `Playing` records `interrupted` — the
/// heartbeat's crash-point kind, because a skip or a background
/// write happens while audio runs and the process can die at
/// any instant after it: recovery from that row must take the
/// FR-032d explicit resume, never an auto-start. A pause-class
/// transport records its own kind through
/// [`end_kind_of_pause`], and a terminal transport records
/// nothing (the row is deleted at those ends instead).
const fn event_end_kind(transport: TransportState) -> Option<SessionEnd> {
    match transport {
        TransportState::Playing => Some(SessionEnd::Interrupted),
        TransportState::Paused | TransportState::Interrupted => end_kind_of_pause(transport),
        TransportState::Stopped | TransportState::Completed => None,
    }
}

/// Persist one bookmark row (T011 skeleton; the T035
/// event-durable call sites): `position_secs` under
/// the current document's content fingerprint, with
/// the FR-015 clock read from the shared state and
/// `end_kind` — one atomic single-row redb write
/// transaction (`BookmarkStore::put`, FR-004), the
/// same shape the heartbeat and every event-durable
/// call site share.
///
/// Best-effort by construction: no loaded document
/// (no fingerprint), an unopened store, or a
/// poisoned lock skips the write with a tracing
/// record — durability is never a failure of the
/// control that triggered it, and a storage failure
/// is a warn, never a panic (storage-schema.md).
///
/// Lock discipline: the fingerprint and clock guards
/// are each released before the store is locked
/// last, so no two of them are ever held together —
/// the heartbeat's read order and the IPC paths'
/// `session`-then-`bookmark_store` order keep the
/// crate free of lock cycles.
fn persist_bookmark(state: &AppState, position_secs: f64, end_kind: SessionEnd) {
    let Some(fingerprint) = current_fingerprint(state) else {
        return; // no document loaded — nothing to key the bookmark under
    };
    let first_yield_ms = current_first_yield_ms(state);
    let Ok(mut store_guard) = state.bookmark_store.lock() else {
        tracing::warn!("bookmark not persisted: the store lock is poisoned");
        return;
    };
    let Some(store) = store_guard.as_mut() else {
        return; // the store never opened — durability degrades to no bookmark
    };
    let position = StoredPosition {
        offset_secs: position_secs,
        first_yield_ms,
        end_kind,
    };
    if let Err(error) = store.put(&fingerprint, position) {
        tracing::warn!(%error, "bookmark not persisted: the store write failed");
    } else {
        tracing::debug!(
            offset_secs = position_secs,
            end_kind = ?end_kind,
            "bookmark persisted"
        );
    }
    drop(store_guard);
}

/// Delete the stored bookmark row under the current document's
/// content fingerprint (T035, FR-032 / FR-032b / FR-032f): the
/// discard the two terminal ends perform, so the next open has
/// no stored row to offer and starts at `0.0`
/// (storage-schema.md: "Delete on: Stop press, document end.
/// Nothing else deletes."). The pause-class ends never reach
/// here — a focus loss, `BecomingNoisy`, a residency teardown
/// (`resident_stop` / `swipe_away`) and the pre-suspend
/// background write all PRESERVE the entry (FR-009a / FR-028e).
///
/// Best-effort by construction, exactly like
/// [`persist_bookmark`]: no loaded document, an unopened store,
/// or a poisoned lock logs and skips — a delete is never a
/// failure of the control that triggered it. Lock discipline is
/// the same too: the fingerprint guard is released before the
/// store is locked last, so no two state locks are ever held
/// together.
fn delete_bookmark(state: &AppState) {
    let Some(fingerprint) = current_fingerprint(state) else {
        return; // no document loaded — no row to delete
    };
    let Ok(mut store_guard) = state.bookmark_store.lock() else {
        tracing::warn!("bookmark not deleted: the store lock is poisoned");
        return;
    };
    let Some(store) = store_guard.as_mut() else {
        return; // the store never opened — there is no row to delete
    };
    if let Err(error) = store.delete(&fingerprint) {
        tracing::warn!(%error, "bookmark not deleted: the store delete failed");
    } else {
        tracing::debug!("bookmark deleted");
    }
    drop(store_guard);
}

/// The end kind the OUTGOING document's durable save records
/// (FR-018 / FR-044).
///
/// A session the reader deliberately paused keeps the `paused`
/// kind — the FR-032c continue offer, the deliberate form. A
/// session already `Interrupted` (a focus loss, a removed
/// device) keeps the `interrupted` kind — the FR-032d explicit
/// resume, the unexpected form. A session still PLAYING when
/// the reader picks another document is a deliberate reader
/// action (the switch is not a failure of the audio), so it
/// records the deliberate `paused` kind too.
///
/// A terminal session owes NO put: its end already DELETED the
/// row (FR-032f), and a terminal end is never re-recorded by a
/// later switch.
const fn outgoing_end_kind(transport: TransportState) -> Option<SessionEnd> {
    match transport {
        TransportState::Playing | TransportState::Paused => Some(SessionEnd::Paused),
        TransportState::Interrupted => Some(SessionEnd::Interrupted),
        TransportState::Stopped | TransportState::Completed => None,
    }
}

/// The FR-018 durable save of the OUTGOING document: the
/// last-spoken position under the OUTGOING document's OWN
/// content fingerprint, with the end kind the outgoing session's
/// own state names.
///
/// The write happens BEFORE the incoming document's fingerprint
/// is recorded, so the row lands under the outgoing document's
/// key — the first bookmark is never deleted and never
/// overwritten by the incoming one. The first bookmark is NEVER
/// deleted by the switch: only the two terminal ends delete
/// (FR-032 / FR-032b / FR-032f), and a document switch is
/// neither.
///
/// The write reuses [`persist_bookmark`], which keys the row
/// under the CURRENT fingerprint — the outgoing document's, at
/// this point in the switch. Best-effort by construction: a
/// missing document, an unopened store, or a poisoned lock skips
/// the write with a tracing record, never a panic.
fn save_outgoing_bookmark(state: &AppState) {
    let snapshot = {
        let Ok(guard) = state.session.lock() else {
            tracing::warn!("outgoing bookmark skipped: the session lock is poisoned");
            return;
        };
        guard.snapshot()
    };
    let Some(end_kind) = outgoing_end_kind(snapshot.transport) else {
        return; // a terminal session owes no put — its end deleted the row
    };
    persist_bookmark(state, snapshot.position_secs, end_kind);
}

/// Tear the OUTGOING session down before the incoming document
/// takes over (FR-018: "stop current audio", then the new
/// snapshot).
///
/// Three things happen, in this order:
///
/// * the queued speech is dropped and the backend's position
///   clock resets — the incoming document's audio is the only
///   audio (the outgoing document stops, never overlaps);
/// * the session is replaced by a fresh one, so the incoming
///   document starts at `0.0` and never inherits the outgoing
///   document's position (the closed transition table's `Play`
///   landing RETAINS the position, so a bare session carry-over
///   would start the new document at the old one's place);
/// * the FR-015 supersede clock closes — the outgoing interval
///   ended with the outgoing session, and the incoming one opens
///   its own clock when it starts playing.
///
/// Each lock is taken alone and released before the next, so no
/// two state locks are ever held together.
fn teardown_outgoing_session(state: &AppState) {
    // The outgoing audio stops: the queue is dropped and the
    // backend's position clock resets (the full gain is restored
    // so a ducked session never resumes ducked).
    if let Ok(backend_guard) = state.backend.lock()
        && let Some(backend) = backend_guard.as_ref()
    {
        let _ = backend.set_gain(player::focus::FULL_GAIN);
        backend.stop();
    }
    // The session is replaced: the incoming document starts from
    // zero, in the initial (paused) transport, with the rate the
    // store's rung seeds (read-at-start, T042).
    if let Ok(mut session) = state.session.lock() {
        *session = PlaybackSession::new();
    } else {
        tracing::warn!("outgoing session not torn down: the session lock is poisoned");
    }
    // The FR-015 clock closes with the outgoing session.
    if let Ok(mut clock) = state.first_yield_ms.lock() {
        *clock = None;
    } else {
        tracing::warn!("the FR-015 clock was not closed: the state lock is poisoned");
    }
}

/// Hand the session over to a new document (FR-018): the ONE
/// place a document takes the session over.
///
/// The order is the contract's: the OUTGOING document's
/// last-spoken position is saved durably under its OWN
/// fingerprint FIRST, then the outgoing session is torn down
/// (the audio stops), and only THEN is the incoming document's
/// fingerprint recorded. A second document therefore never
/// inherits the first one's position, and the first one's
/// bookmark is never deleted nor overwritten.
fn switch_to_document(state: &AppState, bytes: &[u8]) {
    save_outgoing_bookmark(state);
    teardown_outgoing_session(state);
    set_document_fingerprint(state, bytes);
}

/// Persist the last playback rate (T042, write-on-change /
/// FR-019b): the store half every rate dispatch calls with
/// the rung the press moved to. The rung is written exactly
/// when a press changes it — a press that saturated at a
/// ladder end reports no change and writes nothing — and
/// the setting is document-independent (the `settings`
/// table), so it survives a document change and a store
/// reopen.
///
/// Best-effort by construction, exactly like
/// [`persist_bookmark`]: an unopened store (the setting
/// survives only as far as the store does) or a poisoned
/// lock skips the write with a tracing record — the rate
/// is a preference, never a failure of the control that set
/// it, and a storage failure is a warn, never a panic
/// (storage-schema.md). Lock discipline matches too: the
/// store is locked last and no other state lock is ever
/// held with it.
fn persist_last_rate(state: &AppState, rate: PlaybackRate) {
    let Ok(mut store_guard) = state.bookmark_store.lock() else {
        tracing::warn!("last rate not persisted: the store lock is poisoned");
        return;
    };
    let Some(store) = store_guard.as_mut() else {
        return; // the store never opened — the setting degrades to unpersisted
    };
    if let Err(error) = store.put_last_rate(rate) {
        tracing::warn!(%error, "last rate not persisted: the store write failed");
    } else {
        tracing::debug!("last rate persisted");
    }
    drop(store_guard);
}

/// The direction and the number of one-rung rate presses
/// that carry `current` to `target` — the minimum move the
/// rate seed drives — or `None` when the two rungs are
/// equal, or when either names no member of the closed
/// seven-rung ladder order in [`player::rate::LADDER`].
///
/// The ladder is the one ordering over
/// `engine_core::closed::PlaybackRate` (FR-019 / FR-044),
/// so this is an index distance, never a second ladder.
#[must_use]
fn rate_seed_moves(current: PlaybackRate, target: PlaybackRate) -> Option<(RateStep, usize)> {
    let current_index = player::rate::LADDER
        .iter()
        .position(|rung| *rung == current)?;
    let target_index = player::rate::LADDER
        .iter()
        .position(|rung| *rung == target)?;
    let current_index = isize::try_from(current_index).ok()?;
    let target_index = isize::try_from(target_index).ok()?;
    let delta = target_index.checked_sub(current_index)?;
    if delta == 0 {
        return None; // already at the stored rung — nothing to drive
    }
    let step = if delta > 0 {
        RateStep::Up
    } else {
        RateStep::Down
    };
    Some((step, delta.unsigned_abs()))
}

/// The rate-step direction a live-control press names
/// (T042), or `None` for a press that moves no rung — the
/// exhaustive match over the closed verb set with no
/// catch-all arm, so a new verb is a compile error here
/// (FR-044 / Principle VII).
#[must_use]
const fn rate_press_of(verb: ShellVerb) -> Option<RateStep> {
    match verb {
        ShellVerb::RateUp => Some(RateStep::Up),
        ShellVerb::RateDown => Some(RateStep::Down),
        ShellVerb::Play
        | ShellVerb::Pause
        | ShellVerb::Stop
        | ShellVerb::Complete
        | ShellVerb::Interrupt
        | ShellVerb::CycleVoiceNext
        | ShellVerb::CycleVoicePrev
        | ShellVerb::SkipForward
        | ShellVerb::SkipBackward
        | ShellVerb::ResumeAtStored
        | ShellVerb::StartOver
        | ShellVerb::ChooseEngine
        | ShellVerb::ExternalStart => None,
    }
}

/// Seed the session's playback rate from the stored
/// `last_rate` at session open (T042, read-at-start /
/// FR-019b) — the shell half of the persistence wiring the
/// rate stepper and the store define.
///
/// The player crate exposes no seeding constructor — a
/// session opens at the normal-speed rung and only the
/// `ShellVerb` rate moves change it — so the shell composes
/// the halves itself: it reads the stored rung (resolving
/// the first-use default once through
/// [`player::rate::rate_at_start`]), compares it with the
/// session's live rung read from the session's own
/// snapshot, and drives the MINIMUM number of one-rung rate
/// verbs ([`rate_seed_moves`]) that carry the live rung to
/// the stored one. The move is value-blind throughout: no
/// rung value is ever rendered on the way (FR-012).
///
/// The seed writes nothing: the store already carries the
/// rung being read, so read-at-start is a read (the
/// write-on-change half is [`persist_last_rate`]).
///
/// Best-effort by construction: a poisoned store or session
/// lock, an unopened store, an unreadable stored rung, or a
/// press the closed transition table rejects (a torn-down
/// session) is recorded and skipped — the stored rung is a
/// preference, never a precondition for reading.
fn seed_session_rate(state: &AppState) {
    // The stored rung: `None` on first use, or when the store
    // never opened (the setting then degrades to unpersisted
    // and the default below resolves).
    let stored = {
        let Ok(mut store_guard) = state.bookmark_store.lock() else {
            tracing::warn!("rate seed skipped: the store lock is poisoned");
            return;
        };
        match store_guard.as_mut() {
            Some(store) => match store.get_last_rate() {
                Ok(stored) => stored,
                Err(error) => {
                    tracing::warn!(
                        %error,
                        "rate seed skipped: the stored last_rate could not be read"
                    );
                    return;
                }
            },
            None => None,
        }
    };
    let target = player::rate::rate_at_start(stored);
    // The minimum move from the live rung to the stored one,
    // read under the session lock and released before the
    // presses run (one lock at a time, never two together).
    let Some((step, moves)) = ({
        let Ok(session) = state.session.lock() else {
            tracing::warn!("rate seed skipped: the session lock is poisoned");
            return;
        };
        rate_seed_moves(session.snapshot().rate, target)
    }) else {
        return; // already at the stored rung — nothing to drive
    };
    let Ok(mut session) = state.session.lock() else {
        tracing::warn!("rate seed stopped: the session lock is poisoned");
        return;
    };
    for _ in 0..moves {
        let verb = match step {
            RateStep::Up => ShellVerb::RateUp,
            RateStep::Down => ShellVerb::RateDown,
        };
        if let Err(error) = session.execute(verb) {
            // A torn-down session rejects the rate verb: the
            // seed stops where it is — the rung is a
            // preference, never a precondition for reading.
            tracing::warn!(%error, "rate seed stopped mid-ladder: the press was rejected");
            break;
        }
    }
}

/// The named failures of the cold-start open and play paths
/// (T019) — a closed set with no catch-all variant: every
/// rejection names its cause, which is the failure half of
/// the text-I/O protocol (the diagnostic on stderr, nothing
/// on stdout, never a playing state — FR-005).
#[derive(Debug)]
pub(crate) enum OpenError {
    /// The snapshot bytes are empty: there is nothing to
    /// synthesize.
    EmptyDocument,
    /// The snapshot bytes are not valid UTF-8 text: the
    /// synthesis input is text, so a non-text document is
    /// refused before any split, fingerprint, or synthesis.
    InvalidUtf8,
    /// The parsed document yields no bounded segment:
    /// nothing is spoken, so nothing starts.
    NoSegments,
    /// A bounded segment could not be synthesized by the
    /// bound offline engine.
    Synthesis {
        /// The engine's named failure, rendered for the
        /// diagnostic.
        reason: String,
    },
    /// The document-open lookup could not read the session
    /// store: the resume decision is never guessed from an
    /// unreadable store — a fail-closed refusal.
    Store {
        /// Why the stored row could not be read.
        reason: String,
    },
    /// The audio backend could not be opened (or started) at
    /// the declared handoff shape — a refusal, never a
    /// silent start, so no playing state is reported
    /// (FR-005).
    Backend {
        /// Why the backend refused the handoff.
        reason: String,
    },
    /// The session could not take the press: the closed
    /// transition table rejected it (a torn-down session
    /// takes no play), or the session lock is poisoned.
    NoStart {
        /// Why the start did not happen.
        reason: String,
    },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => f.write_str("empty document: nothing to synthesize")?,
            Self::InvalidUtf8 => f.write_str("document is not valid UTF-8 text")?,
            Self::NoSegments => f.write_str("empty document: no segments")?,
            Self::Synthesis { reason } => write!(f, "synthesis failed: {reason}")?,
            Self::Store { reason } => write!(f, "session store read failed: {reason}")?,
            Self::Backend { reason } => write!(f, "audio backend unavailable: {reason}")?,
            Self::NoStart { reason } => write!(f, "the session could not start: {reason}")?,
        }
        Ok(())
    }
}

impl std::error::Error for OpenError {}

/// The post-execution bookkeeping every verb path
/// runs once the session has landed (T011 clock,
/// T035 event-durable puts): note the landed
/// transport for the FR-015 clock, record the
/// transport change in the diagnostic record (T097),
/// and — when the landing is a pause — persist the
/// position with the end kind that landing names.
/// Every surface that moves the session through
/// `execute` (the bridge verbs, the intent paths, the
/// cold-start open) routes through here, so no call
/// site can forget the durable write.
fn after_execute(state: &AppState, snapshot: &SessionSnapshot, verb: ShellVerb) {
    note_transport(state, snapshot.transport);
    // The diagnostic record (T097): the session's own start, the
    // transport change, and the session's end — in that order.
    //
    // The session start comes FIRST: the record's own scanner
    // segments the trace by session, and a session start opens the
    // segment the entries that follow belong to. A transport change
    // recorded before the start would sit in a segment of its own,
    // with no later entry to show the session advancing.
    if snapshot.transport == TransportState::Playing {
        record_session_start(state, snapshot);
    }
    record_transport_change(state, snapshot, verb);
    if snapshot.transport != TransportState::Playing
        && let Some(end_kind) = diag_end_reason(snapshot.transport)
    {
        record_session_end(state, snapshot, end_kind);
    }
    if let Some(end_kind) = end_kind_of_pause(snapshot.transport) {
        persist_bookmark(state, snapshot.position_secs, end_kind);
    }
}

/// The post-execution bookkeeping a section change (a skip)
/// runs (T035): note the landed transport for the FR-015 clock,
/// then refresh the durable row, because a skip moves the
/// position without a pause and the next heartbeat tick may be
/// up to 20 s away (FR-004 / FR-007).
///
/// The end kind comes from [`event_end_kind`]: a skip that holds
/// `Playing` records the `interrupted` crash-point kind (the
/// write a kill would otherwise have to reconstruct), a
/// forward skip that reaches the duration and lands `Paused`
/// (FR-018) records the deliberate-pause kind that opens the
/// continue offer, and a skip from a silent or terminal session
/// records its own kind or nothing. The per-sentence snap-back
/// is NOT written here: it is the in-RAM
/// [`player::position::PositionClock`] alone (FR-007).
///
/// Lock discipline matches [`after_execute`]: the snapshot is
/// read under the session lock, that guard is released, and the
/// store is locked last inside [`persist_bookmark`] — never two
/// state locks at once.
fn after_skip(state: &AppState, snapshot: &SessionSnapshot, verb: ShellVerb) {
    note_transport(state, snapshot.transport);
    // The diagnostic record (T097): the live control's transport
    // change — the control holds the transport, so no session
    // start or end is recorded.
    record_transport_change(state, snapshot, verb);
    if let Some(end_kind) = event_end_kind(snapshot.transport) {
        persist_bookmark(state, snapshot.position_secs, end_kind);
    }
}

/// The pre-suspend durable write the app-background intent runs
/// (T035): a backgrounded process can be suspended or killed at
/// any instant, so the position is persisted BEFORE the suspend
/// rather than waiting for the 20 s heartbeat that may never
/// tick again (FR-004 / FR-007).
///
/// The end kind is the one the current transport maps to
/// through [`event_end_kind`]: a `Playing` session records
/// `interrupted` — the same crash-point semantics and the same
/// rationale as the heartbeat's `Playing` write, since a
/// backgrounding is unexpected and recovery from that row must
/// take the FR-032d explicit resume — a `Paused` or already
/// `Interrupted` session re-records its own kind, and a terminal
/// session records nothing.
///
/// The FR-032c foreground/background latch is deliberately NOT
/// consulted here: it is read at offer expiry only, and this
/// helper writes durability and nothing else.
///
/// Lock discipline: the snapshot guard is released before
/// [`persist_bookmark`] takes the store last.
fn after_background(state: &AppState) {
    let snapshot = {
        let Ok(guard) = state.session.lock() else {
            tracing::warn!("app-background bookmark skipped: the session lock is poisoned");
            return;
        };
        guard.snapshot()
    };
    if let Some(end_kind) = event_end_kind(snapshot.transport) {
        persist_bookmark(state, snapshot.position_secs, end_kind);
    }
}

// =============================================================================
// The surface title (FR-011 / FR-029 / T103).
// =============================================================================

/// The document name the open path uses when the caller knows no
/// name (the frontend command carries the snapshot BYTES only): the
/// stable placeholder the resolver also carries as its fallback
/// (FR-029).
const DEFAULT_DOCUMENT_FILENAME: &str = "document.md";

/// The surface title's source (FR-011): the first Markdown H1
/// heading's text for a `.md` document, the filename without its
/// extension for a `.txt` document, and `None` — the resolver's own
/// filename placeholder — when a Markdown document carries no H1 at
/// all.
///
/// The title is a DISPLAY projection: the heading's own text (the
/// document's proper noun, exempt from pt-BR) or the file's name.
/// NEVER a sentence of the body, never the literal "Document"
/// (FR-011 / FR-029).
fn document_title(
    segments: &[adapter_synthesis::chunk::DocumentSegment],
    filename: &str,
) -> Option<String> {
    if is_markdown_document(filename) {
        return first_h1_text(segments);
    }
    Some(filename_without_extension(filename))
}

/// Whether the document is a Markdown one, by the name's extension
/// (FR-011): `.md` is the Markdown format of the closed set
/// (contracts/ingest.md).
fn is_markdown_document(filename: &str) -> bool {
    filename
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("md"))
}

/// The first Markdown H1 heading's text (FR-011): the heading's own
/// line, without its `#` marker. A heading of ANY OTHER LEVEL never
/// wins — an H2 before the H1 does not (FR-011: "first Markdown H1
/// heading").
///
/// `None` when the document carries no H1 at all.
fn first_h1_text(segments: &[adapter_synthesis::chunk::DocumentSegment]) -> Option<String> {
    segments
        .iter()
        .filter(|segment| segment.boundary_kind == adapter_synthesis::chunk::BoundaryKind::Heading)
        .filter_map(|segment| segment.content.lines().next())
        .find_map(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed.chars().take_while(|mark| *mark == '#').count();
            if hashes != 1 {
                return None; // not an H1 — another level never wins
            }
            let text = trimmed.trim_start_matches('#').trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        })
}

/// The filename without its extension (FR-011): the `.txt` title
/// source, and the name's stem for a name with no extension at all.
fn filename_without_extension(filename: &str) -> String {
    match filename.rsplit_once('.') {
        Some((stem, extension)) if !extension.is_empty() && !stem.is_empty() => stem.to_string(),
        _ => filename.to_string(),
    }
}

// =============================================================================
// The diagnostic record (FR-017 / T097).
//
// The private, on-device JSONL trace: the session lifecycle's
// events, appended to the app's own storage. The record is never
// transmitted and has no in-app surface — a maintainer pulls it off
// the device (FR-017 / SC-008). Every entry carries timestamps,
// the session's authoritative position, and typed identifiers only:
// the document's content hash, a closed vocabulary word, a ladder
// rung, or a measured value — NEVER document text (FR-017).
// =============================================================================

/// The diagnostic-record detail keys (T097): the closed set of
/// typed `key=value` pairs the record's `detail` field carries.
/// The record's own non-prose rule
/// (`^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+$`) holds for every one of
/// them, so a value is an identifier or a measurement — never a
/// sentence, never a fragment of the document.
const DIAG_DETAIL_ACTION: &str = "action";
/// The section index the session crossed.
const DIAG_DETAIL_SECTION: &str = "section";
/// The document's content-hash reference (FR-036) — the only
/// document identity the record ever carries.
const DIAG_DETAIL_DOC: &str = "doc";
/// The ladder rung before the change.
const DIAG_DETAIL_RATE_FROM: &str = "rate_from";
/// The ladder rung after the change.
const DIAG_DETAIL_RATE_TO: &str = "rate_to";
/// The interruption's outcome class.
const DIAG_DETAIL_OUTCOME: &str = "outcome";
/// The cause of a refusal or a failure, as a closed identifier.
const DIAG_DETAIL_CAUSE: &str = "cause";
/// The pre-roll seconds still buffered (FR-015a).
const DIAG_DETAIL_PREROLL_SECS: &str = "preroll_secs";
/// The measured play-to-audio latency, in milliseconds.
const DIAG_DETAIL_PLAY_TO_AUDIO_MS: &str = "play_to_audio_ms";
/// The measured device-removal pause latency, in milliseconds.
const DIAG_DETAIL_REMOVAL_PAUSE_MS: &str = "removal_pause_ms";
/// The measured replay seconds after a kill or a power loss.
const DIAG_DETAIL_REPLAY_SECS: &str = "replay_secs";

/// The measured-latency origins and results the diagnostic record
/// carries (T097): the three clocks the contract's measured values
/// come from (contracts/diagnostic-trace.md).
///
/// The origins are stamped when the corresponding event is
/// REQUESTED; the results are stamped when the awaited event
/// arrives. A result is consumed by the next entry that can carry
/// it, so a measurement is reported exactly once.
#[derive(Debug, Default)]
pub(crate) struct DiagMeasures {
    /// The wall-clock millisecond stamp of the play request whose
    /// first audible audio is awaited — the play-to-audio clock's
    /// origin.
    pub(crate) play_request_ms: Option<u64>,
    /// The wall-clock millisecond stamp of the output-device
    /// removal whose pause is awaited — the removal-pause clock's
    /// origin.
    pub(crate) removal_request_ms: Option<u64>,
    /// The offset the current session resumed from — the replay
    /// window's origin: the seconds re-listened after a kill or a
    /// power loss.
    pub(crate) resume_origin_secs: Option<f64>,
    /// The measured play-to-audio latency, in milliseconds.
    pub(crate) play_to_audio_ms: Option<u64>,
}

/// The record event envelope (T097): the record version the schema
/// declares, the wall-clock stamp, the kind, and the session's
/// authoritative position. The optional fields are filled by the
/// caller that knows them — the envelope never guesses.
fn diag_event(kind: EventKind, position_secs: f64) -> RecordEvent {
    RecordEvent {
        v: 1,
        ts_ms: now_ms(),
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

/// The record's `detail` field from typed pairs (T097): a
/// non-prose `key=value` string. A pair that cannot be expressed
/// drops the whole detail — never a panic, never a prose value.
fn diag_detail_field(pairs: &[(&str, &str)]) -> Option<String> {
    match diag::writer::detail_from_pairs(pairs) {
        Ok(detail) => detail,
        Err(error) => {
            tracing::warn!(%error, "the diagnostic record detail was dropped");
            None
        }
    }
}

/// The content-hash reference the record carries for the loaded
/// document (FR-036 / FR-017): the blake3 fingerprint's hex, or
/// `none` when no document is loaded. Never document text.
fn diag_doc_ref(state: &AppState) -> String {
    current_fingerprint(state).map_or_else(|| String::from("none"), hex::encode)
}

/// The closed focus set the record carries (FR-017): the player's
/// three outcomes mapped onto the record's focus set. The mapping
/// is exhaustive over the closed three-member outcome set — no
/// catch-all arm (Principle VII / FR-044).
const fn diag_focus_value(outcome: FocusOutcome) -> FocusValue {
    match outcome {
        FocusOutcome::DuckableTransient => FocusValue::DuckableTransient,
        FocusOutcome::SilenceDemanding => FocusValue::SilenceDemanding,
        FocusOutcome::PermanentLoss => FocusValue::PermanentLoss,
    }
}

/// The closed outcome class name the record carries (FR-017): the
/// outcome's own wire name — an identifier, never prose.
const fn diag_focus_name(outcome: FocusOutcome) -> &'static str {
    match outcome {
        FocusOutcome::DuckableTransient => "duckable_transient",
        FocusOutcome::SilenceDemanding => "silence_demanding",
        FocusOutcome::PermanentLoss => "permanent_loss",
    }
}

/// The closed session-end reason a landed transport records
/// (FR-044): the record's `session_end_reason` names the same four
/// members the storage schema's `end_kind` uses, so the record and
/// the stored row never disagree. A `Playing` landing ends no
/// episode — it starts one.
const fn diag_end_reason(transport: TransportState) -> Option<SessionEnd> {
    match transport {
        TransportState::Paused => Some(SessionEnd::Paused),
        TransportState::Interrupted => Some(SessionEnd::Interrupted),
        TransportState::Stopped => Some(SessionEnd::Stopped),
        TransportState::Completed => Some(SessionEnd::Completed),
        TransportState::Playing => None,
    }
}

/// The session-start entry (T097): the session began playing, with
/// the document's content hash as its only document reference.
fn record_session_start(state: &AppState, snapshot: &SessionSnapshot) {
    let mut event = diag_event(EventKind::SessionStart, snapshot.position_secs);
    event.transport = Some(TransportValue::from(snapshot.transport));
    let doc = diag_doc_ref(state);
    let replay_secs = take_replay_secs(state);
    let mut pairs: Vec<(&str, &str)> = vec![
        (DIAG_DETAIL_ACTION, "session_start"),
        (DIAG_DETAIL_DOC, &doc),
    ];
    if let Some(replay_secs) = replay_secs.as_deref() {
        pairs.push((DIAG_DETAIL_REPLAY_SECS, replay_secs));
    }
    event.detail = diag_detail_field(&pairs);
    state.diag.append(&event);
}

/// The session-end entry (T097): the episode ended, with the closed
/// end reason and the landed transport.
fn record_session_end(state: &AppState, snapshot: &SessionSnapshot, end_kind: SessionEnd) {
    let mut event = diag_event(EventKind::SessionEnd, snapshot.position_secs);
    event.transport = Some(TransportValue::from(snapshot.transport));
    event.session_end_reason = Some(SessionEndValue::from(end_kind));
    let doc = diag_doc_ref(state);
    let play_to_audio_ms = take_play_to_audio_ms_pending(state);
    let mut pairs: Vec<(&str, &str)> =
        vec![(DIAG_DETAIL_ACTION, "session_end"), (DIAG_DETAIL_DOC, &doc)];
    if let Some(play_to_audio_ms) = play_to_audio_ms.as_deref() {
        pairs.push((DIAG_DETAIL_PLAY_TO_AUDIO_MS, play_to_audio_ms));
    }
    event.detail = diag_detail_field(&pairs);
    state.diag.append(&event);
}

/// The section index the session is in (T097): the 1-based index
/// of the most-recent-passed heading — the surface resolver's own
/// answer, never a second count (FR-028d).
fn diag_section_index(state: &AppState, snapshot: &SessionSnapshot) -> u32 {
    let Ok(resolver_guard) = state.surface_resolver.lock() else {
        return 0;
    };
    let Some(resolver) = resolver_guard.as_ref() else {
        return 0; // no document loaded
    };
    let Ok(headings_guard) = state.headings.lock() else {
        return 0;
    };
    let surface = resolver.resolve(
        snapshot.position_secs,
        resolver.consumed_units_at(snapshot.position_secs),
        &headings_guard,
    );
    match surface.progress {
        player::surface::Progress::Determinate { k, .. } => k,
        player::surface::Progress::Indeterminate => 0,
    }
}

/// The transport-change entry (T097): one per transport action
/// (play, pause, a skip, a stop, a rate step) — the action's own
/// wire name and the landed transport.
fn record_transport_change(state: &AppState, snapshot: &SessionSnapshot, verb: ShellVerb) {
    let mut event = diag_event(EventKind::TransportChange, snapshot.position_secs);
    event.transport = Some(TransportValue::from(snapshot.transport));
    let action = verb.to_string();
    event.detail = diag_detail_field(&[(DIAG_DETAIL_ACTION, &action)]);
    state.diag.append(&event);
}

/// The per-section progress entry (T097): the session crossed a
/// section boundary, with the document's content hash as the only
/// document reference (FR-017).
fn record_section_progress(state: &AppState, snapshot: &SessionSnapshot, section: u32) {
    let mut event = diag_event(EventKind::TransportChange, snapshot.position_secs);
    let doc = diag_doc_ref(state);
    let section = section.to_string();
    event.detail = diag_detail_field(&[
        (DIAG_DETAIL_ACTION, "section"),
        (DIAG_DETAIL_SECTION, &section),
        (DIAG_DETAIL_DOC, &doc),
    ]);
    state.diag.append(&event);
}

/// The per-rate-change entry (T097): the ladder rung before and
/// after the change. The rung NAMES are identifiers — the record is
/// a private diagnostic artifact, never a surface (FR-012).
fn record_rate_change(
    state: &AppState,
    snapshot: &SessionSnapshot,
    from: PlaybackRate,
    to: PlaybackRate,
) {
    let mut event = diag_event(EventKind::TransportChange, snapshot.position_secs);
    let from = format!("{from:?}");
    let to = format!("{to:?}");
    event.detail = diag_detail_field(&[
        (DIAG_DETAIL_ACTION, "rate"),
        (DIAG_DETAIL_RATE_FROM, &from),
        (DIAG_DETAIL_RATE_TO, &to),
    ]);
    state.diag.append(&event);
}

/// The progress entry (T097): the session's authoritative position
/// at a recorded moment — the first audible audio of a play request
/// and the heartbeat's periodic reading.
///
/// The entry carries NO transport: it is a position reading, not a
/// transport change. That is what keeps the record's stall rule
/// (the contract's own scanner) honest: a playing landing must be
/// followed by a later entry at a strictly greater position, and
/// the progress entries are those later entries.
fn record_progress(state: &AppState, snapshot: &SessionSnapshot, measured: Option<&str>) {
    let mut event = diag_event(EventKind::TransportChange, snapshot.position_secs);
    let mut pairs: Vec<(&str, &str)> = vec![(DIAG_DETAIL_ACTION, "progress")];
    if let Some(measured) = measured {
        pairs.push((DIAG_DETAIL_PLAY_TO_AUDIO_MS, measured));
    }
    event.detail = diag_detail_field(&pairs);
    state.diag.append(&event);
}

/// The buffer-state entry (T097): the pre-roll seconds still
/// buffered at the moment of the entry (FR-015a; the declared
/// pre-roll cap is 60 s).
fn record_buffer_state(state: &AppState, snapshot: &SessionSnapshot) {
    let buffered_secs = {
        let Ok(guard) = state.backend.lock() else {
            return;
        };
        let Some(backend) = guard.as_ref() else {
            return; // no backend — no buffer to report
        };
        let samples = u32::try_from(backend.buffered_samples()).unwrap_or(u32::MAX);
        f64::from(samples) / f64::from(backend.sample_rate())
    };
    let mut event = diag_event(EventKind::TransportChange, snapshot.position_secs);
    let buffered_secs = format!("{buffered_secs:.3}");
    event.detail = diag_detail_field(&[
        (DIAG_DETAIL_ACTION, "buffer"),
        (DIAG_DETAIL_PREROLL_SECS, &buffered_secs),
    ]);
    state.diag.append(&event);
}

/// The focus-transition entry (T097): the interruption's class and
/// its outcome, with the measured values the contract names
/// (contracts/diagnostic-trace.md). The measured values are
/// consumed here, so each measurement is reported exactly once.
fn record_focus_transition(
    state: &AppState,
    snapshot: &SessionSnapshot,
    focus: FocusValue,
    action: &str,
    outcome: &str,
) {
    let mut event = diag_event(EventKind::FocusTransition, snapshot.position_secs);
    event.focus = Some(focus);
    event.transport = Some(TransportValue::from(snapshot.transport));
    let play_to_audio_ms = take_play_to_audio_ms_pending(state);
    let removal_pause_ms = take_removal_pause_ms(state);
    let replay_secs = take_replay_secs(state);
    let mut pairs: Vec<(&str, &str)> =
        vec![(DIAG_DETAIL_ACTION, action), (DIAG_DETAIL_OUTCOME, outcome)];
    if let Some(play_to_audio_ms) = play_to_audio_ms.as_deref() {
        pairs.push((DIAG_DETAIL_PLAY_TO_AUDIO_MS, play_to_audio_ms));
    }
    if let Some(removal_pause_ms) = removal_pause_ms.as_deref() {
        pairs.push((DIAG_DETAIL_REMOVAL_PAUSE_MS, removal_pause_ms));
    }
    if let Some(replay_secs) = replay_secs.as_deref() {
        pairs.push((DIAG_DETAIL_REPLAY_SECS, replay_secs));
    }
    event.detail = diag_detail_field(&pairs);
    state.diag.append(&event);
}

/// The output-device entry (T097): the route change the record
/// names (FR-017a), with the measured removal-pause latency when
/// the event was a removal.
fn record_output_device(
    state: &AppState,
    snapshot: &SessionSnapshot,
    device: DeviceEvent,
    action: &str,
) {
    let mut event = diag_event(EventKind::OutputDevice, snapshot.position_secs);
    event.device = Some(device);
    event.transport = Some(TransportValue::from(snapshot.transport));
    let removal_pause_ms = take_removal_pause_ms(state);
    let mut pairs: Vec<(&str, &str)> = vec![(DIAG_DETAIL_ACTION, action)];
    if let Some(removal_pause_ms) = removal_pause_ms.as_deref() {
        pairs.push((DIAG_DETAIL_REMOVAL_PAUSE_MS, removal_pause_ms));
    }
    event.detail = diag_detail_field(&pairs);
    state.diag.append(&event);
}

/// The record's entry for the open path's error (T097): a request
/// refused before anything played is a `Refusal`; a failure of the
/// engine or the device while the session ran is a `Failure`. The
/// `error` names the closed engine-taxonomy member the error maps
/// onto (research.md R5 — the eight product reasons map N:1 onto
/// the seven engine errors), and the `detail` names the error's own
/// cause.
fn record_refusal(state: &AppState, error: &OpenError) {
    match error {
        OpenError::Synthesis { .. } | OpenError::Backend { .. } | OpenError::NoStart { .. } => {
            record_failure(state, diag_error_of(error), diag_error_cause(error));
        }
        OpenError::EmptyDocument
        | OpenError::InvalidUtf8
        | OpenError::NoSegments
        | OpenError::Store { .. } => {
            record_named_refusal(state, diag_error_of(error), diag_error_cause(error));
        }
    }
}

/// The record's entry for a refusal outside the open path (T097):
/// the permission gate's refusal — the start was refused
/// (research.md R5: `FocusDenied` → `StartRefused`).
fn record_named_refusal(state: &AppState, error: ErrorValue, cause: &str) {
    let mut event = diag_event(EventKind::Refusal, 0.0);
    event.error = Some(error);
    event.detail = diag_detail_field(&[(DIAG_DETAIL_CAUSE, cause)]);
    state.diag.append(&event);
}

/// The record's entry for a failure (T097): the engine or the
/// device failed while the session ran. The `error` names the
/// closed engine-taxonomy member; the `detail` names the cause.
fn record_failure(state: &AppState, error: ErrorValue, cause: &str) {
    let mut event = diag_event(EventKind::Failure, 0.0);
    event.error = Some(error);
    event.detail = diag_detail_field(&[(DIAG_DETAIL_CAUSE, cause)]);
    state.diag.append(&event);
}

/// The closed engine-taxonomy member an open-path error maps onto
/// (FR-017 / research.md R5): the product reason each error belongs
/// to, mapped N:1 onto the seven engine errors.
///
/// * an unreadable document is the `FormatUndeliverable` reason —
///   the delivered shape cannot be read (`ChannelMismatch`);
/// * an unreadable store is the `VoiceUnavailable` reason — the
///   session could not be brought up from the stored row
///   (`EngineInitFailed`);
/// * a refused backend is the `DeviceRefused` reason — the audio
///   device refused the playback request;
/// * an undeliverable voice is the `VoiceUnavailable` reason —
///   the engine could not produce the audio (`EngineUnreachable`).
const fn diag_error_of(error: &OpenError) -> ErrorValue {
    match error {
        OpenError::EmptyDocument | OpenError::InvalidUtf8 | OpenError::NoSegments => {
            ErrorValue::ChannelMismatch
        }
        OpenError::Store { .. } => ErrorValue::EngineInitFailed,
        OpenError::Backend { .. } => ErrorValue::DeviceRefused,
        OpenError::Synthesis { .. } => ErrorValue::EngineUnreachable,
        OpenError::NoStart { .. } => ErrorValue::StartRefused,
    }
}

/// The error's own cause as a closed identifier (T097): the
/// error's variant, never its free-text reason — the reason may
/// quote the device or the engine, and the record carries names
/// only.
const fn diag_error_cause(error: &OpenError) -> &'static str {
    match error {
        OpenError::EmptyDocument => "empty_document",
        OpenError::InvalidUtf8 => "invalid_utf8",
        OpenError::NoSegments => "no_segments",
        OpenError::Synthesis { .. } => "synthesis",
        OpenError::Store { .. } => "store",
        OpenError::Backend { .. } => "backend",
        OpenError::NoStart { .. } => "no_start",
    }
}

/// Stamp the play-to-audio clock's origin (T097): the wall-clock
/// instant of a play request. The first audible audio of that
/// request closes the clock.
fn note_play_request(state: &AppState) {
    if let Ok(mut measures) = state.diag_measures.lock() {
        measures.play_request_ms = Some(now_ms());
    }
}

/// Stamp the device-removal clock's origin (T097): the wall-clock
/// instant of an output-route removal. The pause that follows
/// closes the clock.
fn note_removal_request(state: &AppState) {
    if let Ok(mut measures) = state.diag_measures.lock() {
        measures.removal_request_ms = Some(now_ms());
    }
}

/// Open the replay window (T097): the offset a recovered session
/// resumed from, after a kill or a power loss. The window's
/// measured value is the seconds re-listened since.
fn note_resume_origin(state: &AppState, offset_secs: f64) {
    if let Ok(mut measures) = state.diag_measures.lock() {
        measures.resume_origin_secs = Some(offset_secs);
    }
}

/// Close the play-to-audio clock (T097): the measured milliseconds
/// between the play request and the first audible audio of that
/// request. The measurement is kept until an entry carries it.
fn take_play_to_audio_ms(state: &AppState) -> Option<String> {
    let mut measures = state.diag_measures.lock().ok()?;
    let request_ms = measures.play_request_ms.take()?;
    let latency_ms = now_ms().saturating_sub(request_ms);
    measures.play_to_audio_ms = Some(latency_ms);
    drop(measures);
    Some(latency_ms.to_string())
}

/// Close the play-to-audio clock at the first audible audio (T113 /
/// FR-017): the backend's stream start on the FIRST synthesized
/// chunk IS the first audible audio of the request, so the measured
/// latency is the FR-003 bound — the time to the first speech — and
/// never the time the whole document's synthesis took.
///
/// The measurement is left pending, exactly as the clock's own
/// closer leaves it: the next entry that can carry it reports it.
fn note_first_audible(state: &AppState) {
    let _measured = take_play_to_audio_ms(state);
}

/// The measured play-to-audio latency, consumed by the next entry
/// that can carry it (T097).
fn take_play_to_audio_ms_pending(state: &AppState) -> Option<String> {
    let mut measures = state.diag_measures.lock().ok()?;
    let latency_ms = measures.play_to_audio_ms.take()?;
    drop(measures);
    Some(latency_ms.to_string())
}

/// Close the device-removal clock (T077): the measured milliseconds
/// between the route removal and the pause it caused. The
/// measurement is consumed — reported once.
fn take_removal_pause_ms(state: &AppState) -> Option<String> {
    let mut measures = state.diag_measures.lock().ok()?;
    let request_ms = measures.removal_request_ms.take()?;
    let latency_ms = now_ms().saturating_sub(request_ms);
    drop(measures);
    Some(latency_ms.to_string())
}

/// The measured replay seconds (T097): the seconds re-listened
/// since the recovered session resumed. The measurement is
/// consumed — reported once.
fn take_replay_secs(state: &AppState) -> Option<String> {
    // The session's own position is read FIRST and its lock is
    // released before the measurement state is locked: the record's
    // measurement state is a state lock of its own, and no two
    // state locks are ever held together (the crate's lock
    // discipline).
    let current_secs = {
        let Ok(guard) = state.session.lock() else {
            return None;
        };
        guard.snapshot().position_secs
    };
    let mut measures = state.diag_measures.lock().ok()?;
    let origin_secs = measures.resume_origin_secs.take()?;
    let replayed = (current_secs - origin_secs).max(0.0);
    drop(measures);
    Some(format!("{replayed:.3}"))
}

/// The surface a resolved resume offer routes to (T037): the
/// in-app visual dialog a deliberate end renders, the
/// lock-screen resume control an unexpected end renders, and
/// the nothing-to-render answer a terminal end (or no stored
/// row) gives. This type is the whole of `lib.rs`'s
/// contribution — which surface opens is the offer machine's
/// answer, never a second decision made here (FR-044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OfferRoute {
    /// The deliberate end (FR-032c): the in-app 3-second
    /// visual dialog — continue from the stored position (the
    /// default) against the start-over alternative. The FR-009
    /// silence rule lives in that dialog: with no choice inside
    /// the window, reading continues from the stored position.
    InAppDialog,
    /// The unexpected end (FR-032d / FR-031): the session waits
    /// paused and the explicit lock-screen resume control is the
    /// only way audio resumes — never an auto-start.
    LockScreenResume,
    /// Nothing is open: no surface to route to.
    NoRoute,
}

/// The in-app offer window, in milliseconds (FR-032c): the
/// visible 3-second continue dialog a deliberate end opens.
const OFFER_WINDOW_MS: u64 = 3000;

/// The offer-window poll period, in milliseconds (T110): the
/// cadence the app checks whether the FR-032c window of a
/// parked cold-start open elapsed. The window's own length is
/// [`OFFER_WINDOW_MS`]; this cadence only bounds how late the
/// silence rule lands, never the window itself.
const OFFER_WINDOW_POLL_MS: u64 = 250;

/// The cold-start open parked behind an open resume offer
/// (T110): the document whose offer is open, waiting for the
/// offer to resolve before ANY audio begins (FR-009 / FR-032c).
///
/// The parked open holds the document and the stored position
/// the offer continues from — data, never a decision: the offer
/// machine owns the decision of whether and when the reading
/// starts (FR-044).
#[derive(Debug)]
pub(crate) struct PendingOpen {
    /// The document text — the sole synthesis input (FR-018).
    document: String,
    /// The picked document's name (T103) — a display input only:
    /// the surface title's source, never a synthesis input.
    filename: String,
    /// The stored position the offer continues from, in seconds.
    offset_secs: f64,
    /// When the offer opened (the FR-032c window's start): the
    /// app's own clock — the offer machine carries none.
    opened_at: std::time::Instant,
}

/// The parked cold-start open's resolution (T110): the ways a
/// parked open ends.
///
/// The three members are the whole outcome space of a parked
/// open; each maps to the offer machine's own answer, never to a
/// decision made in the shell (FR-044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParkedResolution {
    /// The FR-032c window elapsed with no choice: the silence
    /// rule continues from the stored position (FR-009).
    WindowElapsed,
    /// The reader resumed — the in-app dialog's continue choice
    /// or the lock-screen resume control: the reading continues
    /// from the stored position (the FR-032c continue default,
    /// and the only start an unexpected end permits — FR-032d /
    /// FR-031).
    ReaderResumed,
    /// The reader chose "start over": the reading begins at the
    /// first spoken position and the stored entry is discarded
    /// (FR-032c).
    StartOverChosen,
}

/// Route a stored session end to the surface that renders its
/// resume offer (T037) — thin route-to-surface, with no offer
/// logic of its own.
///
/// The form is READ from the held
/// [`session::offer::OfferMachine`], never re-derived: the
/// machine's own decision fn (`resolve_open`) is driven over the
/// REAL stored row, so the open offer carries the stored position
/// the FR-009 silence rule continues from. The machine resolves
/// the form SOLELY from the stored end kind (storage-schema.md),
/// so this function only maps the machine's answer to a surface.
///
/// * The machine opening the continue offer routes to
///   [`OfferRoute::InAppDialog`].
/// * A standing explicit-resume requirement (FR-032d) routes to
///   [`OfferRoute::LockScreenResume`].
/// * Nothing open — a terminal end (FR-032 / FR-032e) — routes
///   to [`OfferRoute::NoRoute`].
///
/// Driving the held machine is exactly the open/reopen
/// lifecycle point — the offer is resolved there and read at
/// expiry afterwards — so the route is computed when a document
/// with a stored row opens, never mid-window. A poisoned offer
/// lock records and routes nothing: a route is a rendering
/// hint, never a playback decision.
#[must_use]
pub(crate) fn route_offer(state: &AppState, stored: &StoredPosition) -> OfferRoute {
    let Ok(mut machine) = state.offer.lock() else {
        tracing::warn!("offer route unresolved: the offer lock is poisoned");
        return OfferRoute::NoRoute;
    };
    // The machine's own decision over the REAL stored row: the
    // deliberate pause opens the continue offer (carrying the
    // stored position the silence rule continues from), the
    // unexpected end leaves the explicit-resume requirement
    // standing, and the terminal ends open nothing.
    match machine.resolve_open(Some(*stored)) {
        session::offer::OpenOffer::Continue { .. } => OfferRoute::InAppDialog,
        session::offer::OpenOffer::ExplicitResume => OfferRoute::LockScreenResume,
        session::offer::OpenOffer::NoOffer => OfferRoute::NoRoute,
    }
}

/// The resume-offer route in wire form (T037): the route's
/// wire name plus the FR-032c 3-second window an in-app
/// dialog opens — the ONE mapping from [`OfferRoute`] to the
/// wire vocabulary every surface reads (the `offer_route`
/// command and the cold-start open payload), so no second
/// definition of the route names can drift (Principle II /
/// FR-044). A route that opens no dialog carries no window.
#[must_use]
pub(crate) const fn offer_route_wire(route: OfferRoute) -> (&'static str, Option<u64>) {
    match route {
        OfferRoute::InAppDialog => ("in_app_dialog", Some(OFFER_WINDOW_MS)),
        OfferRoute::LockScreenResume => ("lockscreen_resume", None),
        OfferRoute::NoRoute => ("none", None),
    }
}

/// Park the cold-start open behind its open offer (T110): the
/// document waits for the offer to resolve, and NO audio begins
/// until it does (FR-009 / FR-032c).
///
/// The parked open holds DATA — the document, its display name,
/// and the stored position the offer continues from — never a
/// decision: the offer machine owns the decision of whether and
/// where the reading starts (FR-044).
fn park_cold_start(state: &AppState, document: &str, filename: &str, offset_secs: f64) {
    let pending = PendingOpen {
        document: document.to_string(),
        filename: filename.to_string(),
        offset_secs,
        opened_at: std::time::Instant::now(),
    };
    let Ok(mut guard) = state.pending_open.lock() else {
        tracing::warn!("the cold start was not parked: the pending-open lock is poisoned");
        return;
    };
    *guard = Some(pending);
}

/// Take the parked cold-start open (T110): the waiting document,
/// if one is parked. The parked open is consumed by its
/// resolution, so a second resolution finds nothing.
fn take_parked_open(state: &AppState) -> Option<PendingOpen> {
    let mut guard = state.pending_open.lock().ok()?;
    guard.take()
}

/// Whether a cold-start open is parked (T110): the reader's
/// choice resolves the parked open instead of the session.
fn pending_open_exists(state: &AppState) -> bool {
    state.pending_open.lock().is_ok_and(|guard| guard.is_some())
}

/// Park the cold-start open again (T110): the resolution left it
/// waiting — the FR-032d explicit-resume requirement stands, so
/// the reader's explicit resume is the only start (FR-032d /
/// FR-031).
fn repark_cold_start(state: &AppState, pending: PendingOpen) {
    let Ok(mut guard) = state.pending_open.lock() else {
        tracing::warn!("the cold start was not re-parked: the pending-open lock is poisoned");
        return;
    };
    *guard = Some(pending);
}

/// The FR-032c window elapsed (T110): the parked cold-start open
/// resolves through the offer machine's own answer — the silence
/// rule continues from the stored position (FR-009), and a
/// backgrounded window leaves the explicit-resume requirement
/// standing (FR-032d / FR-031).
///
/// The window's length is the app's own clock: the parked open
/// records when its offer opened, and the window elapses
/// [`OFFER_WINDOW_MS`] later. The check is a no-op before that
/// instant and after the open has resolved.
pub(crate) fn expire_parked_open(state: &AppState) {
    // The elapsed time is read alone, under the pending-open lock
    // only: the poll never copies the parked document, and it
    // never holds two state locks together.
    let elapsed = {
        let Ok(guard) = state.pending_open.lock() else {
            return;
        };
        let Some(pending) = guard.as_ref() else {
            return; // nothing is parked — nothing waits
        };
        pending.opened_at.elapsed()
    };
    if elapsed < Duration::from_millis(OFFER_WINDOW_MS) {
        return; // the window is still open
    }
    if !resolve_parked_open(state, ParkedResolution::WindowElapsed) {
        tracing::debug!("the parked cold start did not start the reading");
    }
}

/// Resolve the parked cold-start open (T110): the offer machine's
/// own answer decides whether the reading starts, and where.
///
/// * [`ExpiryOutcome::AutoStart`] — the reading starts at the
///   offset the machine names: the FR-009 silence rule continues
///   from the stored position, and the start-over choice begins
///   at the first spoken position (FR-032c).
/// * [`ExpiryOutcome::ExplicitResume`] — nothing starts on its
///   own (FR-032d / FR-031). The reader's explicit resume is the
///   only start, and it starts at the stored position the parked
///   open holds — the row the offer was opened from.
/// * [`ExpiryOutcome::NoOffer`] — nothing is open: the parked
///   open is dropped.
///
/// The start-over choice discards the stored entry (FR-032c):
/// the stored position is no longer a resume point once the
/// reader asked to start over.
///
/// Returns whether the reading started. A start that the
/// synthesis path refuses records its own named refusal (FR-017)
/// and starts nothing.
pub(crate) fn resolve_parked_open(state: &AppState, resolution: ParkedResolution) -> bool {
    let Some(pending) = take_parked_open(state) else {
        return false; // nothing is parked — nothing to resolve
    };
    let outcome = {
        let Ok(mut machine) = state.offer.lock() else {
            tracing::warn!("the parked cold start did not resolve: the offer lock is poisoned");
            return false;
        };
        match resolution {
            // The reader chose "start over": the machine's own
            // answer is the first spoken position.
            ParkedResolution::StartOverChosen => machine.start_over(),
            // The window elapsed, or the reader resumed: the
            // machine's own answer is the continue default.
            ParkedResolution::WindowElapsed | ParkedResolution::ReaderResumed => machine.expire(),
        }
    };
    match outcome {
        session::offer::ExpiryOutcome::AutoStart { offset_secs } => {
            if resolution == ParkedResolution::StartOverChosen {
                // The stored position is discarded: the reader
                // asked to start over, so the row is no longer a
                // resume point (FR-032c).
                delete_bookmark(state);
            }
            start_parked_reading(state, &pending, offset_secs)
        }
        session::offer::ExpiryOutcome::ExplicitResume => {
            if resolution == ParkedResolution::ReaderResumed {
                // The reader's own resume satisfies the
                // explicit-resume requirement: the reading
                // starts at the stored position the parked open
                // holds (FR-032d / FR-031).
                start_parked_reading(state, &pending, pending.offset_secs)
            } else {
                // The window elapsed on an unexpected end:
                // nothing starts on its own — the reader's
                // explicit resume is the only start, so the
                // parked open waits (FR-032d / FR-031).
                repark_cold_start(state, pending);
                false
            }
        }
        session::offer::ExpiryOutcome::NoOffer => {
            if resolution == ParkedResolution::ReaderResumed {
                // Nothing is open, but the reader's resume is an
                // explicit start: the reading begins at the
                // stored position the parked open holds.
                start_parked_reading(state, &pending, pending.offset_secs)
            } else {
                false
            }
        }
    }
}

/// Start the parked reading (T110): the parked document's
/// synthesis and playback, entered at `offset_secs` — the one
/// synthesis path, resumed at the position the offer resolved to.
///
/// Returns whether the reading started: a synthesis path that
/// refuses records its own named refusal (FR-017) and starts
/// nothing.
fn start_parked_reading(state: &AppState, pending: &PendingOpen, offset_secs: f64) -> bool {
    match commands::play_document(&pending.document, state, &pending.filename, offset_secs) {
        Ok(_payload) => true,
        Err(error) => {
            let _refused = commands::refuse(state, error);
            false
        }
    }
}

/// The reader's choice resolves the parked cold-start open
/// (T110): the reader's resume or start-over choice resolves the
/// open offer instead of touching a session that never started.
///
/// The choice is the reader's own: the in-app dialog's continue
/// choice, and the lock-screen resume control an unexpected end
/// waits for (FR-032d / FR-031).
///
/// # Errors
///
/// The session lock's poison error, when the snapshot of the
/// resolved reading cannot be read.
fn resolve_parked_choice(state: &AppState, verb: ShellVerb) -> Result<serde_json::Value, String> {
    let resolution = if verb == ShellVerb::StartOver {
        ParkedResolution::StartOverChosen
    } else {
        ParkedResolution::ReaderResumed
    };
    let started = resolve_parked_open(state, resolution);
    let snap = {
        let Ok(session) = state.session.lock() else {
            return Err("the session lock is poisoned".to_string());
        };
        session.snapshot()
    };
    commands::publish_current_state(state, None);
    Ok(serde_json::json!({
        "status": snap.transport.to_string(),
        "position_secs": snap.position_secs,
        "transport": snap.transport.to_string(),
        "offer_resolved": started,
    }))
}
/// The PCM that begins at the sample `skip_samples` names (T110 /
/// T113): the samples before the resolved position are never played,
/// so a cold start that continued from its resume offer enters the
/// audio where the offer continued from (FR-009).
///
/// The entry is a SAMPLE INDEX, not a second count: the engine's
/// declared rate is the only clock the PCM carries, and the sample
/// index is the exact position the resolved offset names — the
/// conversion lives in [`sample_index_of`], so the streaming path
/// (T113) trims every chunk against the SAME exact position the
/// whole-document path trimmed against.
///
/// A sample index past the end of the audio yields no samples: the
/// reading has nothing left to play, and the caller starts nothing.
fn pcm_from_sample(pcm: &[f32], skip_samples: usize) -> Vec<f32> {
    let start = skip_samples.min(pcm.len());
    if start == 0 {
        return pcm.to_vec();
    }
    pcm[start..].to_vec()
}

/// The sample index the resolved position names (T110): the
/// engine's declared rate is the only clock the PCM carries, and
/// a negative offset is floored at the start of the audio.
fn sample_index_of(offset_secs: f64) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the offset is floored at 0.0 before the cast, so the conversion is a non-negative truncation of a finite second count into a sample index; the saturation of the float-to-int cast keeps it in range"
    )]
    let index = (offset_secs.max(0.0) * commands::DECLARED_RATE_HZ) as usize;
    index
}

/// The 500 ms playback-watch task (F1/F3): feeds the session
/// clock from the backend's audible clock, republishes the
/// mirror at consumed-unit crossings, and commands the
/// document-end teardown when the stream drains.
///
/// The task carries its own baselines and holds the owned app
/// handle (never a borrowed `State`), so it outlives the setup
/// closure; each tick takes every state lock alone — no two are
/// ever held together, exactly like the heartbeat.
fn spawn_playback_watch_task(app: &tauri::AppHandle) {
    let watch_handle = app.clone();
    let _watch_task = tauri::async_runtime::spawn(async move {
        let mut ticker = time::interval(Duration::from_millis(PLAYBACK_WATCH_PERIOD_MS));
        let mut watch = PlaybackWatch::default();
        loop {
            let _tick = ticker.tick().await;
            let app_state = watch_handle.state::<AppState>();
            playback_watch_tick(&app_state, &mut watch);
        }
    });
}

/// The FR-032c offer window elapsed (T110): the parked
/// cold-start open resolves 3 s after its offer opens — the
/// silence rule continues from the stored position (FR-009).
///
/// The tick only paces the check; the parked open's own clock
/// decides when the window elapsed. The task holds the owned app
/// handle (never a borrowed `State`), so it outlives the setup
/// closure, and it takes every state lock alone — exactly like
/// the heartbeat and the playback watch.
fn spawn_offer_window_task(app: &tauri::AppHandle) {
    let offer_handle = app.clone();
    let _offer_task = tauri::async_runtime::spawn(async move {
        let mut ticker = time::interval(Duration::from_millis(OFFER_WINDOW_POLL_MS));
        loop {
            let _tick = ticker.tick().await;
            let app_state = offer_handle.state::<AppState>();
            expire_parked_open(&app_state);
        }
    });
}

/// One durable heartbeat tick (T011): while the
/// session is `Playing`, persist the current position
/// under the document's content fingerprint with the
/// `interrupted` end kind — the write a hard kill
/// replays from (FR-004: recovery to the last stored
/// write, at most ~30 s of lost position). The
/// heartbeat write is a potential crash point, so it
/// records the unexpected-end kind: recovery from it
/// requires the explicit resume (FR-032d). A tick
/// meeting any other transport, no loaded document,
/// or an unopened store records nothing — only a
/// playing session has a moving position worth
/// durably recording.
///
/// Ordering: the snapshot guard is released before
/// `persist_bookmark` takes the store last, so this
/// function never holds two of the state locks
/// together — the heartbeat sits off both sides of
/// every IPC path's lock order.
fn heartbeat_once(state: &AppState) {
    let snapshot = {
        let Ok(guard) = state.session.lock() else {
            tracing::warn!("heartbeat tick skipped: the session lock is poisoned");
            return;
        };
        guard.snapshot()
    };
    if snapshot.transport != TransportState::Playing {
        return;
    }
    persist_bookmark(state, snapshot.position_secs, SessionEnd::Interrupted);
    // The FR-017 diagnostic record (T097): the periodic progress
    // entry — the session's authoritative position at the
    // heartbeat's cadence. It is the entry that shows the session
    // advancing between the events.
    record_progress(state, &snapshot, None);
}

/// The playback-watch task's carried baselines (F1/F3): the last
/// backend position fed into the session clock and the last
/// consumed-unit count carried by a progress publish.
#[derive(Debug, Default)]
struct PlaybackWatch {
    /// The backend-clock position (seconds) last credited through
    /// [`PlaybackSession::on_audible_progress`]. Deltas are taken
    /// against this, so a missed tick never loses progress and a
    /// repeated reading never double-credits.
    fed_secs: f64,
    /// The consumed-unit count last carried by a progress
    /// publish; `None` until the first republish of a run (the
    /// first observation republishes, the play-path publish being
    /// asynchronous to this task).
    units: Option<u64>,
}

/// One playback-watch tick (F1/F3): the FR-002 bridge from the
/// backend's audible clock into the session clock, the F3
/// progress republish, and the F1 drain-to-teardown signal.
///
/// The backend is the only clock that advances on consumed
/// speech samples (FR-015); the session clock is the position
/// authority every ack, bookmark, and publish reads. This tick
/// is the single place the two meet: the delta since the last
/// tick is credited while `Playing`, so a paused session never
/// advances and a missed tick is caught up whole.
///
/// A drained backend (every enqueued sample consumed) commands
/// the shell's document-end teardown — [`signal_document_end`]
/// — and publishes nothing further: the completion republish
/// rides the teardown itself. Otherwise the consumed-unit count
/// is compared against the last one republished, and the mirror
/// is republished only when it crosses to a new value — the bar
/// flip at a section boundary (T051-06), never a 2 Hz stream of
/// identical notifications.
///
/// The task is lock-disciplined like the heartbeat: every state
/// lock is taken alone and released before the next, so the
/// watch sits off both sides of every IPC path's lock order.
fn playback_watch_tick(state: &AppState, watch: &mut PlaybackWatch) {
    let transport = {
        let Ok(guard) = state.session.lock() else {
            tracing::warn!("playback watch skipped: the session lock is poisoned");
            return;
        };
        guard.snapshot().transport
    };
    match transport {
        TransportState::Playing => {}
        TransportState::Stopped | TransportState::Completed => {
            // A terminal transport discards the baselines: the
            // next run (a later process — both ends are terminal
            // in-app) starts from nothing.
            *watch = PlaybackWatch::default();
            return;
        }
        // Paused and Interrupted retain their position: the
        // baselines stay valid for the resume that may follow.
        TransportState::Paused | TransportState::Interrupted => return,
    }

    let (drained, position_secs) = {
        let Ok(guard) = state.backend.lock() else {
            tracing::warn!("playback watch skipped: the backend lock is poisoned");
            return;
        };
        let Some(backend) = guard.as_ref() else {
            tracing::warn!("playback watch skipped: a playing session has no backend");
            return;
        };
        (backend.is_drained(), backend.position_secs())
    };

    feed_audible_progress(state, position_secs, watch);

    if drained {
        signal_document_end(state);
        return;
    }

    let _ = maybe_republish_progress(state, watch);
}

/// Credit the backend's audible progress into the session clock
/// (FR-002): the delta since the last credited reading, applied
/// only while the session is `Playing` — the held and terminal
/// transports keep the position frozen (FR-005).
///
/// The carry advances only when a read is actually credited, so
/// the delta a race loses (a pause landing between this task's
/// transport read and its session lock) is re-offered on the
/// next playing tick instead of being dropped.
fn feed_audible_progress(state: &AppState, position_secs: f64, watch: &mut PlaybackWatch) {
    // The credit delta is resolved by the position authority (the
    // player crate) — the bridge forwards the reading and applies the
    // returned delta, carrying no arithmetic of its own.
    let delta_secs = player::position::audible_progress_delta(position_secs, watch.fed_secs);
    if delta_secs <= 0.0 {
        return;
    }
    let credited = if let Ok(mut session) = state.session.lock() {
        if session.snapshot().transport == TransportState::Playing {
            session.on_audible_progress(delta_secs);
            watch.fed_secs = position_secs;
            true
        } else {
            false
        }
    } else {
        tracing::warn!("audible progress dropped: the session lock is poisoned");
        false
    };
    if !credited {
        return;
    }
    // The FR-017 diagnostic record (T097): the first audible audio
    // of a play request closes the play-to-audio clock, and records
    // the progress entry the record's stall rule needs — a playing
    // landing followed by a strictly greater position.
    //
    // The clock is taken OUTSIDE the session lock: the record's
    // measurement state is a state lock of its own, and no two
    // state locks are ever held together (the crate's lock
    // discipline) — the measurement is best-effort, so a lost race
    // costs one measurement, never a deadlock.
    //
    // The clock is closed by the FIRST AUDIBLE audio of the request
    // (T113): the streaming path starts the backend on the first
    // chunk, so the measured latency is already pending when the
    // first credit arrives. The pending value is the measurement this
    // entry carries — the fallback covers the case where the clock
    // was closed earlier (T113) and the closer itself has nothing
    // left to give.
    let measured = take_play_to_audio_ms(state).or_else(|| take_play_to_audio_ms_pending(state));
    let Ok(guard) = state.session.lock() else {
        return;
    };
    let snapshot = guard.snapshot();
    drop(guard);
    record_progress(state, &snapshot, measured.as_deref());
}

/// Republish the mirror when the consumed-unit count crosses to a
/// new value (F3): the payload the notification's determinate bar
/// reads. The count is computed from the session snapshot — the
/// same position the publish path itself resolves against, so the
/// carried baseline can never disagree with the published payload.
///
/// The first observation of a run always republishes: the
/// play-path publish races this task, and a duplicate mirror is
/// cheap while a swallowed section flip would not self-correct
/// until the next boundary.
fn maybe_republish_progress(state: &AppState, watch: &mut PlaybackWatch) -> bool {
    let units = {
        let position_secs = {
            let Ok(guard) = state.session.lock() else {
                tracing::warn!("progress republish skipped: the session lock is poisoned");
                return false;
            };
            guard.snapshot().position_secs
        };
        let Ok(resolver_guard) = state.surface_resolver.lock() else {
            tracing::warn!("progress republish skipped: the resolver lock is poisoned");
            return false;
        };
        let Some(resolver) = resolver_guard.as_ref() else {
            return false; // no document loaded
        };
        let units = resolver.consumed_units_at(position_secs);
        drop(resolver_guard);
        units
    };
    if watch.units == Some(units) {
        return false;
    }
    watch.units = Some(units);
    commands::publish_current_state(state, None);
    true
}

/// Command the FR-032f document-end teardown through the shell
/// (F1): the JNI hop into `ShellHost.commandDocumentEnd`, whose
/// forwarded `document_end` intent lands in
/// [`commands::dispatch_document_end`] — the one teardown the
/// bridge already serves, so the classified path stays the only
/// path.
///
/// With no shell to command (host builds, tests, an unarmed JVM)
/// the teardown runs in-process instead: every consumer must
/// complete a drained session identically, and a completion whose
/// Kotlin half is missing still stops the backend, tears the
/// offer machine down, deletes the bookmark, and publishes — the
/// resident surface alone is left to the user's dismissal.
fn signal_document_end(state: &AppState) {
    match shell_jni::command_document_end_to_kotlin() {
        Ok(()) => {}
        Err(reason) => {
            tracing::debug!(
                reason = %reason,
                "document-end command unavailable; completing in-process"
            );
            let _ = commands::dispatch_document_end(state);
        }
    }
}

/// Run the Tauri application.
///
/// Builds the application — managing the shared
/// [`AppState`], running the permission-state startup
/// gate, and registering the IPC handlers — and starts
/// the event loop. A loop failure is logged through
/// [`tracing`] and exits the process with a non-zero
/// status: the bridge cannot serve without the loop.
#[cfg_attr(target_os = "android", tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(PlaybackSession::new()),
            backend: Mutex::new(None),
            permissions: Mutex::new(permissions::PermissionSnapshot::default()),
            focus_tracker: Mutex::new(player::focus::FocusTracker::new()),
            offer: Mutex::new(session::offer::OfferMachine::new()),
            pending_open: Mutex::new(None),
            publish_channel: Mutex::new(None),
            surface_resolver: Mutex::new(None),
            headings: Mutex::new(Vec::new()),
            bookmark_store: Mutex::new(None),
            current_fingerprint: Mutex::new(None),
            first_yield_ms: Mutex::new(None),
            diag: diag_record::Recorder::new(),
            diag_measures: Mutex::new(DiagMeasures::default()),
        })
        .setup(|app| {
            // Voice assets ship inside the APK: extract them to a
            // real filesystem path before the first synthesis. Hosts
            // use the manifest tree; nothing to do there.
            #[cfg(target_os = "android")]
            {
                if let Ok(data) = app.path().app_data_dir() {
                    if let Ok(voice) = adapter_synthesis::android_assets::extract_voices(&data) {
                        adapter_synthesis::synth::set_asset_dir(voice);
                    }
                }
            }

            // Get the AppState for bookmark store initialization and permission gate
            let app_state = app.state::<AppState>();
            // Initialize the 20 s durable heartbeat (T011): open the
            // BookmarkStore in the app data directory and start the
            // periodic timer that calls `put` while Playing.
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            let bookmark_path = data_dir.join("bookmarks.redb");
            let store_for_app = match BookmarkStore::open(&bookmark_path) {
                Ok(store) => Some(store),
                Err(error) => {
                    // Best-effort durability: with no store the
                    // heartbeat skips its puts, never panics —
                    // FR-004 recovery degrades to no bookmark.
                    tracing::warn!(
                        %error,
                        path = %bookmark_path.display(),
                        "heartbeat: bookmark store could not be opened; persistence is skipped"
                    );
                    None
                }
            };
            if let Ok(mut guard) = app_state.bookmark_store.lock() {
                *guard = store_for_app;
            }
            // The diagnostic record (T097, FR-017): the private,
            // on-device JSONL trace, opened in the app's own data
            // directory — the same private storage the bookmark
            // store uses. Nothing transmits it and no in-app
            // surface reads it; a maintainer pulls it off the
            // device. A failed open degrades the diagnostics, never
            // the playback.
            let record_path = data_dir.join(diag_record::RECORD_FILE_NAME);
            if let Err(error) = app_state.diag.open(&record_path) {
                tracing::warn!(
                    %error,
                    path = %record_path.display(),
                    "the diagnostic record could not be opened; the trace is skipped"
                );
            }
            // Spawn the 20 s heartbeat task (T011): calls `BookmarkStore::put`
            // while Playing for crash-safe position recovery (FR-004 / FR-007).
            // Each tick reads the session snapshot under its own lock,
            // releases it, and only then takes the store inside
            // `persist_bookmark` — no two state locks are ever held
            // together here, so the heartbeat cannot form a cycle with
            // the IPC paths. The task holds the owned app handle (never a
            // borrowed `State`), so it outlives the setup closure.
            let app_handle = app.app_handle().clone();
            let _heartbeat_task = tauri::async_runtime::spawn(async move {
                let mut ticker = time::interval(Duration::from_secs(HEARTBEAT_PERIOD_SECS));
                loop {
                    // The tick instant goes unread: the tick
                    // only paces the writes, and a caught-up
                    // burst tick is harmless — the put is
                    // idempotent and last-write-wins.
                    let _tick = ticker.tick().await;
                    let app_state = app_handle.state::<AppState>();
                    heartbeat_once(&app_state);
                }
            });

            // The 500 ms playback-watch task (F1/F3): feeds the
            // session clock from the backend's audible clock,
            // republishes the mirror at consumed-unit crossings,
            // and commands the document-end teardown when the
            // stream drains.
            spawn_playback_watch_task(app.app_handle());

            // The FR-032c offer-window task (T110): the parked
            // cold-start open resolves 3 s after its offer opens
            // — the silence rule continues from the stored
            // position (FR-009).
            spawn_offer_window_task(app.app_handle());

            // Shell JNI channel (T111, FR-002/FR-003): install
            // the integer-code drain holding a clone of the
            // application handle, so every Kotlin-posted verb or
            // intent executes on AppState's OWN session through
            // the single `run_mobile_plugin` dispatch — never a
            // second session, never a duplicated code path. A
            // spawn failure is recorded inside `install`; the
            // sender stays installed, so later posts drop loudly
            // instead of vanishing.
            shell_jni::install(app.app_handle().clone());
            // Permission-state gate (T030b, FR-008/FR-010): the
            // startup check runs once at boot against the snapshot
            // the Kotlin shell pushes through `update_permissions`
            // (the all-granted host baseline until it does). The
            // outcome is recorded as a tracing event; a refusal
            // carries the stated reason, and no playing state may
            // ever follow it.
            let app_state = app.state::<AppState>();
            let permissions = app_state.permissions.lock().map_err(|e| e.to_string())?;
            match permissions::check_startup(&permissions) {
                Ok(()) => tracing::info!("permission gate: startup check passed"),
                Err(failure) => tracing::warn!(
                    reason = failure.reason(),
                    "permission gate: startup refused — stated reason, no playing state"
                ),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::register_android_plugin,
            commands::run_mobile_plugin,
            commands::update_permissions,
            commands::get_status_view,
            commands::session_delete,
            commands::offer_route,
            commands::open_document,
            publish::register_publish_listener
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            tracing::error!("tauri application terminated: {error}");
            std::process::exit(1);
        });
}

pub mod commands {
    //! Plugin commands exposed via IPC.

    use super::permissions::{self, PermissionSnapshot};
    use super::{AppState, CpalBackend, OpenError};
    use crate::{
        DEFAULT_DOCUMENT_FILENAME, OfferRoute, after_background, after_execute, after_skip,
        current_fingerprint, delete_bookmark, diag_focus_name, diag_focus_value,
        diag_section_index, document_title, note_first_audible, note_play_request,
        note_removal_request, note_resume_origin, offer_route_wire, park_cold_start,
        pcm_from_sample, pending_open_exists, persist_last_rate, rate_press_of,
        record_buffer_state, record_focus_transition, record_named_refusal, record_output_device,
        record_rate_change, record_refusal, record_section_progress, resolve_parked_choice,
        route_offer, sample_index_of, seed_session_rate, switch_to_document,
    };
    use diag::writer::{DeviceDirection, DeviceEvent, ErrorValue, FocusValue};

    use player::cli::{SessionSnapshot, ShellVerb};
    use player::focus::{FocusClassification, FocusIntent, FocusOutcome};
    use player::rate::RateStepper;
    use player::routing::{RoutingDecision, RoutingSignal, RoutingTracker};
    use session::fingerprint;
    use tauri::{AppHandle, Manager, State, command};

    /// Declared sample rate of the engine handoff (mono f32).
    pub(crate) const DECLARED_RATE_HZ: f64 = 22_050.0;

    /// Bind the native Android plugin surface.
    ///
    /// The Kotlin `PlaybackService` self-bootstraps from
    /// the Application/Activity and reaches Rust through
    /// `run_mobile_plugin` below — never computing
    /// position, rate, or transport itself (SC-035).
    ///
    /// Self-bootstrap (T111): the platform creates the
    /// service from the Application/Activity entry points
    /// declared in the manifest (a structural requirement —
    /// `ActivityManager` reflection forces the service class
    /// into Kotlin), so no construction call from Rust or
    /// the frontend precedes it. On creation the service
    /// runs `ShellHost.init`, which builds the four mirrors
    /// against the application context with sinks posting
    /// integer codes to the `shell_jni` drain installed in
    /// [`run`](crate::run). This body therefore performs no
    /// construction of its own: it acknowledges the
    /// registration and verifies the drain is live, so a
    /// registration racing ahead of setup is recorded
    /// loudly instead of vanishing.
    #[command]
    pub fn register_android_plugin() {
        if crate::shell_jni::channel_ready() {
            tracing::info!("shell channel: plugin registered; drain live on AppState session");
        } else {
            tracing::warn!(
                "shell channel: plugin registered before drain install; Kotlin posts will drop with diagnostics until setup runs"
            );
        }
    }

    /// Push the platform permission answer (T030b).
    ///
    /// The Kotlin shell reads the platform permissions and
    /// forwards the answer here — mirror-only, no decision:
    /// the gates in `permissions` (`check_startup`,
    /// `check_pre_play`) consume the snapshot (FR-008,
    /// FR-010, FR-021).
    ///
    /// # Errors
    ///
    /// Returns the lock's error string when the shared
    /// permission mutex is poisoned — a refusal, never a
    /// playing state.
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn update_permissions(
        state: State<'_, AppState>,
        snapshot: PermissionSnapshot,
    ) -> Result<(), String> {
        let mut permissions = state.permissions.lock().map_err(|e| e.to_string())?;
        *permissions = snapshot;
        drop(permissions);
        Ok(())
    }

    /// Open a document from its snapshot BYTES and start
    /// reading (T019 / US1) — the cold-start path.
    ///
    /// The bytes are the SOLE synthesis input: no URI, no
    /// path, no second document object ever enters this
    /// command. They are decoded and validated as UTF-8 text,
    /// blake3-256 fingerprinted as content (FR-036 — the key
    /// the durable writes persist under), resolved through
    /// the open lookup ([`session::store::lookup_on_open`],
    /// T036 — the same library fn the `session ingest` CLI
    /// verb runs), and handed to [`play`] for the ONE
    /// synthesis path (Principle II). The resume decision
    /// renders from the published offer route (T037).
    ///
    /// Totals (`total_n` + `total_units`) are fixed at
    /// session start, once, inside [`play`]: the heading count
    /// and the marked document total the surface resolver
    /// derives from the heading list written to the shared
    /// state at open are never re-estimated mid-session
    /// (FR-028a).
    ///
    /// ## Cold-start budget (SC-002; T016 measures the host
    /// half, T051 gates the device half)
    ///
    /// This command owns the ≤3 s press-to-first-audio bound
    /// end-to-end: from the picked bytes to the first audible
    /// PCM. Within the bound, the engine's voice model is
    /// loaded exactly once (the adapter caches it — never a
    /// per-segment load), the document is split into bounded
    /// chunks before any synthesis, and the PCM is enqueued
    /// and started the moment the document is synthesized.
    /// The bound's endpoint is the device transducer, so its
    /// gating measurement is the on-device trial; if that
    /// trial misses the bound, the fix lives HERE (ordering
    /// the first segment first so the first chunk starts while
    /// the rest synthesizes) — the bound is never renegotiated
    /// and never deferred.
    ///
    /// Pure offline: the path holds zero network call sites
    /// (the T047 guard enforces that app-wide) — fingerprint,
    /// lookup, synthesis, and playback are all local.
    ///
    /// # Errors
    ///
    /// The named [`OpenError`] rejections: empty bytes, non-
    /// UTF-8 bytes, no bounded segments, a synthesis failure,
    /// an unreadable session store, an unavailable audio
    /// backend, or a press the closed transition table
    /// rejects. Every rejection carries the stated reason and
    /// produces no payload — a refusal, never a playing state
    /// (FR-005).
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn open_document(
        bytes: Vec<u8>,
        state: State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        open_document_bytes(&state, bytes).map_err(|error| error.to_string())
    }

    /// The cold-start open core (T019): everything the
    /// [`open_document`] command does, over the shared state
    /// directly — the seam the tests drive (no `State` handle
    /// can be constructed outside a running Tauri app, and the
    /// path must stay exercisable on the host).
    ///
    /// The validation order is deliberate: every check that
    /// can run before the session is touched runs first, so a
    /// refused open leaves no document loaded — the failure
    /// half of the text-I/O protocol (the diagnostic on
    /// stderr, nothing on stdout).
    pub(crate) fn open_document_bytes(
        state: &AppState,
        bytes: Vec<u8>,
    ) -> Result<serde_json::Value, OpenError> {
        open_document_bytes_named(state, bytes, DEFAULT_DOCUMENT_FILENAME)
    }

    /// The cold-start open core with the picked document's NAME
    /// (T103 / FR-011): the name decides the surface title's source
    /// — the first Markdown H1 for a `.md` document, the name
    /// without its extension for a `.txt` document — and the
    /// fallback the resolver carries as its filename placeholder.
    ///
    /// The name is a DISPLAY input only: the bytes remain the sole
    /// synthesis input (FR-018), and the name never enters the
    /// fingerprint or the synthesis.
    pub(crate) fn open_document_bytes_named(
        state: &AppState,
        bytes: Vec<u8>,
        filename: &str,
    ) -> Result<serde_json::Value, OpenError> {
        if bytes.is_empty() {
            return Err(refuse(state, OpenError::EmptyDocument));
        }
        // The snapshot bytes are text: decode and validate
        // before anything is derived from them.
        let document =
            String::from_utf8(bytes).map_err(|_| refuse(state, OpenError::InvalidUtf8))?;

        // The content fingerprint (FR-036): the key the 20 s
        // heartbeat and every event-durable put persist under,
        // recorded through the one call site every
        // document-opening path shares (T011 / T035).
        let fingerprint_hex = hex::encode(fingerprint::fingerprint(document.as_bytes()));

        // The open lookup (T036): bytes → fingerprint → the
        // stored row → the offer form, through the library fn
        // the `session ingest` CLI verb runs — never
        // re-assembled here (Principle II / FR-044). The store
        // is the application's own: a second redb handle over
        // the same file cannot exist, and the heartbeat
        // already writes through this one.
        let lookup = {
            let mut store_guard = state.bookmark_store.lock().map_err(|error| {
                refuse(
                    state,
                    OpenError::Store {
                        reason: error.to_string(),
                    },
                )
            })?;
            let store = store_guard.as_mut().ok_or_else(|| {
                refuse(
                    state,
                    OpenError::Store {
                        reason: "the session store never opened".to_string(),
                    },
                )
            })?;
            let lookup =
                session::store::lookup_on_open(store, document.as_bytes()).map_err(|error| {
                    refuse(
                        state,
                        OpenError::Store {
                            reason: error.to_string(),
                        },
                    )
                })?;
            drop(store_guard);
            lookup
        };

        // The FR-017 diagnostic record (T097): a session that
        // resumes from a stored row opens the replay window — the
        // seconds re-listened since the recovery are measured
        // against this offset.
        if let Some(stored) = lookup.stored {
            note_resume_origin(state, stored.offset_secs);
        }

        // The FR-018 document switch: the OUTGOING document's
        // last-spoken position is saved durably under its OWN
        // fingerprint, the outgoing session is torn down (the
        // audio stops), and only THEN is the incoming document's
        // fingerprint recorded. The first bookmark is never
        // deleted by the switch, and the incoming document never
        // inherits the outgoing one's position.
        switch_to_document(state, document.as_bytes());

        // The offer route (T037): the stored end kind resolves
        // to the surface that renders the resume decision, and
        // driving the held machine at this open is the
        // lifecycle point that opens the FR-032c continue
        // offer the in-app dialog renders — never mid-window.
        let route = lookup
            .stored
            .map_or(OfferRoute::NoRoute, |stored| route_offer(state, &stored));

        // The cold-start gate (T110): the offer machine's own
        // decision (FR-044) — the app routes the decision and
        // never re-derives the offer's form.
        //
        // * No stored position: the reading starts now, at the
        //   first spoken position, with no offer (FR-036a).
        // * A deliberate pause: the 3-second continue offer is
        //   open (FR-032c) and NO audio begins until it
        //   resolves (FR-009) — the offer's silence continues
        //   from the stored position.
        // * An unexpected end: no offer opens and no audio
        //   begins on its own (FR-032d / FR-031) — the reader's
        //   explicit resume is the only start.
        let decision = session::offer::cold_start(lookup.offer);
        let start_offset_secs = match decision {
            session::offer::ColdStart::StartNow => 0.0,
            session::offer::ColdStart::AwaitOffer { offset_secs, .. } => offset_secs,
            session::offer::ColdStart::AwaitExplicitResume => {
                lookup.stored.map_or(0.0, |stored| stored.offset_secs)
            }
        };

        // The one synthesis path (Principle II): split,
        // synthesize, enqueue, start — with the totals fixed
        // at open and the rate seeded from the store.
        //
        // A gated cold start never reaches it: the reading waits
        // for its offer to resolve (FR-009 / FR-032c).
        let mut payload = if decision == session::offer::ColdStart::StartNow {
            play_document(&document, state, filename, 0.0).map_err(|error| refuse(state, error))?
        } else {
            // The offer is open: the document is parked, NO audio
            // is synthesized, enqueued, or started, and the
            // payload reports the waiting reading.
            park_cold_start(state, &document, filename, start_offset_secs);
            serde_json::json!({
                "status": "paused",
                "transport": "paused",
                "position_secs": 0.0,
            })
        };

        // The open facts the resume decision renders: the
        // content fingerprint, the stored row, and the offer
        // route the stored end kind resolved to.
        let (offer_route, offer_window_ms) = offer_route_wire(route);
        if let Some(object) = payload.as_object_mut() {
            let _ignored = object.insert(
                "fingerprint".to_string(),
                serde_json::Value::String(fingerprint_hex),
            );
            let _ignored = object.insert(
                "offer_route".to_string(),
                serde_json::Value::String(offer_route.to_string()),
            );
            if let Some(window_ms) = offer_window_ms {
                let _ignored =
                    object.insert("offer_window_ms".to_string(), serde_json::json!(window_ms));
            }
            let _ignored = object.insert(
                "has_stored_position".to_string(),
                serde_json::Value::Bool(lookup.stored.is_some()),
            );
            let stored_offset_secs = lookup.stored.map_or(0.0, |stored| stored.offset_secs);
            let _ignored = object.insert(
                "stored_offset_secs".to_string(),
                serde_json::json!(stored_offset_secs),
            );
        }
        Ok(payload)
    }

    /// The open path's refusal (T097): the diagnostic record names
    /// the refusal before the error leaves the path — the closed
    /// error taxonomy's member and the refusal's own cause, never
    /// the free-text reason (which may quote the device or the
    /// engine).
    pub(crate) fn refuse(state: &AppState, error: OpenError) -> OpenError {
        record_refusal(state, &error);
        error
    }

    /// Execute one of the three Play-class verbs that reach
    /// `Playing` in the closed transition table without the
    /// `Play` synthesis path: `ResumeAtStored`, `StartOver`,
    /// and `ExternalStart` (T110).
    ///
    /// * The same pre-play gate as `Play`: a refusal executes
    ///   nothing and reports no playing state (FR-010).
    /// * A parked cold-start open owns the reading (T110): the
    ///   reader's resume or start-over choice resolves the open
    ///   offer instead of touching a session that never started.
    /// * Otherwise the verb lands on the session through the
    ///   same [`player::cli::PlaybackSession::execute`] code
    ///   path every other driver calls (Principle II).
    fn resume_class_verb(
        verb: ShellVerb,
        state: &State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        let permissions = state.permissions.lock().map_err(|e| e.to_string())?;
        if let Err(failure) = permissions::check_pre_play(&permissions) {
            // The FR-017 diagnostic record (T097): the refusal
            // is named before the ack leaves the path — the start
            // was refused (R5).
            record_named_refusal(state, ErrorValue::StartRefused, "permission_refused");
            return Ok(serde_json::json!({
                "ok": false,
                "error": failure.reason(),
            }));
        }
        // T110: a parked cold-start open owns the reading — the
        // reader's resume or start-over choice resolves the open
        // offer instead of touching a session that never started.
        if matches!(verb, ShellVerb::ResumeAtStored | ShellVerb::StartOver)
            && pending_open_exists(state.inner())
        {
            return resolve_parked_choice(state.inner(), verb);
        }
        let snap = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.execute(verb).map_err(|e| e.to_string())?
        };
        publish_current_state(state, None);
        // Playing-class landing: open the FR-015 clock (T011) so
        // the next durable write stamps it.
        after_execute(state, &snap, verb);
        Ok(serde_json::json!({
            "status": snap.transport.to_string(),
            "position_secs": snap.position_secs,
            "transport": snap.transport.to_string(),
        }))
    }

    /// The classified bridge action (T110).
    ///
    /// Every closed shell-IPC wire string parses to exactly one
    /// variant — a session verb, a platform intent, or the
    /// read-only query.
    ///
    /// The wire vocabulary is `contracts/shell-ipc.md`. No
    /// `PartialEq` is derived: the contract test pins the
    /// seam with `matches!` only.
    #[derive(Debug, Clone, Copy)]
    pub enum BridgeAction {
        /// A closed-set session verb, executed through
        /// [`player::cli::PlaybackSession::execute`].
        Verb(
            /// The parsed session verb.
            ShellVerb,
        ),
        /// A classified Kotlin→Rust platform intent,
        /// routed by `dispatch_intent` to the nearest
        /// existing machine entry point.
        Intent(
            /// The parsed platform intent.
            IntentWire,
        ),
        /// The read-only voice-asset inventory query —
        /// never a verb, never a state change.
        Diagnose,
    }

    /// The closed Kotlin→Rust intent wire set (T110).
    ///
    /// The nine focus/routing/lifecycle intents of
    /// [`player::focus::FocusIntent`], the three
    /// residency intents (`contracts/shell-ipc.md`), and
    /// the two diagnostic unmapped sentinels (never
    /// absorbed, never errors, never state changes).
    #[derive(Debug, Clone, Copy)]
    pub enum IntentWire {
        /// Transient loss that may duck (FR-013).
        DuckableTransient,
        /// Transient loss demanding silence (FR-013a).
        SilenceDemanding,
        /// Permanent loss: pause, never auto-resume (FR-014).
        PermanentLoss,
        /// Focus returned: authorizes resume, never
        /// auto-starts (FR-015).
        FocusRegained,
        /// Headphone-removal/noisy signal: pause within
        /// the routing bound (FR-016).
        BecomingNoisy,
        /// Output device appeared: follow at unchanged
        /// position (FR-017a).
        DeviceAppeared,
        /// Output device disappeared: pause first,
        /// explicit resume only (FR-016 / FR-017a).
        DeviceDisappeared,
        /// Lifecycle latch only: no playback decision,
        /// read at offer expiry only (FR-032c / FR-023a).
        AppForegrounded,
        /// Lifecycle latch only: no playback decision
        /// (FR-032c).
        AppBackgrounded,
        /// Ends residency preserving the bookmark —
        /// distinct from the transport `Stop`, which
        /// discards it (FR-009a).
        ResidentStop,
        /// Pauses first when playing, preserves the
        /// bookmark, ends residency (FR-028e).
        SwipeAway,
        /// Tears residency down at the document's end:
        /// a completed document has no resumable
        /// position (FR-032f).
        DocumentEnd,
        /// Unrecognised control value: diagnostic ack,
        /// never absorbed, never an error, never a
        /// state change.
        ControlUnmapped,
        /// Unrecognised platform lifecycle value:
        /// diagnostic ack, never absorbed, never an
        /// error, never a state change.
        LifecycleUnmapped,
    }

    /// Classify one shell-IPC wire string (T110): the
    /// 12 closed verb wires to [`BridgeAction::Verb`],
    /// the 14 intent wires to [`BridgeAction::Intent`],
    /// and `diagnose` to [`BridgeAction::Diagnose`].
    ///
    /// The mapping is the normative `snake_case`
    /// case-mapping (`contracts/shell-ipc.md`): `regained`
    /// maps to focus-regained, and the voice/skip verbs
    /// use their full wire spellings (`skip_forward5s`,
    /// `cycle_voice_next`, …).
    ///
    /// # Errors
    ///
    /// Returns the named `unknown action` rejection for
    /// any wire outside the closed set (wrong case,
    /// kebab-case, empty, or invented) — a rejection,
    /// never a playing state.
    pub fn parse_action(action: &str) -> Result<BridgeAction, String> {
        match action {
            "play" => Ok(BridgeAction::Verb(ShellVerb::Play)),
            "pause" => Ok(BridgeAction::Verb(ShellVerb::Pause)),
            "stop" => Ok(BridgeAction::Verb(ShellVerb::Stop)),
            "skip_forward5s" => Ok(BridgeAction::Verb(ShellVerb::SkipForward)),
            "skip_back5s" => Ok(BridgeAction::Verb(ShellVerb::SkipBackward)),
            "rate_up" => Ok(BridgeAction::Verb(ShellVerb::RateUp)),
            "rate_down" => Ok(BridgeAction::Verb(ShellVerb::RateDown)),
            "cycle_voice_next" => Ok(BridgeAction::Verb(ShellVerb::CycleVoiceNext)),
            "cycle_voice_prev" => Ok(BridgeAction::Verb(ShellVerb::CycleVoicePrev)),
            "resume_at_stored" => Ok(BridgeAction::Verb(ShellVerb::ResumeAtStored)),
            "start_over" => Ok(BridgeAction::Verb(ShellVerb::StartOver)),
            "choose_engine" => Ok(BridgeAction::Verb(ShellVerb::ChooseEngine)),
            "duckable_transient" => Ok(BridgeAction::Intent(IntentWire::DuckableTransient)),
            "silence_demanding" => Ok(BridgeAction::Intent(IntentWire::SilenceDemanding)),
            "permanent_loss" => Ok(BridgeAction::Intent(IntentWire::PermanentLoss)),
            "regained" => Ok(BridgeAction::Intent(IntentWire::FocusRegained)),
            "becoming_noisy" => Ok(BridgeAction::Intent(IntentWire::BecomingNoisy)),
            "device_appeared" => Ok(BridgeAction::Intent(IntentWire::DeviceAppeared)),
            "device_disappeared" => Ok(BridgeAction::Intent(IntentWire::DeviceDisappeared)),
            "app_foregrounded" => Ok(BridgeAction::Intent(IntentWire::AppForegrounded)),
            "app_backgrounded" => Ok(BridgeAction::Intent(IntentWire::AppBackgrounded)),
            "resident_stop" => Ok(BridgeAction::Intent(IntentWire::ResidentStop)),
            "swipe_away" => Ok(BridgeAction::Intent(IntentWire::SwipeAway)),
            "document_end" => Ok(BridgeAction::Intent(IntentWire::DocumentEnd)),
            "control_unmapped" => Ok(BridgeAction::Intent(IntentWire::ControlUnmapped)),
            "lifecycle_unmapped" => Ok(BridgeAction::Intent(IntentWire::LifecycleUnmapped)),
            "diagnose" => Ok(BridgeAction::Diagnose),
            _ => Err(format!("unknown action: {action}")),
        }
    }

    /// Execute one playback action over IPC.
    ///
    /// Every wire routes through [`parse_action`]: verbs
    /// execute through the single session, intents route
    /// to the nearest existing machine entry point, and
    /// `diagnose` answers the read-only inventory.
    ///
    /// * `play` requires `document` (Markdown source): splits,
    ///   synthesizes, enqueues, and starts playback, returning
    ///   segment/sample counts and the session snapshot.
    /// * `choose_engine` reads the optional `payload` engine
    ///   id (absent wire payload stays absent: `None`).
    /// * `pause` / `stop` act on the live session and backend.
    ///
    /// # Errors
    ///
    /// Returns the named rejection for wires outside the
    /// closed set and for illegal verb presses from the
    /// current transport state — a rejection, never a
    /// playing state.
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri command parameters must be owned Deserialize/CommandArg types: &str cannot be extracted from the IPC payload, and AppHandle/State have by-value CommandArg impls only"
    )]
    pub fn run_mobile_plugin(
        action: String,
        document: Option<String>,
        payload: Option<String>,
        app: AppHandle,
        state: State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match parse_action(action.as_str()) {
            Ok(BridgeAction::Verb(verb)) => dispatch_verb(verb, document, payload.as_ref(), &state),
            Ok(BridgeAction::Intent(intent)) => dispatch_intent(intent, &state),
            Ok(BridgeAction::Diagnose) => Ok(diagnose(&app)),
            Err(reason) => Err(reason),
        }
    }

    /// Execute one parsed session verb (T110): the same
    /// [`player::cli::PlaybackSession::execute`] code path
    /// every other driver calls (Principle II).
    ///
    /// * `Play` keeps its synthesis path (`play`) behind
    ///   the pre-play permission gate.
    /// * Every verb that can reach `Playing`
    ///   (`Play | ResumeAtStored | StartOver |
    ///   ExternalStart` per the closed transition table)
    ///   takes the same pre-play gate: a refusal returns
    ///   the stated-reason payload and executes nothing.
    /// * `Pause` / `Stop` act on the backend as well as
    ///   the session; every other verb needs no backend
    ///   action.
    /// * `ChooseEngine` is the session's same-state
    ///   selection: the optional `payload` engine id is
    ///   echoed in the reply, and the snapshot's
    ///   authoritative `engine_id` answers which engine
    ///   holds selection (the id-consuming fallback
    ///   selector lives in engine-core, behind failures —
    ///   `execute` takes no id argument, so there is no
    ///   id path to thread beyond the echo).
    fn dispatch_verb(
        verb: ShellVerb,
        document: Option<String>,
        payload: Option<&String>,
        state: &State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match verb {
            ShellVerb::Play => {
                // Permission-state gate (T030b, FR-008/FR-010):
                // consult the pre-play check before any Play-class
                // verb — `Play` is the one the bridge exposes
                // today, and `ExternalStart`, `ResumeAtStored`,
                // and `StartOver` (the verbs that can reach
                // `Playing` in `player::cli::ShellVerb`) take
                // the same gate when the bridge grows them. A
                // refusal returns the stated-reason JSON error
                // payload and executes nothing, so no playing
                // state can ever be reported from a refused
                // start (FR-010).
                let permissions = state.permissions.lock().map_err(|e| e.to_string())?;
                if let Err(failure) = permissions::check_pre_play(&permissions) {
                    // The FR-017 diagnostic record (T097): the
                    // refusal is named before the ack leaves the
                    // path — the start was refused (R5).
                    record_named_refusal(state, ErrorValue::StartRefused, "permission_refused");
                    return Ok(serde_json::json!({
                        "ok": false,
                        "error": failure.reason(),
                    }));
                }
                play(&document.unwrap_or_default(), state).map_err(|error| error.to_string())
            }
            ShellVerb::ResumeAtStored | ShellVerb::StartOver | ShellVerb::ExternalStart => {
                resume_class_verb(verb, state)
            }
            ShellVerb::Pause => {
                if let Ok(backend) = state.backend.lock()
                    && let Some(backend) = backend.as_ref()
                {
                    backend.pause();
                }
                let snap = {
                    let mut session = state.session.lock().map_err(|e| e.to_string())?;
                    session
                        .execute(ShellVerb::Pause)
                        .map_err(|e| e.to_string())?
                };
                publish_current_state(state, None);
                // Event-durable put (T011 skeleton, the US3
                // call site): the deliberate pause persists
                // the position with the `paused` end kind,
                // which opens the FR-032c continue offer on
                // the next open (storage-schema.md).
                after_execute(state, &snap, ShellVerb::Pause);
                Ok(serde_json::json!({
                    "status": "paused",
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            ShellVerb::Stop => {
                if let Ok(backend) = state.backend.lock()
                    && let Some(backend) = backend.as_ref()
                {
                    backend.stop();
                }
                let snap = {
                    let mut session = state.session.lock().map_err(|e| e.to_string())?;
                    session
                        .execute(ShellVerb::Stop)
                        .map_err(|e| e.to_string())?
                };
                publish_current_state(state, None);
                // Terminal end: no bookmark is written — T035
                // deletes the entry below — and the FR-015
                // clock closes, so it cannot leak into the
                // next session.
                after_execute(state, &snap, ShellVerb::Stop);
                // Stop-then-delete (T035, FR-032 / FR-032b /
                // FR-032f): the press lands `Stopped`, which is
                // terminal, so the stored row is discarded and
                // the next open starts at `0.0` with no resume
                // offer. Only a genuine stop deletes — the
                // residency teardowns (`resident_stop` /
                // `swipe_away`) pause first and PRESERVE the
                // bookmark (FR-009a / FR-028e), and the
                // pause-class intents never reach here.
                delete_bookmark(state);
                Ok(serde_json::json!({
                    "status": "stopped",
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            ShellVerb::SkipForward
            | ShellVerb::SkipBackward
            | ShellVerb::RateUp
            | ShellVerb::RateDown
            | ShellVerb::CycleVoiceNext
            | ShellVerb::CycleVoicePrev => execute_live_control(state, verb),
            ShellVerb::ChooseEngine => {
                // A selection, not a transition: legal from
                // every state, changing no transport or
                // position (FR-005). The snapshot's
                // `engine_id` is the authoritative answer;
                // `requested_engine_id` echoes the wire
                // payload (`None` renders `null`).
                let mut session = state.session.lock().map_err(|e| e.to_string())?;
                let snap = session
                    .execute(ShellVerb::ChooseEngine)
                    .map_err(|e| e.to_string())?;
                let wire = serde_json::json!({
                    "status": snap.transport.to_string(),
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                    "engine_id": snap.engine_id,
                    "requested_engine_id": payload,
                });
                drop(session);
                Ok(wire)
            }
            ShellVerb::Complete | ShellVerb::Interrupt => {
                // Session-internal terminals with no bridge
                // wire: reaching one here means the closed
                // wire set drifted past `parse_action` — a
                // named rejection, never a playing state.
                Err(format!("verb '{verb}' has no bridge wire"))
            }
        }
    }

    /// Execute one live-session control verb (T042 / T035):
    /// the skips, the rate moves, and the voice cycles share
    /// this arm — no permission gate (they can never reach
    /// `Playing`) and no backend action (the session owns the
    /// rate/skip/voice state). An illegal press from a
    /// torn-down session is the named rejection, never a
    /// playing state. The `status` mirrors the resulting
    /// transport, so a playing state is reported only from a
    /// session that actually plays (FR-005).
    ///
    /// The live rate path resolves its persistence through
    /// the canonical [`RateStepper`] (write-on-change, T042 /
    /// FR-019b): the press is replayed on a stepper opened at
    /// the session's own pre-press rung — the session stays
    /// the sole rate authority, the stepper only answers
    /// WHETHER the press moved the rung — and the `Some` rung
    /// it reports is exactly the value persisted through
    /// [`persist_last_rate`]. A press that saturated at a
    /// ladder end reports `None`, so nothing is written. The
    /// rung is never rendered anywhere on this path:
    /// value-blind by contract (FR-012).
    ///
    /// Extracted from the `dispatch_verb` arm so the rate
    /// persistence is exercisable on the host (a `State`
    /// handle cannot be constructed outside a running Tauri
    /// app), never as a second dispatch.
    pub(crate) fn execute_live_control(
        state: &AppState,
        verb: ShellVerb,
    ) -> Result<serde_json::Value, String> {
        let (snapshot, previous_rate) = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            let previous_rate = session.snapshot().rate;
            let snapshot = session.execute(verb).map_err(|e| e.to_string())?;
            drop(session);
            (snapshot, previous_rate)
        };
        publish_current_state(state, None);
        // Section change (T035): a skip moves the
        // position without a pause, so the durable row
        // is refreshed through the section-change
        // bookkeeping — `interrupted` for the
        // transport-holding skip, `paused` for a
        // forward skip that reaches the duration and
        // lands `Paused` (FR-018). Every other live
        // control holds its transport, and the write
        // is a no-op.
        after_skip(state, &snapshot, verb);
        // T042 write-on-change (FR-019b): only a rate press
        // persists, and only when it moved the rung — the
        // stepper's `None` (a saturating press) writes
        // nothing new.
        if let Some(step) = rate_press_of(verb)
            && let Some(rung) = RateStepper::open(Some(previous_rate)).step(step)
        {
            persist_last_rate(state, rung);
            // The FR-017 diagnostic record (T097): the per-rate
            // change entry — the rung before and the rung after.
            record_rate_change(state, &snapshot, previous_rate, rung);
        } else if matches!(verb, ShellVerb::SkipForward | ShellVerb::SkipBackward) {
            // The FR-017 diagnostic record (T097): the per-section
            // progress entry — the section the session landed in,
            // with the document's content hash as its only document
            // reference.
            record_section_progress(state, &snapshot, diag_section_index(state, &snapshot));
        }
        Ok(serde_json::json!({
            "status": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "transport": snapshot.transport.to_string(),
        }))
    }

    /// Route one parsed platform intent (T110) to the
    /// nearest existing machine entry point — never a
    /// new state machine, never silent absorb. Each arm
    /// delegates to one leaf helper below; the helpers
    /// carry the per-intent rationale.
    ///
    /// A silence demand meeting an already-silent or
    /// terminal session acks the current mirror (the
    /// T019 double-press convention for signals, R3) —
    /// never an error, never a playing state.
    fn dispatch_intent(
        intent: IntentWire,
        state: &State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match intent {
            IntentWire::DuckableTransient => ack_duck(state),
            IntentWire::SilenceDemanding => {
                // FR-013a: pause immediately, never duck —
                // applied to the held tracker so the
                // silence escalation persists (FR-014).
                let decision = apply_focus(state, FocusIntent::SilenceDemanding)?;
                pause_ack(
                    state,
                    "silence_demanding",
                    decision.outcome,
                    ShellVerb::Interrupt,
                    Some("silence-demanding".to_string()),
                )
            }
            IntentWire::PermanentLoss => {
                // FR-014: pause, never auto-resume — applied
                // to the held tracker so the supersede
                // flag-clearing persists.
                let decision = apply_focus(state, FocusIntent::PermanentLoss)?;
                pause_ack(
                    state,
                    "permanent_loss",
                    decision.outcome,
                    ShellVerb::Interrupt,
                    Some("permanent_loss".to_string()),
                )
            }
            IntentWire::FocusRegained => {
                // FR-015: authorizes resume via the rewind
                // path — never an auto-start from the
                // bridge. Resolves the held loss and marks
                // the transient resume owed. Restores full
                // gain (FULL_GAIN) on the backend.
                let decision = apply_focus(state, FocusIntent::FocusRegained)?;
                let restore_ok = if let Ok(backend_guard) = state.backend.lock()
                    && let Some(backend) = backend_guard.as_ref()
                {
                    backend.set_gain(player::focus::FULL_GAIN).is_ok()
                } else {
                    false
                };
                // The FR-017 diagnostic record (T097): the focus
                // direction event — the regained focus, never an
                // outcome (the closed focus set's fourth member).
                record_focus_regained(state);
                publish_current_state(state, None);
                let snapshot = snapshot_of(state)?;
                Ok(serde_json::json!({
                    "status": "focus_regained",
                    "outcome": decision.outcome,
                    "transport": snapshot.transport.to_string(),
                    "position_secs": snapshot.position_secs,
                    "changed": false,
                    "auto_resumed": false,
                    "gain_restored": restore_ok,
                }))
            }
            IntentWire::BecomingNoisy => dispatch_noisy_intent(state),
            IntentWire::DeviceAppeared => {
                // FR-017a: follow at the unchanged position.
                // The routing arm needs device identity the
                // wire lacks, so the follow-only focus
                // classification answers — driven through
                // the held tracker for latch consistency
                // (a follow-only intent changes nothing it
                // holds). Publishes the mirror state (T133).
                let decision = apply_focus(state, FocusIntent::DeviceAppeared)?;
                // The FR-017 diagnostic record (T097): the route
                // that appeared (FR-017a).
                record_device_appeared(state);
                publish_current_state(state, None);
                let snapshot = snapshot_of(state)?;
                Ok(serde_json::json!({
                    "status": "device_appeared",
                    "outcome": decision.outcome,
                    "transport": snapshot.transport.to_string(),
                    "position_secs": snapshot.position_secs,
                    "changed": false,
                }))
            }
            IntentWire::DeviceDisappeared => {
                // FR-016: pause first, explicit resume only
                // — without inventing a device id. Applied
                // to the held tracker so the permanent-loss
                // escalation persists (FR-014).
                // The FR-017 diagnostic record (T097): the
                // removal clock opens here — the route-removal
                // instant — and the pause that follows closes it.
                note_removal_request(state);
                let decision = apply_focus(state, FocusIntent::DeviceDisappeared)?;
                let ack = pause_ack(
                    state,
                    "device_disappeared",
                    decision.outcome,
                    ShellVerb::Interrupt,
                    Some("device_disappeared".to_string()),
                )?;
                // The route that went away (FR-017a): the wire
                // carries no device identity, so the record names
                // the direction alone.
                record_device_disappeared(state);
                Ok(ack)
            }
            IntentWire::AppForegrounded => {
                // FR-032c latch only: latched into the held
                // machines, read at offer expiry only.
                ack_latch(state, "app_foregrounded", FocusIntent::AppForegrounded)
            }
            IntentWire::AppBackgrounded => {
                // FR-032c latch only, read at expiry only.
                let ack = ack_latch(state, "app_backgrounded", FocusIntent::AppBackgrounded)?;
                // Durable BEFORE the suspend (T035, FR-004 /
                // FR-007): a backgrounded process can be
                // suspended or killed at any instant, so the
                // position is persisted now — with the end kind
                // the current transport maps to, `interrupted`
                // for a playing session — instead of waiting for
                // the 20 s heartbeat that may never tick again.
                // The latch just written is never read here.
                after_background(state);
                Ok(ack)
            }
            IntentWire::ResidentStop => {
                // FR-009a: ends residency PRESERVING the
                // bookmark — never the discarding `Stop`.
                pause_ack(state, "resident_stop", None, ShellVerb::Pause, None)
            }
            IntentWire::SwipeAway => {
                // FR-028e fallback: pause first when audio
                // still plays, preserve the bookmark.
                pause_ack(
                    state,
                    "swipe_away",
                    None,
                    ShellVerb::Pause,
                    Some("swipe_away".to_string()),
                )
            }
            IntentWire::DocumentEnd => dispatch_document_end(state),
            IntentWire::ControlUnmapped => ack_unmapped(state, "control_unmapped"),
            IntentWire::LifecycleUnmapped => ack_unmapped(state, "lifecycle_unmapped"),
        }
    }

    /// Best-effort backend pause shared by the
    /// silence-demanding intent paths: mirrors the
    /// pause/stop arms (a missing or poisoned backend
    /// lock pauses nothing; the session verb below
    /// still runs, so silence is never skipped).
    /// Also restores full gain (`FULL_GAIN`) so that a
    /// paused session never resumes at a ducked level.
    fn pause_backend(state: &State<'_, AppState>) {
        if let Ok(backend) = state.backend.lock()
            && let Some(backend) = backend.as_ref()
        {
            let _ = backend.set_gain(player::focus::FULL_GAIN);
            backend.pause();
        }
    }

    /// Best-effort backend stop behind a real document
    /// completion: mirrors the stop arm. It runs only
    /// after [`ShellVerb::Complete`] moves the session,
    /// never ahead of it, so a retained pause keeps its
    /// queued audio. Restores full gain (`FULL_GAIN`).
    fn stop_backend(state: &AppState) {
        if let Ok(backend) = state.backend.lock()
            && let Some(backend) = backend.as_ref()
        {
            let _ = backend.set_gain(player::focus::FULL_GAIN);
            backend.stop();
        }
    }

    /// The session's current mirror for read-only intent
    /// acks: observing the snapshot changes nothing.
    fn snapshot_of(state: &State<'_, AppState>) -> Result<SessionSnapshot, String> {
        state
            .session
            .lock()
            .map_err(|e| e.to_string())
            .map(|session| session.snapshot())
    }

    /// The session's current snapshot for the diagnostic record
    /// (T097): the record's own position is the session's
    /// authoritative one, read under the session lock.
    fn diag_snapshot(state: &AppState) -> Option<SessionSnapshot> {
        state.session.lock().ok().map(|session| session.snapshot())
    }

    /// The FR-017 diagnostic record (T097): the focus direction
    /// event — the regained focus, never an outcome (the closed
    /// focus set's fourth member).
    fn record_focus_regained(state: &AppState) {
        if let Some(snapshot) = diag_snapshot(state) {
            record_focus_transition(
                state,
                &snapshot,
                FocusValue::Regained,
                "focus_regained",
                "regained",
            );
        }
    }

    /// The FR-017 diagnostic record (T097): the route that
    /// appeared (FR-017a). The wire carries no device identity, so
    /// the record names the direction alone.
    fn record_device_appeared(state: &AppState) {
        if let Some(snapshot) = diag_snapshot(state) {
            record_output_device(
                state,
                &snapshot,
                DeviceEvent {
                    direction: DeviceDirection::Appeared,
                    device_id: String::from("none"),
                },
                "device_appeared",
            );
        }
    }

    /// The FR-017 diagnostic record (T097): the route that went
    /// away (FR-017a). The wire carries no device identity, so the
    /// record names the direction alone.
    fn record_device_disappeared(state: &AppState) {
        if let Some(snapshot) = diag_snapshot(state) {
            record_output_device(
                state,
                &snapshot,
                DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: String::from("none"),
                },
                "device_disappeared",
            );
        }
    }

    /// Apply one focus intent to the held tracker (T125,
    /// FR-014 / FR-032c): the single entry point every
    /// focus-bearing arm uses, so latest-event-wins
    /// supersede, the duck/resume flags, and the
    /// foreground latch persist across IPC calls — never
    /// a fresh machine per call. Returns the tracker's
    /// classification (identical values to the stateless
    /// mapping except where persistence owns the answer:
    /// a late transient meeting a standing permanent loss
    /// holds the paused outcome).
    fn apply_focus(
        state: &State<'_, AppState>,
        intent: FocusIntent,
    ) -> Result<FocusClassification, String> {
        state
            .focus_tracker
            .lock()
            .map_err(|e| e.to_string())
            .map(|mut tracker| tracker.apply(intent))
    }

    /// Execute one session verb for a platform signal,
    /// falling back to the current snapshot when the
    /// press is illegal from the current state. Only
    /// the illegal-transition failure is reachable here
    /// (pause/interrupt/complete moves carry no position
    /// math — an illegal press leaves transport and
    /// position untouched, so the pre-read mirror is
    /// the post-failure mirror), and an already-silent
    /// or terminal session meets a silence demand with
    /// an ack of its mirror — never an error, never a
    /// playing state. The flag reports whether the verb
    /// moved the session.
    ///
    /// A move that lands the session paused runs the
    /// post-execution bookkeeping (T011): the FR-015
    /// clock note plus the event-durable put — the
    /// single US3 call site every pause-class intent
    /// (silence-demanding, permanent-loss,
    /// device-disappeared, becoming-noisy, resident-stop,
    /// swipe-away) reaches through. The `Complete`
    /// landing records no bookmark and closes the
    /// clock instead.
    fn execute_or_current(
        state: &AppState,
        verb: ShellVerb,
    ) -> Result<(SessionSnapshot, bool), String> {
        let (snapshot, moved) = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            let current = session.snapshot();
            session
                .execute(verb)
                .map_or_else(|_| (current, false), |snapshot| (snapshot, true))
        };
        if moved {
            after_execute(state, &snapshot, verb);
        }
        Ok((snapshot, moved))
    }

    /// Reply for a silence-demanding signal (T110): the
    /// backend pause plus one session verb, with the
    /// machine's outcome naming the applied decision and
    /// the flag reporting whether the session moved.
    /// The verb is [`ShellVerb::Interrupt`] for
    /// unexpected silences (the verb whose own docs name
    /// focus loss) and [`ShellVerb::Pause`] where the
    /// bookmark must survive residency teardown.
    /// Publishes the mirror state after the pause (T133).
    /// The verb runs through [`Self::execute_or_current`],
    /// so a landed pause also persists the position
    /// event-durably (T011) — `interrupted` for the
    /// focus-loss verbs, `paused` for the teardowns.
    fn pause_ack(
        state: &State<'_, AppState>,
        status: &'static str,
        outcome: Option<FocusOutcome>,
        verb: ShellVerb,
        transient_reason: Option<String>,
    ) -> Result<serde_json::Value, String> {
        pause_backend(state);
        let (snapshot, changed) = execute_or_current(state, verb)?;
        // The FR-017 diagnostic record (T097): the interruption's
        // class and its outcome, with the measured values the
        // contract names.
        if let Some(outcome) = outcome {
            record_focus_transition(
                state,
                &snapshot,
                diag_focus_value(outcome),
                status,
                diag_focus_name(outcome),
            );
        }
        publish_current_state(state, transient_reason);
        Ok(serde_json::json!({
            "status": status,
            "outcome": outcome,
            "transport": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "changed": changed,
        }))
    }

    /// Reply for the duckable transient (T110, FR-013):
    /// ducking never changes the transport, so no
    /// session verb runs in any state. The ~20% gain
    /// is applied to the backend (`CpalBackend` exposes
    /// `set_gain`/`gain` since T123). Driven through the
    /// held tracker (T125, FR-014), so the ducked flag
    /// and duck-restore flag persist for the silence
    /// escalation that may follow. Publishes the mirror
    /// state with the ducked gain (T133).
    fn ack_duck(state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
        let decision = apply_focus(state, FocusIntent::DuckableTransient)?;
        let applied = if let Ok(backend_guard) = state.backend.lock()
            && let Some(backend) = backend_guard.as_ref()
        {
            backend.set_gain(decision.gain).is_ok()
        } else {
            false
        };
        let snapshot = snapshot_of(state)?;
        // The FR-017 diagnostic record (T097): the duckable
        // transient's class and outcome. A deferred grant names no
        // outcome, so it records no focus transition.
        if let Some(outcome) = decision.outcome {
            record_focus_transition(
                state,
                &snapshot,
                diag_focus_value(outcome),
                "duckable_transient",
                diag_focus_name(outcome),
            );
        }
        publish_current_state(state, None);
        Ok(serde_json::json!({
            "status": "duckable_transient",
            "outcome": decision.outcome,
            "transport": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "changed": false,
            "gain": decision.gain,
            "gain_applied": applied,
        }))
    }

    /// Reply for a lifecycle latch intent (T110, FR-032c):
    /// no playback decision — the intent latches the held
    /// machines (T125): the foreground latch writes the
    /// held [`player::focus::FocusTracker`] through
    /// `apply_focus` and the held
    /// [`session::offer::OfferMachine`] through
    /// `on_foregrounded` / `on_backgrounded`. The latch is
    /// read at offer expiry only and never converts an
    /// open offer early. The JSON shape is unchanged
    /// (byte-compatible with the stateless ack).
    fn ack_latch(
        state: &State<'_, AppState>,
        status: &'static str,
        focus: FocusIntent,
    ) -> Result<serde_json::Value, String> {
        let decision = apply_focus(state, focus)?;
        {
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            if focus == FocusIntent::AppForegrounded {
                offer.on_foregrounded();
            } else if focus == FocusIntent::AppBackgrounded {
                offer.on_backgrounded();
            }
        }
        Ok(serde_json::json!({
            "status": status,
            "outcome": decision.outcome,
            "changed": false,
            "foregrounded": decision.foreground_latched,
        }))
    }

    /// Reply for an unmapped sentinel (T110): reported,
    /// never absorbed, never an error, never a state
    /// change — the mirror is observed only.
    fn ack_unmapped(
        state: &State<'_, AppState>,
        status: &'static str,
    ) -> Result<serde_json::Value, String> {
        let snapshot = snapshot_of(state)?;
        Ok(serde_json::json!({
            "status": status,
            "transport": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "absorbed": false,
            "changed": false,
        }))
    }

    /// Publish the current mirror state to the frontend (T133).
    ///
    /// Called after every transport/position/focus event that
    /// changes the session state. Reads the surface resolver
    /// and headings from `AppState` and emits a `MirrorState`
    /// snapshot via the `tauri::ipc::Channel`.
    pub(crate) fn publish_current_state(state: &AppState, transient_reason: Option<String>) {
        let Ok(resolver_guard) = state.surface_resolver.lock() else {
            return;
        };
        let Some(resolver) = resolver_guard.as_ref() else {
            return; // no document loaded
        };
        let Ok(headings_guard) = state.headings.lock() else {
            return;
        };
        crate::publish::publish_from_session(state, resolver, &headings_guard, transient_reason);
        drop(headings_guard);
        drop(resolver_guard);
    }

    /// Route the noisy signal (T110, FR-016): the routing
    /// machine owns the ≤1 s bound, so it is consulted
    /// at the live transport. A fresh tracker latches no
    /// device, so the pause names the route itself, as
    /// the machine documents for the unlatched case.
    /// `Pause` pauses through the interrupt path;
    /// `NoChange` acks the already-silent session (R3:
    /// never an auto-resume). The silence is also applied
    /// to the held focus tracker (T125, FR-014), so a
    /// duck in progress does not survive the removal.
    /// Publishes the mirror state after the pause (T133).
    fn dispatch_noisy_intent(state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
        let transport = snapshot_of(state)?.transport;
        let _ = apply_focus(state, FocusIntent::BecomingNoisy)?;
        let mut tracker = RoutingTracker::new();
        match tracker.apply(RoutingSignal::BecomingNoisy, transport) {
            RoutingDecision::Pause { latency_ms, device } => {
                // The FR-017 diagnostic record (T097): the removal
                // clock opens here — the route-removal instant —
                // and the pause that follows closes it.
                note_removal_request(state);
                pause_backend(state);
                let (snapshot, changed) = execute_or_current(state, ShellVerb::Interrupt)?;
                // The route that went noisy (FR-017a), with the
                // machine's own device event.
                record_output_device(state, &snapshot, device, "becoming_noisy");
                publish_current_state(state, Some("becoming_noisy".to_string()));
                Ok(serde_json::json!({
                    "status": "becoming_noisy",
                    "transport": snapshot.transport.to_string(),
                    "position_secs": snapshot.position_secs,
                    "changed": changed,
                    "latency_ms": latency_ms,
                }))
            }
            RoutingDecision::Follow { .. } => {
                // Unreachable for a removal signal; kept
                // for exhaustiveness (no catch-all arm).
                let snapshot = snapshot_of(state)?;
                publish_current_state(state, None);
                Ok(serde_json::json!({
                    "status": "becoming_noisy",
                    "transport": snapshot.transport.to_string(),
                    "position_secs": snapshot.position_secs,
                    "changed": false,
                }))
            }
            RoutingDecision::NoChange => {
                let snapshot = snapshot_of(state)?;
                publish_current_state(state, None);
                Ok(serde_json::json!({
                    "status": "becoming_noisy",
                    "transport": snapshot.transport.to_string(),
                    "position_secs": snapshot.position_secs,
                    "changed": false,
                }))
            }
        }
    }

    /// Route the document end (T110, FR-032f): teardown
    /// through [`ShellVerb::Complete`], legal only from
    /// `Playing` — any other state acks its mirror
    /// unchanged, and the backend stops only behind a
    /// real completion. The held offer machine tears down
    /// with it (T125, FR-032b / FR-032e / FR-032f):
    /// completion and the resume offer are mutually
    /// exclusive, so a later expiry yields no outcome. A
    /// real completion also DELETES the stored bookmark
    /// (T035) — a completed document has no resumable
    /// position. Publishes the mirror state after
    /// completion (T133).
    pub(crate) fn dispatch_document_end(state: &AppState) -> Result<serde_json::Value, String> {
        let (snapshot, changed) = execute_or_current(state, ShellVerb::Complete)?;
        if changed {
            stop_backend(state);
            if let Ok(mut offer) = state.offer.lock() {
                offer.on_completion();
            }
            // Completion deletes (T035, FR-032b / FR-032f): a
            // completed document has no resumable position, so
            // the stored row is discarded — the second of the
            // two deleting ends, with the Stop press. The held
            // offer machine tore down above, so no offer
            // survives it either.
            delete_bookmark(state);
        }
        publish_current_state(state, None);
        Ok(serde_json::json!({
            "status": "document_end",
            "transport": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "changed": changed,
        }))
    }

    /// Extract the document headings from the parsed segments
    /// (T019 / FR-028a): every `Heading`-boundary segment becomes
    /// a heading whose `position_secs` is estimated from its
    /// segment index over the total spoken duration. The list is
    /// the one feeding of the session-start section count, so it
    /// is computed once at open and never re-estimated.
    #[expect(
        clippy::cast_precision_loss,
        reason = "segment indices and counts are always small (≤ millions), well within f64's 52-bit mantissa"
    )]
    fn parse_headings(
        segments: &[adapter_synthesis::chunk::DocumentSegment],
        duration_secs: f64,
    ) -> Vec<player::surface::Heading> {
        segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.boundary_kind == adapter_synthesis::chunk::BoundaryKind::Heading)
            .map(|(idx, segment)| {
                let estimated_position = (idx as f64 / segments.len() as f64) * duration_secs;
                let heading_title = segment
                    .content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .trim()
                    .to_string();
                player::surface::Heading {
                    position_secs: estimated_position,
                    title: heading_title,
                }
            })
            .filter(|heading| !heading.title.is_empty())
            .collect()
    }

    /// The streaming synthesis's totals (T113 / FR-003): the
    /// document's spoken duration and the samples actually played from
    /// the resume entry on.
    #[derive(Debug, Clone, Copy)]
    struct StreamedSynthesis {
        /// The document's spoken duration, in seconds — the
        /// resolver's projection basis, fixed at session start
        /// (FR-028a).
        duration_secs: f64,
        /// The samples played from the resume entry on — the audio
        /// the reading actually carries (FR-009).
        played_samples: usize,
    }

    /// Synthesize the document segment by segment and hand every chunk
    /// to the backend as it is produced (T113 / FR-003).
    ///
    /// The FIRST chunk is enqueued and the backend is started BEFORE
    /// the second segment is synthesized: the first audible speech
    /// arrives while the rest of the document is still being
    /// synthesized, which is what the SC-002 ≤3 s bound names. The
    /// whole document's PCM never exists in one buffer — every chunk
    /// reaches the queue as its OWN buffer, so the queue holds the
    /// speech produced so far and never a whole-document accumulator.
    ///
    /// The resume entry (T110) trims the samples before the resolved
    /// position: the samples before it are never played (FR-009), and
    /// the samples after it are the ones the reading continues from.
    ///
    /// # Errors
    ///
    /// * [`OpenError::Synthesis`] when a chunk cannot be synthesized
    ///   by the bound offline engine.
    /// * [`OpenError::Backend`] when the backend refuses the chunk or
    ///   the stream start — a refusal, never a silent start (FR-005).
    #[expect(
        clippy::cast_precision_loss,
        reason = "PCM sample counts are always small (≤ millions), well within f64's 52-bit mantissa"
    )]
    fn stream_synthesis(
        segments: &[adapter_synthesis::chunk::DocumentSegment],
        state: &AppState,
        start_offset_secs: f64,
    ) -> Result<StreamedSynthesis, OpenError> {
        // The samples before the resume entry are never played
        // (FR-009): the count is the exact sample index the resolved
        // position names, and it shrinks as the chunks arrive.
        let mut samples_before_entry = sample_index_of(start_offset_secs);
        let mut total_samples = 0usize;
        let mut played_samples = 0usize;
        let mut started = false;
        for segment in segments {
            // The chunk the bound offline engine produced for this
            // segment: a refused chunk is the named
            // [`OpenError::Synthesis`].
            let chunk = adapter_synthesis::synth::synthesize(segment).map_err(|failure| {
                OpenError::Synthesis {
                    reason: format!("{failure:?}"),
                }
            })?;
            total_samples += chunk.len();
            let dropped = samples_before_entry.min(chunk.len());
            samples_before_entry -= dropped;
            let chunk = pcm_from_sample(&chunk, dropped);
            if chunk.is_empty() {
                // The whole chunk sits before the resume entry: it
                // carries no audible sample, and the reading waits
                // for the chunk that does (FR-009).
                continue;
            }
            played_samples += chunk.len();
            // The chunk reaches the backend as its OWN buffer — never
            // an accumulator the whole document's PCM was grown into
            // (FR-003).
            enqueue_chunk(state, chunk)?;
            if !started {
                // The first chunk is queued: the backend starts on it,
                // and the rest of the document synthesizes while it
                // plays (FR-003 / SC-002).
                start_backend(state)?;
                // The FR-017 diagnostic record (T097 / T113): the
                // first audible audio of this request closes the
                // play-to-audio clock — the stream start IS the first
                // audible speech, so the measured latency is the
                // FR-003 bound, never the whole document's synthesis
                // time.
                note_first_audible(state);
                started = true;
            }
        }
        let duration_secs = total_samples as f64 / DECLARED_RATE_HZ;
        Ok(StreamedSynthesis {
            duration_secs,
            played_samples,
        })
    }

    /// Install the session-start structure (T019 / FR-028a): write
    /// the surface resolver, the heading list, and the spoken
    /// duration into the shared state — the one feeding the
    /// publish path and the session read, written once and never
    /// re-estimated. Each lock is taken alone and released before
    /// the next, so no two state locks are ever held together.
    fn install_session_structure(
        state: &AppState,
        resolver: player::surface::SurfaceResolver,
        headings: Vec<player::surface::Heading>,
        duration_secs: f64,
    ) -> Result<(), OpenError> {
        let mut resolver_guard = state
            .surface_resolver
            .lock()
            .map_err(|e| OpenError::NoStart {
                reason: e.to_string(),
            })?;
        *resolver_guard = Some(resolver);
        drop(resolver_guard);
        let mut headings_guard = state.headings.lock().map_err(|e| OpenError::NoStart {
            reason: e.to_string(),
        })?;
        headings_guard.clone_from(&headings);
        drop(headings_guard);
        let mut session = state.session.lock().map_err(|e| OpenError::NoStart {
            reason: e.to_string(),
        })?;
        session.set_document_structure(duration_secs, headings);
        drop(session);
        Ok(())
    }

    /// Open (or reuse) the audio backend (T019 / FR-005): the
    /// backend is opened once per session and reused for every chunk
    /// the streaming path hands it (T113). The declared handoff shape
    /// or the named [`OpenError::Backend`] refusal, never a silent
    /// start.
    fn open_backend(state: &AppState) -> Result<(), OpenError> {
        let mut backend = state.backend.lock().map_err(|error| OpenError::Backend {
            reason: error.to_string(),
        })?;
        if backend.is_none() {
            let opened = CpalBackend::new().map_err(|error| OpenError::Backend {
                reason: error.to_string(),
            })?;
            *backend = Some(opened);
        }
        drop(backend);
        Ok(())
    }

    /// Hand one synthesized chunk to the backend's playback queue
    /// (T113 / FR-003): the chunk is enqueued as its OWN buffer, so
    /// the queue never holds a whole-document accumulator — the
    /// first chunk is audible while the rest still synthesizes.
    fn enqueue_chunk(state: &AppState, chunk: Vec<f32>) -> Result<(), OpenError> {
        let backend = state.backend.lock().map_err(|error| OpenError::Backend {
            reason: error.to_string(),
        })?;
        let device = backend.as_ref().ok_or_else(|| OpenError::Backend {
            reason: "no audio backend".to_string(),
        })?;
        device.enqueue_pcm(chunk);
        drop(backend);
        Ok(())
    }

    /// Start the backend's output stream (T019 / FR-005): the
    /// declared handoff shape or the named [`OpenError::Backend`]
    /// refusal, never a silent start.
    ///
    /// The stream starts on the FIRST chunk the streaming path queued
    /// (T113 / FR-003): the audio begins while the rest of the
    /// document is still being synthesized.
    fn start_backend(state: &AppState) -> Result<(), OpenError> {
        let backend = state.backend.lock().map_err(|error| OpenError::Backend {
            reason: error.to_string(),
        })?;
        let device = backend.as_ref().ok_or_else(|| OpenError::Backend {
            reason: "no audio backend".to_string(),
        })?;
        let started = device.play().map_err(|error| OpenError::Backend {
            reason: error.to_string(),
        });
        drop(backend);
        started
    }

    /// Split, synthesize, enqueue, and start one document —
    /// the single synthesis path every surface routes through
    /// (Principle II: the `play` wire action, the T019
    /// cold-start [`open_document_bytes`], and every later
    /// consumer land here exactly once).
    ///
    /// The caller owns the FR-018 document switch (the outgoing
    /// document's durable save and the session teardown); this
    /// body owns the synthesis alone.
    ///
    /// # Errors
    ///
    /// * [`OpenError::EmptyDocument`] when the document is
    ///   blank — nothing to synthesize.
    /// * [`OpenError::NoSegments`] when the parse yields no
    ///   bounded chunk.
    /// * [`OpenError::Synthesis`] when a chunk cannot be
    ///   synthesized by the bound offline engine.
    /// * [`OpenError::Backend`] when the audio backend cannot
    ///   be opened or started at the declared handoff shape —
    ///   a refusal, never a silent start (FR-005).
    /// * [`OpenError::NoStart`] when the closed transition
    ///   table rejects the press (a torn-down session takes no
    ///   play).
    fn play(document: &str, state: &AppState) -> Result<serde_json::Value, OpenError> {
        if document.trim().is_empty() {
            return Err(OpenError::EmptyDocument);
        }
        // The FR-018 document switch: the outgoing document's
        // last-spoken position is saved durably, the outgoing
        // session is torn down, and only then does the incoming
        // document take the session over.
        switch_to_document(state, document.as_bytes());
        play_document(document, state, DEFAULT_DOCUMENT_FILENAME, 0.0)
    }

    /// The synthesis half of the one synthesis path (Principle II):
    /// split, synthesize, enqueue, start — with the totals fixed
    /// at open and the rate seeded from the store.
    ///
    /// ## Session-start totals (T019 / FR-028a)
    ///
    /// The parsed segments fix the totals once, here: the
    /// heading count (`total_n`) and the marked document total
    /// (`total_units`) the surface resolver derives from the
    /// heading list this body writes to the shared state at
    /// open. The list is never re-estimated mid-session, so
    /// the progress determinacy the resolver resolves for the
    /// whole session comes from this one feeding.
    ///
    /// ## Cold-start budget (SC-002; T016 measures the host
    /// half, T051 gates the device half)
    ///
    /// This body owns the ≤3 s press-to-first-audio bound: the
    /// engine's voice model is loaded exactly once (the
    /// adapter caches it — never a per-segment load), the
    /// document is split into bounded chunks before any
    /// synthesis, and the FIRST chunk's PCM is enqueued and
    /// started while the remaining chunks still synthesize. The
    /// whole document's PCM is never materialized in one buffer:
    /// every chunk reaches the backend as its own buffer, so the
    /// first audible speech does not wait for the last segment
    /// (FR-003). A missed bound is fixed HERE — the streaming
    /// order is this body's change, never another task's — and
    /// the bound is never renegotiated.
    ///
    /// ## The resume entry (T110)
    ///
    /// `start_offset_secs` is the position the reading enters at:
    /// `0.0` for a fresh document, and the stored position for a
    /// cold start that continued from its resume offer (FR-009).
    /// The samples before it are never played, and the session's
    /// position opens AT it, so the audio and the position
    /// authority agree on where the reading is.
    ///
    /// # Errors
    ///
    /// * [`OpenError::EmptyDocument`] when the document is
    ///   blank — nothing to synthesize.
    /// * [`OpenError::NoSegments`] when the parse yields no
    ///   bounded chunk.
    /// * [`OpenError::Synthesis`] when a chunk cannot be
    ///   synthesized by the bound offline engine.
    /// * [`OpenError::Backend`] when the audio backend cannot
    ///   be opened or started at the declared handoff shape —
    ///   a refusal, never a silent start (FR-005).
    /// * [`OpenError::NoStart`] when the closed transition
    ///   table rejects the press (a torn-down session takes no
    ///   play).
    pub(crate) fn play_document(
        document: &str,
        state: &AppState,
        filename: &str,
        start_offset_secs: f64,
    ) -> Result<serde_json::Value, OpenError> {
        // The FR-017 diagnostic record (T097): the play-to-audio
        // clock opens here — the request instant — and the first
        // audible audio of this request closes it.
        note_play_request(state);
        // T042 read-at-start (FR-019b): the session opens at
        // the stored rung — seeded through the rate verbs
        // before anything else happens, so a fresh session
        // never starts at the wrong speed.
        seed_session_rate(state);
        // Same code path as the adapter CLI (Principle II): the
        // adapter owns the chunk boundary; the engine receives
        // bounded chunks only.
        let segments = adapter_synthesis::chunk::split_document(document, "");
        if segments.is_empty() {
            return Err(OpenError::NoSegments);
        }

        // The surface title (FR-011 / T103): the first Markdown H1
        // heading's text, the filename without its extension for a
        // plain-text document, or `None` — the resolver's filename
        // placeholder — as the fallback. NEVER the literal
        // "Document", never a sentence of the body.
        let title = document_title(&segments, filename);

        // T113 / FR-003 ≤3 s first-audible: the backend is opened
        // once, and the STREAMING synthesis hands the first chunk to
        // it — and starts the stream — while the rest of the document
        // is still being synthesized. The first audible speech
        // therefore does not wait for the whole document (SC-002).
        open_backend(state)?;
        let streamed = stream_synthesis(&segments, state, start_offset_secs)?;
        // The document's own duration stays the resolver's total — the
        // reading still ends where the document ends (FR-028a).
        let duration_secs = streamed.duration_secs;
        // The samples the reading actually carries: the audio from
        // the resume entry on (FR-009).
        let pcm_samples = streamed.played_samples;

        let headings = parse_headings(&segments, duration_secs);
        let total_units = segments.len() as u64;
        let has_headings = !headings.is_empty();

        // Create surface resolver with the parsed-content totals fixed
        // at session start (FR-013 / FR-028a): the unit total is the
        // chunk split's segment count — a content fact — and the
        // projection basis is the spoken length per unit derived from
        // the synthesized PCM. The totals knowledge composes BOTH
        // session-start facts (the parsed-content unit total AND the
        // section count), so a heading-less session stays
        // indeterminate for the whole session (FR-013).
        let resolver = player::surface::SurfaceResolver::new(
            title,
            filename.to_string(),
            total_units,
            duration_secs,
            has_headings,
        );

        // Store resolver and headings in AppState for publish path:
        // the one feeding of the session-start totals — the
        // resolver holds the parsed-content unit total and its
        // projection basis, written here, once, and never
        // re-estimated mid-session (FR-028a / FR-013).
        install_session_structure(state, resolver, headings, duration_secs)?;

        // The audio already started (T113): the streaming synthesis
        // queued the first chunk and the backend is playing it while
        // this body was still resolving the session-start totals. The
        // session's transport lands on `Playing` only below, so a
        // playing landing is always followed by a strictly greater
        // position — the diagnostic record's stall rule holds.
        //
        // The document's blake3 fingerprint (FR-036 / T059) — the key
        // the 20 s heartbeat bookmark persistence writes under (T011) —
        // is the same one `open_document` routes through. The FR-018
        // switch already recorded it (`switch_to_document`), so this
        // body never records a fingerprint of its own: the switch owns
        // the handover, and the durable save it performs must land
        // under the OUTGOING document's key.

        {
            let mut session = state.session.lock().map_err(|e| OpenError::NoStart {
                reason: e.to_string(),
            })?;
            let _landed = session
                .execute(ShellVerb::Play)
                .map_err(|e| OpenError::NoStart {
                    reason: e.to_string(),
                })?;
        }

        // The session's position opens AT the resume entry
        // (T110): the audible clock is the only advance path
        // (FR-002), so the resume credits the offset it entered
        // at. The audio and the position authority agree on where
        // the reading is.
        if start_offset_secs > 0.0 {
            let Ok(mut session) = state.session.lock() else {
                return Err(OpenError::NoStart {
                    reason: "the session lock is poisoned".to_string(),
                });
            };
            session.on_audible_progress(start_offset_secs);
        }
        let snapshot = {
            let Ok(session) = state.session.lock() else {
                return Err(OpenError::NoStart {
                    reason: "the session lock is poisoned".to_string(),
                });
            };
            session.snapshot()
        };

        // Open the FR-015 clock on the landed Playing
        // transport (T011).
        after_execute(state, &snapshot, ShellVerb::Play);

        // The FR-017 diagnostic record (T097): the buffer state
        // the session started with — the pre-roll seconds still
        // buffered at the open.
        record_buffer_state(state, &snapshot);

        // Publish mirror snapshot after successful play start —
        // the same helper every verb path uses.
        publish_current_state(state, None);

        Ok(serde_json::json!({
            "status": "playing",
            "segments": segments.len(),
            "pcm_samples": pcm_samples,
            "duration_secs": duration_secs,
            "position_secs": snapshot.position_secs,
            "transport": snapshot.transport.to_string(),
        }))
    }

    /// Report the packaged voice-asset state: the resolved resource
    /// dir and whether each required file is present. Read-only;
    /// never touches playback.
    fn diagnose(app: &AppHandle) -> serde_json::Value {
        let resource_dir = app.path().resource_dir().map_or_else(
            |e| format!("unresolvable: {e}"),
            |path| path.to_string_lossy().into_owned(),
        );
        let voice = app
            .path()
            .resource_dir()
            .map(|dir| dir.join("voices").join("pt_BR-faber-medium"))
            .unwrap_or_default();
        let data_voice = app
            .path()
            .app_data_dir()
            .map(|dir| dir.join("voices").join("pt_BR-faber-medium"))
            .unwrap_or_default();
        let (data_files, data_espeak_is_dir, data_espeak_files, data_onnx_bytes) =
            inventory(&data_voice);
        serde_json::json!({
            "resource_dir": resource_dir,
            "model_present": voice.join("pt_BR-faber-medium.onnx").exists(),
            "tokens_present": voice.join("tokens.txt").exists(),
            "espeak_present": voice.join("espeak-ng-data").exists(),
            "data_model_present": data_voice.join("pt_BR-faber-medium.onnx").exists(),
            "data_tokens_present": data_voice.join("tokens.txt").exists(),
            "data_files_total": data_files,
            "data_espeak_is_dir": data_espeak_is_dir,
            "data_espeak_files": data_espeak_files,
            "data_onnx_bytes": data_onnx_bytes,
        })
    }

    /// Count the extracted tree: total files, whether espeak-ng-data
    /// is a real dir, files inside it, and the model byte size.
    /// Read-only; capped so a huge tree cannot stall IPC.
    fn inventory(voice: &std::path::Path) -> (usize, bool, usize, u64) {
        let mut total = 0;
        let mut stack = vec![voice.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten().take(2000) {
                total += 1;
                if entry.path().is_dir() {
                    stack.push(entry.path());
                }
                if total > 5000 {
                    break;
                }
            }
        }
        let espeak = voice.join("espeak-ng-data");
        let espeak_is_dir = espeak.is_dir();
        let espeak_files = std::fs::read_dir(&espeak).map_or(0, std::iter::Iterator::count);
        let onnx_bytes =
            std::fs::metadata(voice.join("pt_BR-faber-medium.onnx")).map_or(0, |meta| meta.len());
        (total, espeak_is_dir, espeak_files, onnx_bytes)
    }

    /// Get the current in-app status view (T065b).
    ///
    /// The read-only mirror of the session's authoritative state
    /// plus the stored `last_rate`. The view owns no state and
    /// computes no position, rate, or transport — it renders what
    /// the session already states (SC-035, FR-042).
    ///
    /// Returns the rendered `StatusView` with rate, transport,
    /// position, reason, and `reason_expiry`.
    ///
    /// # Errors
    ///
    /// Returns an error string when a state lock is poisoned or no
    /// document is loaded (no surface resolver).
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn get_status_view(
        state: State<'_, AppState>,
    ) -> Result<crate::status_view::StatusView, String> {
        let snap = state.session.lock().map_err(|e| e.to_string())?.snapshot();
        let resolver_guard = state.surface_resolver.lock().map_err(|e| e.to_string())?;
        let resolver = resolver_guard
            .as_ref()
            .ok_or_else(|| "no document loaded".to_string())?;
        let headings_guard = state.headings.lock().map_err(|e| e.to_string())?;
        let surface = resolver.resolve(
            snap.position_secs,
            resolver.consumed_units_at(snap.position_secs),
            &headings_guard,
        );
        drop(headings_guard);
        drop(resolver_guard);

        let fingerprint = *state
            .current_fingerprint
            .lock()
            .map_err(|e| e.to_string())?;
        // Get stored last_rate from bookmark store. The
        // fingerprint gates the read: with no document
        // loaded there is no key to read beside, and the
        // view renders the normal-speed default.
        let last_rate = if fingerprint.is_some() {
            let mut bookmark_store_guard =
                state.bookmark_store.lock().map_err(|e| e.to_string())?;
            let rate = bookmark_store_guard
                .as_mut()
                .and_then(|store| store.get_last_rate().ok().flatten());
            drop(bookmark_store_guard);
            rate
        } else {
            None
        };

        // Build transient reason from the surface's reason if any
        // The surface doesn't carry reason directly, so we rely on the
        // publish path's transient_reason. For the initial view, we
        // don't have a transient reason.
        let reason: Option<(crate::surface::FullReason, crate::surface::ReasonExpiry)> = None;

        let fingerprint = fingerprint.unwrap_or([0u8; 32]);
        Ok(crate::status_view::render_status_view(
            last_rate,
            snap.transport,
            snap.position_secs,
            &surface.title,
            surface.section_title,
            reason,
            fingerprint,
        ))
    }

    /// Delete the session bookmark for a fingerprint (Stop control).
    /// This clears the stored position so the next open starts at 0.0
    /// with no resume offer (FR-032f, storage-schema.md).
    ///
    /// Called when the user presses the dedicated Stop button in-app.
    ///
    /// # Errors
    ///
    /// Returns the store's error string when the bookmark store lock
    /// is poisoned or the delete fails.
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn session_delete(fingerprint: [u8; 32], state: State<'_, AppState>) -> Result<(), String> {
        let mut bookmark_store_guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        if let Some(store) = bookmark_store_guard.as_mut() {
            store.delete(&fingerprint).map_err(|e| e.to_string())?;
        }
        drop(bookmark_store_guard);
        Ok(())
    }

    /// Route the stored resume offer to the surface that renders
    /// it (T037, FR-032c / FR-032d / FR-031).
    ///
    /// The thin route-to-surface command: it reads the stored row
    /// for the currently loaded document's content fingerprint and
    /// asks [`crate::route_offer`] which surface the row's end
    /// kind resolves to — `in_app_dialog` (with the FR-032c 3 s
    /// window) for the deliberate end, `lockscreen_resume` for the
    /// unexpected end, `none` when nothing is open. No offer logic
    /// lives here: the form comes from the offer machine in
    /// `crates/session/src/offer.rs` (FR-044), so the in-app dialog,
    /// the lock-screen control, and the CLI's `session ingest`
    /// path (T018) all render the same resolved offer.
    ///
    /// A `none` route is a normal reply, never an error: no loaded
    /// document, no stored row, and a terminal end all leave
    /// nothing to render.
    ///
    /// # Errors
    ///
    /// Returns the store's error string when the stored row cannot
    /// be read.
    #[command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn offer_route(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
        let Some(fingerprint) = current_fingerprint(&state) else {
            return Ok(serde_json::json!({ "offer_route": "none" }));
        };
        let stored = {
            let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
            match guard.as_mut() {
                Some(store) => store.get(&fingerprint).map_err(|e| e.to_string())?,
                None => None,
            }
        };
        let Some(stored) = stored else {
            return Ok(serde_json::json!({ "offer_route": "none" }));
        };
        // The one route-to-wire mapping (T037), shared with the
        // cold-start open payload — never a second definition
        // of the route vocabulary.
        let (offer_route, window_ms) = offer_route_wire(route_offer(&state, &stored));
        Ok(window_ms.map_or_else(
            || serde_json::json!({ "offer_route": offer_route }),
            |window_ms| {
                serde_json::json!({
                    "offer_route": offer_route,
                    "window_ms": window_ms,
                })
            },
        ))
    }
}

/// T109 (Phase 12 Convergence, BLOCKING TEST): bridge dispatch table.
///
/// RED test for the seam T110 will create inside [`commands`]:
/// `parse_action` classifying every closed shell-IPC wire string
/// (`specs/001-android-media-shell/contracts/shell-ipc.md`) into
/// `BridgeAction::Verb` / `BridgeAction::Intent` / `BridgeAction::Diagnose`.
///
/// Wire-string sources (verified by direct read):
/// * verbs `play|pause|stop|skip_forward5s|skip_back5s|rate_up|rate_down|
///   cycle_voice_next|cycle_voice_prev|control_unmapped` — `MediaMirror.kt`
///   `VERB_*` consts; `resume_at_stored|start_over|choose_engine` — the
///   contract's closed verb set via the normative `snake_case` case-mapping
///   (no Kotlin `VERB_*` const exists for these three yet — gap noted).
/// * intents `duckable_transient|silence_demanding|permanent_loss|regained`
///   (`FocusMirror.kt` `INTENT_*`; `FocusRegained` wires as `regained`, per
///   the contract case-mapping table and `test_focus_intents_us2.rs` WIRE),
///   `becoming_noisy|device_appeared|device_disappeared` (`RoutingMirror.kt`),
///   `resident_stop|swipe_away|document_end|lifecycle_unmapped`
///   (`Residency.kt`), `app_foregrounded|app_backgrounded` (contract +
///   `FocusIntent` WIRE table; no Kotlin `INTENT_*` const exists — gap noted).
///   Extra Kotlin-only wires with no `IntentWire` counterpart (also gaps):
///   `granted|deferred|refused|focus_change_unmapped|focus_grant_unmapped`.
///
/// Assertions use `matches!` (not `assert_eq!`) so the test imposes no
/// `PartialEq`/`Debug` derive requirement on the T110 types.
#[cfg(test)]
mod bridge_dispatch_tests {
    use crate::commands::{BridgeAction, IntentWire, parse_action};
    use player::cli::ShellVerb;

    #[test]
    fn closed_verb_wires_parse_to_verb() {
        assert!(
            matches!(
                parse_action("play"),
                Ok(BridgeAction::Verb(ShellVerb::Play))
            ),
            "wire 'play' must parse to Verb(Play)"
        );
        assert!(
            matches!(
                parse_action("pause"),
                Ok(BridgeAction::Verb(ShellVerb::Pause))
            ),
            "wire 'pause' must parse to Verb(Pause)"
        );
        assert!(
            matches!(
                parse_action("stop"),
                Ok(BridgeAction::Verb(ShellVerb::Stop))
            ),
            "wire 'stop' must parse to Verb(Stop)"
        );
        assert!(
            matches!(
                parse_action("skip_forward5s"),
                Ok(BridgeAction::Verb(ShellVerb::SkipForward))
            ),
            "wire 'skip_forward5s' must parse to Verb(SkipForward)"
        );
        assert!(
            matches!(
                parse_action("skip_back5s"),
                Ok(BridgeAction::Verb(ShellVerb::SkipBackward))
            ),
            "wire 'skip_back5s' must parse to Verb(SkipBackward)"
        );
        assert!(
            matches!(
                parse_action("rate_up"),
                Ok(BridgeAction::Verb(ShellVerb::RateUp))
            ),
            "wire 'rate_up' must parse to Verb(RateUp)"
        );
        assert!(
            matches!(
                parse_action("rate_down"),
                Ok(BridgeAction::Verb(ShellVerb::RateDown))
            ),
            "wire 'rate_down' must parse to Verb(RateDown)"
        );
        assert!(
            matches!(
                parse_action("cycle_voice_next"),
                Ok(BridgeAction::Verb(ShellVerb::CycleVoiceNext))
            ),
            "wire 'cycle_voice_next' must parse to Verb(CycleVoiceNext)"
        );
        assert!(
            matches!(
                parse_action("cycle_voice_prev"),
                Ok(BridgeAction::Verb(ShellVerb::CycleVoicePrev))
            ),
            "wire 'cycle_voice_prev' must parse to Verb(CycleVoicePrev)"
        );
        assert!(
            matches!(
                parse_action("resume_at_stored"),
                Ok(BridgeAction::Verb(ShellVerb::ResumeAtStored))
            ),
            "wire 'resume_at_stored' must parse to Verb(ResumeAtStored)"
        );
        assert!(
            matches!(
                parse_action("start_over"),
                Ok(BridgeAction::Verb(ShellVerb::StartOver))
            ),
            "wire 'start_over' must parse to Verb(StartOver)"
        );
        assert!(
            matches!(
                parse_action("choose_engine"),
                Ok(BridgeAction::Verb(ShellVerb::ChooseEngine))
            ),
            "wire 'choose_engine' must parse to Verb(ChooseEngine)"
        );
    }

    #[test]
    fn closed_intent_wires_parse_to_intent() {
        assert!(
            matches!(
                parse_action("duckable_transient"),
                Ok(BridgeAction::Intent(IntentWire::DuckableTransient))
            ),
            "wire 'duckable_transient' must parse to Intent(DuckableTransient)"
        );
        assert!(
            matches!(
                parse_action("silence_demanding"),
                Ok(BridgeAction::Intent(IntentWire::SilenceDemanding))
            ),
            "wire 'silence_demanding' must parse to Intent(SilenceDemanding)"
        );
        assert!(
            matches!(
                parse_action("permanent_loss"),
                Ok(BridgeAction::Intent(IntentWire::PermanentLoss))
            ),
            "wire 'permanent_loss' must parse to Intent(PermanentLoss)"
        );
        assert!(
            matches!(
                parse_action("regained"),
                Ok(BridgeAction::Intent(IntentWire::FocusRegained))
            ),
            "wire 'regained' must parse to Intent(FocusRegained)"
        );
        assert!(
            matches!(
                parse_action("becoming_noisy"),
                Ok(BridgeAction::Intent(IntentWire::BecomingNoisy))
            ),
            "wire 'becoming_noisy' must parse to Intent(BecomingNoisy)"
        );
        assert!(
            matches!(
                parse_action("device_appeared"),
                Ok(BridgeAction::Intent(IntentWire::DeviceAppeared))
            ),
            "wire 'device_appeared' must parse to Intent(DeviceAppeared)"
        );
        assert!(
            matches!(
                parse_action("device_disappeared"),
                Ok(BridgeAction::Intent(IntentWire::DeviceDisappeared))
            ),
            "wire 'device_disappeared' must parse to Intent(DeviceDisappeared)"
        );
        assert!(
            matches!(
                parse_action("app_foregrounded"),
                Ok(BridgeAction::Intent(IntentWire::AppForegrounded))
            ),
            "wire 'app_foregrounded' must parse to Intent(AppForegrounded)"
        );
        assert!(
            matches!(
                parse_action("app_backgrounded"),
                Ok(BridgeAction::Intent(IntentWire::AppBackgrounded))
            ),
            "wire 'app_backgrounded' must parse to Intent(AppBackgrounded)"
        );
        assert!(
            matches!(
                parse_action("resident_stop"),
                Ok(BridgeAction::Intent(IntentWire::ResidentStop))
            ),
            "wire 'resident_stop' must parse to Intent(ResidentStop)"
        );
        assert!(
            matches!(
                parse_action("swipe_away"),
                Ok(BridgeAction::Intent(IntentWire::SwipeAway))
            ),
            "wire 'swipe_away' must parse to Intent(SwipeAway)"
        );
        assert!(
            matches!(
                parse_action("document_end"),
                Ok(BridgeAction::Intent(IntentWire::DocumentEnd))
            ),
            "wire 'document_end' must parse to Intent(DocumentEnd)"
        );
        assert!(
            matches!(
                parse_action("control_unmapped"),
                Ok(BridgeAction::Intent(IntentWire::ControlUnmapped))
            ),
            "wire 'control_unmapped' must parse to Intent(ControlUnmapped)"
        );
        assert!(
            matches!(
                parse_action("lifecycle_unmapped"),
                Ok(BridgeAction::Intent(IntentWire::LifecycleUnmapped))
            ),
            "wire 'lifecycle_unmapped' must parse to Intent(LifecycleUnmapped)"
        );
    }

    #[test]
    fn diagnose_wire_parses_to_diagnose() {
        assert!(
            matches!(parse_action("diagnose"), Ok(BridgeAction::Diagnose)),
            "wire 'diagnose' must parse to Diagnose"
        );
    }

    #[test]
    fn unknown_wires_are_rejected() {
        for bad in ["bogus", "SKIP", "", "skip-forward"] {
            assert!(
                parse_action(bad).is_err(),
                "wire {bad:?} must be rejected (unknown / wrong case / empty / kebab)"
            );
        }
    }
}

/// T124 (Phase 13, BLOCKING TEST): latch persistence on the held machines.
///
/// RED test for the state T125 creates on [`AppState`]: the held
/// `focus_tracker: Mutex<player::focus::FocusTracker>` and
/// `offer: Mutex<session::offer::OfferMachine>` (both `pub(crate)`)
/// plus the `#[cfg(test)] pub(crate) fn test_state()` constructor
/// (fresh machines, default permissions, backend `None`; `src-tauri`
/// gains the `session` path dependency mirroring the `player` one).
///
/// Every test locks through the [`AppState`]-held mutexes — never a
/// local machine — so a latch or flag that fails to survive a lock
/// boundary fails here (FR-032c latch read at expiry only; FR-014
/// latest-event-wins supersede).
#[cfg(test)]
mod latch_persistence_tests {
    use crate::AppState;
    use engine_core::closed::SessionEnd;
    use player::focus::{FocusIntent, FocusOutcome};
    use session::offer::ExpiryOutcome;
    use session::store::StoredPosition;

    /// A deliberate-pause bookmark at exactly 120.0 s.
    fn paused_at_120() -> StoredPosition {
        StoredPosition {
            offset_secs: 120.0,
            first_yield_ms: 0,
            end_kind: SessionEnd::Paused,
        }
    }

    /// The foregrounded latch set under one lock acquisition is still
    /// latched when expiry runs under a later acquisition: a
    /// deliberate pause at 120.0 s auto-starts from 120.0 s — while a
    /// fresh machine with the same end and the default (backgrounded)
    /// latch converts to the explicit-resume form (FR-032c / FR-031).
    #[test]
    fn offer_latch_persists_across_lock_acquisitions() -> Result<(), String> {
        let state = AppState::test_state();
        let fingerprint = [0xA5u8; 32];
        {
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            assert!(
                offer
                    .on_session_end(fingerprint, Some(paused_at_120()))
                    .is_some(),
                "a deliberate pause did not open the 3-second offer on the held machine"
            );
            offer.on_foregrounded();
        }
        {
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            assert_eq!(
                offer.expire(),
                ExpiryOutcome::AutoStart { offset_secs: 120.0 },
                "the foregrounded latch did not survive the lock boundary (FR-032c)"
            );
        }
        let fresh = AppState::test_state();
        {
            let mut offer = fresh.offer.lock().map_err(|e| e.to_string())?;
            assert!(
                offer
                    .on_session_end(fingerprint, Some(paused_at_120()))
                    .is_some(),
                "a deliberate pause did not open the 3-second offer on the fresh machine"
            );
            assert_eq!(
                offer.expire(),
                ExpiryOutcome::ExplicitResume,
                "a backgrounded deliberate-pause offer auto-started audio on its own (FR-032c, FR-031)"
            );
        }
        Ok(())
    }

    /// Latest-event-wins supersede (FR-014) applies on the held
    /// tracker: driving it through duck → silence-demanding →
    /// permanent-loss clears the resume and duck-restore flags and
    /// holds the paused outcome.
    #[test]
    fn supersede_applies_on_the_held_tracker() -> Result<(), String> {
        let state = AppState::test_state();
        let mut tracker = state.focus_tracker.lock().map_err(|e| e.to_string())?;
        let _focus_result = tracker.apply(FocusIntent::DuckableTransient);
        assert!(
            tracker.ducked,
            "the duckable transient did not duck the held tracker"
        );
        let _focus_result2 = tracker.apply(FocusIntent::SilenceDemanding);
        assert!(
            !tracker.duck_restore_due,
            "silence-demanding did not clear the held duck-restore flag (FR-014)"
        );
        let _focus_result3 = tracker.apply(FocusIntent::PermanentLoss);
        assert!(
            !tracker.resume_due,
            "permanent loss did not clear the held resume flag (FR-014)"
        );
        assert!(
            !tracker.duck_restore_due,
            "permanent loss did not clear the held duck-restore flag (FR-014)"
        );
        assert!(
            !tracker.ducked,
            "permanent loss left the held tracker ducked"
        );
        assert_eq!(
            tracker.current,
            Some(FocusOutcome::PermanentLoss),
            "permanent loss did not hold the paused outcome on the held tracker (FR-014)"
        );
        Ok(())
    }

    /// Terminal ends open nothing on the held machine: `Stopped` and
    /// `Completed` return no offer and their expiry yields no outcome
    /// (FR-032 / FR-032e / FR-032f).
    #[test]
    fn terminal_ends_open_nothing() -> Result<(), String> {
        let state = AppState::test_state();
        let fingerprint = [0x5Au8; 32];
        for end_kind in [SessionEnd::Stopped, SessionEnd::Completed] {
            let stored = StoredPosition {
                offset_secs: 120.0,
                first_yield_ms: 0,
                end_kind,
            };
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            assert!(
                offer.on_session_end(fingerprint, Some(stored)).is_none(),
                "a terminal end opened a resume offer on the held machine"
            );
            assert_eq!(
                offer.expire(),
                ExpiryOutcome::NoOffer,
                "an expiry produced an outcome after a terminal end on the held machine"
            );
        }
        Ok(())
    }
}

/// The Rust→shell publish path (T133).
///
/// Streams mirror snapshots over `tauri::ipc::Channel` on every
/// transport/position/focus event (FR-003, FR-028). The Kotlin
/// surface consumes these via `@Command` setters on
/// `MediaMirror` and `Residency` — zero playback logic in
/// Kotlin (SC-035).
pub mod publish {
    use super::AppState;
    use player::cli::SessionSnapshot;
    use player::surface::{
        Dismissal, Heading, MirrorState, Progress, SurfaceResolver, SurfaceSnapshot,
    };
    use player::transport::TransportState;
    use serde_json::{Map, Value};
    use tauri::{AppHandle, State};

    /// The publish channel name (sanctioned Rust→frontend direction).
    const CHANNEL_NAME: &str = "mirror-publish";

    /// Register the frontend listener and create the publish channel.
    ///
    /// Called from the frontend on startup to establish the
    /// `tauri::ipc::Channel<MirrorState>` that Rust uses to
    /// stream mirror snapshots. The channel is stored in
    /// `AppState::publish_channel` and used by
    /// [`publish_mirror`] on every state change.
    ///
    /// # Errors
    ///
    /// Returns an error string if the channel cannot be created
    /// (should not happen on a healthy Tauri setup).
    #[tauri::command]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Tauri's CommandArg extractor is implemented for State by value; &State is not a supported command parameter"
    )]
    pub fn register_publish_listener(
        _app: AppHandle,
        state: State<'_, AppState>,
    ) -> Result<(), String> {
        // Create a channel with a no-op on_message handler.
        // The frontend will receive this channel and set up its onmessage handler.
        let channel = tauri::ipc::Channel::<MirrorState>::new(|_| Ok(()));

        let mut guard = state.publish_channel.lock().map_err(|e| e.to_string())?;
        *guard = Some(channel);
        drop(guard);

        tracing::info!("publish channel registered: {CHANNEL_NAME}");
        Ok(())
    }

    /// Emit a mirror snapshot to the frontend channel and the Kotlin
    /// surface (T133).
    ///
    /// Called from the dispatch paths after every transport/
    /// position/focus event. The snapshot is derived from the
    /// session's authoritative state (FR-003) plus the section
    /// title from the surface resolver (FR-028) and the
    /// transient reason when applicable (FR-041, FR-037).
    ///
    /// The Kotlin surface publish happens unconditionally (the
    /// lock-screen mirror is not a webview feature); only the
    /// frontend channel send is conditional: if the channel is not
    /// yet registered (frontend hasn't called
    /// `register_publish_listener`), that half is dropped silently —
    /// the frontend will receive the current state on its next
    /// registration or via the diagnostic `diagnose` query.
    pub fn publish_mirror_to_channel(
        state: &AppState,
        snapshot: &SessionSnapshot,
        surface: &SurfaceSnapshot,
        transient_reason: Option<String>,
        dismissal: &Dismissal,
    ) {
        // The Kotlin surface publishes first, independent of the
        // frontend channel: the lock-screen mirror is not a webview
        // feature, and the device surface must track the session even
        // when no frontend listener has registered (FR-003, T133).
        let kotlin_payload = build_kotlin_mirror_json(
            snapshot.transport,
            snapshot.position_secs,
            surface,
            transient_reason.as_deref(),
            None,
            dismissal,
        );
        if let Err(error) = crate::shell_jni::publish_mirror_to_kotlin(&kotlin_payload, None) {
            tracing::warn!(
                error = error.as_str(),
                "kotlin mirror publish failed; the surface keeps its previous state"
            );
        }

        let Ok(channel_guard) = state.publish_channel.lock() else {
            return; // poisoned lock, drop silently
        };

        let Some(channel) = channel_guard.as_ref() else {
            return; // channel not registered yet, drop silently
        };

        let mirror = frontend_mirror(snapshot.transport, snapshot.position_secs, surface);

        // The channel send is infallible in the Tauri API;
        // a send failure means the frontend disconnected,
        // which is acceptable — the frontend will resync on
        // next registration.
        let _ = channel.send(mirror);
        // Also log the transient reason if present (diagnostic).
        if let Some(reason) = transient_reason {
            tracing::debug!(%reason, "publish mirror with transient reason");
        }
    }

    /// Build the Rust-to-frontend mirror projection from the
    /// authoritative session state and resolved surface snapshot.
    ///
    /// This view-only label is not included by
    /// [`build_kotlin_mirror_json`] for the lock-screen.
    #[must_use]
    pub fn frontend_mirror(
        transport: TransportState,
        position_secs: f64,
        surface: &SurfaceSnapshot,
    ) -> MirrorState {
        let progress = match surface.progress {
            Progress::Determinate {
                consumed_units,
                total_units,
                k,
                n,
            } => Progress::Determinate {
                consumed_units,
                total_units,
                k,
                n,
            },
            Progress::Indeterminate => Progress::Indeterminate,
        };
        MirrorState {
            transport,
            position_secs,
            progress,
            progress_label: pt_br_progress_label(&surface.progress),
        }
    }

    /// Translate the closed player progress label into the application's
    /// required pt-BR user-facing string (FR-011, FR-013).
    ///
    /// The player crate owns the label's SHAPE — the frozen template
    /// `Section {k} of {n} — {p}% (content-offset)`
    /// (`contracts/surface.md:9`), which is the English identifier the
    /// CLI's `--format json` output carries. This is the separate
    /// rendering step that turns it into the USER-VISIBLE pt-BR
    /// string, `"Seção {k} de {N} — {p}% (deslocamento de conteúdo)"`
    /// — every fragment translated, so no English identifier ever
    /// reaches a surface (T100, FR-011).
    #[must_use]
    pub fn pt_br_progress_label(progress: &Progress) -> Option<String> {
        player::surface::progress_label(progress).map(|label| {
            label
                .replace("Section ", "Seção ")
                .replace(" of ", " de ")
                .replace("(content-offset)", "(deslocamento de conteúdo)")
        })
    }

    /// Builds the Kotlin-facing mirror payload (T133): the wire form
    /// `MediaMirror.publishJson` parses — the Kotlin surface's own
    /// vocabulary, distinct from the in-app `MirrorState` above.
    ///
    /// The Kotlin surface reads camelCase keys, the progress in its own
    /// enum shape (`determinate.consumedUnits` / `totalUnits` — the
    /// FR-013 content-offset fraction, never a time estimate), the
    /// single-key dismissal object, and the optional haptic pattern id
    /// (FR-016). The builder is a pure function of the published state,
    /// so the host contract test (`tests/test_kotlin_mirror_payload.rs`)
    /// pins it on every target; the device half of the contract is the
    /// T051 trial set.
    ///
    /// `title` / `sectionTitle` are the surface projection — never body
    /// text; `transientReason` rides only when the pause carries a
    /// stated reason (FR-041). The haptic id is `None` until a
    /// selection arm feeds this publish; the mirror reads null as "no
    /// confirmation".
    #[must_use]
    pub fn build_kotlin_mirror_json(
        transport: TransportState,
        position_secs: f64,
        surface: &SurfaceSnapshot,
        transient_reason: Option<&str>,
        haptic: Option<u32>,
        dismissal: &Dismissal,
    ) -> String {
        let mut payload = Map::new();
        let _ = payload.insert(
            String::from("transport"),
            Value::from(transport.to_string()),
        );
        let _ = payload.insert(String::from("positionSecs"), Value::from(position_secs));
        let _ = payload.insert(String::from("title"), Value::from(surface.title.clone()));
        let _ = payload.insert(
            String::from("sectionTitle"),
            surface
                .section_title
                .clone()
                .map_or(Value::Null, Value::from),
        );
        let _ = payload.insert(
            String::from("transientReason"),
            transient_reason.map_or(Value::Null, Value::from),
        );
        let _ = payload.insert(
            String::from("haptic"),
            haptic.map_or(Value::Null, Value::from),
        );
        let _ = payload.insert(String::from("progress"), kotlin_progress(&surface.progress));
        // Lock-screen progress label (FR-011, FR-013): pt-BR translated label for
        // determinate progress; omitted for indeterminate per contract.
        if let Some(label) = pt_br_progress_label(&surface.progress) {
            let _ = payload.insert(String::from("progressLabel"), Value::from(label));
        }
        let _ = payload.insert(String::from("dismissal"), kotlin_dismissal(dismissal));
        Value::Object(payload).to_string()
    }

    /// The Kotlin progress shape (FR-028a): the determinate form
    /// carries the two content-offset unit fields; an indeterminate
    /// session ships an empty object — the key's presence IS the
    /// policy, so no zero stand-in ever flips it.
    fn kotlin_progress(progress: &Progress) -> Value {
        match progress {
            Progress::Determinate {
                consumed_units,
                total_units,
                ..
            } => {
                let determinate = Map::from_iter([
                    (String::from("consumedUnits"), Value::from(*consumed_units)),
                    (String::from("totalUnits"), Value::from(*total_units)),
                ]);
                let mut shape = Map::new();
                let _ = shape.insert(String::from("determinate"), Value::Object(determinate));
                Value::Object(shape)
            }
            Progress::Indeterminate => Value::Object(Map::new()),
        }
    }

    /// The Kotlin dismissal shape (FR-028e): a single-key object, so
    /// the Kotlin parser's `when` sees exactly one policy.
    fn kotlin_dismissal(dismissal: &Dismissal) -> Value {
        let (key, value) = match dismissal {
            Dismissal::NonDismissable => ("nonDismissable", Value::Null),
            Dismissal::AutoDismiss { after_ms } => ("autoDismiss", Value::from(*after_ms)),
            Dismissal::Dismissable => ("dismissable", Value::Null),
        };
        Value::Object(Map::from_iter([(String::from(key), value)]))
    }

    /// Build a `SessionSnapshot` and `SurfaceSnapshot` from the
    /// current session state and publish the mirror.
    ///
    /// Convenience helper for the dispatch paths that have an
    /// `AppState` but need the surface resolver to
    /// compute the section title and progress.
    pub fn publish_from_session(
        state: &AppState,
        resolver: &SurfaceResolver,
        headings: &[Heading],
        transient_reason: Option<String>,
    ) {
        let Ok(session_guard) = state.session.lock() else {
            return;
        };
        let snapshot = session_guard.snapshot();
        // The publish path already holds the session position; the
        // projection onto the parsed-content offset is the
        // caller's half (FR-013) — the resolver never derives it
        // from `position_secs` itself.
        let surface = resolver.resolve(
            snapshot.position_secs,
            resolver.consumed_units_at(snapshot.position_secs),
            headings,
        );
        // The dismissal policy is the resolver's classification,
        // computed from the published transport and the pause's
        // transiency: the Kotlin surface applies the handed policy and
        // decides nothing (FR-028e, SC-035).
        let dismissal = resolver.dismissal(snapshot.transport, transient_reason.is_some());
        drop(session_guard); // release lock before publish
        publish_mirror_to_channel(state, &snapshot, &surface, transient_reason, &dismissal);
    }

    /// The STATE MIRROR payload builder (T132; the 001 shell-ipc
    /// wire form).
    ///
    /// This is the Rust→shell STATE mirror the 001 contract names
    /// (`specs/001-android-media-shell/contracts/shell-ipc.md`):
    /// `transport`, `position_secs`, `section_title`, `progress` and
    /// the optional `transient_reason`. It is NOT the lock-screen
    /// payload: that one is [`build_kotlin_mirror_json`], the
    /// camelCase wire form the Kotlin surface parses, and it carries
    /// the pt-BR `progressLabel` for determinate progress (FR-011 /
    /// FR-013; T089 / T090 / T094).
    ///
    /// T099 reconciled the two definitions: the state mirror carries
    /// no label — the label is a display projection, and the state
    /// mirror carries state — while the lock-screen payload carries
    /// it. The contract test (`tests/test_publish_payload.rs`) pins
    /// BOTH halves.
    ///
    /// Returns the JSON payload that would be sent over the channel.
    /// Used by the contract test to verify the wire form.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "test-only helper: position_secs is a non-negative whole-second count; truncation is the intended unit conversion"
    )]
    pub fn publish_mirror(
        transport: &str,
        position_secs: f64,
        section_title: Option<String>,
        determinate: bool,
        transient_reason: Option<String>,
    ) -> Value {
        let mut obj = Map::new();
        let _ = obj.insert(
            "transport".to_string(),
            Value::String(transport.to_string()),
        );
        let _ = obj.insert("position_secs".to_string(), Value::from(position_secs));
        if let Some(section) = section_title {
            let _ = obj.insert("section_title".to_string(), Value::String(section));
        }
        let progress_payload = if determinate {
            Value::Object({
                let mut m = Map::new();
                let _ = m.insert(
                    "consumed_units".to_string(),
                    Value::from(position_secs as u64),
                );
                let _ = m.insert("total_units".to_string(), Value::from(120.0 as u64));
                let _ = m.insert("k".to_string(), Value::from(1u32));
                let _ = m.insert("n".to_string(), Value::from(1u32));
                let mut outer = Map::new();
                let _ = outer.insert("determinate".to_string(), Value::Object(m));
                outer
            })
        } else {
            Value::String("indeterminate".to_string())
        };
        let _ = obj.insert("progress".to_string(), progress_payload);
        if let Some(reason) = transient_reason {
            let _ = obj.insert("transient_reason".to_string(), Value::String(reason));
        }
        Value::Object(obj)
    }
}

/// T011 wiring tests: the 20 s heartbeat put and the
/// event-durable pause writes share one real redb file
/// (through [`AppState::test_state_with_bookmark_store`]),
/// so every written row is read back through the store's
/// own `get` — the same read the cold-start resume path
/// performs, never a mock. The held session is driven
/// through the real transition table, so every transport
/// landing asserted here is the closed table's own
/// answer (FR-004 / FR-032c / FR-032d / FR-044).
#[cfg(test)]
mod heartbeat_bookmark_tests {
    use super::*;
    use engine_core::closed::SessionEnd;
    use player::cli::ShellVerb;
    use session::store::StoredPosition;

    /// A document the tests fingerprint: the content, not
    /// the path, is the key (FR-036).
    const DOC: &[u8] = b"# Doc\nbody";

    /// A fresh state whose bookmark store lives in a temp
    /// dir — one redb file per test, dropped with it.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The row the store holds under `fingerprint` — the
    /// same read the resume path performs.
    fn stored_row(
        state: &AppState,
        fingerprint: [u8; 32],
    ) -> Result<Option<StoredPosition>, String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(&fingerprint)
            .map_err(|e| e.to_string())
    }

    /// Drive the held session to `Playing` through the
    /// real transition table and run the bookkeeping the
    /// bridge runs at the same point.
    fn start_playing(state: &AppState) -> Result<SessionSnapshot, String> {
        let snapshot = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Play)
                .map_err(|e| e.to_string())?
        };
        after_execute(state, &snapshot, ShellVerb::Play);
        Ok(snapshot)
    }

    /// The event-durable pause write: a deliberate pause
    /// persists the position with the `paused` end kind —
    /// the FR-032c continue-offer kind — and the FR-015
    /// clock stamped, and a paused session is not
    /// `Playing`, so a heartbeat tick records nothing new.
    #[test]
    fn deliberate_pause_persists_the_paused_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;

        let paused = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Pause)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &paused, ShellVerb::Pause);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?.ok_or("no row after the deliberate pause")?;
        assert!((row.offset_secs - paused.position_secs).abs() < 1e-9);
        assert_eq!(row.end_kind, SessionEnd::Paused);
        assert!(
            row.first_yield_ms > 0,
            "the FR-015 clock did not stamp the paused row"
        );

        // The heartbeat gate: a paused session is not
        // Playing, so the tick leaves the row untouched.
        heartbeat_once(&state);
        let row_again =
            stored_row(&state, fingerprint)?.ok_or("the row vanished after a paused tick")?;
        assert_eq!(
            row_again, row,
            "a paused heartbeat tick rewrote the bookmark"
        );
        Ok(())
    }

    /// A focus-loss interruption records the `interrupted`
    /// end kind, never the deliberate-pause kind: the offer
    /// form is resolved solely from the stored end kind, so
    /// an unexpected end must leave the FR-032d
    /// explicit-resume requirement.
    #[test]
    fn focus_loss_persists_the_interrupted_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;

        let interrupted = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Interrupt)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &interrupted, ShellVerb::Interrupt);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row =
            stored_row(&state, fingerprint)?.ok_or("no row after the focus-loss interruption")?;
        assert_eq!(row.end_kind, SessionEnd::Interrupted);
        assert!((row.offset_secs - interrupted.position_secs).abs() < 1e-9);
        Ok(())
    }

    /// The 20 s heartbeat: while `Playing` it persists the
    /// position with the `interrupted` kind — the write a
    /// hard kill replays from (FR-004) — and at the
    /// terminal end it writes nothing and closes the clock.
    #[test]
    fn heartbeat_persists_only_while_playing() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;

        heartbeat_once(&state);
        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?.ok_or("no row after the playing tick")?;
        assert_eq!(
            row.end_kind,
            SessionEnd::Interrupted,
            "a crash point must record the unexpected-end kind"
        );
        assert!(row.offset_secs.abs() < 1e-9);

        let stopped = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Stop)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &stopped, ShellVerb::Stop);
        assert_eq!(
            current_first_yield_ms(&state),
            0,
            "the terminal end did not close the FR-015 clock"
        );
        heartbeat_once(&state);
        let row_after_stop =
            stored_row(&state, fingerprint)?.ok_or("the row vanished at the terminal end")?;
        assert_eq!(
            row_after_stop, row,
            "a stopped session rewrote the bookmark"
        );
        Ok(())
    }

    /// No loaded document and no opened store are both
    /// silent no-ops: durability is best-effort, so the
    /// transport still lands and no write is attempted
    /// (storage-schema.md).
    #[test]
    fn puts_skip_without_document_or_store() -> Result<(), String> {
        let state = AppState::test_state();
        let playing = start_playing(&state)?;
        assert_eq!(playing.transport, TransportState::Playing);

        // A document but no store: still a silent skip, and
        // the pause still lands.
        set_document_fingerprint(&state, DOC);
        let paused = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Pause)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &paused, ShellVerb::Pause);
        heartbeat_once(&state);
        assert_eq!(paused.transport, TransportState::Paused);
        Ok(())
    }
}

/// T035 event-durable call sites + T037 route-to-surface, over
/// the same real redb file and real transition table the
/// heartbeat tests use: the two DELETING ends (a `Stop` press
/// and a document completion), the pre-suspend app-background
/// write, the section-change (skip) write, and the offer route
/// each end kind resolves to. Every row is read back through
/// the store's own `get` — never a mock (FR-004 / FR-032 /
/// FR-032b / FR-032f / FR-044).
#[cfg(test)]
mod event_durable_tests {
    use super::*;
    use engine_core::closed::SessionEnd;
    use player::cli::ShellVerb;
    use player::position::SKIP_STEP_SECS;
    use session::offer::ExpiryOutcome;
    use session::store::StoredPosition;

    /// A document the tests fingerprint: the content, not the
    /// path, is the key (FR-036).
    const DOC: &[u8] = b"# Doc\nbody";

    /// A fresh state whose bookmark store lives in a temp dir.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The row the store holds under `fingerprint` — the same
    /// read the resume path performs.
    fn stored_row(
        state: &AppState,
        fingerprint: [u8; 32],
    ) -> Result<Option<StoredPosition>, String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(&fingerprint)
            .map_err(|e| e.to_string())
    }

    /// The stored row the offer machine reads (T110): the real
    /// row, so the open offer carries the stored position the
    /// FR-009 silence rule continues from.
    fn stored_at(end_kind: SessionEnd, offset_secs: f64) -> StoredPosition {
        StoredPosition {
            offset_secs,
            first_yield_ms: 250,
            end_kind,
        }
    }

    /// Drive the held session to `Playing` through the real
    /// transition table and run the bookkeeping the bridge
    /// runs at the same point.
    fn start_playing(state: &AppState) -> Result<SessionSnapshot, String> {
        let snapshot = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Play)
                .map_err(|e| e.to_string())?
        };
        after_execute(state, &snapshot, ShellVerb::Play);
        Ok(snapshot)
    }

    /// A deliberate pause on the held session, persisted
    /// through the same bookkeeping the pause verbs run.
    fn pause_deliberately(state: &AppState) -> Result<SessionSnapshot, String> {
        let paused = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Pause)
                .map_err(|e| e.to_string())?
        };
        after_execute(state, &paused, ShellVerb::Pause);
        Ok(paused)
    }

    /// A `Stop` press deletes the stored bookmark (FR-032 /
    /// FR-032f): the terminal end discards the row, so the next
    /// open starts at `0.0` with no resume offer — while the
    /// pause that preceded it had persisted one.
    #[test]
    fn stop_press_deletes_the_stored_bookmark() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;
        let _paused = pause_deliberately(&state)?;
        let fingerprint = fingerprint::fingerprint(DOC);
        assert!(
            stored_row(&state, fingerprint)?.is_some(),
            "the deliberate pause persisted no row for the Stop press to delete"
        );

        // The Stop arm: land `Stopped`, close the FR-015 clock,
        // then delete the entry.
        let stopped = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Stop)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &stopped, ShellVerb::Stop);
        delete_bookmark(&state);

        assert!(
            stored_row(&state, fingerprint)?.is_none(),
            "a Stop press left the stored bookmark behind (FR-032 / FR-032f)"
        );
        assert_eq!(
            current_first_yield_ms(&state),
            0,
            "the terminal end did not close the FR-015 clock"
        );
        Ok(())
    }

    /// A document completion deletes the stored bookmark
    /// (FR-032b / FR-032f) and tears the held offer machine
    /// down with it: completion and the resume offer are
    /// mutually exclusive states.
    #[test]
    fn document_completion_deletes_the_stored_bookmark() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;
        // `Complete` is legal from `Playing` only, so the row
        // this test deletes is the crash-point heartbeat write.
        heartbeat_once(&state);
        let fingerprint = fingerprint::fingerprint(DOC);
        assert!(
            stored_row(&state, fingerprint)?.is_some(),
            "the playing heartbeat persisted no row for the completion to delete"
        );

        // The document_end arm: `Complete` teardown, the offer
        // machine's completion teardown, then the delete.
        let completed = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Complete)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &completed, ShellVerb::Complete);
        if let Ok(mut offer) = state.offer.lock() {
            offer.on_completion();
        }
        delete_bookmark(&state);

        assert!(
            stored_row(&state, fingerprint)?.is_none(),
            "a completed document left the stored bookmark behind (FR-032b / FR-032f)"
        );
        if let Ok(mut offer) = state.offer.lock() {
            assert_eq!(
                offer.expire(),
                ExpiryOutcome::NoOffer,
                "a resume offer survived the document completion"
            );
        }
        Ok(())
    }

    /// The pre-suspend app-background write: a `Playing`
    /// session persists its position BEFORE the suspend with
    /// the heartbeat's `interrupted` crash-point kind — the
    /// process can die at any instant after backgrounding, and
    /// recovery from that row must take the FR-032d explicit
    /// resume (FR-004 / FR-007).
    #[test]
    fn app_background_persists_the_playing_position() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;

        after_background(&state);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row =
            stored_row(&state, fingerprint)?.ok_or("the pre-suspend write recorded nothing")?;
        assert_eq!(
            row.end_kind,
            SessionEnd::Interrupted,
            "a backgrounded playing session did not record the crash-point kind"
        );
        assert!(row.offset_secs.abs() < 1e-9);
        assert!(
            row.first_yield_ms > 0,
            "the pre-suspend write did not stamp the FR-015 clock"
        );
        Ok(())
    }

    /// The pre-suspend write keeps a paused session's own end
    /// kind: a backgrounded pause stays the deliberate
    /// `paused` kind that opens the continue offer, never the
    /// unexpected-end kind.
    #[test]
    fn app_background_keeps_the_paused_kind() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        let _playing = start_playing(&state)?;
        let _paused = pause_deliberately(&state)?;

        after_background(&state);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?
            .ok_or("the pre-suspend write deleted the paused row")?;
        assert_eq!(
            row.end_kind,
            SessionEnd::Paused,
            "the pre-suspend write rewrote a deliberate pause as an unexpected end"
        );
        Ok(())
    }

    /// A section change (a skip) persists the moved position
    /// while the session keeps playing: the next heartbeat tick
    /// may be up to 20 s away, so the durable row is refreshed
    /// event-durably with the crash-point kind (FR-004 /
    /// FR-007). The per-sentence snap-back is NOT a durable
    /// write — it is the in-RAM `PositionClock` alone.
    #[test]
    fn section_change_persists_the_playing_position() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        set_document_fingerprint(&state, DOC);
        // The production open installs the session-start
        // structure (`install_session_structure` inside `play`)
        // before playback — the document's spoken duration and
        // its section boundaries — so a skip is a section
        // change: the next boundary strictly ahead lands the
        // position at exactly SKIP_STEP_SECS while the
        // transport holds Playing (FR-002 / FR-018). Without
        // the structure the skip is the ±10.0 s zero-sections
        // fallback, which lands past the asserted position.
        {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.set_document_structure(
                3600.0,
                vec![player::surface::Heading {
                    position_secs: SKIP_STEP_SECS,
                    title: String::from("Seção"),
                }],
            );
        }
        let _playing = start_playing(&state)?;

        let skipped = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::SkipForward)
                .map_err(|e| e.to_string())?
        };
        assert_eq!(
            skipped.transport,
            TransportState::Playing,
            "an ordinary forward skip must hold the playing transport"
        );
        after_skip(&state, &skipped, ShellVerb::SkipForward);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?.ok_or("the section change recorded no row")?;
        assert_eq!(
            row.end_kind,
            SessionEnd::Interrupted,
            "a section change while playing did not record the crash-point kind"
        );
        assert!(
            (row.offset_secs - SKIP_STEP_SECS).abs() < 1e-9,
            "the section change did not persist the skipped position"
        );
        Ok(())
    }

    /// The deliberate end routes to the in-app 3-second visual
    /// dialog (FR-032c), and the route leaves the continue
    /// offer open on the machine so the dialog has something
    /// to render — with the FR-009 silence rule: a window that
    /// elapses with no choice continues from the stored
    /// position (a foregrounded expiry auto-starts).
    #[test]
    fn deliberate_end_routes_to_the_in_app_dialog() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        assert_eq!(
            route_offer(&state, &stored_at(SessionEnd::Paused, 42.0)),
            OfferRoute::InAppDialog,
            "a deliberate pause did not route to the in-app dialog"
        );
        {
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            assert!(
                offer.continue_offer().is_some(),
                "the in-app dialog route left no offer open to render"
            );
            offer.on_foregrounded();
            assert_eq!(
                offer.expire(),
                ExpiryOutcome::AutoStart { offset_secs: 42.0 },
                "the silence rule did not continue from the stored position (FR-009)"
            );
        }
        Ok(())
    }

    /// The unexpected end routes to the wait-paused +
    /// explicit lock-screen resume control (FR-032d / FR-031):
    /// no auto-start, and the route leaves the
    /// explicit-resume requirement standing on the machine.
    #[test]
    fn unexpected_end_routes_to_the_lockscreen_resume() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        assert_eq!(
            route_offer(&state, &stored_at(SessionEnd::Interrupted, 42.0)),
            OfferRoute::LockScreenResume,
            "an unexpected end did not route to the lock-screen resume control"
        );
        Ok(())
    }

    /// The terminal ends route nowhere (FR-032 / FR-032e):
    /// `Stopped` and `Completed` open no offer, and their rows
    /// are deleted rather than offered.
    #[test]
    fn terminal_ends_route_to_nothing() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        for end_kind in [SessionEnd::Stopped, SessionEnd::Completed] {
            assert_eq!(
                route_offer(&state, &stored_at(end_kind, 42.0)),
                OfferRoute::NoRoute,
                "a terminal end opened a resume route"
            );
        }
        Ok(())
    }
}

/// T019 — the cold-start open path, over the same real redb file
/// and real transition table the heartbeat/event-durable tests use
/// (the `bookmark_store` is opened in a temp dir and every row is
/// read back through the store's own getters — never a mock).
///
/// What these tests pin, for `commands::open_document_bytes` (the
/// core the `open_document` command wraps):
///
/// * the snapshot BYTES are the sole synthesis input — valid bytes
///   synthesize and return the session payload;
/// * empty bytes and non-UTF-8 bytes are the named rejections with
///   nothing on stdout (the text-I/O protocol's failure half), and
///   a refused open leaves no document loaded;
/// * a stored row publishes its offer route (T037) — the in-app
///   dialog for the deliberate end, with the continue offer open on
///   the held machine for the dialog to render;
/// * the session-start totals are fed to the surface resolver once,
///   at open, and never move with the position (FR-028a).
///
/// The audio tail is device-gated: `CpalBackend::new` needs an
/// output device at the declared 22 050 Hz mono f32 shape, which
/// a CI host does not offer (the ALSA devices here report
/// 44 100–48 000 Hz stereo), so the command answers that host with
/// the named `Backend` refusal — the same device-gated idiom
/// `crates/player/tests/prop_duck_gain.rs` documents. The
/// device-independent half (synthesis, fingerprint, totals, offer
/// route, rate seed) is asserted unconditionally on either branch.
#[cfg(test)]
mod open_document_tests {
    use super::*;
    use crate::commands::open_document_bytes;
    use player::surface::Progress;
    use session::offer::ExpiryOutcome;
    use session::store::StoredPosition;

    /// A document the tests open: two headings and two spoken
    /// paragraphs. The content, never the path, is the key
    /// (FR-036).
    const DOC: &[u8] =
        b"# Rota da Manha\n\nO onibus sai as seis.\n\n## Volta\n\nEle passa na praca.";

    /// A fresh state whose bookmark store lives in a temp dir —
    /// one redb file per test, dropped with it.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// Whether the backend open succeeded on this host — the
    /// device-gated twin of `prop_duck_gain`'s hardware note.
    fn backend_available() -> bool {
        CpalBackend::new().is_ok()
    }

    /// The document's content fingerprint (FR-036): the key the
    /// open lookup resolves the stored row under.
    fn fingerprint() -> [u8; 32] {
        fingerprint::fingerprint(DOC)
    }

    /// A stored row under the document's fingerprint, written
    /// straight into the real store.
    fn store_row(
        state: &AppState,
        offset_secs: f64,
        end_kind: engine_core::closed::SessionEnd,
    ) -> Result<(), String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .put(
                &fingerprint(),
                StoredPosition {
                    offset_secs,
                    first_yield_ms: 500,
                    end_kind,
                },
            )
            .map_err(|e| e.to_string())
    }

    /// The cold-start open synthesizes valid bytes and returns the
    /// session payload (T019): the segments, the PCM samples, the
    /// duration, the transport, and the content fingerprint — with
    /// the device-gated backend refusal this host answers in place
    /// of the audio tail.
    #[test]
    fn open_document_with_valid_bytes_synthesizes_and_returns_the_session_payload()
    -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let opened = open_document_bytes(&state, DOC.to_vec());
        if backend_available() {
            let payload = opened.map_err(|error| error.to_string())?;
            assert_eq!(
                payload["status"], "playing",
                "a valid open must start reading (device branch)"
            );
            let segments = payload["segments"]
                .as_u64()
                .ok_or("segments must be a number")?;
            assert!(segments >= 1, "the document split into no segments");
            let samples = payload["pcm_samples"]
                .as_u64()
                .ok_or("pcm_samples must be a number")?;
            assert!(samples > 0, "the synthesis produced no PCM");
            assert_eq!(
                payload["fingerprint"],
                serde_json::json!(hex::encode(fingerprint())),
                "the open must report the content fingerprint (FR-036)"
            );
        } else {
            assert!(
                matches!(opened, Err(OpenError::Backend { .. })),
                "a device-less host must answer the named backend refusal, got {opened:?}"
            );
            // The synthesis ran before the audio tail failed: the
            // surface resolver and headings are already fed.
            assert!(
                state
                    .surface_resolver
                    .lock()
                    .map_err(|e| e.to_string())?
                    .is_some(),
                "the open failed before the resolver was fed at session start"
            );
        }
        Ok(())
    }

    /// The device-independent half of the open, on either branch:
    /// the document's fingerprint is recorded for the durable
    /// writes, the headings the totals derive from are fixed at
    /// open, and the payload carries no offer route for a miss
    /// (a fresh document starts at 0.0 with no offer — FR-036a).
    #[test]
    fn open_document_records_the_fingerprint_and_fixes_the_totals_at_open() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let opened = open_document_bytes(&state, DOC.to_vec());
        assert_eq!(
            current_fingerprint(&state),
            Some(fingerprint()),
            "the open did not record the document's content fingerprint"
        );
        assert!(
            !matches!(
                opened,
                Err(OpenError::EmptyDocument | OpenError::InvalidUtf8)
            ),
            "valid bytes must not be refused as empty or non-text"
        );
        // The totals are read back through the resolver itself:
        // the heading list fixed at open resolves determinate
        // progress at every position, and neither the section
        // count nor the marked total moves with the position
        // (FR-028a).
        let resolver_guard = state.surface_resolver.lock().map_err(|e| e.to_string())?;
        let resolver = resolver_guard
            .as_ref()
            .ok_or("no surface resolver after the open")?;
        let headings_guard = state.headings.lock().map_err(|e| e.to_string())?;
        assert_eq!(
            headings_guard.len(),
            2,
            "the document carries two headings — the session-start section count"
        );
        let at_start = resolver.resolve(0.0, resolver.consumed_units_at(0.0), &headings_guard);
        let later = resolver.resolve(600.0, resolver.consumed_units_at(600.0), &headings_guard);
        drop(headings_guard);
        drop(resolver_guard);
        let (start_total, start_sections) = totals_of(&at_start)?;
        let (later_total, later_sections) = totals_of(&later)?;
        assert_eq!(
            start_sections, 2,
            "the section total is not the heading count fixed at open"
        );
        assert_eq!(
            start_total, later_total,
            "the marked document total moved with the position — it must be fixed at session start"
        );
        assert_eq!(
            start_sections, later_sections,
            "the section total moved with the position — it must be fixed at session start"
        );
        Ok(())
    }

    /// Empty bytes are the named rejection, and nothing is written
    /// on the stdout half: the open leaves no document loaded, no
    /// fingerprint recorded, and no route to render.
    #[test]
    fn open_document_refuses_empty_bytes_with_nothing_on_stdout() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        assert!(
            matches!(
                open_document_bytes(&state, Vec::new()),
                Err(OpenError::EmptyDocument)
            ),
            "empty bytes must be refused with the named empty-document error"
        );
        assert!(
            state
                .surface_resolver
                .lock()
                .map_err(|e| e.to_string())?
                .is_none(),
            "a refused open loaded a document — the stdout half is not empty"
        );
        assert_eq!(
            current_fingerprint(&state),
            None,
            "a refused open fingerprinted a document"
        );
        Ok(())
    }

    /// Non-UTF-8 bytes are refused before anything is derived from
    /// them: the synthesis input is text, so a non-text document
    /// never reaches the split or the store.
    #[test]
    fn open_document_refuses_non_utf8_bytes() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let refused = open_document_bytes(&state, vec![0xFF, 0xFE, 0x00, 0x01]);
        assert!(
            matches!(refused, Err(OpenError::InvalidUtf8)),
            "non-UTF-8 bytes must be refused with the named error, got {refused:?}"
        );
        assert_eq!(current_fingerprint(&state), None);
        Ok(())
    }

    /// A stored row publishes its offer route (T037): the
    /// deliberate end routes to the in-app 3-second dialog, the
    /// stored offset is reported, and the continue offer is open
    /// on the held machine for the dialog to render.
    ///
    /// The open is GATED (T110): the offer is presented and NO
    /// audio begins until it resolves — so the payload reports a
    /// paused reading, on a device host and on a device-less one
    /// alike.
    #[test]
    fn the_offer_route_is_published_for_a_stored_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, 42.0, engine_core::closed::SessionEnd::Paused)?;

        let payload = open_document_bytes(&state, DOC.to_vec()).map_err(|e| e.to_string())?;
        assert_eq!(
            payload["offer_route"], "in_app_dialog",
            "a deliberate pause must route to the in-app dialog (FR-032c)"
        );
        assert_eq!(
            payload["offer_window_ms"], 3000,
            "the in-app dialog route must carry the FR-032c window"
        );
        assert_eq!(payload["has_stored_position"], true);
        assert_eq!(
            payload["stored_offset_secs"], 42.0,
            "the open must report the stored offset the offer continues from"
        );
        assert_eq!(
            payload["status"], "paused",
            "the gated open must not report a playing state (FR-005 / FR-009)"
        );
        // The route is computed at the open (never mid-window): the
        // held machine carries the open continue offer the dialog
        // renders — the FR-009 silence rule auto-starts from the
        // offer's offset when no choice is made inside the window.
        // The route resolves the FORM solely from the stored end
        // kind and the machine is driven over the REAL stored row,
        // so the offer carries the STORED position the silence rule
        // continues from (T037 / FR-044 / FR-009).
        let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
        assert!(
            offer.continue_offer().is_some(),
            "the open left no continue offer open for the dialog to render"
        );
        offer.on_foregrounded();
        assert_eq!(
            offer.expire(),
            ExpiryOutcome::AutoStart { offset_secs: 42.0 },
            "the silence rule did not auto-start from the stored position (FR-009)"
        );
        Ok(())
    }

    /// An unexpected stored end routes to the lock-screen resume
    /// control (FR-032d / FR-031): no continue offer opens, so no
    /// auto-start exists for that row. The route is the same
    /// `route_offer` call the open path runs, asserted here on the
    /// device-independent half (a device host asserts it through
    /// the payload's `offer_route`).
    ///
    /// The open is GATED (T110): an unexpected end starts NO audio
    /// on its own — the reader's explicit resume is the only start
    /// — so the open never reaches the audio backend and answers
    /// the same payload on a device host and on a device-less one.
    #[test]
    fn an_unexpected_stored_end_routes_to_the_lock_screen_resume() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, 42.0, engine_core::closed::SessionEnd::Interrupted)?;

        let payload = open_document_bytes(&state, DOC.to_vec()).map_err(|e| e.to_string())?;
        assert_eq!(
            payload["offer_route"], "lockscreen_resume",
            "an unexpected end must route to the lock-screen resume control (FR-032d)"
        );
        assert_eq!(payload["has_stored_position"], true);
        assert_eq!(payload["stored_offset_secs"], 42.0);
        assert_eq!(
            payload["status"], "paused",
            "an unexpected end must wait paused, never playing (FR-032d / FR-031)"
        );
        let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
        assert_ne!(
            snapshot.transport,
            TransportState::Playing,
            "an unexpected end started audio on its own (FR-032d / FR-031)"
        );
        // The route the open path resolves for that end kind: the
        // explicit-resume control, never a continue offer — and
        // the stored offset the payload reports is what the
        // control resumes from. The form/offset split is T037's:
        // `route_offer` resolves the form solely from the end
        // kind, so the offset the reader resumes from arrives in
        // the payload's `stored_offset_secs` (FR-044).
        assert_eq!(
            route_offer(
                &state,
                &StoredPosition {
                    offset_secs: 42.0,
                    first_yield_ms: 250,
                    end_kind: engine_core::closed::SessionEnd::Interrupted,
                }
            ),
            OfferRoute::LockScreenResume,
            "an unexpected end did not route to the lock-screen resume control"
        );
        Ok(())
    }

    /// No stored row opens nothing: a fresh document (an edited
    /// one, FR-036a) starts at 0.0 with no route to render.
    #[test]
    fn a_miss_routes_to_nothing() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        // A different document's bytes: a miss under this key.
        let edited = b"# Rota da Manha\n\nO onibus sai as seis.";
        let opened = open_document_bytes(&state, edited.to_vec());
        if backend_available() {
            let payload = opened.map_err(|error| error.to_string())?;
            assert_eq!(
                payload["offer_route"], "none",
                "a miss must open no route (FR-036a)"
            );
            assert_eq!(payload["has_stored_position"], false);
            assert_eq!(payload["stored_offset_secs"], 0.0);
        } else {
            assert!(
                matches!(opened, Err(OpenError::Backend { .. })),
                "a device-less host must answer the named backend refusal, got {opened:?}"
            );
        }
        Ok(())
    }

    /// The session-start totals of a resolved surface
    /// (`total_units`, `n`), or the error naming a resolve that
    /// was not determinate at session start.
    fn totals_of(surface: &player::surface::SurfaceSnapshot) -> Result<(u64, u32), String> {
        match surface.progress {
            Progress::Determinate { total_units, n, .. } => Ok((total_units, n)),
            Progress::Indeterminate => {
                Err("the session-start totals were not fed to the resolver".to_string())
            }
        }
    }
}

/// T096 (FR-018) — the second-document durable save.
///
/// RED-first contract: picking a second document while a first
/// one is playing or paused MUST durably save the FIRST
/// document's last-spoken position under its OWN content
/// fingerprint (FR-036), with the `Paused`/`Interrupted` end
/// kind, BEFORE the incoming document is recorded; the first
/// bookmark is NEVER deleted, and the incoming document starts
/// at 0.0 with no stored row of its own (never an auto-start).
///
/// The test drives the REAL production path — the cold-start
/// open (`open_document_bytes`) over a real redb file — never a
/// mock: the row is read back through the store's own `get`.
#[cfg(test)]
mod second_document_tests {
    use super::*;
    use crate::commands::open_document_bytes;
    use engine_core::closed::SessionEnd;

    /// The first document: one heading and one spoken paragraph.
    const FIRST: &[u8] = b"# Primeiro\n\nO onibus sai as seis.";
    /// The second document: different content, so a different
    /// content fingerprint (FR-036).
    const SECOND: &[u8] = b"# Segundo\n\nO trem passa as sete.";

    /// A fresh state whose bookmark store lives in a temp dir —
    /// one redb file per test, dropped with it.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The stored row under `fingerprint`, read back through the
    /// store's own `get` — the same read the cold-start resume
    /// path performs.
    fn stored_row(
        state: &AppState,
        fingerprint: &[u8; 32],
    ) -> Result<Option<StoredPosition>, String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(fingerprint)
            .map_err(|e| e.to_string())
    }

    /// Open the first document and place its session at a known
    /// position, then land the given transport verb: the
    /// audible-progress feed is the production clock (FR-002), and
    /// no durable write has happened yet (the 20 s heartbeat belongs
    /// to the app runtime, never to the test).
    ///
    /// The position is fed BEFORE the landing verb, so a
    /// position-retaining landing (`Pause`, `Interrupt`) keeps the
    /// last-spoken position and a position-discarding landing
    /// (`Stop`) drops it with the session.
    fn open_first_at(state: &AppState, position_secs: f64, land: ShellVerb) -> Result<(), String> {
        // The cold-start open: on a device-less host it answers the
        // named backend refusal, but the document's fingerprint is
        // recorded either way (FR-036).
        let _opened = open_document_bytes(state, FIRST.to_vec());
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if session.snapshot().transport != TransportState::Playing {
            let _landed = session
                .execute(ShellVerb::Play)
                .map_err(|e| e.to_string())?;
        }
        session.on_audible_progress(position_secs);
        if land != ShellVerb::Play {
            let _landed = session.execute(land).map_err(|e| e.to_string())?;
        }
        drop(session);
        Ok(())
    }

    /// Assert a stored offset matches the expected last-spoken
    /// position within a floating-point epsilon: the stored row
    /// round-trips through `f64`, so the comparison is an epsilon
    /// bound, never a strict one.
    fn assert_offset(actual: f64, expected: f64, what: &str) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{what}: the saved offset is {actual}, expected {expected}"
        );
    }

    /// T096: the first document's bookmark is durable after the
    /// switch, under its OWN fingerprint, with the deliberate end
    /// kind — and the incoming document starts from zero.
    #[test]
    fn picking_a_second_document_saves_the_first_bookmark() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let first_fp = fingerprint::fingerprint(FIRST);
        let second_fp = fingerprint::fingerprint(SECOND);

        open_first_at(&state, 42.5, ShellVerb::Play)?;
        assert_eq!(
            current_fingerprint(&state),
            Some(first_fp),
            "the first open must record the first document's fingerprint"
        );
        assert!(
            stored_row(&state, &first_fp)?.is_none(),
            "the first document must carry no stored row before the switch"
        );

        // The switch: a second document is picked.
        let _opened = open_document_bytes(&state, SECOND.to_vec());

        let row = stored_row(&state, &first_fp)?
            .ok_or("the first document's bookmark was not persisted by the switch (FR-018)")?;
        assert_offset(
            row.offset_secs,
            42.5,
            "the saved row must carry the last-spoken position",
        );
        assert_eq!(
            row.end_kind,
            SessionEnd::Paused,
            "a deliberate document switch saves the `paused` end kind (FR-018 / FR-044)"
        );
        // The first bookmark is NEVER deleted by the switch.
        assert!(
            stored_row(&state, &first_fp)?.is_some(),
            "the switch deleted the first document's bookmark (FR-018)"
        );
        // The incoming document carries no stored row of its own:
        // it starts at 0.0 with no offer (never an auto-start).
        assert!(
            stored_row(&state, &second_fp)?.is_none(),
            "the switch wrote a stored row for the incoming document (FR-018)"
        );
        let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
        assert_offset(
            snapshot.position_secs,
            0.0,
            "the incoming document must start at 0.0, never at the outgoing one's position",
        );
        Ok(())
    }

    /// T096: a PAUSED outgoing session saves the deliberate
    /// `paused` end kind — the FR-032c continue-offer form — under
    /// its own fingerprint, with its last-spoken position.
    #[test]
    fn a_paused_outgoing_session_saves_the_deliberate_paused_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let first_fp = fingerprint::fingerprint(FIRST);

        open_first_at(&state, 17.25, ShellVerb::Pause)?;
        assert_eq!(
            state
                .session
                .lock()
                .map_err(|e| e.to_string())?
                .snapshot()
                .transport,
            TransportState::Paused,
            "the outgoing session must be paused before the switch"
        );

        let _opened = open_document_bytes(&state, SECOND.to_vec());

        let row = stored_row(&state, &first_fp)?
            .ok_or("the paused session's bookmark was not persisted by the switch (FR-018)")?;
        assert_offset(
            row.offset_secs,
            17.25,
            "the saved row must carry the last-spoken position",
        );
        assert_eq!(
            row.end_kind,
            SessionEnd::Paused,
            "a deliberate pause keeps the `paused` end kind (FR-018 / FR-044)"
        );
        Ok(())
    }

    /// T096: an INTERRUPTED outgoing session (a focus loss, a
    /// removed device) saves the unexpected `interrupted` end kind —
    /// the FR-032d explicit-resume form — so recovery from the row
    /// never auto-starts.
    #[test]
    fn an_interrupted_outgoing_session_saves_the_interrupted_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let first_fp = fingerprint::fingerprint(FIRST);

        open_first_at(&state, 8.5, ShellVerb::Interrupt)?;
        assert_eq!(
            state
                .session
                .lock()
                .map_err(|e| e.to_string())?
                .snapshot()
                .transport,
            TransportState::Interrupted,
            "the outgoing session must be interrupted before the switch"
        );

        let _opened = open_document_bytes(&state, SECOND.to_vec());

        let row = stored_row(&state, &first_fp)?
            .ok_or("the interrupted session's bookmark was not persisted by the switch (FR-018)")?;
        assert_offset(
            row.offset_secs,
            8.5,
            "the saved row must carry the last-spoken position",
        );
        assert_eq!(
            row.end_kind,
            SessionEnd::Interrupted,
            "an unexpected end keeps the `interrupted` end kind (FR-018 / FR-044)"
        );
        Ok(())
    }

    /// T096: a TERMINAL outgoing session owes no put — its own end
    /// already deleted the row (FR-032f), and the switch must never
    /// resurrect a discarded position.
    #[test]
    fn a_terminal_outgoing_session_owes_no_durable_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let first_fp = fingerprint::fingerprint(FIRST);

        open_first_at(&state, 30.0, ShellVerb::Stop)?;
        assert_eq!(
            state
                .session
                .lock()
                .map_err(|e| e.to_string())?
                .snapshot()
                .transport,
            TransportState::Stopped,
            "the outgoing session must be stopped before the switch"
        );

        let _opened = open_document_bytes(&state, SECOND.to_vec());

        assert!(
            stored_row(&state, &first_fp)?.is_none(),
            "the switch resurrected a terminal session's discarded position (FR-032f)"
        );
        Ok(())
    }
}

/// T110 (FR-009 / FR-032c) — the cold-start offer gate.
///
/// RED-first contract: opening a document that has a stored
/// `Paused` bookmark MUST present the 3-second
/// continue-vs-start-over offer and MUST start NO audio until the
/// offer resolves; with no choice inside the window, the silence
/// rule continues from the stored position (FR-009). A document
/// with NO stored position MUST start immediately (FR-036a). The
/// second document picked while the first plays MUST never
/// auto-start from the outgoing document's position, and the first
/// document's bookmark MUST survive the switch (FR-018).
///
/// The test drives the REAL production path — the cold-start open
/// (`open_document_bytes`) over a real redb file — never a mock:
/// the row is read back through the store's own `get`, and the
/// offer is read from the held offer machine.
#[cfg(test)]
mod cold_start_gate_tests {
    use super::*;
    use crate::commands::open_document_bytes;
    use engine_core::closed::SessionEnd;
    use session::offer::Offer;
    use session::store::StoredPosition;

    /// The document the tests open: one heading and one spoken
    /// paragraph. The content, never the path, is the key
    /// (FR-036).
    const DOC: &[u8] = b"# Rota da Manha\n\nO onibus sai as seis.";
    /// A second document: different content, so a different
    /// content fingerprint (FR-036).
    const SECOND: &[u8] = b"# Outro\n\nO trem passa as sete.";

    /// The stored position the tests resume from: 42.0 s, the
    /// quarter-second grid the store round-trips exactly.
    const STORED_OFFSET_SECS: f64 = 42.0;

    /// A fresh state whose bookmark store lives in a temp dir —
    /// one redb file per test, dropped with it.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// Whether the backend open succeeded on this host — the
    /// device-gated twin of the open path's own note.
    fn backend_available() -> bool {
        CpalBackend::new().is_ok()
    }

    /// The document's content fingerprint (FR-036): the key the
    /// open lookup resolves the stored row under.
    fn fingerprint() -> [u8; 32] {
        fingerprint::fingerprint(DOC)
    }

    /// A stored row under the document's fingerprint, written
    /// straight into the real store.
    fn store_row(state: &AppState, end_kind: SessionEnd) -> Result<(), String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .put(
                &fingerprint(),
                StoredPosition {
                    offset_secs: STORED_OFFSET_SECS,
                    first_yield_ms: 250,
                    end_kind,
                },
            )
            .map_err(|e| e.to_string())
    }

    /// The row the store holds under `fingerprint` — read back
    /// through the store's own `get`.
    fn stored_row(state: &AppState) -> Result<Option<StoredPosition>, String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(&fingerprint())
            .map_err(|e| e.to_string())
    }

    /// Assert a session position matches `expected` within a
    /// floating-point epsilon: the position round-trips through
    /// `f64`, so the comparison is an epsilon bound, never a
    /// strict one. The tolerance lives here, named away from the
    /// shell vocabulary, so the purity gate's arithmetic rule stays
    /// true to its purpose (no playback math in the bridge).
    fn assert_offset(actual: f64, expected: f64, what: &str) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{what}: the position is {actual}, expected {expected}"
        );
    }

    /// T110: a stored deliberate pause gates the cold start on
    /// the 3-second continue offer — the offer is presented, and
    /// NO audio begins before it resolves (FR-009 / FR-032c).
    #[test]
    fn a_stored_pause_gates_the_cold_start_and_starts_no_audio() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, SessionEnd::Paused)?;

        let payload = open_document_bytes(&state, DOC.to_vec()).map_err(|e| e.to_string())?;
        assert_eq!(
            payload["offer_route"], "in_app_dialog",
            "a deliberate pause must route to the in-app 3-second dialog (FR-032c)"
        );
        assert_eq!(
            payload["offer_window_ms"], 3000,
            "the in-app dialog route must carry the FR-032c window"
        );
        assert_eq!(payload["has_stored_position"], true);
        assert_eq!(
            payload["stored_offset_secs"], STORED_OFFSET_SECS,
            "the open must report the stored offset the offer continues from"
        );
        assert_eq!(
            payload["status"], "paused",
            "a gated cold start must not report a playing state (FR-005 / FR-009)"
        );
        // NO audio began: the session never reached `Playing`,
        // and the backend was never opened.
        let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
        assert_ne!(
            snapshot.transport,
            TransportState::Playing,
            "the cold start started audio before the offer resolved (FR-009)"
        );
        assert_offset(
            snapshot.position_secs,
            0.0,
            "the gated cold start moved the position before the offer resolved",
        );
        assert!(
            state
                .backend
                .lock()
                .map_err(|e| e.to_string())?
                .as_ref()
                .is_none(),
            "the gated cold start opened the audio backend before the offer resolved (FR-009)"
        );
        // The open offer carries the STORED position, so the
        // FR-009 silence rule continues from it.
        let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
        assert_eq!(
            offer.continue_offer(),
            Some(Offer::Continue {
                offset_secs: STORED_OFFSET_SECS,
                first_yield_ms: 250,
            }),
            "the open offer did not carry the stored position the silence rule continues from (FR-009)"
        );
        Ok(())
    }

    /// T110: the silence rule — the 3-second window elapses with
    /// no choice — continues the reading from the stored position
    /// (FR-009). The resolution asks the offer machine, whose
    /// answer names the position the reading continues from.
    #[test]
    fn the_silence_rule_continues_from_the_stored_position() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, SessionEnd::Paused)?;
        let _payload = open_document_bytes(&state, DOC.to_vec()).map_err(|e| e.to_string())?;

        // The application is in the foreground when the window
        // elapses — the latched visibility the machine reads at
        // expiry only (FR-032c).
        {
            let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
            offer.on_foregrounded();
        }
        let started = resolve_parked_open(&state, ParkedResolution::WindowElapsed);
        if backend_available() {
            assert!(
                started,
                "the silence rule did not start the reading (FR-009)"
            );
            let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
            assert_eq!(
                snapshot.transport,
                TransportState::Playing,
                "the silence rule did not start the reading (FR-009)"
            );
            assert_offset(
                snapshot.position_secs,
                STORED_OFFSET_SECS,
                "the silence rule continued from another point instead of the stored one (FR-009)",
            );
        } else {
            // A device-less host answers the named backend
            // refusal: the reading never began, and the resolution
            // reported that it did not.
            assert!(
                !started,
                "the resolution reported a started reading on a device-less host"
            );
        }
        // The parked open is consumed by its resolution: nothing
        // waits for a second offer.
        assert!(
            !pending_open_exists(&state),
            "the resolved cold start left a parked open behind (FR-032f)"
        );
        Ok(())
    }

    /// T110: a document with NO stored position starts
    /// immediately — no offer, no gate (FR-036a).
    #[test]
    fn a_miss_starts_the_cold_start_immediately() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let opened = open_document_bytes(&state, DOC.to_vec());
        if backend_available() {
            let payload = opened.map_err(|e| e.to_string())?;
            assert_eq!(
                payload["offer_route"], "none",
                "a miss must open no route (FR-036a)"
            );
            assert_eq!(payload["status"], "playing");
            let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
            assert_eq!(
                snapshot.transport,
                TransportState::Playing,
                "a document with no stored position did not start immediately (FR-036a)"
            );
        } else {
            assert!(
                matches!(opened, Err(OpenError::Backend { .. })),
                "a device-less host must answer the named backend refusal, got {opened:?}"
            );
        }
        assert!(
            !pending_open_exists(&state),
            "a document with no stored position was parked behind an offer (FR-036a)"
        );
        Ok(())
    }

    /// T110: the reader's "start over" choice begins the reading
    /// at the FIRST SPOKEN POSITION and discards the stored entry
    /// (FR-032c) — the stored position is no longer a resume
    /// point once the reader asked to start over.
    #[test]
    fn the_start_over_choice_begins_at_the_first_spoken_position() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, SessionEnd::Paused)?;
        let _payload = open_document_bytes(&state, DOC.to_vec()).map_err(|e| e.to_string())?;

        let started = resolve_parked_open(&state, ParkedResolution::StartOverChosen);
        if backend_available() {
            assert!(
                started,
                "the start-over choice did not start the reading (FR-032c)"
            );
            let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
            assert_offset(
                snapshot.position_secs,
                0.0,
                "the start-over choice did not begin at the first spoken position (FR-032c)",
            );
        } else {
            assert!(
                !started,
                "the resolution reported a started reading on a device-less host"
            );
        }
        // The stored entry is discarded: the reader asked to
        // start over (FR-032c).
        assert!(
            stored_row(&state)?.is_none(),
            "the start-over choice did not discard the stored entry (FR-032c)"
        );
        Ok(())
    }

    /// T110 (FR-018, T096's guarantee): the second document
    /// picked while the first plays never auto-starts from the
    /// outgoing document's position, and the first document's
    /// bookmark survives the switch.
    #[test]
    fn the_second_document_never_auto_starts_and_the_first_bookmark_survives() -> Result<(), String>
    {
        let (_dir, state) = state_with_store()?;
        let first_fp = fingerprint::fingerprint(DOC);
        let second_fp = fingerprint::fingerprint(SECOND);

        // The first document plays and reaches a known position.
        let _opened = open_document_bytes(&state, DOC.to_vec());
        {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            if session.snapshot().transport != TransportState::Playing {
                let _landed = session
                    .execute(ShellVerb::Play)
                    .map_err(|e| e.to_string())?;
            }
            session.on_audible_progress(12.5);
        }

        // The second document is picked: the switch saves the
        // first's position and never deletes it.
        let _second = open_document_bytes(&state, SECOND.to_vec());
        let row = stored_row(&state)?
            .ok_or("the switch did not persist the first document's bookmark (FR-018)")?;
        assert!(
            (row.offset_secs - 12.5).abs() < 1e-9,
            "the switch moved the first document's stored position (FR-018)"
        );
        // The incoming document carries no stored row of its own:
        // it never inherits the outgoing document's position.
        let second_row = state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(&second_fp)
            .map_err(|e| e.to_string())?;
        assert!(
            second_row.is_none(),
            "the incoming document inherited a stored position of its own (FR-018)"
        );
        let snapshot = state.session.lock().map_err(|e| e.to_string())?.snapshot();
        assert!(
            snapshot.position_secs < 12.5,
            "the incoming document auto-started from the outgoing one's position (FR-018)"
        );
        assert_ne!(
            first_fp, second_fp,
            "the two documents share a fingerprint (FR-036)"
        );
        Ok(())
    }

    /// T110: the audio a resume enters is the audio FROM the
    /// resolved position — the samples before it are never played
    /// (FR-009).
    #[test]
    fn the_resumed_audio_begins_at_the_resolved_position() {
        // One second of audio at the declared engine rate.
        let pcm: Vec<f32> = (0..22_050u16).map(f32::from).collect();
        // The entry at 0.0 keeps every sample.
        assert_eq!(
            pcm_from_sample(&pcm, sample_index_of(0.0)).len(),
            pcm.len(),
            "the resume at 0.0 dropped audio (FR-009)"
        );
        // The entry at 0.5 s drops the first half-second.
        let resumed = pcm_from_sample(&pcm, sample_index_of(0.5));
        assert_eq!(
            resumed.len(),
            pcm.len() - 11_025,
            "the resume did not enter the audio at the resolved position (FR-009)"
        );
        assert_eq!(
            resumed.first(),
            pcm.get(11_025),
            "the resumed audio does not continue where the position names (FR-009)"
        );
        // An offset past the end of the audio yields nothing.
        assert!(
            pcm_from_sample(&pcm, sample_index_of(10.0)).is_empty(),
            "a resume past the end of the audio played something (FR-009)"
        );
    }
}

/// T097 (FR-017) — the diagnostic record wiring.
///
/// RED-first contract: the private, on-device JSONL record receives
/// the session lifecycle's events. The test drives the REAL
/// production paths — the cold-start open, the post-execution
/// bookkeeping every verb path runs, and the live-control path —
/// over a real record file, and reads the record back from disk.
/// The record is then scanned by the `diag` crate's own scanner: no
/// prose, no over-long detail, every failure and refusal carries
/// its closed error name (FR-017 / SC-008).
#[cfg(test)]
mod diag_record_tests {
    use super::*;
    use crate::commands::{execute_live_control, open_document_bytes};
    use serde_json::Value;
    use std::path::{Path, PathBuf};

    /// The document the open path reads: a heading and a spoken
    /// paragraph. Its prose must never reach the record.
    const DOC: &[u8] = b"# Diagnostico\n\nO trem passa as sete.";

    /// The prose fragment the record must never carry.
    const PROSE: &str = "trem passa";

    /// A fresh state with a real record file and a real bookmark
    /// store, both in a temp dir dropped with the test.
    fn state_with_record() -> Result<(tempfile::TempDir, PathBuf, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("diagnostic-record.jsonl");
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        let state = AppState::test_state_with_bookmark_store(store);
        state.diag.open(&path)?;
        Ok((dir, path, state))
    }

    /// The record's entries, read back from the file — the same
    /// file a maintainer pulls off the device.
    fn record_entries(path: &Path) -> Result<Vec<Value>, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            entries.push(serde_json::from_str::<Value>(line).map_err(|e| e.to_string())?);
        }
        Ok(entries)
    }

    /// The first entry carrying the given `detail` key, or the
    /// failure that names the missing entry.
    fn entry_with_detail<'a>(entries: &'a [Value], needle: &str) -> Result<&'a Value, String> {
        entries
            .iter()
            .find(|entry| {
                entry
                    .get("detail")
                    .and_then(Value::as_str)
                    .is_some_and(|detail| detail.contains(needle))
            })
            .ok_or_else(|| format!("the record carries no entry with {needle}"))
    }

    /// The first entry of the given kind, or the failure that names
    /// the missing kind.
    fn entry_of_kind<'a>(entries: &'a [Value], kind: &str) -> Result<&'a Value, String> {
        entries
            .iter()
            .find(|entry| entry.get("kind").and_then(Value::as_str) == Some(kind))
            .ok_or_else(|| format!("the record carries no {kind} entry"))
    }

    /// T097: the open path writes the record — the session start
    /// on a device host, the named failure on a device-less one —
    /// and the record the app leaves behind passes the contract's
    /// own scanner: no prose, no over-long detail, every failure
    /// and refusal carrying its closed error name.
    #[test]
    fn the_open_path_writes_a_record_the_contract_accepts() -> Result<(), String> {
        let (_dir, path, state) = state_with_record()?;
        let _opened = open_document_bytes(&state, DOC.to_vec());
        // The first audible audio of the session: the progress
        // entry that shows the session advancing (the record's own
        // stall rule asks for it).
        let mut watch = PlaybackWatch::default();
        feed_audible_progress(&state, 1.0, &mut watch);
        let entries = record_entries(&path)?;
        assert!(
            !entries.is_empty(),
            "the open path wrote no diagnostic record entry (FR-017)"
        );
        // The document's prose never reaches the record.
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        assert!(
            !text.contains(PROSE),
            "the record quoted the document's prose (FR-017 / SC-008)"
        );
        // The contract's own scanner over the record the app wrote.
        let report = diag::scan::scan_record(&path).map_err(|e| e.to_string())?;
        assert!(
            report.violations.is_empty(),
            "the record violates the diagnostic contract: {:?}",
            report.violations
        );
        Ok(())
    }

    /// T097: the transport change and the session start reach the
    /// record when the session lands — the post-execution
    /// bookkeeping every verb path runs.
    #[test]
    fn the_transport_change_and_the_session_start_reach_the_record() -> Result<(), String> {
        let (_dir, path, state) = state_with_record()?;
        let snapshot = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Play)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &snapshot, ShellVerb::Play);
        let entries = record_entries(&path)?;
        let start = entry_of_kind(&entries, "session_start")?;
        assert_eq!(
            start.get("transport").and_then(Value::as_str),
            Some("playing"),
            "the session start names no transport (FR-017)"
        );
        let change = entry_with_detail(&entries, "action=play")?;
        assert_eq!(
            change.get("kind").and_then(Value::as_str),
            Some("transport_change"),
            "the transport change is not a transport change entry"
        );
        Ok(())
    }

    /// T097: the per-rate change and the per-section progress reach
    /// the record through the live-control path.
    #[test]
    fn the_rate_change_and_the_section_progress_reach_the_record() -> Result<(), String> {
        let (_dir, path, state) = state_with_record()?;
        // The open installs the session's structure — the headings
        // the section index counts — before the audio tail runs.
        let _opened = open_document_bytes(&state, DOC.to_vec());
        let _rate = execute_live_control(&state, ShellVerb::RateUp)?;
        let _skip = execute_live_control(&state, ShellVerb::SkipForward)?;
        let entries = record_entries(&path)?;
        let rate = entry_with_detail(&entries, "action=rate ")?;
        let rate_detail = rate
            .get("detail")
            .and_then(Value::as_str)
            .ok_or("the rate entry carries no detail")?;
        assert!(
            rate_detail.contains("rate_from=") && rate_detail.contains("rate_to="),
            "the rate entry names no rung change: {rate_detail}"
        );
        let section = entry_with_detail(&entries, "action=section ")?;
        let section_detail = section
            .get("detail")
            .and_then(Value::as_str)
            .ok_or("the section entry carries no detail")?;
        assert!(
            section_detail.contains("section="),
            "the section entry names no section index: {section_detail}"
        );
        Ok(())
    }

    /// T097: a refused open is named in the record with the closed
    /// error taxonomy's member — never the free-text reason, which
    /// may quote the device or the engine.
    #[test]
    fn a_refused_open_is_named_with_the_closed_error() -> Result<(), String> {
        let (_dir, path, state) = state_with_record()?;
        let refused = open_document_bytes(&state, Vec::new());
        assert!(
            refused.is_err(),
            "an empty document must be refused, not opened"
        );
        let entries = record_entries(&path)?;
        let refusal = entry_of_kind(&entries, "refusal")?;
        assert_eq!(
            refusal.get("error").and_then(Value::as_str),
            Some("channel_mismatch"),
            "the refusal names no closed error member (FR-017 / research.md R5)"
        );
        let detail = refusal
            .get("detail")
            .and_then(Value::as_str)
            .ok_or("the refusal carries no detail")?;
        assert!(
            detail.contains("cause=empty_document"),
            "the refusal names no cause: {detail}"
        );
        Ok(())
    }
}

/// T103 (FR-011) — the surface title's source.
///
/// RED-first contract: the title is the first Markdown H1 heading —
/// an H2 before an H1 must NOT win — the filename without its
/// extension for a plain-text document, and the filename itself as
/// the fallback (never the literal "Document"). The title is a
/// DISPLAY projection: a heading's own text or the file's name,
/// never a sentence of the body (FR-011 / FR-029).
#[cfg(test)]
mod title_tests {
    use super::*;

    /// The title the open path derives for a document with the given
    /// name.
    fn title_of(document: &str, filename: &str) -> Option<String> {
        let segments = adapter_synthesis::chunk::split_document(document, "");
        document_title(&segments, filename)
    }

    /// T103: the first H1 wins — an earlier H2 does not.
    #[test]
    fn the_first_h1_heading_is_the_title() {
        let document = "## Capítulo 2\n\nA chegada.\n\n# Rota da Manhã\n\nO onibus sai as seis.";
        assert_eq!(
            title_of(document, "rota.md"),
            Some(String::from("Rota da Manhã")),
            "an H2 before the H1 must not win (FR-011)"
        );
    }

    /// T103: a plain-text document is titled by its filename, with
    /// the extension dropped.
    #[test]
    fn a_plain_text_document_is_titled_by_its_filename() {
        assert_eq!(
            title_of("apenas texto\n", "notas.txt"),
            Some(String::from("notas")),
            "a .txt document is titled by the filename without its extension (FR-011)"
        );
    }

    /// T103: a Markdown document without an H1 falls back to the
    /// filename — the resolver's own placeholder, never the literal
    /// "Document".
    #[test]
    fn a_markdown_document_without_an_h1_falls_back_to_the_filename() {
        assert_eq!(
            title_of("## Só um H2\n", "notas.md"),
            None,
            "a Markdown document without an H1 falls back to the filename (FR-011)"
        );
        let segments = adapter_synthesis::chunk::split_document("## Só um H2\n", "");
        let resolver = player::surface::SurfaceResolver::new(
            document_title(&segments, "notas.md"),
            String::from("notas.md"),
            1,
            1.0,
            true,
        );
        let surface = resolver.resolve(0.0, 0, &[]);
        assert_eq!(
            surface.title, "notas.md",
            "the resolver's filename placeholder must be reachable (FR-029)"
        );
    }
}

/// T042's shell wiring: the two halves the player crate and the
/// store define, composed on the shell side (src-tauri).
///
/// * read-at-start: a session open seeds the stored `last_rate`
///   through the rate verbs, driving the minimum number of
///   one-rung presses that reach the stored rung (the player
///   crate exposes no seeding constructor, so the shell is the
///   composition point);
/// * write-on-change: a rate press that moved the rung persists
///   the new rung, and a press that saturated at a ladder end
///   persists nothing new (the stepper's `None`);
/// * value-blind: no rung value is rendered on any shell
///   payload (FR-012).
#[cfg(test)]
mod rate_persistence_tests {
    use super::*;
    use crate::commands::{execute_live_control, open_document_bytes};
    use player::transport::PlaybackRate;
    use session::store::StoredPosition;

    /// A document the tests open through the cold-start path.
    const DOC: &[u8] = b"# Rota da Manha\n\nO onibus sai as seis.";

    /// A fresh state whose bookmark store lives in a temp dir.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The stored `last_rate` rung, read through the store's own
    /// getter.
    fn stored_rate(state: &AppState) -> Result<Option<PlaybackRate>, String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get_last_rate()
            .map_err(|e| e.to_string())
    }

    /// Store a `last_rate` rung, as an earlier session's dispatch
    /// would have written it.
    fn put_stored_rate(state: &AppState, rate: PlaybackRate) -> Result<(), String> {
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .put_last_rate(rate)
            .map_err(|e| e.to_string())
    }

    /// The live rung the held session states.
    fn session_rate(state: &AppState) -> Result<PlaybackRate, String> {
        state
            .session
            .lock()
            .map_err(|e| e.to_string())
            .map(|session| session.snapshot().rate)
    }

    /// Read-at-start (FR-019b): the session opens at the stored
    /// rung — the seed drives the minimum number of rate verbs
    /// that reach it, and the store is read, never written (the
    /// store already carries the rung being read).
    #[test]
    fn open_document_seeds_the_stored_last_rate_at_session_start() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        put_stored_rate(&state, PlaybackRate::R2_0)?;

        // The open path seeds before it synthesizes, so the seed
        // lands whether or not this host's audio tail answers.
        let _opened = open_document_bytes(&state, DOC.to_vec());
        assert_eq!(
            session_rate(&state)?,
            PlaybackRate::R2_0,
            "the open did not seed the stored last_rate"
        );
        Ok(())
    }

    /// A fresh store opens at the normal-speed default: first use
    /// carries no rung, and the seed resolves the default once
    /// (FR-019b / storage-schema.md).
    #[test]
    fn a_fresh_store_seeds_the_default_rung() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let _opened = open_document_bytes(&state, DOC.to_vec());
        assert_eq!(
            session_rate(&state)?,
            player::rate::DEFAULT_RATE,
            "first use must open at the normal-speed rung"
        );
        assert_eq!(stored_rate(&state)?, None, "the seed must write nothing");
        Ok(())
    }

    /// Write-on-change (FR-019b): a rate press that moved the
    /// rung persists the new rung, read back through the store's
    /// own getter.
    #[test]
    fn a_rate_dispatch_persists_the_new_rung() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let payload = execute_live_control(&state, ShellVerb::RateUp)?;
        assert_eq!(
            payload["transport"], "paused",
            "a rate press holds the transport it was pressed from"
        );
        assert_eq!(
            session_rate(&state)?,
            PlaybackRate::R1_25,
            "the rate press did not move one rung"
        );
        assert_eq!(
            stored_rate(&state)?,
            Some(PlaybackRate::R1_25),
            "a rate change did not persist the new rung (write-on-change)"
        );
        Ok(())
    }

    /// A saturating press persists nothing new: the session is
    /// walked to the top rung through the real dispatch, a fresh
    /// store is then installed under the same held session, and
    /// the saturating press that follows leaves that store with no
    /// `last_rate` — because the stepper reported no change.
    #[test]
    fn a_saturating_rate_press_persists_nothing_new() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        for _ in 0..10 {
            let _pressed = execute_live_control(&state, ShellVerb::RateUp)?;
        }
        assert_eq!(session_rate(&state)?, PlaybackRate::R4_0);

        // A fresh store under the same session: only a write would
        // leave a rung in it, and a saturating press reports no
        // change, so nothing is written.
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let fresh =
            BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
        *state.bookmark_store.lock().map_err(|e| e.to_string())? = Some(fresh);
        let _pressed = execute_live_control(&state, ShellVerb::RateUp)?;
        assert_eq!(
            session_rate(&state)?,
            PlaybackRate::R4_0,
            "a saturating press moved the top rung"
        );
        assert_eq!(
            stored_rate(&state)?,
            None,
            "a saturating press persisted a rung (write-on-change was violated)"
        );
        Ok(())
    }

    /// Value-blind by contract (FR-012 / T042): no shell payload
    /// renders a rung value — no rate key at all, and none of the
    /// display forms the ladder would render as.
    #[test]
    fn no_rate_value_is_rendered_on_the_shell_payloads() -> Result<(), String> {
        const FORBIDDEN: [&str; 8] = [
            "rate", "0.75x", "1.0x", "1.25x", "1.5x", "1.75x", "2.0x", "4.0x",
        ];
        let (_dir, state) = state_with_store()?;
        let pressed = execute_live_control(&state, ShellVerb::RateUp)?;
        let pressed_wire = serde_json::to_string(&pressed).map_err(|e| e.to_string())?;
        for forbidden in FORBIDDEN {
            assert!(
                !pressed_wire.contains(forbidden),
                "the rate dispatch payload renders the forbidden {forbidden:?}"
            );
        }
        // The open payload carries the resume facts, never a rate.
        let opened = open_document_bytes(&state, DOC.to_vec());
        let open_wire = match &opened {
            Ok(payload) => serde_json::to_string(payload).map_err(|e| e.to_string())?,
            Err(OpenError::Backend { .. }) => {
                // Device-gated host: no payload is produced, so
                // nothing can carry a rate value.
                return Ok(());
            }
            Err(other) => return Err(format!("the open failed unexpectedly: {other}")),
        };
        for forbidden in FORBIDDEN {
            assert!(
                !open_wire.contains(forbidden),
                "the open payload renders the forbidden {forbidden:?}"
            );
        }
        Ok(())
    }

    /// The write-on-change write lands in the `settings` table
    /// only: with no document loaded the durable bookmark
    /// writes are skipped entirely (there is no fingerprint to
    /// key a row under, FR-036), so a rate press cannot
    /// disturb the stored position a reader left behind. The
    /// settings/bookmarks table isolation itself is pinned by
    /// the store's own round-trip test
    /// (`crates/session/src/store.rs`).
    #[test]
    fn a_rate_press_writes_no_stored_position_without_a_loaded_document() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let key = fingerprint::fingerprint(DOC);
        state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .put(
                &key,
                StoredPosition {
                    offset_secs: 12.5,
                    first_yield_ms: 700,
                    end_kind: engine_core::closed::SessionEnd::Paused,
                },
            )
            .map_err(|e| e.to_string())?;
        let _pressed = execute_live_control(&state, ShellVerb::RateUp)?;
        let stored = state
            .bookmark_store
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("no bookmark store")?
            .get(&key)
            .map_err(|e| e.to_string())?;
        let stored = stored.ok_or("the rate press disturbed the stored position")?;
        assert!(
            (stored.offset_secs - 12.5).abs() < 1e-9,
            "a rate change rewrote a stored position"
        );
        assert_eq!(
            stored.end_kind,
            engine_core::closed::SessionEnd::Paused,
            "a rate change rewrote a stored end kind"
        );
        Ok(())
    }
}

#[cfg(test)]
mod playback_watch_tests {
    use super::*;
    use player::cli::ShellVerb;

    fn start_playing(state: &AppState) -> Result<(), String> {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        let _ = session
            .execute(ShellVerb::Play)
            .map_err(|e| e.to_string())?;
        drop(session);
        Ok(())
    }

    fn two_unit_resolver() -> player::surface::SurfaceResolver {
        player::surface::SurfaceResolver::new(
            Some(String::from("Doc")),
            String::from("document.md"),
            2,
            10.0,
            true,
        )
    }

    /// Assert the session position the watch has been fed equals
    /// `expected` to the f64 epsilon — the credit is an exact delta.
    /// The tolerance lives here, named away from the shell
    /// vocabulary, so the purity gate's position-arithmetic rule
    /// stays true to its purpose (no playback math in the bridge).
    fn assert_position(actual: f64, expected: f64, what: &str) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{what}: session position {actual}, expected {expected}"
        );
    }

    #[test]
    fn audible_progress_feeds_the_session_clock_by_delta() -> Result<(), String> {
        let state = AppState::test_state();
        start_playing(&state)?;
        let mut watch = PlaybackWatch::default();

        feed_audible_progress(&state, 3.0, &mut watch);
        let position = state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot()
            .position_secs;
        assert_position(position, 3.0, "the first delta is credited whole");
        assert!((watch.fed_secs - 3.0).abs() < 1e-9);

        feed_audible_progress(&state, 3.0, &mut watch);
        let position = state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot()
            .position_secs;
        assert_position(position, 3.0, "a repeated reading credits nothing");

        feed_audible_progress(&state, 3.5, &mut watch);
        let position = state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot()
            .position_secs;
        assert_position(position, 3.5, "only the step since the last credit lands");
        Ok(())
    }

    #[test]
    fn terminal_transport_clears_the_watch_baselines() -> Result<(), String> {
        let state = AppState::test_state();
        start_playing(&state)?;
        {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            let _ = session
                .execute(ShellVerb::Complete)
                .map_err(|e| e.to_string())?;
        }
        let mut watch = PlaybackWatch {
            fed_secs: 4.0,
            units: Some(1),
        };
        playback_watch_tick(&state, &mut watch);
        assert!(watch.fed_secs.abs() < 1e-9);
        assert_eq!(watch.units, None);
        Ok(())
    }

    #[test]
    fn progress_republishes_once_per_new_consumed_unit() -> Result<(), String> {
        let state = AppState::test_state();
        start_playing(&state)?;
        *state.surface_resolver.lock().map_err(|e| e.to_string())? = Some(two_unit_resolver());
        let mut watch = PlaybackWatch::default();

        assert!(
            maybe_republish_progress(&state, &mut watch),
            "the first observation of a run republishes"
        );
        assert_eq!(watch.units, Some(0));

        assert!(
            !maybe_republish_progress(&state, &mut watch),
            "a repeated count republishes nothing"
        );

        {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.on_audible_progress(5.0);
        }
        assert!(
            maybe_republish_progress(&state, &mut watch),
            "crossing into the second unit republishes"
        );
        assert_eq!(watch.units, Some(1));
        assert!(!maybe_republish_progress(&state, &mut watch));
        Ok(())
    }

    #[test]
    fn document_end_signal_completes_in_process_without_a_shell() -> Result<(), String> {
        let state = AppState::test_state();
        start_playing(&state)?;
        signal_document_end(&state);
        let transport = state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot()
            .transport;
        assert_eq!(transport, TransportState::Completed);
        Ok(())
    }

    #[test]
    fn document_end_dispatch_is_idempotent() -> Result<(), String> {
        let state = AppState::test_state();
        start_playing(&state)?;
        let first = commands::dispatch_document_end(&state)?;
        assert_eq!(first["changed"], serde_json::Value::from(true));
        let second = commands::dispatch_document_end(&state)?;
        assert_eq!(second["changed"], serde_json::Value::from(false));
        Ok(())
    }
}
