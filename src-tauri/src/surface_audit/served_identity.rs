//! C-H8 served-identity twin (RED until T006 wiring lands).

use std::fs;
use std::path::Path;

/// Parse `frontendDist` from tauri.conf.json (naive, no serde dep).
fn frontend_dist(manifest_dir: &Path) -> Option<String> {
    let conf = manifest_dir.join("tauri.conf.json");
    let text = fs::read_to_string(&conf).ok()?;
    // find "frontendDist": "..."
    let key = "\"frontendDist\"";
    let idx = text.find(key)?;
    let rest = &text[idx + key.len()..];
    let q1 = rest.find('"')? + 1;
    let q2 = rest[q1..].find('"')? + q1;
    Some(rest[q1..q2].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_audit_served_identity_binds_status_view_as_bundle() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dist_rel = frontend_dist(manifest_dir).expect("frontendDist missing");
        let bundle = manifest_dir.join(&dist_rel).join("index.html");
        let src = manifest_dir.join("src/status_view.html");
        assert!(
            bundle.exists(),
            "served bundle is missing: {}",
            bundle.display()
        );
        let bundle_bytes = fs::read(&bundle).expect("read bundle");
        let src_bytes = fs::read(&src).expect("read status_view.html");
        assert_eq!(
            bundle_bytes, src_bytes,
            "served bundle is the dist/index.html bench, byte-differs from src-tauri/src/status_view.html"
        );
    }
}
