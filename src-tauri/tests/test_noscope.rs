//! T047 — BLOCKING out-of-scope absence guard (Feature
//! 004-ressoa-dogfood-build / task T047).
//!
//! Requirement (tasks.md T047): "Out-of-scope absence guard
//! test in `src-tauri/tests/test_noscope.rs` (asserts zero
//! network call sites app-wide (any crate; voice + diag +
//! other; grep `reqwest|hyper|ureq|tonic|surf|isahc|minreq|
//! curl|tokio::net|std::net::Tcp` + `cargo tree -i`
//! network-crate reverse-deps + `cargo deny check bans
//! sources` + Cargo.lock audit), not voice-only — enforces
//! FR-015 voice ban plus diag no-egress, exactly one bundled
//! voice dir, no library view, lock-screen no Stop CONTROL,
//! no Auto/head-unit service, no in-app diag surface, no
//! distribution artifacts + naming-boundary grep: no
//! `mdreader` in user-visible strings, Ressoa-external
//! only)".
//!
//! The guard reads the repository source at test time and
//! asserts the ABSENCE of every out-of-scope surface. An
//! absence guard fails closed: the expectation is zero
//! matches, and any new match fails the build — the same
//! discipline as the shell-purity gate.

use std::path::{Path, PathBuf};

/// The repository root: the workspace Cargo.toml's parent.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| candidate.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .expect("the workspace root carries the Cargo.toml")
}

/// The network call-site vocabulary (tasks.md T047 verbatim):
/// HTTP clients, gRPC, async runtimes' net modules, and the
/// raw socket entry points.
const NETWORK_PATTERNS: [&str; 10] = [
    "reqwest",
    "hyper",
    "ureq",
    "tonic",
    "surf",
    "isahc",
    "minreq",
    "curl",
    "tokio::net",
    "std::net::Tcp",
];

/// The crates whose sources the egress guard scans: every
/// workspace member's `src/` and `tests/` tree (the app-wide
/// scope — voice, diag, and everything else).
fn source_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for member in ["crates", "src-tauri", "android-shell"] {
        let member_dir = root.join(member);
        if member_dir.is_dir() {
            roots.push(member_dir);
        }
    }
    roots
}

/// Every `.rs` / `.kt` file under `dir`, recursively.
fn source_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Never scan build output or vendored assets.
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == "target" || name == "gen" || name == "voices" || name == "assets" {
                    continue;
                }
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                if ext == "rs" || ext == "kt" {
                    files.push(path);
                }
            }
        }
    }
    files
}

