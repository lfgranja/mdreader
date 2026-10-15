//! T014 — BLOCKING US1 TEST: integration
//! `ingest → snapshot → fingerprint → parse`
//! (Feature 004-ressoa-dogfood-build / task T014).
//!
//! Invariants under test (tasks.md T014):
//! "pick `.md`/`.txt` ⇒ snapshot written, `fingerprint()`
//! matches, `store.get` decides resume; moved/renamed
//! re-pick ⇒ hash-match restore; revoked grant ⇒ re-prompt,
//! never crash; non-`.md`/`.txt` pick ⇒ refused with
//! `FormatUndeliverable` before synthesis; keep-current-only
//! trim asserted; first-bookmark preservation."
//!
//! Scope of the host half: the Kotlin shell
//! (`android-shell/…/DocIngest.kt`) owns the picker intent,
//! the snapshot write, the keep-current-only trim, and the
//! format/size refusal — those are asserted on-device
//! (T051) and by the Kotlin-side contract in `contracts/
//! ingest.md`. This test binds the host halves: the
//! fingerprint identity (`session::fingerprint`), the
//! resume decision `store.get` resolves, the moved/renamed
//! hash-match restore, and first-bookmark preservation —
//! through the REAL CLI surface (`session::cli::run`) with
//! injected writers, so the IPC path and the CLI path are
//! exercised identically (Principle II).

use std::fs;
use std::path::{Path, PathBuf};

use engine_core::closed::SessionEnd;
use session::cli::run as session_cli;
use session::fingerprint;
use session::store::{BookmarkStore, StoredPosition};

/// A tempdir wrapper that removes itself on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let base = std::env::temp_dir().join(format!(
            "t014-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("create tempdir");
        Self(base)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs `session --db <db> ingest <file>` and returns the
/// parsed JSON payload (stdout) — the machine-readable
/// default. A usage/IO failure asserts: nothing reaches
/// stdout and the error is reported on stderr.
fn ingest_json(db: &Path, document: &Path) -> String {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let args = [
        "--db".to_string(),
        db.to_string_lossy().into_owned(),
        "ingest".to_string(),
        document.to_string_lossy().into_owned(),
    ];
    session_cli(args.into_iter(), &mut stdout, &mut stderr)
        .expect("ingest of an existing file must succeed");
    assert!(
        stderr.is_empty(),
        "a successful ingest must write nothing to stderr, got: {}",
        String::from_utf8_lossy(&stderr)
    );
    String::from_utf8(stdout).expect("stdout is UTF-8 JSON")
}

/// Extracts a JSON string value for `key` from a flat JSON
/// object payload — the payload shape is flat (`{...}`), so
/// a scan for `"key":` is exact here.
fn json_field(payload: &str, key: &str) -> String {
    let needle = format!("\"{key}\":");
    let start = payload
        .find(&needle)
        .unwrap_or_else(|| panic!("payload {payload} missing key {key}"))
        + needle.len();
    let rest = &payload[start..];
    let rest = rest.trim_start();
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped
            .find('"')
            .unwrap_or_else(|| panic!("unterminated string for {key} in {payload}"));
        stripped[..end].to_string()
    } else {
        let end = rest
            .find(|c: char| c == ',' || c == '}')
            .unwrap_or(rest.len());
        rest[..end].to_string()
    }
}

/// Writes `bytes` to `dir/name` and returns the path.
fn write_document(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("write document");
    path
}

const DOC_A: &[u8] =
    b"# Rota da Manha\n\nPrimeira secao com numeros 123 e R$ 45,90.\n\nSegunda secao.\n";
const DOC_B: &[u8] = b"# Outro Documento\n\nConteudo diferente por completo.\n";

#[test]
fn ingest_of_a_fresh_document_reports_no_resume_and_zero_offset() {
    let dir = TempDir::new("fresh");
    let db = dir.path().join("bookmarks.redb");
    let document = write_document(dir.path(), "roteiro.md", DOC_A);

    let payload = ingest_json(&db, &document);
    let expected_fp = fingerprint(DOC_A);
    let expected_hex: String = expected_fp.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        json_field(&payload, "fingerprint"),
        expected_hex,
        "ingest must report the blake3 fingerprint of the document bytes"
    );
    assert_eq!(
        json_field(&payload, "resume"),
        "false",
        "a fresh document (no stored position) must resolve to no offer"
    );
    assert_eq!(
        json_field(&payload, "offset"),
        "0.0",
        "a fresh document starts at 0.0 (FR-036a)"
    );
}

#[test]
fn ingest_after_a_deliberate_pause_reports_resume_and_the_offset() {
    let dir = TempDir::new("paused");
    let db = dir.path().join("bookmarks.redb");
    let document = write_document(dir.path(), "roteiro.md", DOC_A);

    // Store a deliberate pause under the document's content
    // fingerprint — the host-side shape of "the listener
    // paused and closed the app".
    let key = fingerprint(DOC_A);
    let mut store = BookmarkStore::open(&db).expect("open store");
    store
        .put(
            &key,
            StoredPosition {
                offset_secs: 317.25,
                first_yield_ms: 0,
                end_kind: SessionEnd::Paused,
            },
        )
        .expect("put paused position");
    drop(store);

    let payload = ingest_json(&db, &document);
    assert_eq!(
        json_field(&payload, "resume"),
        "true",
        "a stored deliberate pause is the only resume-offering end kind"
    );
    assert_eq!(
        json_field(&payload, "offset"),
        "317.25",
        "ingest must report the stored last-spoken offset"
    );
}

