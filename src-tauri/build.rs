//! Tauri build script.
//!
//! Generates the Tauri application context and the Android
//! build scaffolding. The Android manifest is hand-edited
//! for the `mediaPlayback` foreground service — Tauri 2
//! exposes no plugin API for it (`mobile::update_manifest`
//! does not exist) — so `scripts/check-manifest.sh`
//! re-verifies the built manifest on every build.

fn main() {
    tauri_build::build()
}
