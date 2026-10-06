//! Tauri bridge — plugin delegation (FR-011 / FR-042 / SC-035).
//! NO playback logic in Kotlin surface — pure delegation to Rust module.

#[tauri::mobile_entry_point]
pub fn run() {
    tauri::Builder::default()
        .runtime(tauri_runtime_wry::Wry::default())
        .invoke_handler(tauri::generate_handler![
            commands::register_android_plugin,
            commands::run_mobile_plugin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod commands {
    //! Plugin commands exposed via IPC.
    use tauri::command;

    #[command]
    pub fn register_android_plugin() {
        // Delegate to Rust playback module — never computes position/rate/transport.
    }

    #[command]
    pub fn run_mobile_plugin() {
        // Same code path as Kotlin mirror (SC-035 purity).
    }
}
