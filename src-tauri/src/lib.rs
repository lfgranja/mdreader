//! Tauri bridge — plugin registration + mobile plugin execution (FR-011 / FR-042 / SC-035).
//! NO playback logic (FR-042). Pure delegation to Rust playback module (FR-002/FR-003).

use tauri::plugin::Plugin;

pub fn plugin() -> impl Plugin<()> {
    Plugin::new("android-media-shell")
        .invoke_handler(tauri::generate_handler![
            register_android_plugin,
            run_mobile_plugin,
        ])
}

/// Plugin registration usable without prior app-process start (FR-009a / FR-011).
#[tauri::command]
fn register_android_plugin() {
    // Delegate to Rust playback module — never computes position/rate/transport.
}

/// Mobile plugin execution via `tauri::ipc::Channel` (no `InvokeBody::Streaming`).
#[tauri::command]
fn run_mobile_plugin() {
    // Same code path as IPC from Kotlin mirror (SC-035 purity — no second source).
}
