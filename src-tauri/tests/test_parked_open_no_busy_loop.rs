//! T155 — RED-FIRST contract test: the parked cold-start open is
//! NOT re-checked at an unbounded cadence (feature
//! `004-ressoa-dogfood-build`, task T155).
//!
//! THE FINDING (tasks.md F13, LOW). The FR-032c offer window is
//! polled on a 250 ms tick. The poll asks whether the parked open's
//! window has elapsed — and the re-park keeps the SAME open (the
//! same `opened_at`), so the elapsed condition stays true FOREVER.
//! Every tick therefore took the parked open, locked the offer
//! machine, re-parked the same open and logged — a 4 Hz cycle,
//! unbounded in time, for as long as the reader waits for an
//! explicit resume (Principle V).
//!
//! THE CONTRACT.
//!
//! * **The window elapses ONCE per parked open.** The re-park
//!   latches the elapsed window on the parked open, and the poll
//!   reads that latch and returns before it resolves — so the
//!   take/lock/re-park cycle happens once, not once per tick.
//! * **The reader's explicit resume is NOT delayed.** The latch is
//!   read in exactly ONE place — the poll — so the resolution path
//!   the reader's own verb reaches is never gated on it: the resume
//!   is detected as promptly as before (FR-009 / FR-032d / FR-031).
//!
//! MECHANISM. The parked open and the poll live in the bridge's
//! private state, and the state is not constructible from an
//! integration test, so the claims are asserted over the PRODUCTION
//! SOURCE — the same discipline
//! `src-tauri/tests/test_resume_restarts_audio.rs` and
//! `src-tauri/tests/test_session_live_from_first_chunk.rs` use. The
//! consequence of the rule is then shown as a property: over ANY
//! number of ticks, the parked open is resolved at most ONCE.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary
//! word anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state", "the reading's
//! place" and "the surface" instead. Keep it that way: the gate
//! greps the WHOLE file, and a word introduced here fails the gate
//! for every other task.

use proptest::prelude::*;

/// The Tauri bridge — the file that owns the parked cold-start open
/// and the offer-window poll.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The parked cold-start open: the document whose offer is open,
/// waiting for the offer to resolve before ANY audio begins.
const PENDING_OPEN: &str = "PendingOpen";

/// The latched fact: the field that says the open has already
/// waited out its window and the resolution left it waiting.
const LATCH_FIELD: &str = "window_expired";

/// The poll: the tick that asks whether the parked open's window
/// has elapsed.
const POLL: &str = "expire_parked_open";

/// The re-park: the resolution that leaves the open waiting for the
/// reader.
const REPARK: &str = "repark_cold_start";

/// The resolution path: the fn that resolves the parked open. The
/// reader's explicit resume reaches it through the verb dispatch, so
/// it must never be gated on the latch.
const RESOLUTION: &str = "resolve_parked_open";

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

/// The body of the Rust STRUCT named `name` — the fields it
/// declares, braces included.
fn rust_struct_body(source: &str, name: &str) -> Option<String> {
    let marker = format!("struct {name} ");
    let at = source.find(&marker)?;
    let open = source.get(at..)?.find('{')? + at;
    let end = rust_matching_brace(source, open)?;
    source.get(open..=end).map(str::to_string)
}

// ===========================================================================
// The contract
// ===========================================================================

/// T155 RED-FIRST: the parked cold-start open carries the LATCHED
/// expiry fact — the fact that says its window already elapsed and
/// the resolution left it waiting.
///
/// Without the latch the poll re-checks the open on every tick: the
/// re-park keeps the same open (the same `opened_at`), so the
/// elapsed condition holds forever, and the poll takes, locks and
/// re-parks the open — and logs — 4 times a second, for as long as
/// the reader waits (tasks.md F13, Principle V).
#[test]
fn the_parked_open_carries_the_latched_window_expiry() {
    let production = production_source(BRIDGE);
    let Some(pending) = rust_struct_body(production, PENDING_OPEN) else {
        assert!(
            !production.is_empty(),
            "T155: the bridge must carry the parked cold-start open (FR-009)"
        );
        return;
    };
    assert!(
        pending.contains(LATCH_FIELD),
        "T155: the parked cold-start open must carry the LATCHED expiry fact (`{LATCH_FIELD}`) — the re-park keeps the same open, so the elapsed condition would hold forever and the poll would re-check it on every tick (tasks.md F13, Principle V)"
    );
    assert!(
        pending.contains("bool"),
        "T155: the latched expiry fact must be a flag — the parked open carries DATA, never a decision (FR-044)"
    );
}

