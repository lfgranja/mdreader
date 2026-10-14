//! T040b — BLOCKING US4 CLI contract test for the `player progress`
//! verb (Feature 004-ressoa-dogfood-build, task T040b).
//!
//! Requirement (tasks.md T040b, verbatim): "CLI contract test for
//! progress verb in `crates/player/tests/test_progress_cli.rs`
//! (--consumed/--total/--k/--n and --indeterminate JSON shapes, frozen
//! label "Section {k} of {N} — {p}% (content-offset)", zero-sections
//! indeterminate+null, --format json; FAIL-red before T030b/T041)"
//!
//! The verb is the CLI surface of Principle II: `player progress`
//! calls the same progress render function the Tauri IPC path calls
//! (T030b, normative flag shape `progress (--consumed <u> --total <u>
//! --k <k> --n <n> | --indeterminate) --format json`), and its output
//! mirrors `contracts/surface.md`: the content-offset progress plus
//! the FROZEN label template (English identifier — this CLI emits the
//! identifier string directly; the rendered pt-BR translation is a
//! separate rendering step).
//!
//! **This test is RED by gate design.** The `player progress` verb does
//! not exist yet (T030b is unwritten), so the current binary exits
//! non-zero on every invocation below (clap: "unrecognized subcommand
//! 'progress'"). That failure is the required pre-implementation red
//! state — T030b/T041 land against this contract.
//!
//! The pinned contract (what T030b/T041 must implement):
//!
//! - **Determinate** (`--consumed <u> --total <u> --k <k> --n <n>`):
//!   exactly the keys `progress`, `label`, `section_title`, with
//!   - `progress`: the `Progress::Determinate` wire form — the
//!     externally-tagged `{"determinate": {"consumed_units": …,
//!     "total_units": …, "k": …, "n": …}}` object the mirror payload
//!     carries (content-offset units, never audio-duration or
//!     time-based; the shape pinned by
//!     `crates/player/tests/contract/test_shell_ipc_us4.rs`);
//!   - `label`: the frozen template
//!     `"Section {k} of {N} — {p}% (content-offset)"`, where **{p} is
//!     the integer percent** — `(consumed × 100) / total` in integer
//!     arithmetic, truncated toward zero, with **no decimals**
//!     (CHOSEN FORMAT: `30/120 ⇒ 25`, `1/3 ⇒ 33`; stated here because
//!     the contract freezes the template but not the rounding);
//!   - `section_title`: `null` — this verb names no section (the flag
//!     set carries no section input); `null` is the contract's
//!     absent-section form.
//! - **Indeterminate** (`--indeterminate`, and the zero-sections case
//!   `--k 0 --n 0`): `progress` is the `"indeterminate"` tag, `label`
//!   is `null` (no determinable percent), `section_title` is `null` —
//!   zero sections ⇒ indeterminate + null, whole session, no flip
//!   (contracts/surface.md; FR-028a).
//! - **Protocol**: payload on stdout, diagnostic on stderr, exit 0 on
//!   success — the text-I/O protocol every verb honours.
//!
//! Invoked through the real `player` binary via
//! `std::process::Command` (the `crates/session/tests/
//! test_ingest_cli.rs` pattern), so the exact surface a pipe or a
//! Tauri IPC shim drives is what the test runs.

use std::process::{Command, Output, Stdio};

use serde_json::Value;

/// The path of the built `player` binary cargo exposes to this
/// integration test.
const BIN: &str = env!("CARGO_BIN_EXE_player");

