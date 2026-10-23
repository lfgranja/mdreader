//! T147 — RED-FIRST contract test: the offer-route command is
//! READ-ONLY over the resolved decision (feature
//! `004-ressoa-dogfood-build`, task T147).
//!
//! THE FINDING (tasks.md F5, MEDIUM). The `offer_route` command
//! asked the bridge's own route-resolution function which surface a
//! stored row resolves to, and that function DRIVES the offer
//! machine — the machine's decision function clears the open offer
//! and the explicit-resume requirement and re-derives them from the
//! stored row. The in-app surface polls the command on a timer
//! (every 250 ms), so a UI timer was mutating the FR-044 decision
//! core — and the invariant the bridge documented ("the route is
//! computed when a document with a stored row opens, never
//! mid-window") was false (FR-044 / FR-009).
//!
//! The mutation is not benign in principle: a re-derivation re-opens
//! an offer the window already consumed, so the resolved decision
//! does not survive a poll. The runtime half of this file shows
//! exactly that on the machine itself: the reads leave the decision
//! alone, a re-derivation resurrects it.
//!
//! THE CONTRACT.
//!
//! * **The decision is driven EXACTLY ONCE** in the whole bridge —
//!   at the cold-start open, which is the open/reopen lifecycle
//!   point the offer's own documentation names. A second driver
//!   would be a second decision (FR-044).
//! * **The route command never drives the machine.** It READS the
//!   route the open published, so the machine's resolved decision
//!   survives every poll unchanged.
//! * **The machine's own decision survives repeated reads.** The
//!   runtime half, on the machine the shell holds: the reads leave
//!   the decision alone, and a re-derivation — what the poll used to
//!   do — resurrects the consumed offer.
//!
//! MECHANISM. The bridge's verb dispatch and its state are private
//! to the plugin module, and the command needs the framework's own
//! state extractor, so the two shell-side claims are asserted over
//! the PRODUCTION SOURCE — the same discipline
//! `src-tauri/tests/test_resume_restarts_audio.rs` and
//! `src-tauri/tests/test_session_live_from_first_chunk.rs` use. The
//! runtime half IS reachable: the session crate's offer machine is
//! public, so the decision's own behaviour is asserted against the
//! real machine.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary
//! word anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state", "the reading's
//! place" and "the surface" instead. Keep it that way: the gate
//! greps the WHOLE file, and a word introduced here fails the gate
//! for every other task.

use std::collections::BTreeSet;

use engine_core::closed::SessionEnd;
use session::offer::{ExpiryOutcome, OfferMachine, OpenOffer};
use session::store::StoredPosition;

use proptest::prelude::*;

/// The Tauri bridge — the file that owns the offer machine, the
/// route resolution and the route command.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The bridge's own route-resolution call: the function that drives
/// the offer machine and answers which surface a stored row resolves
/// to. Its NAME is the bridge's vocabulary; the fact that the call
/// drives the machine is the contract.
const ROUTE_RESOLUTION_CALL: &str = "route_offer(";

/// The bridge's cold-start open: the open/reopen lifecycle point the
/// offer's own documentation names as the ONE place the decision is
/// resolved.
const OPEN_PATH: &str = "open_document_bytes_named";

/// The route command the in-app surface polls.
const ROUTE_COMMAND: &str = "offer_route";

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

// ===========================================================================
// The contract — the shell side
// ===========================================================================

/// T147 RED-FIRST: the offer machine's decision is driven EXACTLY
/// ONCE in the whole bridge — at the cold-start open, the
/// open/reopen lifecycle point the offer's own documentation names.
///
/// Today the route command drives it too, on every poll — so the
/// decision is driven twice, and a UI timer rewrites the FR-044
/// decision core (FR-044 / FR-009, tasks.md F5).
#[test]
fn the_offer_machines_decision_is_driven_exactly_once() {
    let production = production_source(BRIDGE);
    let sites = call_sites(production, ROUTE_RESOLUTION_CALL);
    assert_eq!(
        sites.len(),
        1,
        "T147: the offer machine's decision must be driven EXACTLY ONCE in the bridge — at the cold-start open, the open/reopen lifecycle point. {} call sites drive it, so a surface poll (every 250 ms) rewrites the FR-044 decision core (FR-044, FR-009, tasks.md F5); the call sites are at {sites:?}",
        sites.len()
    );
}

/// T147: that ONE resolution happens in the cold-start open — the
/// place the offer's own documentation names as the lifecycle point
/// where the offer is resolved and read at expiry afterwards.
#[test]
fn the_single_resolution_happens_at_the_cold_start_open() {
    let production = production_source(BRIDGE);
    assert!(
        rust_fn_body(production, OPEN_PATH).is_some(),
        "T147: the bridge must expose the cold-start open — the open/reopen lifecycle point (FR-009)"
    );
    let Some(open_path) = rust_fn_body(production, OPEN_PATH) else {
        return;
    };
    assert!(
        open_path.contains(ROUTE_RESOLUTION_CALL),
        "T147: the cold-start open must drive the offer machine's decision — it is the ONE lifecycle point where the offer is resolved, and the route it resolves is published for the surfaces to read (FR-009, FR-044)"
    );
}

