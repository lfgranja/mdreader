//! Emit the packaged voice-asset manifest (host-side).
//!
//! Walks `assets/voices/` (all bundled voices + shared resources) and
//! writes one relative path per line to `$OUT_DIR/voice_manifest.txt`,
//! embedded by the Android extractor via `include_str!`. Directory
//! listing on device proved unreliable (nested `open_dir` yields
//! nothing on the test device while `open()` on explicit file paths
//! works), so the copy follows this manifest instead of walking the
//! asset tree.

use std::path::{Path, PathBuf};

/// Voice catalog tree packaged into the APK assets.
const VOICES_ROOT: &str = "assets/voices";

fn main() {
    println!("cargo::rerun-if-changed={VOICES_ROOT}");
    let root = Path::new(VOICES_ROOT);
    let mut entries = Vec::new();
    walk(root, root, &mut entries);
    entries.sort();
    let out =
        PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("voice_manifest.txt");
    let content = entries.join("\n");
    std::fs::write(&out, content).unwrap_or_default();
}

/// Collect file paths relative to `VOICES_ROOT`, skipping nothing:
/// every regular file ships, directories are recreated from the
/// file parents on device.
fn walk(root: &Path, dir: &Path, entries: &mut Vec<String>) {
    let Ok(listed) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in listed.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, entries);
        } else if let Ok(relative) = path.strip_prefix(root)
            && let Some(name) = relative.to_str()
        {
            entries.push(name.to_owned());
        }
    }
}