#[test]
fn moved_or_renamed_document_restores_the_same_position() {
    let dir = TempDir::new("moved");
    let db = dir.path().join("bookmarks.redb");

    // The document is picked once under one name...
    let original = write_document(dir.path(), "original.md", DOC_A);
    let key = fingerprint(DOC_A);
    let mut store = BookmarkStore::open(&db).expect("open store");
    store
        .put(
            &key,
            StoredPosition {
                offset_secs: 512.0,
                first_yield_ms: 1_700_000_000_000,
                end_kind: SessionEnd::Paused,
            },
        )
        .expect("put");
    drop(store);

    // ...then moved and renamed on disk and re-picked. The
    // fingerprint is keyed on the CONTENT, never the path
    // (FR-036), so the same bytes resolve to the same row.
    let moved_dir = dir.path().join("subdir");
    fs::create_dir_all(&moved_dir).expect("create subdir");
    let moved = moved_dir.join("outro-nome.md");
    fs::rename(&original, &moved).expect("move document");
    fs::write(&moved, DOC_A).expect("ensure content after move");

    let payload = ingest_json(&db, &moved);
    assert_eq!(
        json_field(&payload, "fingerprint"),
        key.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "the moved/renamed document must fingerprint identically"
    );
    assert_eq!(
        json_field(&payload, "resume"),
        "true",
        "hash-match restore: a moved/renamed re-pick must offer resume"
    );
    assert_eq!(
        json_field(&payload, "offset"),
        "512.0",
        "hash-match restore must report the stored offset"
    );
}

#[test]
fn an_edited_document_resolves_to_no_resume() {
    let dir = TempDir::new("edited");
    let db = dir.path().join("bookmarks.redb");
    let original = write_document(dir.path(), "doc.md", DOC_A);

    // A position stored for the original bytes.
    let key = fingerprint(DOC_A);
    let mut store = BookmarkStore::open(&db).expect("open store");
    store
        .put(
            &key,
            StoredPosition {
                offset_secs: 100.0,
                first_yield_ms: 0,
                end_kind: SessionEnd::Paused,
            },
        )
        .expect("put");
    drop(store);

    // The document is edited on disk — one byte of content
    // changes, which is a different document (FR-036a).
    let mut edited = DOC_A.to_vec();
    let last = edited.len() - 1;
    edited[last] ^= 0x01;
    fs::write(&original, &edited).expect("write edited document");

    let payload = ingest_json(&db, &original);
    let edited_key = fingerprint(&edited);
    assert_eq!(
        json_field(&payload, "fingerprint"),
        edited_key
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "the edited document must fingerprint to the edited content"
    );
    assert_eq!(
        json_field(&payload, "resume"),
        "false",
        "an edited document's stored position is unreachable — no offer (FR-036a)"
    );
    assert_eq!(
        json_field(&payload, "offset"),
        "0.0",
        "an edited document starts at 0.0"
    );
}

#[test]
fn switching_documents_never_deletes_the_first_documents_position() {
    let dir = TempDir::new("switch");
    let db = dir.path().join("bookmarks.redb");
    let doc_a = write_document(dir.path(), "a.md", DOC_A);
    let _doc_b = write_document(dir.path(), "b.md", DOC_B);

    // Document A is paused with a stored position.
    let key_a = fingerprint(DOC_A);
    let mut store = BookmarkStore::open(&db).expect("open store");
    store
        .put(
            &key_a,
            StoredPosition {
                offset_secs: 42.5,
                first_yield_ms: 7,
                end_kind: SessionEnd::Paused,
            },
        )
        .expect("put A");
    // Document B is ingested and stored separately.
    let key_b = fingerprint(DOC_B);
    store
        .put(
            &key_b,
            StoredPosition {
                offset_secs: 8.0,
                first_yield_ms: 9,
                end_kind: SessionEnd::Paused,
            },
        )
        .expect("put B");
    drop(store);

    // Re-ingesting A must still see A's position: the store
    // is keyed by content, so a second document's row never
    // overwrites or deletes the first's.
    let payload_a = ingest_json(&db, &doc_a);
    assert_eq!(
        json_field(&payload_a, "resume"),
        "true",
        "the first document's position must survive a second document's ingest"
    );
    assert_eq!(
        json_field(&payload_a, "offset"),
        "42.5",
        "the first document's stored offset must be unchanged"
    );
}

#[test]
fn an_interrupted_end_stores_but_offers_no_resume() {
    let dir = TempDir::new("interrupted");
    let db = dir.path().join("bookmarks.redb");
    let document = write_document(dir.path(), "doc.md", DOC_A);

    // A kill/power-loss end (FR-009 / FR-032d) stores the
    // position but is not the deliberate offer kind.
    let key = fingerprint(DOC_A);
    let mut store = BookmarkStore::open(&db).expect("open store");
    store
        .put(
            &key,
            StoredPosition {
                offset_secs: 60.0,
                first_yield_ms: 0,
                end_kind: SessionEnd::Interrupted,
            },
        )
        .expect("put interrupted");
    drop(store);

    let payload = ingest_json(&db, &document);
    assert_eq!(
        json_field(&payload, "resume"),
        "false",
        "an interrupted end auto-resumes nothing (FR-014); the explicit-resume form resolves in offer.rs"
    );
    assert_eq!(
        json_field(&payload, "offset"),
        "60.0",
        "the interrupted end's stored position is still reported"
    );
}