/// T147: the route command NEVER drives the offer machine — it reads
/// the route the open published. A command that drove the machine
/// would mutate the resolved decision on every poll (FR-044 /
/// FR-009).
#[test]
fn the_route_command_never_drives_the_offer_machine() {
    let production = production_source(BRIDGE);
    assert!(
        rust_fn_body(production, ROUTE_COMMAND).is_some(),
        "T147: the bridge must expose the route command the in-app surface polls (FR-009)"
    );
    let Some(command) = rust_fn_body(production, ROUTE_COMMAND) else {
        return;
    };
    assert!(
        !command.contains(ROUTE_RESOLUTION_CALL),
        "T147: the route command must NOT drive the offer machine — the poll runs on a timer (every 250 ms), and driving the machine mutates the resolved decision (a re-derivation re-opens an offer the window already consumed), so a UI timer would rewrite the FR-044 decision core (FR-044, FR-009, tasks.md F5)"
    );
}

// ===========================================================================
// The contract — the machine's own behaviour
// ===========================================================================

/// A stored row of the given end kind — the row the decision is
/// resolved from.
const fn stored_row(end_kind: SessionEnd) -> StoredPosition {
    StoredPosition {
        offset_secs: 12.5,
        first_yield_ms: 250,
        end_kind,
    }
}

/// T147: the machine's resolved decision SURVIVES repeated reads —
/// the same decision, unchanged, poll after poll.
///
/// The read is the only thing a surface poll may do. The reads
/// leave the decision exactly where the resolution left it, and the
/// decision a re-derivation would give is the mutation the route
/// command must not make (FR-044 / FR-009).
#[test]
fn the_resolved_decision_survives_repeated_reads() {
    let mut machine = OfferMachine::new();
    // The ONE resolution — the cold-start open.
    let decision = machine.resolve_open(Some(stored_row(SessionEnd::Paused)));
    assert!(
        matches!(decision, OpenOffer::Continue { .. }),
        "T147: a deliberate pause must resolve the continue form (FR-032c)"
    );
    // The window elapses: the offer is consumed.
    machine.on_foregrounded();
    let outcome = machine.expire();
    assert!(
        matches!(outcome, ExpiryOutcome::AutoStart { .. }),
        "T147: the silence rule must auto-start from the stored place (FR-009)"
    );
    // Repeated READS: the decision is unchanged.
    for read in 0..8 {
        assert!(
            machine.continue_offer().is_none(),
            "T147: read {read} changed the resolved decision — a read must never re-open the consumed offer (FR-044, FR-009)"
        );
    }
    // A re-derivation — what the poll used to do — RESURRECTS the
    // consumed offer: the mutation the route command must not make.
    let again = machine.resolve_open(Some(stored_row(SessionEnd::Paused)));
    assert!(
        matches!(again, OpenOffer::Continue { .. }),
        "T147: a re-derivation resolves the same form (FR-044)"
    );
    assert!(
        machine.continue_offer().is_some(),
        "T147: a re-derivation re-opens the consumed offer — the mutation a surface poll caused, and the reason the route command must only READ (FR-044, FR-009, tasks.md F5)"
    );
}

/// The strategy over the closed set of the stored end kinds the
/// decision resolves from.
fn end_kind() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![
        Just(SessionEnd::Paused),
        Just(SessionEnd::Interrupted),
        Just(SessionEnd::Stopped),
        Just(SessionEnd::Completed),
    ]
}

// T147: over EVERY stored end kind, the reads leave the machine's
// resolved decision exactly where the resolution left it — and a
// re-derivation is the only thing that can change it.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_the_reads_never_change_the_resolved_decision(kind in end_kind()) {
        let mut machine = OfferMachine::new();
        let _decision = machine.resolve_open(Some(stored_row(kind)));
        machine.on_foregrounded();
        let _outcome = machine.expire();
        let after_expiry = machine.continue_offer();
        for read in 0..8 {
            prop_assert_eq!(
                machine.continue_offer(),
                after_expiry,
                "read {} changed the resolved decision — a read must never re-open or drop the offer (FR-044, FR-009)",
                read
            );
        }
        // A re-derivation is the ONLY mutation, and it is the one
        // the route command must not make: the deliberate pause's
        // offer comes back even after the window consumed it.
        let _again = machine.resolve_open(Some(stored_row(kind)));
        let rederived = machine.continue_offer();
        if kind == SessionEnd::Paused {
            prop_assert!(
                rederived.is_some(),
                "a re-derivation must re-open the deliberate pause's consumed offer — the mutation the poll caused (FR-044, FR-009)"
            );
        } else {
            prop_assert_eq!(
                rederived,
                after_expiry,
                "a re-derivation of a {:?} row must leave the offer as the reads left it",
                kind
            );
        }
    }
}