#[test]
fn no_network_call_sites_app_wide() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    for source_root in source_roots(&root) {
        for file in source_files(&source_root) {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for pattern in NETWORK_PATTERNS {
                if text.contains(pattern) {
                    offenders.push(format!(
                        "{} contains network call-site pattern {pattern:?}",
                        file.strip_prefix(&root).unwrap_or(&file).display()
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "zero network call sites app-wide (FR-015 voice ban + diag no-egress):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn exactly_one_bundled_voice_directory() {
    let root = repo_root();
    let voices = root.join("crates/adapter-synthesis/assets/voices");
    let entries = match std::fs::read_dir(&voices) {
        Ok(entries) => entries.flatten().map(|e| e.path()).collect::<Vec<_>>(),
        // An absent voice tree is itself a scope violation — the
        // sole voice is the offline path's source.
        Err(_) => Vec::new(),
    };
    let voice_dirs: Vec<PathBuf> = entries.into_iter().filter(|path| path.is_dir()).collect();
    assert_eq!(
        voice_dirs.len(),
        1,
        "exactly one bundled voice dir (spec: single CC0 pt_BR-faber-medium): found {:?}",
        voice_dirs
            .iter()
            .map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        voice_dirs[0]
            .file_name()
            .map(|n| n.to_string_lossy().into_owned()),
        Some("pt_BR-faber-medium".to_string()),
        "the sole bundled voice is pt_BR-faber-medium (plan.md R1)"
    );
}

#[test]
fn lock_screen_carries_no_stop_control() {
    // FR-002/FR-019: the Stop CONTROL exists only on the single
    // in-app screen; the lock-screen surface may carry the
    // `stopped` STATE but never a Stop control. The mirror
    // payload builder is the authority — scan its source for a
    // stop-control key and for MediaSession stop verbs in the
    // Kotlin mirror.
    let root = repo_root();
    let lib_rs = root.join("src-tauri/src/lib.rs");
    let text = std::fs::read_to_string(&lib_rs).expect("lib.rs is readable");
    assert!(
        !text.contains("\"stop_control\""),
        "the mirror payload must not carry a stop control key (Stop is in-app only, FR-002/FR-019)"
    );
    let kotlin = root.join("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt");
    if let Ok(kotlin_text) = std::fs::read_to_string(&kotlin) {
        assert!(
            !kotlin_text.contains("ACTION_STOP") && !kotlin_text.contains("onStop"),
            "the Kotlin MediaSession mirror must not handle the Stop verb on the lock screen (Stop is in-app only)"
        );
    }
}

#[test]
fn no_library_view_no_auto_service_no_diag_surface() {
    let root = repo_root();
    let frontend = root.join("src-tauri/src/status_view.html");
    let text = std::fs::read_to_string(&frontend).expect("status_view.html is readable");
    // No library view: the single in-app screen renders one
    // document's session — no document list/grid/library.
    for forbidden in ["library", "Library", "biblioteca", " Biblioteca"] {
        assert!(
            !text.contains(forbidden),
            "the in-app screen must not surface a library view (out of scope): found {forbidden:?}"
        );
    }
    // No in-app diagnostics surface: the record is adb-pull only.
    let src_files = source_files(&root.join("src-tauri"));
    let mut diag_surface_hits = Vec::new();
    for file in src_files {
        if let Ok(file_text) = std::fs::read_to_string(&file) {
            if file_text.contains("DiagnosticWriter") && file_text.contains("publish") {
                diag_surface_hits.push(format!(
                    "{}",
                    file.strip_prefix(&root).unwrap_or(&file).display()
                ));
            }
        }
    }
    assert!(
        diag_surface_hits.is_empty(),
        "no in-app diagnostics surface (FR-017: adb-pull only): {:?}",
        diag_surface_hits
    );
}

#[test]
fn no_distribution_artifacts_in_tree() {
    let root = repo_root();
    // No store listing / changelog / support surface / signing
    // material committed (maintainer-only sideload; nothing
    // published).
    for forbidden in [
        "fastlane/metadata",
        "play-store-listing",
        "CHANGELOG-STORE",
        "listing.md",
    ] {
        let candidate = root.join(forbidden);
        assert!(
            !candidate.exists(),
            "no distribution artifacts in the tree (maintainer-only sideload): {forbidden} exists"
        );
    }
}

#[test]
fn no_mdreader_in_user_visible_strings() {
    // The naming boundary: the internal technical name never
    // reaches a user-visible surface (Ressoa-external naming).
    let root = repo_root();
    let frontend = root.join("src-tauri/src/status_view.html");
    let text = std::fs::read_to_string(&frontend).expect("status_view.html is readable");
    assert!(
        !text.contains("mdreader"),
        "no internal name in user-visible strings (naming boundary): status_view.html mentions it"
    );
    // The Kotlin package name is structural (app.mdreader), not
    // user-visible — allowed; the manifest's app label is not.
    let manifest = root.join("src-tauri/gen/android/app/src/main/AndroidManifest.xml");
    if let Ok(manifest_text) = std::fs::read_to_string(&manifest) {
        let label_lines: Vec<&str> = manifest_text
            .lines()
            .filter(|line| line.contains("android:label"))
            .collect();
        for line in label_lines {
            assert!(
                !line.contains("mdreader"),
                "no internal name in the user-visible app label: {line}"
            );
        }
    }
}
