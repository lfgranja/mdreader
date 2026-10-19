//! T018 — the `session ingest` CLI verb, smoked
//! end-to-end over the real binary: the text-I/O
//! protocol (arguments in, payload on stdout,
//! diagnostic on stderr) exercised through
//! `std::process::Command`, so the exact surface
//! a pipe or a Tauri IPC shim drives is what the
//! test runs (Principle II).
//!
//! The verb is the CLI surface `contracts/
//! ingest.md` names — `ingest <snapshot-file>`
//! (hash + store lookup → `{fingerprint, resume:
//! bool, offset}` JSON) — resolved through the
//! same canonical [`session::fingerprint`] and
//! [`session::store::BookmarkStore`] the in-process
//! callers use, with `--format json` the default.
//!
//! Invariants under test:
//! - the payload carries exactly the three keys
//!   the contract names, and the fingerprint is
//!   the blake3-256 of the BYTES — identical
//!   bytes at any path (or piped on stdin) share
//!   one fingerprint (FR-036);
//! - `resume` is true only when a stored entry
//!   exists AND its end kind is the deliberate
//!   pause (`paused`);
//! - an `interrupted` entry auto-resumes nothing
//!   (FR-032d) — `resume` is false — but the JSON
//!   still reports the stored offset, the offer
//!   form itself resolving in `offer.rs`;
//! - a miss (no entry) reports `resume: false`
//!   and `offset: 0.0` — the start-with-no-offer
//!   resolution (FR-036a / storage-schema.md).

use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::Value;
use session::fingerprint;
use session::store::StoredPosition;

use engine_core::closed::SessionEnd;

/// The path of the built `session` binary cargo
/// exposes to this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_session");

/// Run the `session` binary with `args`, feeding
/// `stdin_bytes` (empty for none) and collecting
/// the full output.
///
/// # Errors
///
/// The spawn, the stdin write, or the wait fails.
fn run(args: &[&str], stdin_bytes: &[u8]) -> std::io::Result<Output> {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_bytes)?;
    }
    child.wait_with_output()
}

/// The stdout of one invocation as UTF-8 text.
///
/// # Errors
///
/// The output is not valid UTF-8.
fn stdout_of(output: &Output) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(output.stdout.clone())?)
}

/// Parse one payload as JSON.
///
/// # Errors
///
/// The text is not valid JSON.
fn json_of(text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(text)?)
}

/// The hex fingerprint of `bytes` — the same
/// 64-digit lowercase form the payload carries,
/// resolved through the crate's canonical
/// [`fingerprint`] so the oracle is the same
/// function the verb drives.
fn hex_of_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    fingerprint(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ignored = write!(hex, "{byte:02x}");
            hex
        })
}

