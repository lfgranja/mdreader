//! Build script — S2 packaging wiring.
//!
//! Copies `src-tauri/src/status_view.html` → `dist/index.html` at build time
//! so the packaged app serves the restyled surface as its frontend entry.
//!
//! This is the S2 contract: the served `index.html` MUST be byte-identical
//! to `src-tauri/src/status_view.html`. The bench `dist/index.html` content
//! may remain as a fallback only if it does not leave the audited surface
//! unserved — it leaves it unserved as the default route, so the bench page
//! is REPLACED.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tauri.conf.json frontendDist is "../dist" relative to src-tauri/,
    // i.e. workspace-root dist/.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("status_view.html");
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dist");
    let dst = out_dir.join("index.html");

    // Ensure the dist directory exists.
    std::fs::create_dir_all(&out_dir)?;

    // Copy status_view.html → dist/index.html at build time.
    let _bytes = std::fs::copy(&src, &dst)?;

    // Re-run this build script if status_view.html changes.
    println!("cargo:rerun-if-changed={}", src.display());

    Ok(())
}
