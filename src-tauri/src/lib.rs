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
        .runtime(tauri_runtime_wry::Wry::default())
        .manage(AppState {
            session: Mutex::new(PlaybackSession::new()),
            backend: Mutex::new(None),
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
    use tauri::{State, command};

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
        state: State<'_, AppState>,
    ) -> Result<serde_json::Value, String> {
        match action.as_str() {
            "play" => play(document.unwrap_or_default(), &state),
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
}
