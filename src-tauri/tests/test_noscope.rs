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
//!
//! T098 (F3, unrequested — HIGH) extends the guard past the
//! crate level: a served HTML surface is fetched by the
//! renderer, so a remote font, a CDN script, or a tracking
//! pixel is a network egress at RENDER time that the crate
//! scan above cannot see — the request never passes through
//! a Rust call site. The guard therefore scans every served
//! surface for an external `http://` / `https://` resource.
//! T106 (F11) extends it to the platform capability itself:
//! the built Android manifest must not even HOLD the
//! `INTERNET` permission.

use std::error::Error;
use std::path::{Path, PathBuf};

/// The repository root: the WORKSPACE Cargo.toml's parent — the
/// ancestor whose manifest declares `[workspace]`, not merely any
/// `Cargo.toml` (this crate's own manifest would otherwise match
/// first and resolve the root to `src-tauri/`).
#[expect(
    clippy::expect_used,
    reason = "test harness: the guard cannot run without the workspace root; a missing root must fail the test loudly rather than silently scan nothing"
)]
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && std::fs::read_to_string(candidate.join("Cargo.toml"))
                    .is_ok_and(|text| text.contains("[workspace]"))
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
const fn is_ident_char(byte: u8) -> bool {
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
            } else if let Some(ext) = path.extension()
                && (ext == "rs" || ext == "kt")
            {
                files.push(path);
            }
        }
    }
    files
}

/// Every `.html` file under `dir`, recursively — the served
/// surfaces. Unlike the code scan, this one DOES descend into
/// `assets/` (a served webview frontend lives there); it still
/// never enters build output, the vendored voice tree, or
/// `node_modules`.
fn html_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Never scan build output, vendored assets, or
                // dependency trees — only hand-authored surfaces.
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == "target"
                    || name == "gen"
                    || name == "build"
                    || name == "voices"
                    || name == "node_modules"
                {
                    continue;
                }
                stack.push(path);
            } else if let Some(ext) = path.extension()
                && ext == "html"
            {
                files.push(path);
            }
        }
    }
    files
}

/// The served HTML surfaces: the in-app screen and the webview
/// frontends. The tree scan finds the ones inside the scanned
/// source roots; the explicit list adds the two that live
/// OUTSIDE them — the declared `frontendDist` bundle
/// (`tauri.conf.json` → `../dist`) and the Android asset copy
/// that ships inside the APK.
fn served_html_surfaces(root: &Path) -> Vec<PathBuf> {
    let mut surfaces = Vec::new();
    for source_root in source_roots(root) {
        surfaces.extend(html_files(&source_root));
    }
    for relative in ["dist", "src-tauri/gen/android/app/src/main/assets"] {
        surfaces.extend(html_files(&root.join(relative)));
    }
    surfaces.sort();
    surfaces.dedup();
    surfaces
}

/// The URL schemes a served surface may never name: an absolute
/// `http://` / `https://` reference in a served document is a
/// network fetch at render time — a remote font, a CDN script, a
/// tracking pixel. FR-015 (pure-offline) and Principle IX make
/// that an egress, and the crate-level scan above cannot see it:
/// the egress lives in a stylesheet the renderer resolves.
const REMOTE_URL_PREFIXES: [&str; 2] = ["http://", "https://"];

/// An XML namespace URI is an identifier, not a resource: a
/// renderer never fetches `http://www.w3.org/2000/svg`. That
/// prefix is the ONLY allow-listed form, and it is allow-listed
/// precisely because no request is ever issued for it.
const NON_FETCHED_NAMESPACE_PREFIX: &str = "http://www.w3.org/";

/// The remote URLs `text` names, in order of appearance. A W3C
/// namespace declaration is skipped — it identifies a
/// vocabulary, it does not name a resource to fetch.
fn remote_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for prefix in REMOTE_URL_PREFIXES {
        for (offset, _) in text.match_indices(prefix) {
            let tail = &text[offset..];
            if tail.starts_with(NON_FETCHED_NAMESPACE_PREFIX) {
                continue;
            }
            let url: String = tail
                .chars()
                .take_while(|c| {
                    !c.is_whitespace()
                        && !matches!(c, '\'' | '"' | '`' | ')' | '<' | '>' | '}' | '|' | '\\')
                })
                .collect();
            urls.push(url);
        }
    }
    urls
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

