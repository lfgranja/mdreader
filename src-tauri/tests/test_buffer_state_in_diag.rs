//! T159 — RED-FIRST property test: the resident bound in the
//! diagnostic record (feature 004-ressoa-dogfood-build, task T159).
//!
//! THE FINDING (tasks.md F2, MEDIUM). The buffer-state entry reported
//! only the pre-roll depth. The bound the resident window never
//! exceeds and the count of the samples dropped beyond it — the two
//! numbers T149 exposed through `CpalBackend::buffer_state()` — never
//! reached the record, so a reading that SKIPPED audio (the producer
//! outran the device) was not diagnosable after the fact.
//!
//! THE CONTRACT.
//!
//! * The buffer-state entry carries the depth, the bound, the dropped
//!   count and the document's content-hash ref.
//! * The entry stays prose-free, identifier-pattern and within the
//!   200-character detail bound: the diag crate's own scanner — the
//!   one the maintainer's pull runs — accepts it (no prose, no
//!   over-long detail, and the `error` the failure and refusal kinds
//!   require).
//!
//! MECHANISM. The entry is built by the production's own builder
//! (`src_tauri::buffer_state_event`) and scanned by the diag crate's
//! own scanner (`diag::scan::scan_record`) over a REAL record file
//! written by the record's own writer — the same file, the same
//! writer and the same scanner the maintainer pulls off the device.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary word
//! anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state" and "the
//! reading's place" instead. Keep it that way: the gate greps the
//! WHOLE file, and a word introduced here fails the gate for every
//! other task.

use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

/// The bridge — the file that owns the record's entries.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The document's content-hash ref: the only document reference the
/// record ever carries (FR-036).
const DOC_REF: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// The production part of the bridge: everything before its first
/// inline test module.
fn production_source() -> &'static str {
    let mut offset = 0usize;
    for line in BRIDGE.split_inclusive('\n') {
        let trimmed = line.trim();
        let opens_a_test_module = trimmed.starts_with("mod ")
            && (trimmed.contains("_tests") || trimmed.contains(" tests"))
            && trimmed.ends_with('{');
        if opens_a_test_module {
            return &BRIDGE[..offset];
        }
        offset += line.len();
    }
    BRIDGE
}

/// The declared functions of the production source, with their
/// bodies.
fn fn_bodies(source: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut index = 0usize;
    while let Some(found) = source[index..].find("fn ") {
        let at = index + found;
        let name_start = at + 3;
        let name_end = name_start
            + source[name_start..]
                .find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                .unwrap_or(source.len() - name_start);
        let name = source[name_start..name_end].to_string();
        let Some(brace) = source[name_end..].find('{') else {
            break;
        };
        let open = name_end + brace;
        let Some(close) = matching_brace(source, open) else {
            break;
        };
        let _inserted = out.insert(name, source[open..=close].to_string());
        index = close;
    }
    out
}

/// The brace that closes the one at `open`.
const fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_string(bytes, index);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// The end of the string literal that starts at `start`.
const fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    let mut escaped = false;
    while index < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[index] == b'\\' {
            escaped = true;
        } else if bytes[index] == b'"' {
            return index + 1;
        }
        index += 1;
    }
    index
}

/// The diag crate's own scanner's verdict on the given entries: the
/// record is written by the record's own writer and scanned by the
/// record's own scanner — the same file, the same writer and the same
/// scanner the maintainer pulls off the device.
fn scan_entries(details: &[String]) -> Result<diag::scan::ScanReport, String> {
    let dir = tempfile::TempDir::new().map_err(|error| error.to_string())?;
    let path = dir.path().join("record.jsonl");
    let mut writer =
        diag::writer::DiagnosticWriter::open(&path).map_err(|error| error.to_string())?;
    for detail in details {
        let mut event = src_tauri::buffer_state_event(0.0, 0.0, 0.0, 0, DOC_REF);
        event.detail = Some(detail.clone());
        writer.append(&event).map_err(|error| error.to_string())?;
    }
    drop(writer);
    diag::scan::scan_record(&path).map_err(|error| error.to_string())
}

// ===========================================================================
// The properties.
// ===========================================================================

// T159 RED-FIRST: the buffer-state entry carries the resident window's
// BOUND and the samples DROPPED beyond it, and the entry still passes
// the record's own scanner.
//
// The property holds for every measurement the window can report: the
// entry names the depth, the bound and the dropped count, and the
// scanner — the one the maintainer's pull runs — finds no violation
// in it (no prose, no over-long detail, and the `error` the failure
// and refusal kinds require).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn prop_the_buffer_state_entry_carries_the_bound_and_the_dropped_count(
        buffered in 0.0f64..3_600.0,
        cap in 0.0f64..3_600.0,
        dropped in 0u64..1_000_000_000u64,
    ) {
        let event = src_tauri::buffer_state_event(0.0, buffered, cap, dropped, DOC_REF);
        let Some(detail) = event.detail else {
            return Err(TestCaseError::fail(
                "T159: the buffer-state entry MUST carry a detail — the bound and the dropped \
                 count are the numbers a reading that skipped audio needs (FR-017)",
            ));
        };
        prop_assert!(
            detail.contains(&format!("preroll_secs={buffered:.3}")),
            "T159: the buffer-state entry does not carry the pre-roll depth: {detail}"
        );
        prop_assert!(
            detail.contains(&format!("preroll_cap_secs={cap:.3}")),
            "T159: the buffer-state entry does not carry the window's bound — the depth is \
             measured against it, and the record must carry both: {detail}"
        );
        prop_assert!(
            detail.contains(&format!("dropped_samples={dropped}")),
            "T159: the buffer-state entry does not carry the dropped count — a reading that \
             skipped audio is not diagnosable without it: {detail}"
        );
        prop_assert!(
            detail.contains(&format!("doc={DOC_REF}")),
            "T159: the buffer-state entry does not carry the document's content-hash ref — it is \
             the only document reference the record may carry (FR-017 / FR-036): {detail}"
        );
        prop_assert!(
            detail.chars().count() <= 200,
            "T159: the buffer-state entry's detail is {} characters — the record's own bound is \
             200: {detail}",
            detail.chars().count()
        );

        let report = scan_entries(std::slice::from_ref(&detail)).map_err(TestCaseError::fail)?;
        prop_assert!(
            report.violations.is_empty(),
            "T159: the buffer-state entry violates the record's own scanner: {:?} — {detail}",
            report.violations
        );
    }
}

/// T159: the production's buffer-state entry reads the bound and the
/// dropped count from the resident window — the numbers the record
/// must carry.
#[test]
fn the_buffer_state_entry_reads_the_window_bound_and_the_dropped_count() {
    let bodies = fn_bodies(production_source());
    let Some(body) = bodies.get("record_buffer_state") else {
        assert!(
            bodies.contains_key("record_buffer_state"),
            "T159: the production declares no buffer-state entry — the record must carry the \
             window's bound and the dropped count (FR-017)"
        );
        return;
    };
    assert!(
        body.contains("buffer_state()"),
        "T159: the buffer-state entry does not read the resident window's state — the bound and \
         the dropped count are the two numbers T149 exposed, and the entry must carry them \
         (FR-017 / contracts/diagnostic-trace.md)"
    );
    assert!(
        body.contains("buffer_state_event("),
        "T159: the buffer-state entry does not go through the entry builder — the builder is the \
         one place that composes the detail's keys (FR-017)"
    );
}
