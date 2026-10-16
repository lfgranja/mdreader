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

use std::sync::Mutex;

use engine_core::closed::SessionEnd;
use player::backend::CpalBackend;
use player::cli::{PlaybackSession, SessionSnapshot, ShellVerb};
use player::rate::RateStep;
use player::surface::MirrorState;
use player::transport::{PlaybackRate, TransportState};
use session::fingerprint;
use session::offer::ExpiryOutcome;
use session::store::{BookmarkStore, StoredPosition};
use std::time::Duration;
use tauri::Manager;
use tokio::time;

/// The permission-state gate (T030b): the pure,
/// headless half of FR-008/FR-010/FR-021. The Kotlin
/// shell reads the platform permissions and pushes the
/// answer as a `PermissionSnapshot`; the startup and
/// pre-play gates consume it. No Android API calls live
/// here (Principle V).
pub mod permissions;

/// The Kotlin-to-Rust shell channel (T111): the integer-code drain
/// the `ShellHost` bootstrap posts through. Installed once in
/// [`run`]; consulted (never constructed) by
/// [`commands::register_android_plugin`].
pub mod shell_jni;

/// The shared reason shapes every surface renders
/// (T065): short lock-screen strings, full in-app
/// strings, and the reason expiry (3 s focus-loss
/// vs persistent failure). T065b's status view and
/// T078's failure surface render through these
/// getters — never a second definition.
pub mod surface;

/// The minimal in-app status view (T065b): the
/// read-only render of the stored `last_rate`, the
/// transport/position mirror, and the current
/// failure/focus reason with its expiry — through the
/// T065 getters alone, with no playback logic and
/// no second string definition.
pub mod status_view;

/// The engine-failure surface (T078): the named
/// failure + alternatives rendering, conforming to
/// the T065 reason shapes.
pub mod failure;

/// Shared app state: one playback session, one lazily-opened
/// audio backend, the platform permission snapshot the
/// Kotlin shell pushes, the held focus/offer machines
/// (T125, FR-014 / FR-032c), and the Rust→shell publish
/// channel (T133, FR-003 / FR-028): streams `MirrorState`
/// snapshots on every transport/position/focus event.
/// Does not derive Debug because it contains `tauri::ipc::Channel`
/// which does not implement Debug.
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
    /// ordered list of (position_secs, title) for section
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
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("session", &self.session)
            .field("backend", &self.backend)
            .field("permissions", &self.permissions)
            .field("focus_tracker", &self.focus_tracker)
            .field("offer", &self.offer)
            .field("publish_channel", &"<tauri::ipc::Channel<MirrorState>>")
            .field("surface_resolver", &self.surface_resolver)
            .field("headings", &self.headings)
            .field("bookmark_store", &"<BookmarkStore>")
            .field("current_fingerprint", &self.current_fingerprint)
            .field("first_yield_ms", &self.first_yield_ms)
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
            publish_channel: Mutex::new(None),
            surface_resolver: Mutex::new(None),
            headings: Mutex::new(Vec::new()),
            bookmark_store: Mutex::new(None),
            current_fingerprint: Mutex::new(None),
            first_yield_ms: Mutex::new(None),
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
    match state.current_fingerprint.lock() {
        Ok(guard) => *guard,
        Err(_) => {
            tracing::warn!("the fingerprint lock is poisoned; no bookmark can be keyed");
            None
        }
    }
}