/// Run the `player` binary with `args`, collecting the full output.
///
/// # Errors
///
/// The spawn or the wait fails.
fn run(args: &[&str]) -> std::io::Result<Output> {
    Command::new(BIN)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

/// Run one `progress` invocation and read the contract payload out of
/// stdout, asserting the success half of the text-I/O protocol and
/// the exact key set.
///
/// # Errors
///
/// The invocation fails (non-zero exit — the current red state),
/// stdout is not UTF-8 or not JSON, stderr was written on success, or
/// the payload keys are not the contract's exact
/// `{label, progress, section_title}`.
fn progress_payload(args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let output = run(args)?;
    if !output.status.success() {
        return Err(format!(
            "`player {}` exited with {} — the progress verb is unimplemented (T030b); stderr: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    // The success half of the protocol: the payload on stdout,
    // nothing on stderr.
    if !output.stderr.is_empty() {
        return Err(format!(
            "`player {}` wrote to stderr on success: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let text = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&text)?;
    let object = payload
        .as_object()
        .ok_or("the progress payload is not a JSON object")?;
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["label", "progress", "section_title"],
        "the progress payload must carry exactly the contract's three keys, got {text}"
    );
    Ok(payload)
}

/// The frozen label template (contracts/surface.md): the English
/// identifier string the CLI emits directly — `{p}` is the INTEGER
/// percent `(consumed × 100) / total` in integer arithmetic,
/// truncated toward zero, with no decimals.
fn frozen_label(k: u32, n: u32, consumed: u64, total: u64) -> String {
    let percent = consumed.saturating_mul(100) / total.max(1);
    format!("Section {k} of {n} — {percent}% (content-offset)")
}

#[test]
fn determinate_progress_reports_the_contract_shape_and_frozen_label()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = progress_payload(&[
        "progress",
        "--consumed",
        "30",
        "--total",
        "120",
        "--k",
        "1",
        "--n",
        "5",
        "--format",
        "json",
    ])?;
    // The progress carries the content-offset shape: parsed units,
    // never audio-duration or time-based values.
    assert_eq!(
        payload["progress"],
        serde_json::json!({"determinate": {
            "consumed_units": 30,
            "total_units": 120,
            "k": 1,
            "n": 5,
        }}),
        "the determinate progress must carry the externally-tagged content-offset object"
    );
    // The frozen label: 30/120 ⇒ the integer percent 25.
    assert_eq!(
        payload["label"],
        serde_json::json!("Section 1 of 5 — 25% (content-offset)"),
        "the label must render the frozen template with the integer percent"
    );
    // This verb names no section: the null is the contract's
    // absent-section form.
    assert_eq!(
        payload["section_title"],
        serde_json::json!(null),
        "the progress CLI carries a null section_title (no section input flag)"
    );
    Ok(())
}

#[test]
fn the_frozen_label_uses_the_integer_percent_with_no_decimals()
-> Result<(), Box<dyn std::error::Error>> {
    // The rounding is pinned: 1/3 ⇒ 33 (integer division truncates
    // toward zero; no decimals, no percent sign spacing change).
    let expected = frozen_label(2, 4, 1, 3);
    assert_eq!(expected, "Section 2 of 4 — 33% (content-offset)");
    let payload = progress_payload(&[
        "progress",
        "--consumed",
        "1",
        "--total",
        "3",
        "--k",
        "2",
        "--n",
        "4",
        "--format",
        "json",
    ])?;
    assert_eq!(
        payload["label"], serde_json::json!(expected),
        "the label must render the integer percent with no decimals"
    );
    Ok(())
}

#[test]
fn the_indeterminate_flag_reports_the_indeterminate_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = progress_payload(&["progress", "--indeterminate", "--format", "json"])?;
    // The indeterminate progress carries no units (FR-028a).
    assert_eq!(
        payload["progress"],
        serde_json::json!("indeterminate"),
        "the --indeterminate progress must be the indeterminate tag"
    );
    assert_eq!(
        payload["label"],
        serde_json::json!(null),
        "an indeterminate progress has no percent to render — the label is null"
    );
    assert_eq!(
        payload["section_title"],
        serde_json::json!(null),
        "the indeterminate shape carries a null section title"
    );
    Ok(())
}

#[test]
fn zero_sections_resolve_indeterminate_with_a_null_label()
-> Result<(), Box<dyn std::error::Error>> {
    // Zero sections (k = 0, n = 0, no marked units): the whole session
    // is indeterminate and nothing renders — the percent is
    // undeterminable, so the label and the section title are null
    // (contracts/surface.md "zero sections → indeterminate + no
    // title").
    let payload = progress_payload(&[
        "progress",
        "--consumed",
        "0",
        "--total",
        "0",
        "--k",
        "0",
        "--n",
        "0",
        "--format",
        "json",
    ])?;
    assert_eq!(
        payload["progress"],
        serde_json::json!("indeterminate"),
        "zero sections must resolve indeterminate progress, never a zero-percent bar"
    );
    assert_eq!(
        payload["label"],
        serde_json::json!(null),
        "zero sections carry a null label (no percent to render)"
    );
    assert_eq!(
        payload["section_title"],
        serde_json::json!(null),
        "zero sections carry a null section title"
    );
    Ok(())
}
