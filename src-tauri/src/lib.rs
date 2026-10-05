//! Tauri bridge — plugin delegation (FR-011 / FR-042 / SC-035).
//! NO playback logic in Kotlin surface — pure delegation to Rust module.

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
