//! T158 — RED-FIRST property test: the producer's back-pressure
//! (feature 004-ressoa-dogfood-build, task T158).
//!
//! THE FINDING (tasks.md F1, HIGH). The app-shell producer was a
//! ONE-PASS loop: it handed every synthesized unit to the resident
//! window in ONE call, and the window's bound (60 s) DROPS the
//! overrun — the audio beyond the window is dropped, the reading
//! SKIPS that stretch, and the place the reader is told (credited
//! from the consumed samples alone, FR-002) no longer names the
//! audio the reader actually heard.
//!
//! THE CONTRACT.
//!
//! * The producer is DEMAND-DRIVEN: the audio reaches the resident
//!   window only as fast as the window can take it. The window's
//!   room is the back-pressure — the producer waits for the device
//!   instead of losing the overrun.
//! * The audio the reader hears is the WHOLE document: the queue
//!   drops NOTHING, so the reading never skips a stretch.
//! * The place the reader is told is the audio the reader heard: the
//!   audible clock counts the consumed samples, and every produced
//!   sample is consumed.
//!
//! MECHANISM. The resident window is the production's own
//! (`player::backend::ResidentQueue` — the bounded window T149
//! introduced), and the back-pressure rule is the production's own
//! (`src_tauri::producer::admits`), so the property below runs
//! against the SAME window and the SAME rule the app shell uses. The
//! producer's own wiring is asserted over the production SOURCE —
//! the discipline `src-tauri/tests/test_resume_restarts_audio.rs`
//! and `src-tauri/tests/test_session_live_from_first_chunk.rs` use:
//! the handover must go through the rule, never around it.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary word
//! anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state" and "the
//! reading's place" instead. Keep it that way: the gate greps the
//! WHOLE file, and a word introduced here fails the gate for every
//! other task.

use std::collections::{BTreeMap, BTreeSet};

use engine_core::format::DeclaredFormat;
use player::backend::{MAX_PREROLL_SECS, ResidentQueue};
use proptest::prelude::*;

/// The bridge — the file that owns the producer.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The production part of the bridge: everything before its first
/// inline test module. A test module's own text is not the
/// production's evidence of the contract.
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

/// The production source with every comment removed: the contract
/// reads CODE, never prose — a doc comment that names a call proves
/// nothing about the route that carries it.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let mut in_string = false;
        let mut escaped = false;
        let mut cut = line.len();
        for (index, ch) in line.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }
            if ch == '"' {
                in_string = true;
                continue;
            }
            if ch == '/' && line[index + 1..].starts_with('/') {
                cut = index;
                break;
            }
        }
        out.push_str(&line[..cut]);
    }
    out
}

/// The declared functions of the production source, with their
/// bodies: the name and the brace-matched body text.
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

/// The brace that closes the one at `open`, skipping the string
/// literals and the character literals.
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

/// The functions a body calls, by name: the identifier in front of
/// every opening parenthesis.
fn calls_in(body: &str) -> BTreeSet<String> {
    let bytes = body.as_bytes();
    let mut out = BTreeSet::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            index = skip_string(bytes, index);
            continue;
        }
        if bytes[index] == b'(' {
            let mut start = index;
            while start > 0
                && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_')
            {
                start -= 1;
            }
            let name = &body[start..index];
            if !name.is_empty() {
                // A path's last segment is the call's own name.
                let name = name.rsplit("::").next().unwrap_or(name);
                let _inserted = out.insert(name.to_string());
            }
        }
        index += 1;
    }
    out
}

/// Whether the production's handover reaches the back-pressure rule:
/// every function that hands audio to the resident window applies the
/// rule, directly or through the helper it calls.
fn handover_reaches_the_rule(bodies: &BTreeMap<String, String>) -> bool {
    let handoffs: Vec<&String> = bodies
        .iter()
        .filter(|(_, body)| body.contains("enqueue_chunk("))
        .map(|(name, _)| name)
        .collect();
    if handoffs.is_empty() {
        return false;
    }
    handoffs
        .iter()
        .all(|name| reaches_the_rule(bodies, name, 0))
}

/// Whether the named function applies the back-pressure rule, within
/// a bounded number of hops.
fn reaches_the_rule(bodies: &BTreeMap<String, String>, name: &str, depth: usize) -> bool {
    if depth > 2 {
        return false;
    }
    let Some(body) = bodies.get(name) else {
        return false;
    };
    if body.contains("admits(") {
        return true;
    }
    calls_in(body)
        .iter()
        .any(|callee| reaches_the_rule(bodies, callee, depth + 1))
}

