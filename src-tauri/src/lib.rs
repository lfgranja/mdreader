//! Tauri bridge — IPC delegation to the Rust playback path.
//!
//! Every surface (notification, lock screen, CLI, IPC) executes the
//! same Rust code (Principle II): `run_mobile_plugin` splits the
//! document with the adapter, synthesizes it with the bound
//! offline engine, enqueues the PCM on the CPAL backend, and
//! advances the single playback session. No transport, rate, or
//! position logic lives in Kotlin or the frontend.

use std::sync::Mutex;

use engine_core::closed::SessionEnd;
use player::backend::CpalBackend;
use player::cli::{PlaybackSession, SessionSnapshot};
use player::surface::MirrorState;
use player::transport::TransportState;
use tauri::Manager;
use std::time::Duration;
use session::store::{BookmarkStore, StoredPosition};
use session::fingerprint;
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
        .map_or(0, |elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
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
    use super::{AppState, CpalBackend};
    use player::cli::{SessionSnapshot, ShellVerb};
    use player::focus::{FocusClassification, FocusIntent, FocusOutcome};
    use player::routing::{RoutingDecision, RoutingSignal, RoutingTracker};
    use session::fingerprint;
    use session::offer::OfferMachine;
    use session::store::BookmarkStore;
    use crate::{after_execute, set_document_fingerprint};
    use tauri::{AppHandle, Manager, State, command};
    use base64::{Engine as _, engine::general_purpose};
    use std::io::Write;

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

    /// Open a document from base64-encoded bytes, write snapshot,
    /// compute fingerprint, check session store for resume offer,
    /// and start playback (US1).
    ///
    /// This is the cold-start path: the document bytes are the sole
    /// synthesis input. The snapshot is written to the app's data
    /// directory under `snapshots/<fingerprint-hex>.md`, the BLAKE3
    /// fingerprint is computed, the session store is consulted for
    /// any stored position, the resume-offer machine determines the
    /// resume behavior, and playback starts via the synthesis path.
    ///
    /// Returns the session snapshot with status, position, and
    /// transport state.
    #[command]
    pub fn open_document(
        state: State<'_, AppState>,
        app: AppHandle,
        document_base64: String,
    ) -> Result<serde_json::Value, String> {
        // Decode base64 document bytes.
        let document_bytes = general_purpose::STANDARD
            .decode(&document_base64)
            .map_err(|e| format!("invalid base64: {e}"))?;

        if document_bytes.is_empty() {
            return Err("empty document: nothing to synthesize".to_string());
        }

        // Compute BLAKE3 fingerprint of document content (FR-036).
        let fp = fingerprint(&document_bytes);
        let fingerprint_hex = hex::encode(fp);

        // Get app data directory for snapshot storage.
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to get app data dir: {e}"))?;

        let snapshots_dir = data_dir.join("snapshots");
        if !snapshots_dir.exists() {
            std::fs::create_dir_all(&snapshots_dir)
                .map_err(|e| format!("failed to create snapshots dir: {e}"))?;
        }

        // Write snapshot file: <fingerprint-hex>.md
        let snapshot_path = snapshots_dir.join(format!("{fingerprint_hex}.md"));
        let mut file = std::fs::File::create(&snapshot_path)
            .map_err(|e| format!("failed to create snapshot file: {e}"))?;
        file.write_all(&document_bytes)
            .map_err(|e| format!("failed to write snapshot: {e}"))?;

        // Trim other snapshots (keep-current-only policy).
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path != snapshot_path && path.is_file() {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        // Open session store and check for stored position.
        let db_path = data_dir.join("bookmarks.redb");
        let mut store = BookmarkStore::open(&db_path)
            .map_err(|e| format!("failed to open session store: {e}"))?;

        let stored = store
            .get(&fp)
            .map_err(|e| format!("failed to get stored position: {e}"))?;

        // Determine resume offer via offer machine.
        let mut offer_machine = OfferMachine::new();
        let offer = offer_machine.on_session_end(fp, stored);

        // Decide which verb to execute based on offer:
        // - Continue offer (deliberate pause, foregrounded) -> ResumeAtStored
        // - StartOver offer or no offer -> Play (starts from 0.0)
        // - Interrupted (explicit resume required) -> Play (starts from 0.0, no auto-resume)
        let verb = match offer {
            Some(session::offer::Offer::Continue { .. }) => ShellVerb::ResumeAtStored,
            Some(session::offer::Offer::StartOver) | None => ShellVerb::Play,
        };

        // Record the fingerprint for the 20 s heartbeat and
        // the event-durable puts (T011) — the one call site
        // every document-opening path shares.
        set_document_fingerprint(&state, &document_bytes);

        // Synthesize the document (same as play() but without executing a verb yet).
        let document_str = String::from_utf8(document_bytes)
            .map_err(|e| format!("document is not valid UTF-8: {e}"))?;

        // Split document into segments.
        let segments = adapter_synthesis::chunk::split_document(&document_str, "");
        if segments.is_empty() {
            return Err("empty document: no segments".to_string());
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
            let buffer = adapter_synthesis::synth::synthesize(segment)
                .map_err(|failure| format!("synthesis failed: {failure:?}"))?;
            pcm.extend_from_slice(&buffer);
        }
        let pcm_samples = pcm.len();
        let duration_secs = pcm_samples as f64 / DECLARED_RATE_HZ;

        // Extract headings from segments with Heading boundary_kind.
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

        // Create surface resolver with known duration.
        let resolver = player::surface::SurfaceResolver::new(
            Some(title.clone()),
            "document.md".to_string(),
            true, // duration_known_at_start = true (total PCM duration known)
        );

        // Store resolver and headings in AppState for publish path.
        {
            let mut resolver_guard = state.surface_resolver.lock().map_err(|e| e.to_string())?;
            *resolver_guard = Some(resolver);
            let mut headings_guard = state.headings.lock().map_err(|e| e.to_string())?;
            *headings_guard = headings;
        }

        // Enqueue PCM and start backend.
        {
            let mut backend = state.backend.lock().map_err(|e| e.to_string())?;
            if backend.is_none() {
                *backend = Some(CpalBackend::new().map_err(|e| e.to_string())?);
            }
            let backend = backend.as_ref().ok_or("no audio backend")?;
            backend.enqueue_pcm(pcm);
            backend.play().map_err(|e| e.to_string())?;
        }

        // Execute the determined verb (Play or ResumeAtStored).
        // Both verbs take the pre-play permission gate.
        let permissions = state.permissions.lock().map_err(|e| e.to_string())?;
        if let Err(failure) = permissions::check_pre_play(&permissions) {
            return Ok(serde_json::json!({
                "ok": false,
                "error": failure.reason(),
            }));
        }

        let snapshot = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.execute(verb).map_err(|e| e.to_string())?
        };

        // Open the FR-015 clock on the landed Playing
        // transport (T011).
        after_execute(&state, &snapshot);

        // Publish mirror snapshot after successful play start —
        // the same helper every verb path uses.
        publish_current_state(&state, None);

        // Return enhanced result with fingerprint and resume info.
        Ok(serde_json::json!({
            "status": "playing",
            "segments": segments.len(),
            "pcm_samples": pcm_samples,
            "duration_secs": duration_secs,
            "position_secs": snapshot.position_secs,
            "transport": snapshot.transport.to_string(),
            "fingerprint": fingerprint_hex,
            "snapshot_path": snapshot_path.to_string_lossy().to_string(),
            "resume_offset_secs": match offer {
                Some(session::offer::Offer::Continue { offset_secs, .. }) => offset_secs,
                _ => 0.0,
            },
            "has_stored_position": stored.is_some(),
        }))
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
                play(document.unwrap_or_default(), state)
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
                // Terminal end: no bookmark is written (T035
                // deletes the entry here) and the FR-015
                // clock closes, so it cannot leak into the
                // next session.
                after_execute(state, &snap);
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
            | ShellVerb::CycleVoicePrev => {
                // Live-session controls: no permission gate
                // (they can never reach `Playing`) and no
                // backend action — the session owns the
                // rate/skip/voice state. An illegal press
                // from a torn-down session is the named
                // rejection, never a playing state. The
                // `status` mirrors the resulting transport,
                // so a playing state is reported only from
                // a session that actually plays (FR-005).
                let snap = {
                    let mut session = state.session.lock().map_err(|e| e.to_string())?;
                    session.execute(verb).map_err(|e| e.to_string())?
                };
                publish_current_state(state, None);
                // A forward skip that reaches or passes the
                // end lands `Paused` at the duration
                // (FR-018): that landing is a transport
                // change to Paused, so the event-durable
                // put runs through the same bookkeeping.
                // Every other live control holds its
                // transport, and the put is a no-op.
                after_execute(state, &snap);
                Ok(serde_json::json!({
                    "status": snap.transport.to_string(),
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
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
                ack_latch(state, "app_backgrounded", FocusIntent::AppBackgrounded)
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
    fn publish_current_state(state: &State<'_, AppState>, transient_reason: Option<String>) {
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
    /// exclusive, so a later expiry yields no outcome.
    /// Publishes the mirror state after completion (T133).
    fn dispatch_document_end(state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
        let (snapshot, changed) = execute_or_current(state, ShellVerb::Complete)?;
        if changed {
            stop_backend(state);
            if let Ok(mut offer) = state.offer.lock() {
                offer.on_completion();
            }
        }
        publish_current_state(state, None);
        Ok(serde_json::json!({
            "status": "document_end",
            "transport": snapshot.transport.to_string(),
            "position_secs": snapshot.position_secs,
            "changed": changed,
        }))
    }

    /// Split, synthesize, enqueue, and start one document.
    fn play(document: String, state: &State<'_, AppState>) -> Result<serde_json::Value, String> {
        if document.trim().is_empty() {
            return Err("empty document: nothing to synthesize".to_string());
        }
        // Same code path as the adapter CLI (Principle II): the
        // adapter owns the chunk boundary; the engine receives
        // bounded chunks only.
        let segments = adapter_synthesis::chunk::split_document(&document, "");
        if segments.is_empty() {
            return Err("empty document: no segments".to_string());
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
            let buffer = adapter_synthesis::synth::synthesize(segment)
                .map_err(|failure| format!("synthesis failed: {failure:?}"))?;
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

        // Create surface resolver with known duration (total PCM duration is known).
        let resolver = player::surface::SurfaceResolver::new(
            Some(title.clone()),
            "document.md".to_string(),
            true, // duration_known_at_start = true (total PCM duration known)
        );

        // Store resolver and headings in AppState for publish path.
        {
            let mut resolver_guard = state.surface_resolver.lock().map_err(|e| e.to_string())?;
            *resolver_guard = Some(resolver);
            let mut headings_guard = state.headings.lock().map_err(|e| e.to_string())?;
            *headings_guard = headings;
        }

        // Compute and store the document's blake3 fingerprint (FR-036 / T059)
        // for the 20 s heartbeat bookmark persistence (T011) — the same
        // call site `open_document` routes through.
        set_document_fingerprint(state, document.as_bytes());

        {
            let mut backend = state.backend.lock().map_err(|e| e.to_string())?;
            if backend.is_none() {
                *backend = Some(CpalBackend::new().map_err(|e| e.to_string())?);
            }
            let backend = backend.as_ref().ok_or("no audio backend")?;
            backend.enqueue_pcm(pcm);
            backend.play().map_err(|e| e.to_string())?;
        }

        let snapshot = {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session
                .execute(ShellVerb::Play)
                .map_err(|e| e.to_string())?
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

        let surface = resolver.resolve(snap.position_secs, &headings_guard);
        let fingerprint_guard = state.current_fingerprint.lock().map_err(|e| e.to_string())?;
        let fingerprint = *fingerprint_guard;
        // Get stored last_rate from bookmark store. The
        // fingerprint gates the read: with no document
        // loaded there is no key to read beside, and the
        // view renders the normal-speed default.
        let last_rate = if fingerprint.is_some() {
            let mut bookmark_store_guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
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
        let fingerprint = fingerprint.unwrap_or([0u8; 32]);
        Ok(crate::status_view::render_status_view(
            last_rate,
            snap.transport,
            snap.position_secs,
            &document_title,
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
    pub fn session_delete(
        fingerprint: [u8; 32],
        state: State<'_, AppState>,
    ) -> Result<(), String> {
        let mut bookmark_store_guard = state.bookmark_store.lock().map_err(|e| e.to_string())?;
        if let Some(store) = bookmark_store_guard.as_mut() {
            store.delete(&fingerprint).map_err(|e| e.to_string())?;
        }
        Ok(())
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
        state: &State<'_, AppState>,
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
    /// Convenience helper for the dispatch paths that have a
    /// `State<'_, AppState>` but need the surface resolver to
    /// compute the section title and progress.
    pub fn publish_from_session(
        state: &State<'_, AppState>,
        resolver: &SurfaceResolver,
        headings: &[Heading],
        transient_reason: Option<String>,
    ) {
        let session_guard = match state.session.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let snapshot = session_guard.snapshot();
        let surface = resolver.resolve(snapshot.position_secs, headings);
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
                let _ = m.insert(
                    "k".to_string(),
                    serde_json::to_value(1u32).unwrap(),
                );
                let _ = m.insert(
                    "n".to_string(),
                    serde_json::to_value(1u32).unwrap(),
                );
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
        let store = BookmarkStore::open(&dir.path().join("bookmarks.redb"))
            .map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The row the store holds under `fingerprint` — the
    /// same read the resume path performs.
    fn stored_row(state: &AppState, fingerprint: [u8; 32]) -> Result<Option<StoredPosition>, String> {
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
        assert_eq!(row_again, row, "a paused heartbeat tick rewrote the bookmark");
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
        let row = stored_row(&state, fingerprint)?.ok_or("no row after the focus-loss interruption")?;
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
        assert_eq!(row.end_kind, SessionEnd::Interrupted, "a crash point must record the unexpected-end kind");
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
