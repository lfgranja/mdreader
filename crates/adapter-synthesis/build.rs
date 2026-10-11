//! Emit the packaged voice-asset manifest (host-side).
//!
//! Walks `assets/voices/pt_BR-faber-medium` and writes one relative
//! path per line to `$OUT_DIR/voice_manifest.txt`, embedded by the
//! Android extractor via `include_str!`. Directory listing on device
//! proved unreliable (nested `open_dir` yields nothing on the test
//! device while `open()` on explicit file paths works), so the copy
//! follows this manifest instead of walking the asset tree.

use std::path::{Path, PathBuf};

/// Voice tree packaged into the APK assets.
const VOICE_DIR: &str = "assets/voices/pt_BR-faber-medium";

fn main() {
    println!("cargo::rerun-if-changed={VOICE_DIR}");
    let mut entries = Vec::new();
    walk(Path::new(VOICE_DIR), Path::new(""), &mut entries);
    entries.sort();
    let out =
        PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("voice_manifest.txt");
    let content = entries.join("\n");
    std::fs::write(&out, content).unwrap_or_default();
}

/// Collect file paths relative to `VOICE_DIR`, skipping nothing:
/// every regular file ships, directories are recreated from the
/// file parents on device.
fn walk(dir: &Path, prefix: &Path, entries: &mut Vec<String>) {
    let Ok(listed) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in listed.flatten() {
        let relative = prefix.join(entry.file_name());
        if entry.path().is_dir() {
            walk(&entry.path(), &relative, entries);
        } else if let Some(name) = relative.to_str() {
            entries.push(name.to_owned());
        }
    }
}
