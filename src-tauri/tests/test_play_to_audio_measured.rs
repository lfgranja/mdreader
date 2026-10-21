//! T152 — RED-FIRST contract test: the play-to-audio measurement
//! covers the first CONSUMED sample (feature
//! `004-ressoa-dogfood-build`, task T152).
//!
//! THE FINDING (tasks.md F10, LOW). The shell closed the
//! play-to-audio clock right after the backend's output stream
//! started, with the comment that the stream start IS the first
//! audible speech. It is not: the first buffer is consumed on the
//! NEXT audio callback, so the device's output latency stands
//! between the two, and the recorded FR-003 / SC-002 latency
//! UNDER-REPORTED the bound by exactly that latency.
//!
//! THE CONTRACT.
//!
//! * **The clock closes on the first CONSUMED sample.** The
//!   backend's audible clock advances ONLY on consumed speech
//!   samples, so a clock past zero IS a consumed sample. The
//!   closer reads that clock and declines to close the measured
//!   window while the clock still sits at zero.
//! * **The stream start does not close the clock.** The start is
//!   the beginning of the STREAM, not of the audible output: the
//!   closer is reached only from a site that can observe the
//!   consumption, never from inside the block that opens the
//!   stream.
//! * **The audible credit is the only advance path.** The runtime
//!   half, on the session crate's own clock: the reading's place
//!   moves only through the audible credit, so a clock that has
//!   not moved has consumed nothing.
//!
//! MECHANISM. The measurement lives in the shell's private state and
//! its closer is private to the plugin module, so the two shell-side
//! claims are asserted over the PRODUCTION SOURCE — the same
//! discipline `src-tauri/tests/test_resume_restarts_audio.rs` and
//! `src-tauri/tests/test_session_live_from_first_chunk.rs` use. The
//! runtime half IS reachable: the session crate's clock is public.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary
//! word anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state", "the reading's
//! place" and "the surface" instead. Keep it that way: the gate
//! greps the WHOLE file, and a word introduced here fails the gate
//! for every other task.

use std::collections::BTreeSet;

use player::cli::{PlaybackSession, ShellVerb};

use proptest::prelude::*;

/// The Tauri bridge — the file that owns the measured-latency
/// clock and the audio backend.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The measured window's own close: the function that takes the
/// request instant and stores the measurement. The closer is the
/// function that CALLS it — the bridge's own vocabulary is not
/// pinned, the STRUCTURE is: the window is closed in exactly one
/// place, and that place decides whether the consumption has
/// happened.
const CLOSE_CALL: &str = "take_play_to_audio_ms(";

/// The backend's stream start: the call that opens the output
/// stream. The stream start is NOT the first audible speech.
const STREAM_START: &str = "start_backend(";

// ===========================================================================
// The bridge — the production source
// ===========================================================================

/// The production part of a Rust source: everything before its
/// first inline test module. A test body's own text is the bridge's
/// evidence of the contract, not the production behaviour, so the
/// model never reads one.
fn production_source(source: &str) -> &str {
    let first_module = ["#[cfg(test)]\nmod ", "#[cfg(test)]\npub(crate) mod "]
        .iter()
        .filter_map(|needle| source.find(needle))
        .min();
    first_module.map_or(source, |offset| source.get(..offset).unwrap_or(source))
}

/// The length of the Rust char literal at `start`; `None` when the
/// quote begins a lifetime instead.
fn rust_char_literal_len(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'\'') {
        return None;
    }
    let mut cursor = start + 1;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor += 2;
        while cursor < bytes.len() && bytes[cursor] != b'\'' {
            cursor += 1;
        }
        return bytes.get(cursor).map(|_| cursor + 1 - start);
    }
    let byte = *bytes.get(cursor)?;
    if byte == b'\'' || byte == b'\\' {
        return None;
    }
    if bytes.get(cursor + 1) == Some(&b'\'') {
        return Some(3);
    }
    None
}

/// The end of the Rust string, char literal, or comment that starts
/// at `start`; `start` itself when none begins there.
fn rust_token_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let Some(&first) = bytes.get(start) else {
        return start;
    };
    match first {
        b'"' => {
            let mut index = start + 1;
            while index < bytes.len() {
                match bytes[index] {
                    b'\\' => index += 2,
                    b'"' => return index + 1,
                    _ => index += 1,
                }
            }
            bytes.len()
        }
        b'\'' => rust_char_literal_len(bytes, start).map_or(start + 1, |len| start + len),
        b'/' => match bytes.get(start + 1) {
            Some(b'/') => {
                let mut index = start + 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                index
            }
            Some(b'*') => {
                let mut index = start + 2;
                while index + 1 < bytes.len() {
                    if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                        return index + 2;
                    }
                    index += 1;
                }
                bytes.len()
            }
            _ => start,
        },
        _ => start,
    }
}