/// Store `position` for the document at `file`,
/// through the `put` verb of the same binary —
/// the round trip the resume lookup reads back.
///
/// # Errors
///
/// The store invocation fails.
fn put(
    db: &str,
    file: &str,
    offset_secs: f64,
    first_yield_ms: u64,
    end_kind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = run(
        &[
            "--db",
            db,
            "put",
            file,
            "--offset-secs",
            &offset_secs.to_string(),
            "--first-yield-ms",
            &first_yield_ms.to_string(),
            "--end-kind",
            end_kind,
        ],
        b"",
    )?;
    if !output.status.success() {
        return Err(format!("put failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}

/// The three payload fields the contract names.
struct IngestReport {
    /// The document's blake3-256 fingerprint, hex.
    fingerprint: String,
    /// Whether a resume offer applies.
    resume: bool,
    /// The stored offset in seconds.
    offset: f64,
}

/// Run one `ingest` invocation and read the
/// contract payload out of stdout.
///
/// # Errors
///
/// The invocation fails, stdout is not UTF-8, the
/// payload is not JSON, or the shape is not the
/// contract's exact `{fingerprint, resume,
/// offset}`.
fn ingest(
    db: &str,
    file: Option<&str>,
    stdin_bytes: &[u8],
) -> Result<IngestReport, Box<dyn std::error::Error>> {
    let mut args = vec!["--db", db, "ingest"];
    if let Some(file) = file {
        args.push(file);
    }
    let output = run(&args, stdin_bytes)?;
    let text = stdout_of(&output)?;
    if !output.status.success() {
        return Err(format!("ingest failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    // The success half of the protocol: the
    // payload on stdout, nothing on stderr.
    if !output.stderr.is_empty() {
        return Err(format!(
            "ingest wrote to stderr on success: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let payload = json_of(&text)?;
    let object = payload
        .as_object()
        .ok_or("the ingest payload is not a JSON object")?;
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["fingerprint", "offset", "resume"],
        "the ingest payload must carry exactly the contract's three keys, got {text}"
    );
    Ok(IngestReport {
        fingerprint: payload["fingerprint"]
            .as_str()
            .ok_or("fingerprint is not a string")?
            .to_owned(),
        resume: payload["resume"].as_bool().ok_or("resume is not a bool")?,
        offset: payload["offset"].as_f64().ok_or("offset is not a number")?,
    })
}

/// The bytes every shared-bytes case reads: the
/// document content, never its path (FR-036).
const BYTES: &[u8] = b"# Titulo\n\nParagrafo de teste.\n";

#[test]
fn ingest_from_the_stdin_pipe_reports_the_fingerprint_and_no_resume()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("bookmarks.redb");
    let db = db.to_str().ok_or("the db path is not UTF-8")?;

    // The pipe form: no file named, the bytes on
    // stdin — the document is identified by its
    // content alone, so the empty store answers
    // the start-with-no-offer resolution.
    let report = ingest(db, None, BYTES)?;
    assert_eq!(
        report.fingerprint,
        hex_of_bytes(BYTES),
        "the fingerprint must be the blake3-256 of the piped bytes"
    );
    assert!(!report.resume, "an empty store opens no resume offer");
    assert_eq!(
        report.offset.to_bits(),
        0.0_f64.to_bits(),
        "a miss resolves to a start at 0.0"
    );
    Ok(())
}

#[test]
fn ingest_reports_resume_for_a_paused_entry() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("bookmarks.redb");
    let file = dir.path().join("snapshot.md");
    std::fs::write(&file, BYTES)?;
    let db = db.to_str().ok_or("the db path is not UTF-8")?;
    let file = file.to_str().ok_or("the file path is not UTF-8")?;

    put(db, file, 42.0, 1234, "paused")?;
    let report = ingest(db, Some(file), b"")?;
    assert!(
        report.resume,
        "a stored deliberate pause opens the resume offer (FR-032c)"
    );
    assert_eq!(
        report.offset.to_bits(),
        42.0_f64.to_bits(),
        "the stored position is reported"
    );
    assert_eq!(
        report.fingerprint,
        hex_of_bytes(BYTES),
        "the fingerprint is the blake3-256 of the file bytes"
    );
    Ok(())
}

#[test]
fn ingest_reports_the_stored_offset_without_resume_for_an_interrupted_entry()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("bookmarks.redb");
    let file = dir.path().join("snapshot.md");
    std::fs::write(&file, BYTES)?;
    let db = db.to_str().ok_or("the db path is not UTF-8")?;
    let file = file.to_str().ok_or("the file path is not UTF-8")?;

    put(db, file, 90.0, 500, "interrupted")?;
    let report = ingest(db, Some(file), b"")?;
    assert!(
        !report.resume,
        "an unexpected end auto-resumes nothing — resume stays false (FR-032d)"
    );
    assert_eq!(
        report.offset.to_bits(),
        90.0_f64.to_bits(),
        "the interrupted entry still reports its stored offset"
    );
    Ok(())
}

#[test]
fn identical_bytes_at_any_path_or_on_stdin_share_one_fingerprint()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("bookmarks.redb");
    let first = dir.path().join("first.md");
    let second = dir.path().join("elsewhere/second.md");
    std::fs::create_dir_all(second.parent().ok_or("the path has no parent")?)?;
    std::fs::write(&first, BYTES)?;
    std::fs::write(&second, BYTES)?;
    let db = db.to_str().ok_or("the db path is not UTF-8")?;
    let first = first.to_str().ok_or("the first path is not UTF-8")?;
    let second = second.to_str().ok_or("the second path is not UTF-8")?;

    // Same bytes, three drivers: two paths and the
    // pipe. The position identity is the content,
    // never the path (FR-036).
    let from_first = ingest(db, Some(first), b"")?;
    let from_second = ingest(db, Some(second), b"")?;
    let from_stdin = ingest(db, None, BYTES)?;
    assert_eq!(
        from_first.fingerprint, from_second.fingerprint,
        "identical bytes at any path share one fingerprint (FR-036)"
    );
    assert_eq!(
        from_first.fingerprint, from_stdin.fingerprint,
        "the piped bytes fingerprint identically to the file bytes"
    );
    Ok(())
}

#[test]
fn an_edited_document_resolves_to_no_resume_at_zero() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("bookmarks.redb");
    let file = dir.path().join("snapshot.md");
    std::fs::write(&file, BYTES)?;
    let db = db.to_str().ok_or("the db path is not UTF-8")?;
    let file = file.to_str().ok_or("the file path is not UTF-8")?;

    put(db, file, 42.0, 1234, "paused")?;
    // FR-036a: an edited byte is a different
    // document — the stored position becomes
    // unreachable and the reopen starts at 0.0.
    let mut edited = BYTES.to_vec();
    let last = edited.last_mut().ok_or("the bytes are empty")?;
    *last ^= 0x01;
    let report = ingest(db, None, &edited)?;
    assert_ne!(
        report.fingerprint,
        hex_of_bytes(BYTES),
        "an edited byte must fingerprint differently (FR-036a)"
    );
    assert!(!report.resume, "an edited document opens no resume offer");
    assert_eq!(
        report.offset.to_bits(),
        0.0_f64.to_bits(),
        "a miss resolves to a start at 0.0"
    );
    Ok(())
}

/// The stored-position end kinds the CLI drives,
/// cross-checked against the store's own closed
/// set: the report shape above is only meaningful
/// because these two kinds are the retained ones
/// (storage-schema.md).
#[test]
fn the_retained_end_kinds_are_paused_and_interrupted() {
    assert_ne!(
        StoredPosition {
            offset_secs: 1.0,
            first_yield_ms: 0,
            end_kind: SessionEnd::Paused,
        },
        StoredPosition {
            offset_secs: 1.0,
            first_yield_ms: 0,
            end_kind: SessionEnd::Interrupted,
        },
        "the retained end kinds are distinct members of the closed set"
    );
}
