//! Build script for Tauri app.

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
    tauri_build::build()
}