// ===========================================================================
// The properties.
// ===========================================================================

/// The resident window's bound, in samples: the window's own bound
/// (in seconds) at the declared 22 050 Hz.
fn cap_samples() -> usize {
    let cap_secs = MAX_PREROLL_SECS;
    let hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the bound is a positive constant (60 s) and the declared 22 050 Hz makes their product a small positive sample count"
    )]
    let samples = (cap_secs * hz) as usize;
    samples
}

/// The seconds a sample count names at the declared 22 050 Hz.
#[expect(
    clippy::cast_precision_loss,
    reason = "PCM sample counts are always small (≤ millions), well within f64's 52-bit mantissa"
)]
fn secs(samples: usize) -> f64 {
    samples as f64 / f64::from(DeclaredFormat::SAMPLE_RATE)
}

// T158 RED-FIRST: the producer never drops the audio the reader
// must hear, and the place the reader is told is the audio the
// reader heard.
//
// The producer here outruns the device: it offers every sample as
// fast as it can. The window's room decides how much of it the
// queue takes, so the producer WAITS for the device instead of
// losing the overrun. The property holds for any document length
// and any device speed: the queue drops nothing, every produced
// sample is consumed, and the place the audible clock reports (the
// consumed samples at the declared 22 050 Hz) is the whole document's
// own length — the audio the reader actually heard.
proptest! {
    #[test]
    fn prop_the_producer_never_drops_the_audio_the_reader_must_hear(
        units in proptest::collection::vec(1usize..4_000, 1..24),
        drain in 1usize..2_000,
    ) {
        let cap = cap_samples();
        let mut window = ResidentQueue::new();
        let mut pending: Vec<f32> = Vec::new();
        let mut produced = 0usize;
        let mut consumed = 0usize;

        for size in units {
            pending.extend(std::iter::repeat_n(1.0_f32, size));
            produced += size;
            // The producer offers the pending audio as fast as the
            // window takes it; the window's room is the whole
            // back-pressure.
            while !pending.is_empty() {
                let room = src_tauri::producer::admits(window.len(), cap, pending.len());
                if room == 0 {
                    // The window is full: the device takes its
                    // samples, and the producer offers the rest.
                    for _ in 0..drain {
                        if window.pop_front().is_some() {
                            consumed += 1;
                        }
                    }
                    continue;
                }
                let piece: Vec<f32> = pending.drain(..room).collect();
                window.push(&piece);
            }
        }
        // The device plays out the window.
        while window.pop_front().is_some() {
            consumed += 1;
        }

        prop_assert_eq!(
            window.dropped_samples(),
            0,
            "T158: the resident window dropped {} samples — the overrun beyond the window is \
             DROPPED, so the reading SKIPS that stretch of the document and the place the reader \
             is told no longer names the audio the reader actually heard (FR-002)",
            window.dropped_samples()
        );
        prop_assert_eq!(
            consumed,
            produced,
            "T158: the reader heard {} of the {} produced samples — the audio the reader did not \
             hear is a stretch the document's own length does not account for (FR-002)",
            consumed,
            produced
        );
        // The place the reader is told: the audible clock counts the
        // CONSUMED samples at the declared 22 050 Hz (FR-002) — the only
        // advance path — so the place names the end of the audio the
        // reader heard. The sample counts are the exact contract; the
        // seconds are their projection at the declared 22 050 Hz.
        prop_assert_eq!(
            consumed,
            produced,
            "T158: the place the reader is told ({} s of the document's {} s) must be the audio \
             the reader heard: the reader heard {} of the {} produced samples, so the reading \
             SKIPS a stretch of the document (FR-002)",
            secs(consumed),
            secs(produced),
            consumed,
            produced
        );
    }
}

/// T158: the production's handover goes THROUGH the back-pressure
/// rule — the producer never hands a whole unit to the window in
/// one call, so the window's bound can never drop the overrun.
#[test]
fn the_producer_handover_goes_through_the_back_pressure_rule() {
    let source = strip_comments(production_source());
    let bodies = fn_bodies(&source);
    assert!(
        !bodies.is_empty(),
        "T158: the production source declares no function — the contract cannot be read"
    );
    assert!(
        handover_reaches_the_rule(&bodies),
        "T158: the producer hands the audio to the resident window WITHOUT the back-pressure \
         rule: the window's bound then DROPS the overrun, the reading SKIPS that stretch of the \
         document, and the place the reader is told (credited from the consumed samples alone) \
         no longer names the audio the reader actually heard (FR-002). Every function that hands \
         audio to the window must apply the window's room."
    );
}
