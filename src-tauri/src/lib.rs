//! Tauri bridge — IPC delegation to the Rust playback path.
//!
//! Every surface (notification, lock screen, CLI, IPC) executes the
//! same Rust code (Principle II): `run_mobile_plugin` splits the
//! document with the adapter, synthesizes it with the bound
//! offline engine, enqueues the PCM on the CPAL backend, and
//! advances the single playback session. No transport, rate, or
//! position logic lives in Kotlin or the frontend.

use std::sync::Mutex;

use player::backend::CpalBackend;
use player::cli::PlaybackSession;
use tauri::Manager;

/// The permission-state gate (T030b): the pure,
/// headless half of FR-008/FR-010/FR-021. The Kotlin
/// shell reads the platform permissions and pushes the
/// answer as a `PermissionSnapshot`; the startup and
/// pre-play gates consume it. No Android API calls live
/// here (Principle V).
pub mod permissions;

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
/// audio backend, and the platform permission snapshot the
/// Kotlin shell pushes.
#[derive(Debug)]
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
            commands::update_permissions
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
    use player::cli::ShellVerb;
    use tauri::{AppHandle, Manager, State, command};

    /// Declared sample rate of the engine handoff (mono f32).
    const DECLARED_RATE_HZ: f64 = 22_050.0;

    /// Bind the native Android plugin surface.
    ///
    /// The Kotlin `PlaybackService` self-bootstraps from
    /// the Application/Activity and reaches Rust through
    /// `run_mobile_plugin` below — never computing
    /// position, rate, or transport itself (SC-035).
    #[command]
    pub fn register_android_plugin() {
        // The Kotlin PlaybackService self-bootstraps from the
        // Application/Activity and reaches Rust through
        // `run_mobile_plugin` below — never computing
        // position/rate/transport itself.
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

    /// Execute one playback action over IPC.
    ///
    /// * `play` requires `document` (Markdown source): splits,
    ///   synthesizes, enqueues, and starts playback, returning
    ///   segment/sample counts and the session snapshot.
    /// * `pause` / `stop` act on the live session and backend.
    #[command]
    pub fn run_mobile_plugin(
        action: String,
        document: Option<String>,
        app: AppHandle,
        state: State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match action.as_str() {
            "play" => {
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
                play(document.unwrap_or_default(), &state)
            }
            "diagnose" => Ok(diagnose(&app)),
            "pause" => {
                if let Ok(backend) = state.backend.lock() {
                    if let Some(backend) = backend.as_ref() {
                        backend.pause();
                    }
                }
                let mut session = state.session.lock().map_err(|e| e.to_string())?;
                let snap = session
                    .execute(ShellVerb::Pause)
                    .map_err(|e| e.to_string())?;
                Ok(serde_json::json!({
                    "status": "paused",
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            "stop" => {
                if let Ok(backend) = state.backend.lock() {
                    if let Some(backend) = backend.as_ref() {
                        backend.stop();
                    }
                }
                let mut session = state.session.lock().map_err(|e| e.to_string())?;
                let snap = session
                    .execute(ShellVerb::Stop)
                    .map_err(|e| e.to_string())?;
                Ok(serde_json::json!({
                    "status": "stopped",
                    "position_secs": snap.position_secs,
                    "transport": snap.transport.to_string(),
                }))
            }
            _ => Err(format!("unknown action: {action}")),
        }
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
        let mut pcm = Vec::new();
        for segment in &segments {
            let buffer = adapter_synthesis::synth::synthesize(segment)
                .map_err(|failure| format!("synthesis failed: {failure:?}"))?;
            pcm.extend_from_slice(&buffer);
        }
        let pcm_samples = pcm.len();
        let duration_secs = pcm_samples as f64 / DECLARED_RATE_HZ;

        {
            let mut backend = state.backend.lock().map_err(|e| e.to_string())?;
            if backend.is_none() {
                *backend = Some(CpalBackend::new().map_err(|e| e.to_string())?);
            }
            let backend = backend.as_ref().ok_or("no audio backend")?;
            backend.enqueue_pcm(pcm);
            backend.play().map_err(|e| e.to_string())?;
        }

        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        let snapshot = session
            .execute(ShellVerb::Play)
            .map_err(|e| e.to_string())?;
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