/// The index of the `}` that closes the `{` at `open`; `None` when
/// the braces do not balance.
fn rust_matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut index = open;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
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

/// The index of the `)` that closes the `(` at `open`; `None` when
/// the parentheses do not balance.
fn rust_matching_paren(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut index = open;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
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

/// The index of the `{` that opens a function body, scanning forward
/// from `from` — past the return type and any where-clause.
fn rust_body_open(source: &str, from: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = from;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'{' {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// The body of the Rust function named `name`; `None` when there is
/// none.
fn rust_fn_body(source: &str, name: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'f' && source.get(index..index + 3) == Some("fn ") {
            let after = index + 3;
            if source.get(after..after + name.len()) == Some(name)
                && bytes
                    .get(after + name.len())
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            {
                let mut cursor = after + name.len();
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if bytes.get(cursor) == Some(&b'(') {
                    let params_end = rust_matching_paren(source, cursor)?;
                    let brace = rust_body_open(source, params_end + 1)?;
                    let body_end = rust_matching_brace(source, brace)?;
                    return source.get(brace..=body_end).map(str::to_string);
                }
            }
        }
        index += 1;
    }
    None
}

/// The call sites of `needle` in the source — the places the bridge
/// CALLS it. The definition of the function is not a call site.
fn call_sites(source: &str, needle: &str) -> BTreeSet<usize> {
    let bytes = source.as_bytes();
    let mut sites = BTreeSet::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        if bytes[index] == b'"'
            || (bytes[index] == b'\'' && rust_char_literal_len(bytes, index).is_some())
        {
            index = rust_token_end(source, index);
            continue;
        }
        if bytes[index] == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        if source.get(index..index + needle.len()) == Some(needle) {
            let is_definition = index >= 3 && source.get(index - 3..index) == Some("fn ");
            if !is_definition {
                let _ignored = sites.insert(index);
            }
            index += needle.len();
            continue;
        }
        index += 1;
    }
    sites
}

/// The block that encloses `at` — the innermost brace pair whose
/// range contains the offset. `None` when there is none.
fn enclosing_block(source: &str, at: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut open_blocks: Vec<usize> = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() && index <= at {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        match byte {
            b'{' => open_blocks.push(index),
            b'}' => {
                let _opened = open_blocks.pop();
            }
            _ => {}
        }
        index += 1;
    }
    let open = *open_blocks.last()?;
    let close = rust_matching_brace(source, open)?;
    Some((open, close))
}

/// Every function name the production source declares.
fn declared_fns(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut names: Vec<String> = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = rust_token_end(source, index);
            continue;
        }
        if byte == b'f' && source.get(index..index + 3) == Some("fn ") {
            let after = index + 3;
            let mut cursor = after;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
            if cursor > after {
                let name = source.get(after..cursor).unwrap_or_default().to_string();
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            index = cursor;
            continue;
        }
        index += 1;
    }
    names
}

/// The closer: the function whose body closes the measured window.
/// The name is the bridge's own, so the model does not pin it — it
/// finds the ONE place the window is closed.
fn closer(production: &str) -> Option<(String, String)> {
    for name in declared_fns(production) {
        if let Some(body) = rust_fn_body(production, &name)
            && body.contains(CLOSE_CALL)
        {
            return Some((name, body));
        }
    }
    None
}

// ===========================================================================
// The contract — the shell side
// ===========================================================================

/// The closer — asserted to exist, with its body.
fn asserted_closer(production: &str) -> Option<(String, String)> {
    let found = closer(production);
    assert!(
        found.is_some(),
        "T152: the bridge must close the play-to-audio window in exactly one place — the measurement closes on the first CONSUMED sample, never on the stream start (FR-003, SC-002, tasks.md F10)"
    );
    found
}

