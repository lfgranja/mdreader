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

/// Shared app state: one playback session, one lazily-opened
/// audio backend.
pub struct AppState {
    /// The single playback session (sole position authority).
    session: Mutex<PlaybackSession>,
    /// The audio backend, opened on first play.
    backend: Mutex<Option<CpalBackend>>,
}

#[cfg_attr(target_os = "android", tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(PlaybackSession::new()),
            backend: Mutex::new(None),
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::register_android_plugin,
            commands::run_mobile_plugin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod commands {
    //! Plugin commands exposed via IPC.

    use super::{AppState, CpalBackend};
    use player::cli::ShellVerb;
    use tauri::{AppHandle, Manager, State, command};

    /// Declared sample rate of the engine handoff (mono f32).
    const DECLARED_RATE_HZ: f64 = 22_050.0;

    #[command]
    pub fn register_android_plugin() {
        // The Kotlin PlaybackService self-bootstraps from the
        // Application/Activity and reaches Rust through
        // `run_mobile_plugin` below — never computing
        // position/rate/transport itself.
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
            "play" => play(document.unwrap_or_default(), &state),
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