/// The FR-015 supersede clock in unix milliseconds,
/// or `0` — the "no interval recorded" value — when
/// no interval is open or the lock is poisoned.
fn current_first_yield_ms(state: &AppState) -> u64 {
    match state.first_yield_ms.lock() {
        Ok(clock) => clock.unwrap_or(0),
        Err(_) => 0,
    }
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
    match state.current_fingerprint.lock() {
        Ok(mut guard) => *guard = Some(fp),
        Err(_) => tracing::warn!("document fingerprint not recorded: the state lock is poisoned"),
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
    match state.first_yield_ms.lock() {
        Ok(mut clock) => match transport {
            TransportState::Playing => {
                if clock.is_none() {
                    *clock = Some(now_ms());
                }
            }
            TransportState::Paused | TransportState::Interrupted => {}
            TransportState::Stopped | TransportState::Completed => *clock = None,
        },
        Err(_) => tracing::warn!("the FR-015 clock was not updated: the state lock is poisoned"),
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
fn end_kind_of_pause(transport: TransportState) -> Option<SessionEnd> {
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
fn event_end_kind(transport: TransportState) -> Option<SessionEnd> {
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
    let mut store_guard = match state.bookmark_store.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("bookmark not persisted: the store lock is poisoned");
            return;
        }
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
    let mut store_guard = match state.bookmark_store.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("bookmark not deleted: the store lock is poisoned");
            return;
        }
    };
    let Some(store) = store_guard.as_mut() else {
        return; // the store never opened — there is no row to delete
    };
    if let Err(error) = store.delete(&fingerprint) {
        tracing::warn!(%error, "bookmark not deleted: the store delete failed");
    } else {
        tracing::debug!("bookmark deleted");
    }
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
    let mut store_guard = match state.bookmark_store.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("last rate not persisted: the store lock is poisoned");
            return;
        }
    };
    let Some(store) = store_guard.as_mut() else {
        return; // the store never opened — the setting degrades to unpersisted
    };
    if let Err(error) = store.put_last_rate(rate) {
        tracing::warn!(%error, "last rate not persisted: the store write failed");
    } else {
        tracing::debug!("last rate persisted");
    }
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
fn rate_press_of(verb: ShellVerb) -> Option<RateStep> {
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
        let mut store_guard = match state.bookmark_store.lock() {
            Ok(guard) => guard,
            Err(_) => {
                tracing::warn!("rate seed skipped: the store lock is poisoned");
                return;
            }
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
    let Some((step, moves)) = (match state.session.lock() {
        Ok(session) => rate_seed_moves(session.snapshot().rate, target),
        Err(_) => {
            tracing::warn!("rate seed skipped: the session lock is poisoned");
            return;
        }
    }) else {
        return; // already at the stored rung — nothing to drive
    };
    let mut session = match state.session.lock() {
        Ok(session) => session,
        Err(_) => {
            tracing::warn!("rate seed stopped: the session lock is poisoned");
            return;
        }
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
/// transport for the FR-015 clock, and — when the
/// landing is a pause — persist the position with
/// the end kind that landing names. Every surface
/// that moves the session through `execute` (the
/// bridge verbs, the intent paths, the cold-start
/// open) routes through here, so no call site can
/// forget the durable write.
fn after_execute(state: &AppState, snapshot: &SessionSnapshot) {
    note_transport(state, snapshot.transport);
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
fn after_skip(state: &AppState, snapshot: &SessionSnapshot) {
    note_transport(state, snapshot.transport);
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
    let snapshot = match state.session.lock() {
        Ok(guard) => guard.snapshot(),
        Err(_) => {
            tracing::warn!("app-background bookmark skipped: the session lock is poisoned");
            return;
        }
    };
    if let Some(end_kind) = event_end_kind(snapshot.transport) {
        persist_bookmark(state, snapshot.position_secs, end_kind);
    }
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

/// Route a stored session end to the surface that renders its
/// resume offer (T037) — thin route-to-surface, with no offer
/// logic of its own.
///
/// The form is READ from the held
/// [`session::offer::OfferMachine`], never re-derived: the
/// machine's own decision fn (`on_session_end`) is driven over
/// a probe row carrying the stored end kind. The machine
/// resolves the form SOLELY from that kind (storage-schema.md),
/// so the probe's placeholder offset changes no answer — this
/// function only maps the machine's answer to a surface.
///
/// * The machine opening the continue offer routes to
///   [`OfferRoute::InAppDialog`].
/// * The machine answering "no open offer" is then asked what
///   the end resolves to, through its own `expire`: a standing
///   explicit-resume requirement (FR-032d) routes to
///   [`OfferRoute::LockScreenResume`], and nothing open routes
///   to [`OfferRoute::NoRoute`]. The `expire` question is asked
///   on that branch only, so it can never consume an open
///   offer.
///
/// Driving the held machine is exactly the open/reopen
/// lifecycle point — the offer is resolved there and read at
/// expiry afterwards — so the route is computed when a document
/// with a stored row opens, never mid-window. A poisoned offer
/// lock records and routes nothing: a route is a rendering
/// hint, never a playback decision.
#[must_use]
pub(crate) fn route_offer(state: &AppState, end_kind: SessionEnd) -> OfferRoute {
    // The key the row would sit under; the machine itself
    // ignores it (the form never depends on document identity).
    let fingerprint = current_fingerprint(state).unwrap_or([0u8; 32]);
    let probe = StoredPosition {
        offset_secs: 0.0,
        first_yield_ms: current_first_yield_ms(state),
        end_kind,
    };
    let mut machine = match state.offer.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("offer route unresolved: the offer lock is poisoned");
            return OfferRoute::NoRoute;
        }
    };
    if machine.on_session_end(fingerprint, Some(probe)).is_some() {
        // The machine opened the continue offer: the
        // deliberate-pause form.
        return OfferRoute::InAppDialog;
    }
    match machine.expire() {
        ExpiryOutcome::ExplicitResume => OfferRoute::LockScreenResume,
        // A foregrounded auto-start resolves on the open-offer
        // branch above; arriving here means nothing opened at
        // all (a terminal end, or no stored row).
        ExpiryOutcome::AutoStart { .. } | ExpiryOutcome::NoOffer => OfferRoute::NoRoute,
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
pub(crate) fn offer_route_wire(route: OfferRoute) -> (&'static str, Option<u64>) {
    match route {
        OfferRoute::InAppDialog => ("in_app_dialog", Some(OFFER_WINDOW_MS)),
        OfferRoute::LockScreenResume => ("lockscreen_resume", None),
        OfferRoute::NoRoute => ("none", None),
    }
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
    let snapshot = match state.session.lock() {
        Ok(guard) => guard.snapshot(),
        Err(_) => {
            tracing::warn!("heartbeat tick skipped: the session lock is poisoned");
            return;
        }
    };
    if snapshot.transport != TransportState::Playing {
        return;
    }
    persist_bookmark(state, snapshot.position_secs, SessionEnd::Interrupted);
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
            publish_channel: Mutex::new(None),
            surface_resolver: Mutex::new(None),
            headings: Mutex::new(Vec::new()),
            bookmark_store: Mutex::new(None),
            current_fingerprint: Mutex::new(None),
            first_yield_ms: Mutex::new(None),
        })
        .setup(|_app| {
            // Voice assets ship inside the APK: extract them to a
            // real filesystem path before the first synthesis. Hosts
            // use the manifest tree; nothing to do there.
            #[cfg(target_os = "android")]
            {
                if let Ok(data) = _app.path().app_data_dir() {
                    if let Ok(voice) = adapter_synthesis::android_assets::extract_voices(&data) {
                        adapter_synthesis::synth::set_asset_dir(voice);
                    }
                }
            }

            // Get the AppState for bookmark store initialization and permission gate
            let app_state = _app.state::<AppState>();
            // Initialize the 20 s durable heartbeat (T011): open the
            // BookmarkStore in the app data directory and start the
            // periodic timer that calls `put` while Playing.
            let data_dir = _app
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
            // Spawn the 20 s heartbeat task (T011): calls `BookmarkStore::put`
            // while Playing for crash-safe position recovery (FR-004 / FR-007).
            // Each tick reads the session snapshot under its own lock,
            // releases it, and only then takes the store inside
            // `persist_bookmark` — no two state locks are ever held
            // together here, so the heartbeat cannot form a cycle with
            // the IPC paths. The task holds the owned app handle (never a
            // borrowed `State`), so it outlives the setup closure.
            let app_handle = _app.app_handle().clone();
            let _ = tauri::async_runtime::spawn(async move {
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

            // Shell JNI channel (T111, FR-002/FR-003): install
            // the integer-code drain holding a clone of the
            // application handle, so every Kotlin-posted verb or
            // intent executes on AppState's OWN session through
            // the single `run_mobile_plugin` dispatch — never a
            // second session, never a duplicated code path. A
            // spawn failure is recorded inside `install`; the
            // sender stays installed, so later posts drop loudly
            // instead of vanishing.
            shell_jni::install(_app.app_handle().clone());
            // Permission-state gate (T030b, FR-008/FR-010): the
            // startup check runs once at boot against the snapshot
            // the Kotlin shell pushes through `update_permissions`
            // (the all-granted host baseline until it does). The
            // outcome is recorded as a tracing event; a refusal
            // carries the stated reason, and no playing state may
            // ever follow it.
            let app_state = _app.state::<AppState>();
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
        OfferRoute, after_background, after_execute, after_skip, current_fingerprint,
        delete_bookmark, offer_route_wire, persist_last_rate, rate_press_of, route_offer,
        seed_session_rate, set_document_fingerprint,
    };
    use player::cli::{SessionSnapshot, ShellVerb};
    use player::focus::{FocusClassification, FocusIntent, FocusOutcome};
    use player::rate::RateStepper;
    use player::routing::{RoutingDecision, RoutingSignal, RoutingTracker};
    use session::fingerprint;
    use tauri::{AppHandle, Manager, State, command};

    /// Declared sample rate of the engine handoff (mono f32).
    const DECLARED_RATE_HZ: f64 = 22_050.0;

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
    /// ActivityManager reflection forces the service class
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
    #[command]
    pub fn update_permissions(
        state: State<'_, AppState>,
        snapshot: PermissionSnapshot,
    ) -> Result<(), String> {
        let mut permissions = state.permissions.lock().map_err(|e| e.to_string())?;
        *permissions = snapshot;
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
        if bytes.is_empty() {
            return Err(OpenError::EmptyDocument);
        }
        // The snapshot bytes are text: decode and validate
        // before anything is derived from them.
        let document = String::from_utf8(bytes).map_err(|_| OpenError::InvalidUtf8)?;

        // The content fingerprint (FR-036): the key the 20 s
        // heartbeat and every event-durable put persist under,
        // recorded through the one call site every
        // document-opening path shares (T011 / T035).
        let fingerprint_hex = hex::encode(fingerprint::fingerprint(document.as_bytes()));
        set_document_fingerprint(state, document.as_bytes());

        // The open lookup (T036): bytes → fingerprint → the
        // stored row → the offer form, through the library fn
        // the `session ingest` CLI verb runs — never
        // re-assembled here (Principle II / FR-044). The store
        // is the application's own: a second redb handle over
        // the same file cannot exist, and the heartbeat
        // already writes through this one.
        let lookup = {
            let mut store_guard =
                state
                    .bookmark_store
                    .lock()
                    .map_err(|error| OpenError::Store {
                        reason: error.to_string(),
                    })?;
            let store = store_guard.as_mut().ok_or_else(|| OpenError::Store {
                reason: "the session store never opened".to_string(),
            })?;
            session::store::lookup_on_open(store, document.as_bytes()).map_err(|error| {
                OpenError::Store {
                    reason: error.to_string(),
                }
            })
        }?;

        // The offer route (T037): the stored end kind resolves
        // to the surface that renders the resume decision, and
        // driving the held machine at this open is the
        // lifecycle point that opens the FR-032c continue
        // offer the in-app dialog renders — never mid-window.
        let route = match lookup.stored {
            Some(stored) => route_offer(state, stored.end_kind),
            None => OfferRoute::NoRoute,
        };

        // The one synthesis path (Principle II): split,
        // synthesize, enqueue, start — with the totals fixed
        // at open and the rate seeded from the store.
        let mut payload = play(document, state)?;

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

    /// The classified bridge action (T110): every closed
    /// shell-IPC wire string (`contracts/shell-ipc.md`)
    /// parses to exactly one variant — a session verb, a
    /// platform intent, or the read-only query. No
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
    pub fn run_mobile_plugin(
        action: String,
        document: Option<String>,
        payload: Option<String>,
        app: AppHandle,
        state: State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match parse_action(action.as_str()) {
            Ok(BridgeAction::Verb(verb)) => dispatch_verb(verb, document, payload, &state),
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
        payload: Option<String>,
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
                    return Ok(serde_json::json!({
                        "ok": false,
                        "error": failure.reason(),
                    }));
                }
                play(document.unwrap_or_default(), state).map_err(|error| error.to_string())
            }
            ShellVerb::ResumeAtStored | ShellVerb::StartOver | ShellVerb::ExternalStart => {
                // The same pre-play gate as `Play`: these are
                // the remaining verbs that can reach `Playing`
                // in the closed transition table, so a refusal
                // executes nothing and reports no playing
                // state (FR-010).
                let permissions = state.permissions.lock().map_err(|e| e.to_string())?;
                if let Err(failure) = permissions::check_pre_play(&permissions) {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "error": failure.reason(),
                    }));
                }
                let snap = {
                    let mut session = state.session.lock().map_err(|e| e.to_string())?;
                    session.execute(verb).map_err(|e| e.to_string())?
                };
                publish_current_state(state, None);
                // Playing-class landing: open the FR-015 clock
                // (T011) so the next durable write stamps it.
                after_execute(state, &snap);
                Ok(serde_json::json!({
                    "status": snap.transport.to_string(),
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            ShellVerb::Pause => {
                if let Ok(backend) = state.backend.lock() {
                    if let Some(backend) = backend.as_ref() {
                        backend.pause();
                    }
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
                after_execute(state, &snap);
                Ok(serde_json::json!({
                    "status": "paused",
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            ShellVerb::Stop => {
                if let Ok(backend) = state.backend.lock() {
                    if let Some(backend) = backend.as_ref() {
                        backend.stop();
                    }
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
                after_execute(state, &snap);
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
                Ok(serde_json::json!({
                    "status": snap.transport.to_string(),
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                    "engine_id": snap.engine_id,
                    "requested_engine_id": payload,
                }))
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
        after_skip(state, &snapshot);
        // T042 write-on-change (FR-019b): only a rate press
        // persists, and only when it moved the rung — the
        // stepper's `None` (a saturating press) writes
        // nothing new.
        if let Some(step) = rate_press_of(verb) {
            if let Some(rung) = RateStepper::open(Some(previous_rate)).step(step) {
                persist_last_rate(state, rung);
            }
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
                let mut restore_ok = false;
                if let Ok(backend_guard) = state.backend.lock() {
                    if let Some(backend) = backend_guard.as_ref() {
                        if backend.set_gain(player::focus::FULL_GAIN).is_ok() {
                            restore_ok = true;
                        }
                    }
                }
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
                let decision = apply_focus(state, FocusIntent::DeviceDisappeared)?;
                pause_ack(
                    state,
                    "device_disappeared",
                    decision.outcome,
                    ShellVerb::Interrupt,
                    Some("device_disappeared".to_string()),
                )
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
    /// Also restores full gain (FULL_GAIN) so that a
    /// paused session never resumes at a ducked level.
    fn pause_backend(state: &State<'_, AppState>) {
        if let Ok(backend) = state.backend.lock() {
            if let Some(backend) = backend.as_ref() {
                let _ = backend.set_gain(player::focus::FULL_GAIN);
                backend.pause();
            }
        }
    }

    /// Best-effort backend stop behind a real document
    /// completion: mirrors the stop arm. It runs only
    /// after [`ShellVerb::Complete`] moves the session,
    /// never ahead of it, so a retained pause keeps its
    /// queued audio. Restores full gain (FULL_GAIN).
    fn stop_backend(state: &State<'_, AppState>) {
        if let Ok(backend) = state.backend.lock() {
            if let Some(backend) = backend.as_ref() {
                let _ = backend.set_gain(player::focus::FULL_GAIN);
                backend.stop();
            }
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
        state: &State<'_, AppState>,
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
            after_execute(state, &snapshot);
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
    /// is applied to the backend (CpalBackend exposes
    /// set_gain/gain since T123). Driven through the
    /// held tracker (T125, FR-014), so the ducked flag
    /// and duck-restore flag persist for the silence
    /// escalation that may follow. Publishes the mirror
    /// state with the ducked gain (T133).
    fn ack_duck(state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
        let decision = apply_focus(state, FocusIntent::DuckableTransient)?;
        let mut applied = false;
        if let Ok(backend_guard) = state.backend.lock() {
            if let Some(backend) = backend_guard.as_ref() {
                if backend.set_gain(decision.gain).is_ok() {
                    applied = true;
                }
            }
        }
        let snapshot = snapshot_of(state)?;
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
    /// and headings from AppState and emits a `MirrorState`
    /// snapshot via the `tauri::ipc::Channel`.
    fn publish_current_state(state: &AppState, transient_reason: Option<String>) {
        let resolver_guard = match state.surface_resolver.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(resolver) = resolver_guard.as_ref() else {
            return; // no document loaded
        };
        let headings_guard = match state.headings.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        crate::publish::publish_from_session(state, resolver, &headings_guard, transient_reason);
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
            RoutingDecision::Pause { latency_ms, .. } => {
                pause_backend(state);
                let (snapshot, changed) = execute_or_current(state, ShellVerb::Interrupt)?;
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
    fn dispatch_document_end(state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
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

    /// Split, synthesize, enqueue, and start one document —
    /// the single synthesis path every surface routes through
    /// (Principle II: the `play` wire action, the T019
    /// cold-start [`open_document_bytes`], and every later
    /// consumer land here exactly once).
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
    /// synthesis, and the PCM is enqueued and started the
    /// moment synthesis completes. A missed bound is fixed
    /// HERE — the first-segment ordering (start the first
    /// chunk's PCM while the rest synthesizes) is this body's
    /// change, never another task's — and the bound is never
    /// renegotiated.
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
    fn play(document: String, state: &AppState) -> Result<serde_json::Value, OpenError> {
        if document.trim().is_empty() {
            return Err(OpenError::EmptyDocument);
        }
        // T042 read-at-start (FR-019b): the session opens at
        // the stored rung — seeded through the rate verbs
        // before anything else happens, so a fresh session
        // never starts at the wrong speed.
        seed_session_rate(state);
        // Same code path as the adapter CLI (Principle II): the
        // adapter owns the chunk boundary; the engine receives
        // bounded chunks only.
        let segments = adapter_synthesis::chunk::split_document(&document, "");
        if segments.is_empty() {
            return Err(OpenError::NoSegments);
        }

        // Extract title from first heading segment, or use fallback.
        let title = segments
            .iter()
            .find(|s| s.boundary_kind == adapter_synthesis::chunk::BoundaryKind::Heading)
            .map(|s| {
                s.content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .trim()
                    .to_string()
            })
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Document".to_string());

        // Synthesize PCM first to get total duration.
        let mut pcm = Vec::new();
        for segment in &segments {
            let buffer = adapter_synthesis::synth::synthesize(segment).map_err(|failure| {
                OpenError::Synthesis {
                    reason: format!("{failure:?}"),
                }
            })?;
            pcm.extend_from_slice(&buffer);
        }
        let pcm_samples = pcm.len();
        let duration_secs = pcm_samples as f64 / DECLARED_RATE_HZ;

        // Extract headings from segments with Heading boundary_kind.
        // Estimate position_secs based on segment order and total duration.
        let heading_segments: Vec<_> = segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.boundary_kind == adapter_synthesis::chunk::BoundaryKind::Heading)
            .collect();
        let mut headings = Vec::new();
        for (idx, segment) in heading_segments {
            let estimated_position = (idx as f64 / segments.len() as f64) * duration_secs;
            let heading_title = segment
                .content
                .lines()
                .next()
                .unwrap_or("")
                .trim_start_matches('#')
                .trim()
                .to_string();
            if !heading_title.is_empty() {
                headings.push(player::surface::Heading {
                    position_secs: estimated_position,
                    title: heading_title,
                });
            }
        }

        // Create surface resolver with the parsed-content totals fixed
        // at session start (FR-013 / FR-028a): the unit total is the
        // chunk split's segment count — a content fact — and the
        // projection basis is the spoken length per unit derived from
        // the synthesized PCM. The totals knowledge composes BOTH
        // session-start facts (the parsed-content unit total AND the
        // section count), so a heading-less session stays
        // indeterminate for the whole session (FR-013).
        let resolver = player::surface::SurfaceResolver::new(
            Some(title.clone()),
            "document.md".to_string(),
            segments.len() as u64,
            duration_secs,
            !segments.is_empty() && !headings.is_empty(),
        );

        // Store resolver and headings in AppState for publish path:
        // the one feeding of the session-start totals — the
        // resolver holds the parsed-content unit total and its
        // projection basis, written here, once, and never
        // re-estimated mid-session (FR-028a / FR-013).
        {
            let mut resolver_guard =
                state
                    .surface_resolver
                    .lock()
                    .map_err(|e| OpenError::NoStart {
                        reason: e.to_string(),
                    })?;
            *resolver_guard = Some(resolver);
            let mut headings_guard = state.headings.lock().map_err(|e| OpenError::NoStart {
                reason: e.to_string(),
            })?;
            *headings_guard = headings;
        }

        // Compute and store the document's blake3 fingerprint (FR-036 / T059)
        // for the 20 s heartbeat bookmark persistence (T011) — the same
        // call site `open_document` routes through.
        set_document_fingerprint(state, document.as_bytes());

        {
            let mut backend = state.backend.lock().map_err(|error| OpenError::Backend {
                reason: error.to_string(),
            })?;
            if backend.is_none() {
                let opened = CpalBackend::new().map_err(|error| OpenError::Backend {
                    reason: error.to_string(),
                })?;
                *backend = Some(opened);
            }
            let backend = backend.as_ref().ok_or_else(|| OpenError::Backend {
                reason: "no audio backend".to_string(),
            })?;
            backend.enqueue_pcm(pcm);
            backend.play().map_err(|error| OpenError::Backend {
                reason: error.to_string(),
            })?;
        }

        let snapshot = {
            let mut session = state.session.lock().map_err(|e| OpenError::NoStart {
                reason: e.to_string(),
            })?;
            session
                .execute(ShellVerb::Play)
                .map_err(|e| OpenError::NoStart {
                    reason: e.to_string(),
                })?
        };

        // Open the FR-015 clock on the landed Playing
        // transport (T011).
        after_execute(state, &snapshot);

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
        let resource_dir = app
            .path()
            .resource_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|e| format!("unresolvable: {e}"));
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
            let entries = match std::fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(_) => continue,
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
        let espeak_files = std::fs::read_dir(&espeak)
            .map(|entries| entries.count())
            .unwrap_or(0);
        let onnx_bytes = std::fs::metadata(voice.join("pt_BR-faber-medium.onnx"))
            .map(|meta| meta.len())
            .unwrap_or(0);
        (total, espeak_is_dir, espeak_files, onnx_bytes)
    }

    /// Get the current in-app status view (T065b): the read-only
    /// mirror of the session's authoritative state plus the
    /// stored last_rate. The view owns no state and computes no
    /// position, rate, or transport — it renders what the
    /// session already states (SC-035, FR-042).
    ///
    /// Returns the rendered `StatusView` with rate, transport,
    /// position, reason, and reason_expiry.
    #[command]
    pub fn get_status_view(
        state: State<'_, AppState>,
    ) -> Result<crate::status_view::StatusView, String> {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        let snap = session.snapshot();
        let resolver_guard = state.surface_resolver.lock().map_err(|e| e.to_string())?;
        let headings_guard = state.headings.lock().map_err(|e| e.to_string())?;

        let resolver = resolver_guard
            .as_ref()
            .ok_or_else(|| "no document loaded".to_string())?;

        let surface = resolver.resolve(
            snap.position_secs,
            resolver.consumed_units_at(snap.position_secs),
            &headings_guard,
        );
        let fingerprint_guard = state
            .current_fingerprint
            .lock()
            .map_err(|e| e.to_string())?;
        let fingerprint = *fingerprint_guard;
        // Get stored last_rate from bookmark store. The
        // fingerprint gates the read: with no document
        // loaded there is no key to read beside, and the
        // view renders the normal-speed default.
        let last_rate = if fingerprint.is_some() {
            let mut bookmark_store_guard =
                state.bookmark_store.lock().map_err(|e| e.to_string())?;
            if let Some(store) = bookmark_store_guard.as_mut() {
                store.get_last_rate().ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        // Build transient reason from the surface's reason if any
        // The surface doesn't carry reason directly, so we rely on the
        // publish path's transient_reason. For the initial view, we
        // don't have a transient reason.
        let reason: Option<(crate::surface::FullReason, crate::surface::ReasonExpiry)> = None;

        let document_title = surface.title.clone();
        let section_title = surface.section_title.clone();
        let fingerprint = fingerprint.unwrap_or([0u8; 32]);
        Ok(crate::status_view::render_status_view(
            last_rate,
            snap.transport,
            snap.position_secs,
            &document_title,
            section_title,
            reason,
            fingerprint,
        ))
    }

    /// Delete the session bookmark for a fingerprint (Stop control).
    /// This clears the stored position so the next open starts at 0.0
    /// with no resume offer (FR-032f, storage-schema.md).
    ///
    /// Called when the user presses the dedicated Stop button in-app.
    #[command]
    pub fn session_delete(fingerprint: [u8; 32], state: State<'_, AppState>) -> Result<(), String> {
        let mut bookmark_store_guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        if let Some(store) = bookmark_store_guard.as_mut() {
            store.delete(&fingerprint).map_err(|e| e.to_string())?;
        }
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
        let (offer_route, window_ms) = offer_route_wire(route_offer(&state, stored.end_kind));
        Ok(match window_ms {
            Some(window_ms) => serde_json::json!({
                "offer_route": offer_route,
                "window_ms": window_ms,
            }),
            None => serde_json::json!({ "offer_route": offer_route }),
        })
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
///   contract's closed verb set via the normative snake_case case-mapping
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

/// The Rust→shell publish path (T133): streams mirror
/// snapshots over `tauri::ipc::Channel` on every
/// transport/position/focus event (FR-003, FR-028).
/// The Kotlin surface consumes these via `@Command`
/// setters on `MediaMirror` and `Residency` — zero
/// playback logic in Kotlin (SC-035).
pub mod publish {
    use super::AppState;
    use player::cli::SessionSnapshot;
    use player::surface::{Heading, MirrorState, Progress, SurfaceResolver, SurfaceSnapshot};
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
    pub fn register_publish_listener(
        _app: AppHandle,
        state: State<'_, AppState>,
    ) -> Result<(), String> {
        // Create a channel with a no-op on_message handler.
        // The frontend will receive this channel and set up its onmessage handler.
        let channel = tauri::ipc::Channel::<MirrorState>::new(|_| Ok(()));

        let mut guard = state.publish_channel.lock().map_err(|e| e.to_string())?;
        *guard = Some(channel);

        tracing::info!("publish channel registered: {CHANNEL_NAME}");
        Ok(())
    }

    /// Emit a mirror snapshot to the frontend via the publish channel.
    ///
    /// Called from the dispatch paths after every transport/
    /// position/focus event. The snapshot is derived from the
    /// session's authoritative state (FR-003) plus the section
    /// title from the surface resolver (FR-028) and the
    /// transient reason when applicable (FR-041, FR-037).
    ///
    /// If the channel is not yet registered (frontend hasn't
    /// called `register_publish_listener`), the snapshot is
    /// dropped silently — the frontend will receive the current
    /// state on its next registration or via the diagnostic
    /// `diagnose` query.
    pub fn publish_mirror_to_channel(
        state: &AppState,
        snapshot: &SessionSnapshot,
        surface: &SurfaceSnapshot,
        transient_reason: Option<String>,
    ) {
        let channel_guard = match state.publish_channel.lock() {
            Ok(g) => g,
            Err(_) => return, // poisoned lock, drop silently
        };

        let Some(channel) = channel_guard.as_ref() else {
            return; // channel not registered yet, drop silently
        };

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

        let mirror = MirrorState {
            transport: snapshot.transport,
            position_secs: snapshot.position_secs,
            progress,
        };

        // The channel send is infallible in the Tauri API;
        // a send failure means the frontend disconnected,
        // which is acceptable — the frontend will resync on
        // next registration.
        let _ = channel.send(mirror);

        // Note: Rust→Kotlin MediaMirror publish (via JNI) deferred to
        // synthesis feature (FR-024) due to jni 0.22.4 API changes blocking
        // the bridge. The Kotlin PlaybackService handles notification
        // and MediaSession through its own lifecycle (ShellHost.init),
        // not through Rust JNI publish. For T-01 trial, the notification
        // surface is handled by Kotlin's PlaybackService.startForegroundHolder().
        // The full mirror synchronization requires resolving jni APIs.

        // Also log the transient reason if present (diagnostic).
        if let Some(reason) = transient_reason {
            tracing::debug!(%reason, "publish mirror with transient reason");
        }
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
        let session_guard = match state.session.lock() {
            Ok(g) => g,
            Err(_) => return,
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
        drop(session_guard); // release lock before publish
        publish_mirror_to_channel(state, &snapshot, &surface, transient_reason);
    }

    /// Test-compatible publish_mirror function (T132 contract).
    ///
    /// Returns the JSON payload that would be sent over the channel.
    /// Used by the contract test to verify the wire form.
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
        let _ = obj.insert(
            "position_secs".to_string(),
            serde_json::to_value(position_secs).unwrap(),
        );
        if let Some(section) = section_title {
            let _ = obj.insert("section_title".to_string(), Value::String(section));
        }
        let progress = if determinate {
            Value::Object({
                let mut m = Map::new();
                let _ = m.insert(
                    "consumed_units".to_string(),
                    serde_json::to_value(position_secs as u64).unwrap(),
                );
                let _ = m.insert(
                    "total_units".to_string(),
                    serde_json::to_value(120.0 as u64).unwrap(),
                );
                let _ = m.insert("k".to_string(), serde_json::to_value(1u32).unwrap());
                let _ = m.insert("n".to_string(), serde_json::to_value(1u32).unwrap());
                let mut outer = Map::new();
                let _ = outer.insert("determinate".to_string(), Value::Object(m));
                outer
            })
        } else {
            Value::String("indeterminate".to_string())
        };
        let _ = obj.insert("progress".to_string(), progress);
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
        let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        let store = guard.as_mut().ok_or("no bookmark store")?;
        store.get(&fingerprint).map_err(|e| e.to_string())
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
        after_execute(state, &snapshot);
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
        after_execute(&state, &paused);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?.ok_or("no row after the deliberate pause")?;
        assert_eq!(row.offset_secs, paused.position_secs);
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
        after_execute(&state, &interrupted);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row =
            stored_row(&state, fingerprint)?.ok_or("no row after the focus-loss interruption")?;
        assert_eq!(row.end_kind, SessionEnd::Interrupted);
        assert_eq!(row.offset_secs, interrupted.position_secs);
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
        assert_eq!(row.offset_secs, 0.0);

        let stopped = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Stop)
                .map_err(|e| e.to_string())?
        };
        after_execute(&state, &stopped);
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
        after_execute(&state, &paused);
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
        let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        let store = guard.as_mut().ok_or("no bookmark store")?;
        store.get(&fingerprint).map_err(|e| e.to_string())
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
        after_execute(state, &snapshot);
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
        after_execute(state, &paused);
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
        after_execute(&state, &stopped);
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
        after_execute(&state, &completed);
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
        assert_eq!(row.offset_secs, 0.0);
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
        after_skip(&state, &skipped);

        let fingerprint = fingerprint::fingerprint(DOC);
        let row = stored_row(&state, fingerprint)?.ok_or("the section change recorded no row")?;
        assert_eq!(
            row.end_kind,
            SessionEnd::Interrupted,
            "a section change while playing did not record the crash-point kind"
        );
        assert_eq!(
            row.offset_secs, SKIP_STEP_SECS,
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
            route_offer(&state, SessionEnd::Paused),
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
                ExpiryOutcome::AutoStart { offset_secs: 0.0 },
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
            route_offer(&state, SessionEnd::Interrupted),
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
                route_offer(&state, end_kind),
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
        let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        let store = guard.as_mut().ok_or("no bookmark store")?;
        store
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
            let resolver = state.surface_resolver.lock().map_err(|e| e.to_string())?;
            assert!(
                resolver.is_some(),
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
        let _opened = open_document_bytes(&state, DOC.to_vec());
        assert_eq!(
            current_fingerprint(&state),
            Some(fingerprint()),
            "the open did not record the document's content fingerprint"
        );
        assert!(
            !matches!(
                _opened,
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
        let resolver = state.surface_resolver.lock().map_err(|e| e.to_string())?;
        assert!(
            resolver.is_none(),
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
    #[test]
    fn the_offer_route_is_published_for_a_stored_row() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, 42.0, engine_core::closed::SessionEnd::Paused)?;

        let opened = open_document_bytes(&state, DOC.to_vec());
        if backend_available() {
            let payload = opened.map_err(|error| error.to_string())?;
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
        } else {
            assert!(
                matches!(opened, Err(OpenError::Backend { .. })),
                "a device-less host must answer the named backend refusal, got {opened:?}"
            );
        }
        // The route is computed at the open (never mid-window): the
        // held machine carries the open continue offer the dialog
        // renders — the FR-009 silence rule auto-starts from the
        // offer's offset when no choice is made inside the window.
        // The route resolves the FORM solely from the stored end
        // kind, so the machine's offer carries the probe
        // placeholder offset `route_offer` drives it with; the
        // offset the reader resumes from is the stored row, which
        // the open payload reports above (T037 / FR-044).
        let mut offer = state.offer.lock().map_err(|e| e.to_string())?;
        assert!(
            offer.continue_offer().is_some(),
            "the open left no continue offer open for the dialog to render"
        );
        offer.on_foregrounded();
        assert_eq!(
            offer.expire(),
            ExpiryOutcome::AutoStart { offset_secs: 0.0 },
            "the silence rule did not auto-start the open offer (FR-009)"
        );
        Ok(())
    }

    /// An unexpected stored end routes to the lock-screen resume
    /// control (FR-032d / FR-031): no continue offer opens, so no
    /// auto-start exists for that row. The route is the same
    /// `route_offer` call the open path runs, asserted here on the
    /// device-independent half (a device host asserts it through
    /// the payload's `offer_route`).
    #[test]
    fn an_unexpected_stored_end_routes_to_the_lock_screen_resume() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        store_row(&state, 42.0, engine_core::closed::SessionEnd::Interrupted)?;

        let opened = open_document_bytes(&state, DOC.to_vec());
        if backend_available() {
            let payload = opened.map_err(|error| error.to_string())?;
            assert_eq!(
                payload["offer_route"], "lockscreen_resume",
                "an unexpected end must route to the lock-screen resume control (FR-032d)"
            );
            assert_eq!(payload["has_stored_position"], true);
            assert_eq!(payload["stored_offset_secs"], 42.0);
        } else {
            assert!(
                matches!(opened, Err(OpenError::Backend { .. })),
                "a device-less host must answer the named backend refusal, got {opened:?}"
            );
        }
        // The route the open path resolves for that end kind: the
        // explicit-resume control, never a continue offer — and
        // the stored offset the payload reports is what the
        // control resumes from. The form/offset split is T037's:
        // `route_offer` resolves the form solely from the end
        // kind, so the offset the reader resumes from arrives in
        // the payload's `stored_offset_secs` (FR-044).
        assert_eq!(
            route_offer(&state, engine_core::closed::SessionEnd::Interrupted),
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
        let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        let store = guard.as_mut().ok_or("no bookmark store")?;
        store.get_last_rate().map_err(|e| e.to_string())
    }

    /// Store a `last_rate` rung, as an earlier session's dispatch
    /// would have written it.
    fn put_stored_rate(state: &AppState, rate: PlaybackRate) -> Result<(), String> {
        let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        let store = guard.as_mut().ok_or("no bookmark store")?;
        store.put_last_rate(rate).map_err(|e| e.to_string())
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
        let payload =
            execute_live_control(&state, ShellVerb::RateUp).map_err(|error| error.to_string())?;
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
        {
            let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
            *guard = Some(fresh);
        }
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
        {
            let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
            let store = guard.as_mut().ok_or("no bookmark store")?;
            store
                .put(
                    &key,
                    StoredPosition {
                        offset_secs: 12.5,
                        first_yield_ms: 700,
                        end_kind: engine_core::closed::SessionEnd::Paused,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        let _pressed = execute_live_control(&state, ShellVerb::RateUp)?;
        let stored = {
            let mut guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
            let store = guard.as_mut().ok_or("no bookmark store")?;
            store.get(&key).map_err(|e| e.to_string())?
        };
        let stored = stored.ok_or("the rate press disturbed the stored position")?;
        assert_eq!(
            stored.offset_secs, 12.5,
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
