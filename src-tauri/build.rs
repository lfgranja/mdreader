//! Build script for Tauri app.
//
//! Copies the status view into the served bundle, generates the Android
//! project via `tauri_build`, then applies the FR-011 debug ingest overlay
//! so `DebugIngestActivity` is declared and `android:debuggable` is set on
//! the generated Android project (needed for ADB-based TTFA validation).

use std::process::Command;

fn main() {
    // Copy status_view.html to dist/index.html for served bundle identity (005)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let src = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("status_view.html");
    let dist_dir = std::path::Path::new(&manifest_dir).join("..").join("dist");
    let _ = std::fs::create_dir_all(&dist_dir);
    let dst = dist_dir.join("index.html");
    if src.exists() {
        let _ = std::fs::copy(&src, &dst);
    }
    println!("cargo:rerun-if-changed=src/status_view.html");

    // Generate the Android project (this creates/overwrites src-tauri/gen/)
    tauri_build::build();

    // Apply FR-011 debug ingest overlay: declare DebugIngestActivity and
    // android:debuggable in the generated Android project so ADB automation
    // can inject documents and read the diagnostic record on device.
    let overlay_script = std::path::Path::new(&manifest_dir)
        .join("..")
        .join("scripts")
        .join("apply-debug-ingest-overlay.sh");
    if overlay_script.exists() {
        let _ = Command::new("bash").arg(overlay_script).status();
    }

    // Ensure android:debuggable="true" on the generated debug manifest so
    // `run-as` works on device for reading the diagnostic record.
    let debug_manifest = std::path::Path::new(&manifest_dir)
        .join("..")
        .join("gen")
        .join("android")
        .join("app")
        .join("src")
        .join("debug")
        .join("AndroidManifest.xml");
    if let Ok(content) = std::fs::read_to_string(&debug_manifest)
        && !content.contains("android:debuggable=\"true\"")
    {
        let updated = content.replace("<application>", "<application android:debuggable=\"true\">");
        let _ = std::fs::write(&debug_manifest, updated);
    }
    println!("cargo:rerun-if-changed=../scripts/apply-debug-ingest-overlay.sh");
}