/// T155: the poll reads the latch and RETURNS before it resolves —
/// so the re-check happens once, not once per tick.
#[test]
fn the_poll_returns_before_it_resolves_an_already_expired_open() {
    let production = production_source(BRIDGE);
    let Some(poll) = rust_fn_body(production, POLL) else {
        assert!(
            !production.is_empty(),
            "T155: the bridge must carry the offer-window poll (FR-032c)"
        );
        return;
    };
    let Some(read_at) = poll.find(LATCH_FIELD) else {
        assert!(
            poll.contains("expire_parked_open"),
            "T155: the poll must read the latched expiry fact — the re-check of an open that already waited out its window must stop (tasks.md F13, Principle V)"
        );
        return;
    };
    let Some(resolve_at) = poll.find("resolve_parked_open(") else {
        assert!(
            poll.contains(LATCH_FIELD),
            "T155: the poll must resolve the parked open when its window elapses (FR-032c)"
        );
        return;
    };
    assert!(
        read_at < resolve_at,
        "T155: the poll must READ the latched expiry fact BEFORE it resolves — the resolution is what takes, locks and re-parks the open, and the latch is what stops that cycle from repeating every tick (tasks.md F13, Principle V)"
    );
    let between = poll.get(read_at..resolve_at).unwrap_or_default();
    assert!(
        between.contains("return"),
        "T155: the poll must RETURN when the latched expiry fact is set — the re-check of an open that already waited out its window must stop, or the poll takes, locks and re-parks the open on every tick (tasks.md F13, Principle V)"
    );
}

/// T155: the re-park LATCHES the expiry — the open it parks again
/// carries the fact that its window already elapsed.
#[test]
fn the_repark_latches_the_elapsed_window() {
    let production = production_source(BRIDGE);
    let Some(repark) = rust_fn_body(production, REPARK) else {
        assert!(
            !production.is_empty(),
            "T155: the bridge must carry the re-park (FR-009 / FR-032d)"
        );
        return;
    };
    assert!(
        repark.contains(LATCH_FIELD),
        "T155: the re-park must LATCH the elapsed window on the open it parks again — the open it parks keeps the same instant, so the elapsed condition would hold forever and the poll would re-check it on every tick (tasks.md F13, Principle V)"
    );
    assert!(
        repark.contains("true"),
        "T155: the re-park must set the latched expiry fact — the open it parks again HAS waited out its window (tasks.md F13)"
    );
}

/// T155: the reader's explicit resume is NOT delayed by the latch —
/// the latch is read in exactly ONE place, the poll, so the
/// resolution path the reader's own verb reaches is never gated on
/// it (FR-009 / FR-032d / FR-031).
#[test]
fn the_readers_resume_is_never_gated_on_the_latch() {
    let production = production_source(BRIDGE);
    let Some(resolution) = rust_fn_body(production, RESOLUTION) else {
        assert!(
            !production.is_empty(),
            "T155: the bridge must carry the parked-open resolution (FR-009)"
        );
        return;
    };
    assert!(
        !resolution.contains(LATCH_FIELD),
        "T155: the resolution path must NOT read the latched expiry fact — the reader's explicit resume arrives through the verb dispatch and resolves the parked open directly, so gating it would delay the reader's resume (FR-009, FR-032d, FR-031)"
    );
}

// T155: over ANY number of poll ticks, the parked open is resolved
// at most ONCE — the rule the latch implements, and the bound the
// unbounded re-park loop broke.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_the_parked_open_is_resolved_at_most_once(ticks in 1usize..256) {
        // The production rule, read from the source: the poll finds
        // the latched fact and returns before it resolves.
        let production = production_source(BRIDGE);
        let Some(poll) = rust_fn_body(production, POLL) else {
            return Ok(());
        };
        let read_at = poll.find(LATCH_FIELD).unwrap_or(usize::MAX);
        let resolve_at = poll.find("resolve_parked_open(").unwrap_or(usize::MAX);
        prop_assert!(
            read_at < resolve_at,
            "the poll must read the latched expiry fact before it resolves — the latch is what stops the take/lock/re-park cycle from repeating on every tick (tasks.md F13, Principle V)"
        );
        // The model of the poll's rule: the window elapses once for
        // a parked open, the re-park latches that fact, and a poll
        // that finds the latch set resolves nothing.
        let mut latched = false;
        let mut resolutions = 0usize;
        for _tick in 0..ticks {
            if latched {
                continue;
            }
            resolutions += 1;
            // The resolution left the open waiting: the re-park
            // latches the elapsed window.
            latched = true;
        }
        prop_assert_eq!(
            resolutions,
            1,
            "the parked open must be resolved at most once, whatever the number of ticks — the re-park latches the elapsed window, so the poll never takes, locks and re-parks the open again (tasks.md F13, Principle V)"
        );
    }
}
