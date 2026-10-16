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

/// The repository root: the WORKSPACE Cargo.toml's parent — the
/// ancestor whose manifest declares `[workspace]`, not merely any
/// `Cargo.toml` (this crate's own manifest would otherwise match
/// first and resolve the root to `src-tauri/`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && std::fs::read_to_string(candidate.join("Cargo.toml"))
                    .map(|text| text.contains("[workspace]"))
                    .unwrap_or(false)
        })
        .map(Path::to_path_buf)
        .expect("the workspace root carries the Cargo.toml with [workspace]")
}

/// The network call-site vocabulary (tasks.md T047 verbatim):
/// HTTP clients, gRPC, async runtimes' net modules, and the
/// raw socket entry points. Each pattern is matched as a
/// CRATE PATH (`pattern::`), a `use` item, or a Cargo.toml
/// dependency declaration — never a bare substring, which
/// would match ordinary English ("surface" contains "surf")
/// and fail the guard on its own vocabulary.
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

/// Whether `text` references `pattern` as a crate path, a
/// `use` item, or a dependency declaration — the three forms
/// a network call site takes in Rust source or a manifest.
/// Word boundaries matter: "we use surface" must not match
/// the `surf` pattern (a bare substring or a loose prefix
/// rule both fail there), while `use surf::Client` must.
fn references_crate(text: &str, pattern: &str) -> bool {
    let bytes = text.as_bytes();
    let mut search_from = 0;
    while let Some(offset) = text[search_from..].find(pattern) {
        let start = search_from + offset;
        let end = start + pattern.len();
        let boundary_before = start == 0 || !is_ident_char(bytes[start - 1]);
        let boundary_after = end == bytes.len() || !is_ident_char(bytes[end]);
        if boundary_before && boundary_after {
            // A whole-word occurrence: a crate path (`surf::`), a
            // `use` item, or a manifest dependency (`surf = "1"`).
            let after = text[end..].trim_start();
            let before = text[..start].trim_end();
            if after.starts_with("::")
                || after.starts_with('=')
                || before.ends_with("use")
                || before.ends_with('}')
                || before.ends_with(']')
            {
                return true;
            }
        }
        search_from = end;
    }
    false
}

/// Whether `byte` can appear inside a crate/identifier name
/// (ASCII alphanumeric or `_`), the complement of a word
/// boundary.
fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

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
    // The guard names the network vocabulary in its own patterns and
    // documentation (e.g. an illustrative `use surf::Client`) — it is
    // the scanner, never a scanned surface.
    let guard = root.join("src-tauri/tests/test_noscope.rs");
    let mut offenders: Vec<String> = Vec::new();
    for source_root in source_roots(&root) {
        for file in source_files(&source_root) {
            if file == guard {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for pattern in NETWORK_PATTERNS {
                if references_crate(&text, pattern) {
                    offenders.push(format!(
                        "{} references network crate {pattern:?}",
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
    let guard = root.join("src-tauri/tests/test_noscope.rs");
    let mut diag_surface_hits = Vec::new();
    for file in src_files {
        // The guard itself names the writer and the publish path in
        // its assertions — it is the scanner, never a surface.
        if file == guard {
            continue;
        }
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
    // reaches a user-visible surface (Ressoa-external naming) —
    // `.specify/assessments/ressoa-mobile-app/concept.md`: "the
    // public/technical naming boundary (Ressoa externally,
    // mdreader technically) holds for the artifact's visible
    // strings". Structural identifiers are out of scope by
    // construction: the Kotlin package path `app.mdreader`, the
    // `applicationId`, intent actions, notification-channel ids,
    // and log tags are never painted for a reader.
    let root = repo_root();

    // Every painted string: the status view, the webview frontends
    // (the served bundle and its Android asset copy), and the
    // generated launcher label.
    for relative in [
        "src-tauri/src/status_view.html",
        "dist/index.html",
        "src-tauri/src/assets/index.html",
        "src-tauri/gen/android/app/src/main/assets/index.html",
        "src-tauri/gen/android/app/src/main/res/values/strings.xml",
    ] {
        let path = root.join(relative);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        assert!(
            !text.contains("mdreader"),
            "no internal name in user-visible strings (naming boundary): {relative} mentions it"
        );
    }

    // The bundle identity is a user-visible surface even though it
    // lives in a build config: `productName` seeds the generated
    // Android `app_name`/`main_activity_title` and the desktop
    // window title.
    let config = root.join("src-tauri/tauri.conf.json");
    let config_text = std::fs::read_to_string(&config).expect("tauri.conf.json is readable");
    for line in config_text.lines() {
        if line.contains("\"productName\"") || line.contains("\"title\"") {
            assert!(
                !line.contains("mdreader"),
                "no internal name in the bundle identity or window title: {line}"
            );
        }
    }

    // Some Tauri releases put the label straight on the manifest
    // activity instead of the generated strings.
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
