//! Tauri build script.
//!
//! Generates the Tauri application context and the Android
//! build scaffolding. The Android manifest is hand-edited
//! for the `mediaPlayback` foreground service — Tauri 2
//! exposes no plugin API for it (`mobile::update_manifest`
//! does not exist) — so `scripts/check-manifest.sh`
//! re-verifies the built manifest on every build.

fn main() {
    // S2 packaging wiring (005): the packaged app serves
    // src-tauri/src/status_view.html as the bundle entry. Copy
    // it to dist/index.html at build time so the served-identity
    // twin stays green.
    let src = std::path::Path::new("src/status_view.html");
    let dst_dir = std::path::Path::new("../dist");
    let _ = std::fs::create_dir_all(dst_dir);
    let dst = dst_dir.join("index.html");
    if src.exists() {
        let _ = std::fs::copy(src, &dst);
    }

    tauri_build::build();
}