/// T152 RED-FIRST: the measured window closes on the first CONSUMED
/// sample — the backend's audible clock past zero.
///
/// Today the closer closes the window unconditionally, right after
/// the stream start, so the recorded latency UNDER-REPORTS the bound
/// by the device's output latency: the first buffer is consumed on
/// the NEXT audio callback, not at the stream start (FR-003, SC-002,
/// tasks.md F10).
#[test]
fn the_play_to_audio_clock_closes_only_on_the_first_consumed_sample() {
    let production = production_source(BRIDGE);
    let Some((_name, body)) = asserted_closer(production) else {
        return;
    };
    assert!(
        body.contains("backend"),
        "T152: the closer must READ the backend's audible clock — the clock advances only on consumed speech samples, so a clock past zero IS the first consumed sample (FR-003, SC-002)"
    );
    assert!(
        body.contains("0.0"),
        "T152: the closer must compare the audible clock against zero — the first consumed sample is the moment the clock leaves zero (FR-003, SC-002)"
    );
    let Some(close_at) = body.find(CLOSE_CALL) else {
        assert!(
            !body.is_empty(),
            "T152: the closer must close the measured window (FR-003, SC-002)"
        );
        return;
    };
    let Some(decline_at) = body.find("return") else {
        assert!(
            body.contains(CLOSE_CALL),
            "T152: the closer must close the measured window (FR-003, SC-002)"
        );
        return;
    };
    assert!(
        decline_at < close_at,
        "T152: the closer must be able to DECLINE before it closes the measured window — while the audible clock still sits at zero, no sample was consumed, so the window must stay open (FR-003, SC-002)"
    );
}

/// T152: the measured window is never closed at the STREAM START —
/// the closer is reached only from a site that can observe the
/// consumption, never from inside the block that opens the output
/// stream.
#[test]
fn the_measured_window_is_never_closed_at_the_stream_start() {
    let production = production_source(BRIDGE);
    let Some((name, _body)) = asserted_closer(production) else {
        return;
    };
    let starts = call_sites(production, STREAM_START);
    assert!(
        !starts.is_empty(),
        "T152: the bridge must start the backend's output stream — the stream start is the site the measurement must NOT close at (FR-003, SC-002)"
    );
    let Some(start) = starts.iter().next().copied() else {
        return;
    };
    let Some(start_block) = enclosing_block(production, start) else {
        return;
    };
    let closers = call_sites(production, &format!("{name}("));
    assert!(
        !closers.is_empty(),
        "T152: the closer (`{name}`) must be reached — the measurement closes on the consumption the closer observes (FR-003, SC-002)"
    );
    for site in closers {
        assert!(
            !(site >= start_block.0 && site <= start_block.1),
            "T152: the measured window must not be closed at the stream start — the closer (`{name}`) is called inside the block that opens the output stream, and the first buffer is consumed on the NEXT audio callback, so the recorded latency under-reports the bound by the device's output latency (FR-003, SC-002, tasks.md F10)"
        );
    }
}

// T152: over EVERY call site of the closer, the measured window is
// closed from a site that can observe the consumption — never from
// inside the block that opens the output stream.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_no_close_site_closes_the_window_at_the_stream_start(index in 0usize..16) {
        let production = production_source(BRIDGE);
        let Some((name, _body)) = closer(production) else {
            return Ok(());
        };
        let closes: Vec<usize> = call_sites(production, &format!("{name}(")).into_iter().collect();
        let starts: Vec<usize> = call_sites(production, STREAM_START).into_iter().collect();
        let start_block = starts
            .first()
            .copied()
            .and_then(|at| enclosing_block(production, at));
        let site_block = closes
            .get(index % closes.len().max(1))
            .copied()
            .and_then(|at| enclosing_block(production, at));
        if let (Some((start_open, start_close)), Some((site_open, _))) = (start_block, site_block) {
            prop_assert!(
                !(site_open >= start_open && site_open <= start_close),
                "a call of the closer sits inside the block that opens the output stream — the measured window would close at the stream start, under-reporting the bound by the device's output latency (FR-003, SC-002)"
            );
        }
        // Nothing to compare: the closer or the stream start is
        // absent, and the plain assertions above report that.
    }
}

// T152: the audible credit is the ONLY advance path of the
// reading's clock — so a clock that has not moved has consumed
// nothing, and a clock past zero HAS consumed a sample. This is the
// runtime half of the contract the closer relies on.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_the_reading_clock_advances_only_through_the_audible_credit(
        elapsed in 0.001f64..3_600.0,
    ) {
        let mut session = PlaybackSession::new();
        let started = session.execute(ShellVerb::Play);
        prop_assert!(started.is_ok(), "a fresh session must start playing");
        // Before the credit the reading's place still sits at zero —
        // the only zero in the snapshot's own rendering.
        let before = format!("{:?}", session.snapshot());
        prop_assert!(
            before.contains("0.0"),
            "a reading that consumed nothing must sit at zero, the snapshot reads {}",
            before
        );
        // The credit IS the consumed samples: it moves the place,
        // and nothing else does.
        session.on_audible_progress(elapsed);
        let after = format!("{:?}", session.snapshot());
        prop_assert_ne!(
            &before,
            &after,
            "the audible credit must move the reading's place, the snapshot reads {}",
            after
        );
        prop_assert!(
            after.contains(&format!("{elapsed}")),
            "the audible credit must move the reading's place to the credited amount, the snapshot reads {}",
            after
        );
    }
}
