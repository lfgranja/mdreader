//! Served-identity twin — C-H8 (contract S2).
//!
//! The packaged app MUST serve `src-tauri/src/status_view.html` as its
//! frontend entry. This test asserts byte-identity between the served
//! bundle (`<frontendDist>/index.html`) and the audited surface.
//!
//! RED: this test MUST fail at T005 time because the bundle serves
//! `dist/index.html` (a tokens-less bench), which byte-differs from the
//! audited `status_view.html`. T006 wires the S2 path so this test
//! passes at acceptance.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "workspace root must exist".to_string())?
        .to_path_buf())
}

fn frontend_dist() -> Result<String, Box<dyn std::error::Error>> {
    let manifest = fs::read_to_string(workspace_root()?.join("src-tauri/tauri.conf.json"))
        .map_err(|e| format!("tauri.conf.json must exist: {e}"))?;
    // tauri.conf.json is JSON; extract the frontendDist value with a
    // targeted scan (no serde dependency — keep the twin dependency-free).
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"frontendDist\"") {
            let after_eq = trimmed.split_once(':').map_or("", |x| x.1);
            let s = after_eq.trim().trim_matches(|c| c == '"' || c == ',');
            return Ok(s.to_string());
        }
    }
    Err("frontendDist key must be present in tauri.conf.json".into())
}

/// Serve the bundle entry that the packaged app actually uses.
fn served_bundle_entry() -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    let fd = frontend_dist()?;
    let path = workspace_root()?
        .join("src-tauri")
        .join(&fd)
        .join("index.html");
    let bytes = fs::read(&path)?;
    Ok((path.to_string_lossy().to_string(), bytes))
}

/// Read the audited surface file.
fn audited_surface() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(fs::read(
        workspace_root()?.join("src-tauri/src/status_view.html"),
    )?)
}

/// The served bundle entry is byte-identical to the audited surface.
#[test]
fn surface_audit_served_identity_binds_status_view_as_bundle()
-> Result<(), Box<dyn std::error::Error>> {
    let audited = audited_surface()?;
    let served = served_bundle_entry()?;
    let (path, bytes) = served;
    assert_eq!(
        bytes, audited,
        "served bundle entry '{path}' is byte-different from the audited surface 'src-tauri/src/status_view.html' — \
         the packaged app does not serve the restyled surface; S2 wiring is incomplete"
    );
    Ok(())
}