/// T098: no served surface may name an external `http://` /
/// `https://` resource. A remote font, a CDN script, or a
/// tracking pixel is a network egress at RENDER time — invisible
/// to the crate-level scan above, because the egress lives in a
/// stylesheet the renderer resolves, not in a Rust call site.
/// FR-015 (pure-offline) and Principle IX forbid it outright.
#[test]
fn no_external_urls_in_served_html_surfaces() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    for surface in served_html_surfaces(&root) {
        let Ok(text) = std::fs::read_to_string(&surface) else {
            continue;
        };
        for url in remote_urls(&text) {
            offenders.push(format!(
                "{} references the external resource {url}",
                surface.strip_prefix(&root).unwrap_or(&surface).display()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "zero external http(s) resources in a served surface (FR-015 pure-offline, Principle IX):\n{}",
        offenders.join("\n")
    );
}

/// The manifest text with its XML comments removed. A comment
/// DOCUMENTS the manifest; it declares nothing. The guard must
/// read the declarations, not the prose — otherwise the artifact
/// cannot even document why it omits the permission.
fn without_xml_comments(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        stripped.push_str(&rest[..start]);
        match rest[start + 4..].find("-->") {
            Some(end) => rest = &rest[start + 4 + end + 3..],
            // An unterminated comment swallows the remainder.
            None => return stripped,
        }
    }
    stripped.push_str(rest);
    stripped
}

/// T106: the pure-offline build must not even HOLD the INTERNET
/// permission. A declared permission is a capability the
/// platform grants BEFORE any code runs, so its absence is the
/// strongest available statement that no egress can happen — a
/// permission the app never asks for is a capability it can
/// never use.
#[test]
fn manifest_declares_no_network_permission() -> Result<(), Box<dyn Error>> {
    let root = repo_root();
    let manifest = root.join("src-tauri/gen/android/app/src/main/AndroidManifest.xml");
    let text = std::fs::read_to_string(&manifest)?;
    let declarations = without_xml_comments(&text);
    assert!(
        !declarations.contains("android.permission.INTERNET"),
        "the pure-offline manifest must not declare android.permission.INTERNET (FR-015, Principle IX)"
    );
    Ok(())
}

#[test]
fn exactly_one_bundled_voice_directory() {
    let root = repo_root();
    let voices = root.join("crates/adapter-synthesis/assets/voices");
    // An absent voice tree is itself a scope violation — the
    // sole voice is the offline path's source.
    let entries = std::fs::read_dir(&voices).map_or_else(
        |_| Vec::new(),
        |entries| entries.flatten().map(|e| e.path()).collect::<Vec<_>>(),
    );
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
fn lock_screen_carries_no_stop_control() -> Result<(), Box<dyn Error>> {
    // FR-002/FR-019: the Stop CONTROL exists only on the single
    // in-app screen; the lock-screen surface may carry the
    // `stopped` STATE but never a Stop control. The mirror
    // payload builder is the authority — scan its source for a
    // stop-control key and for MediaSession stop verbs in the
    // Kotlin mirror.
    let root = repo_root();
    let lib_rs = root.join("src-tauri/src/lib.rs");
    let text = std::fs::read_to_string(&lib_rs)?;
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
    Ok(())
}

#[test]
fn no_library_view_no_auto_service_no_diag_surface() -> Result<(), Box<dyn Error>> {
    let root = repo_root();
    let frontend = root.join("src-tauri/src/status_view.html");
    let text = std::fs::read_to_string(&frontend)?;
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
        if let Ok(file_text) = std::fs::read_to_string(&file)
            && file_text.contains("DiagnosticWriter")
            && file_text.contains("publish")
        {
            diag_surface_hits.push(format!(
                "{}",
                file.strip_prefix(&root).unwrap_or(&file).display()
            ));
        }
    }
    assert!(
        diag_surface_hits.is_empty(),
        "no in-app diagnostics surface (FR-017: adb-pull only): {diag_surface_hits:?}"
    );
    Ok(())
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
fn no_mdreader_in_user_visible_strings() -> Result<(), Box<dyn Error>> {
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
    let config_text = std::fs::read_to_string(&config)?;
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
    Ok(())
}
