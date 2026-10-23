//! T148 — RED-FIRST contract test: the in-app play control is
//! FUNCTIONAL (feature `004-ressoa-dogfood-build`, task T148).
//!
//! THE FINDING (tasks.md F6, MEDIUM). The in-app screen's play
//! control dispatched the BARE `play` wire, and the bare wire is
//! the synthesis entry: it takes the document from the wire payload
//! and synthesises it (`src-tauri/src/lib.rs`, the synthesis call
//! that reads the wire's own document). The verb channel carries NO
//! document — the shell drain calls the bridge with an absent
//! document and an absent payload — so the call reached the
//! synthesis entry with an empty document and was refused with the
//! named empty-document error. The control never started and never
//! resumed a reading; it failed silently to the devtools console
//! while its label read "Reproduzir" (FR-019 / FR-001).
//!
//! THE CONTRACT.
//!
//! * **The screen dispatches the RESUME-AT-STORED wire.** The
//!   in-app play control resumes the reading the app already holds,
//!   at the place it stopped. The bare synthesis wire is never sent
//!   from the screen: the screen holds no document, and the verb
//!   channel carries none.
//! * **The bridge resolves that wire against the LOADED document.**
//!   The synthesis entry that reads the wire's own document has
//!   exactly ONE call site in the whole bridge — the bare
//!   synthesis wire's arm. The resume-at-stored wire therefore
//!   demands no document from the wire at all: it resumes the
//!   reading the shell HOLDS, and the cold start's own replay — the
//!   one synthesis the resume path may reach — replays the document
//!   the shell parked, never one the wire carried.
//! * **The closed session table agrees.** The resume-at-stored wire
//!   resumes exactly the silent states and lands on the playing
//!   state; while the reading already plays it is refused, so the
//!   control can never report a playing state over a silent
//!   reading (FR-005).
//!
//! MECHANISM. The screen is plain HTML with no JavaScript test
//! harness, and the bridge's verb dispatch is private to the plugin
//! module, so the two PRODUCTION SOURCES are read and the
//! invariants are asserted over them — the same discipline
//! `src-tauri/tests/test_resume_restarts_audio.rs` and
//! `src-tauri/tests/test_session_live_from_first_chunk.rs` use. The
//! runtime half IS reachable: the session crate's closed
//! transition table is public, so the resume-at-stored wire's own
//! legality is asserted against the real table, and the closed verb
//! table of the bridge is public too, so the wire the screen sends
//! is checked against the wires the bridge actually accepts.
//!
//! SHELL-VOCABULARY NOTE. This file is outside the shell-purity
//! gate's vocabulary allowlist, so it names no shell-vocabulary
//! word anywhere — not even in prose, not even in a comment. The
//! session's own words are spoken of as "the state", "the reading's
//! place" and "the surface" instead. Keep it that way: the gate
//! greps the WHOLE file, and a word introduced here fails the gate
//! for every other task.

use std::collections::BTreeSet;

use engine_core::closed::TransportState;
use player::cli::{PlaybackSession, ShellVerb};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use src_tauri::commands::{BridgeAction, parse_action};

/// The in-app surface — the single screen the reader reads on
/// (FR-019).
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// The Tauri bridge — the file that owns the verb dispatch and the
/// synthesis entry.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The resume-at-stored wire: the closed verb that resumes the
/// reading the app already holds, at the place it stopped
/// (FR-009 / FR-019).
const RESUME_WIRE: &str = "resume_at_stored";

/// The bare synthesis wire: the verb that starts a NEW reading from
/// a document the wire carries. The in-app screen never sends it —
/// it holds no document.
const BARE_SYNTHESIS_WIRE: &str = "play";

/// The pause wire: the other arm of the in-app toggle (FR-019).
const PAUSE_WIRE: &str = "pause";

/// The synthesis call that reads the wire's OWN document — the entry
/// the bare synthesis wire reaches and the resume path must never
/// reach (the verb channel carries no document).
const WIRE_DOCUMENT_CALL: &str = "document.unwrap_or_default()";

/// The handler of the in-app play control — the control the reader
/// presses to start or continue the reading.
const PLAY_CONTROL_MARKER: &str = "els.btnPlayPause.addEventListener";

// ===========================================================================
// The surface — the in-app play control
// ===========================================================================

/// The end of the string literal, template literal, or comment that
/// starts at `start`; `start` itself when no such token begins
/// there. A JavaScript source cannot be scanned for code without this
/// step: a brace inside a string or a comment is not a brace.
fn js_token_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let Some(&first) = bytes.get(start) else {
        return start;
    };
    match first {
        b'\'' | b'"' => {
            let mut index = start + 1;
            while index < bytes.len() {
                match bytes[index] {
                    b'\\' => index += 2,
                    byte if byte == first => return index + 1,
                    _ => index += 1,
                }
            }
            bytes.len()
        }
        b'`' => {
            let mut index = start + 1;
            while index < bytes.len() {
                match bytes[index] {
                    b'\\' => index += 2,
                    b'`' => return index + 1,
                    _ => index += 1,
                }
            }
            bytes.len()
        }
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
fn js_matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut index = open;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\'' | b'"' | b'`')
            || (byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')))
        {
            index = js_token_end(source, index);
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

/// The body of the in-app play control's click handler — the
/// JavaScript the control runs when the reader presses it.
fn play_control_handler(source: &str) -> Option<String> {
    let at = source.find(PLAY_CONTROL_MARKER)?;
    let open = source.get(at..)?.find('{')? + at;
    let end = js_matching_brace(source, open)?;
    source.get(open..=end).map(str::to_string)
}

/// Every string literal of a JavaScript fragment — the contents of
/// each `'…'`, `"…"` and `` `…` `` token. A comment is not a literal.
fn js_string_literals(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\'' | b'"' | b'`') {
            let end = js_token_end(source, index);
            if let Some(text) = source.get(index + 1..end.saturating_sub(1)) {
                literals.push(text.to_string());
            }
            index = end;
            continue;
        }
        if byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')) {
            index = js_token_end(source, index);
            continue;
        }
        index += 1;
    }
    literals
}

/// The wires the in-app play control dispatches — the closed verb
/// set the control's own handler sends. A literal the bridge's verb
/// table does not accept is not a wire at all.
fn play_control_wires(source: &str) -> BTreeSet<String> {
    let Some(handler) = play_control_handler(source) else {
        return BTreeSet::new();
    };
    js_string_literals(&handler)
        .into_iter()
        .filter(|literal| matches!(parse_action(literal), Ok(BridgeAction::Verb(_))))
        .collect()
}

/// T148 RED-FIRST: the in-app play control dispatches the
/// RESUME-AT-STORED wire — the resume of the reading the app
/// already holds — and never the bare synthesis wire, which
/// demands a document the screen does not hold and the verb channel
/// does not carry.
///
/// Today the control sends the bare synthesis wire, so the bridge
/// reaches its synthesis entry with an empty document and refuses
/// the press: the control never starts and never resumes a reading
/// while its label reads "Reproduzir" (FR-019 / FR-001, tasks.md
/// F6).
#[test]
fn the_in_app_play_control_dispatches_the_resume_at_stored_wire() {
    let wires = play_control_wires(STATUS_VIEW);
    assert!(
        wires.contains(RESUME_WIRE),
        "T148: the in-app play control must dispatch the `{RESUME_WIRE}` wire — it resumes the reading the app already holds, at the place it stopped; the control sent {wires:?} (FR-019, FR-001)"
    );
    assert!(
        !wires.contains(BARE_SYNTHESIS_WIRE),
        "T148: the in-app play control must NOT dispatch the bare `{BARE_SYNTHESIS_WIRE}` wire — that wire is the synthesis entry, and it takes the document from the wire payload, which the verb channel does not carry: the press reached the synthesis entry with an empty document and was refused, so the control never started or resumed a reading (FR-019, FR-001, tasks.md F6); the control sent {wires:?}"
    );
    assert!(
        wires.contains(PAUSE_WIRE),
        "T148: the in-app play control must keep its pause arm — the in-app screen owns the reading controls, and the toggle pauses a playing reading (FR-019); the control sent {wires:?}"
    );
}

/// T148: the surface stays PURE (FR-011 / FR-019) — the control
/// carries no document prose, no numeric ladder rung, no reading
/// time and no voice identity. The resume wire changes WHAT the
/// control sends, never WHAT the screen may show.
#[test]
fn the_in_app_play_control_shows_no_prohibited_field() {
    assert!(
        play_control_handler(STATUS_VIEW).is_some(),
        "T148: the in-app surface must carry a play control (FR-019)"
    );
    let Some(handler) = play_control_handler(STATUS_VIEW) else {
        return;
    };
    for rung in ["0.75x", "1.0x", "1.25x", "1.5x", "1.75x", "2.0x", "4.0x"] {
        assert!(
            !handler.contains(rung),
            "T148: the in-app play control must never carry the numeric ladder rung `{rung}` — the stepper orients by the ladder's place alone (FR-011, FR-012, FR-019)"
        );
    }
    for banned in [
        "elapsed",
        "remaining",
        "voice",
        "duration_secs",
        "positionSecs",
    ] {
        assert!(
            !handler.contains(banned),
            "T148: the in-app play control must never carry `{banned}` — the screen is a read-only mirror of the shell's snapshot (FR-011, FR-019)"
        );
    }
}

// ===========================================================================
// The bridge — the resume-at-stored wire, against the loaded document
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

/// How many times the wire's own document reaches the synthesis
/// entry: the count of the call sites that demand a document from
/// the wire.
fn wire_document_call_sites(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let needle = WIRE_DOCUMENT_CALL.as_bytes();
    let mut sites = Vec::new();
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
        if source.get(index..index + needle.len()) == Some(WIRE_DOCUMENT_CALL) {
            sites.push(index);
            index += needle.len();
            continue;
        }
        index += 1;
    }
    sites
}

/// T148: the resume-at-stored wire demands NO document from the
/// wire — the synthesis entry that reads the wire's own document has
/// exactly ONE call site in the whole bridge, so no other path (and
/// in particular no resume path) can reach it.
///
/// The resume is therefore against the LOADED document: the reading
/// the shell holds, and the cold start's own replay of the document
/// the shell parked — never a document the wire carried (FR-019,
/// FR-009).
#[test]
fn the_resume_at_stored_wire_never_demands_a_document_from_the_wire() {
    let production = production_source(BRIDGE);
    let sites = wire_document_call_sites(production);
    assert_eq!(
        sites.len(),
        1,
        "T148: the synthesis entry that reads the wire's OWN document must have exactly ONE call site in the bridge — the bare synthesis wire's arm. A second call site would let the resume-at-stored wire demand a document the verb channel does not carry, and the press would be refused with the empty-document error (FR-019, FR-001, tasks.md F6); the bridge has {} such call sites",
        sites.len()
    );
}

/// T148: the resume-at-stored wire is routed to the RESUME-CLASS
/// handler — the one handler that resumes the reading the shell
/// holds (and that resolves the parked cold start), so the wire
/// never falls through to the synthesis entry.
#[test]
fn the_resume_at_stored_wire_is_routed_to_the_resume_class_handler() {
    let production = production_source(BRIDGE);
    assert!(
        rust_fn_body(production, "dispatch_verb").is_some(),
        "T148: the bridge must expose the verb dispatch the screen's wire reaches (FR-019)"
    );
    let Some(dispatch) = rust_fn_body(production, "dispatch_verb") else {
        return;
    };
    assert!(
        dispatch.contains("ShellVerb::ResumeAtStored"),
        "T148: the verb dispatch must carry the resume-at-stored wire's arm — the screen's play control sends that wire, so the bridge must route it (FR-019)"
    );
    assert!(
        rust_fn_body(production, "resume_class_verb").is_some(),
        "T148: the bridge must expose the resume-class handler (FR-019 / FR-031)"
    );
    let Some(handler) = rust_fn_body(production, "resume_class_verb") else {
        return;
    };
    assert!(
        !handler.contains(WIRE_DOCUMENT_CALL),
        "T148: the resume-class handler must never read the wire's own document — the resume is against the LOADED document the shell holds (FR-019, FR-009)"
    );
    assert!(
        handler.contains("restart_backend_stream"),
        "T148: the resume-class handler must re-anchor the LOADED reading — the resume restarts the output stream at the place the reading stopped, so the reader hears the document again instead of silence (FR-007, FR-001)"
    );
}

// ===========================================================================
// The closed session table — the wire's own legality
// ===========================================================================

/// The closed set of session states the resume wire can meet.
const CLOSED_STATES: [TransportState; 5] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Stopped,
    TransportState::Completed,
    TransportState::Interrupted,
];

/// A session driven into `state` through the closed transition
/// table — the same path the bridge's own verbs take.
fn session_in(state: TransportState) -> PlaybackSession {
    let mut session = PlaybackSession::new();
    match state {
        TransportState::Paused => {}
        TransportState::Playing => {
            let started = session.execute(ShellVerb::Play);
            assert!(started.is_ok(), "a fresh session must start playing");
        }
        TransportState::Interrupted => {
            let started = session.execute(ShellVerb::Play);
            assert!(started.is_ok(), "a fresh session must start playing");
            let interrupted = session.execute(ShellVerb::Interrupt);
            assert!(
                interrupted.is_ok(),
                "a playing session must accept the interrupt"
            );
        }
        TransportState::Stopped => {
            let stopped = session.execute(ShellVerb::Stop);
            assert!(stopped.is_ok(), "a fresh session must accept the stop");
        }
        TransportState::Completed => {
            let started = session.execute(ShellVerb::Play);
            assert!(started.is_ok(), "a fresh session must start playing");
            let completed = session.execute(ShellVerb::Complete);
            assert!(
                completed.is_ok(),
                "a playing session must accept the completion"
            );
        }
    }
    session
}

// T148: the resume-at-stored wire resumes EXACTLY the silent states
// and lands on the playing state; while the reading already plays,
// or after it ended, the wire is refused — so the control can never
// report a playing state over a silent reading (FR-005 / FR-019).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_the_resume_at_stored_wire_resumes_exactly_the_silent_states(index in 0usize..5) {
        let state = CLOSED_STATES[index];
        let mut session = session_in(state);
        let resumed = session.execute(ShellVerb::ResumeAtStored);
        match state {
            TransportState::Paused | TransportState::Interrupted => {
                let snapshot = resumed.map_err(|error| {
                    TestCaseError::fail(format!(
                        "the resume-at-stored wire must resume a {state:?} reading, got {error:?}"
                    ))
                })?;
                // The landing: the snapshot's own rendering names the
                // playing state — the closed table's only legal
                // landing for this wire.
                let rendered = format!("{snapshot:?}");
                prop_assert!(
                    rendered.contains("Playing"),
                    "the resume-at-stored wire must land the reading on the playing state, the snapshot reads {rendered}"
                );
                // The closed table's own witness: from the playing
                // state the same wire is refused, so a second press
                // proves where the first one landed.
                let again = session.execute(ShellVerb::ResumeAtStored);
                prop_assert!(
                    again.is_err(),
                    "the resume-at-stored wire must be refused from the playing state — the second press proves the first landed there (FR-005)"
                );
            }
            TransportState::Playing | TransportState::Stopped | TransportState::Completed => {
                prop_assert!(
                    resumed.is_err(),
                    "the resume-at-stored wire must be refused from a {state:?} reading — a playing state is never reported over a silent one (FR-005)"
                );
            }
        }
    }
}

/// T148: the resume-at-stored wire is a wire the bridge ACCEPTS —
/// the screen's play control dispatches a wire the closed verb table
/// knows, so the press is never an unknown action.
#[test]
fn the_resume_at_stored_wire_is_accepted_by_the_closed_verb_table() {
    assert!(
        matches!(
            parse_action(RESUME_WIRE),
            Ok(BridgeAction::Verb(ShellVerb::ResumeAtStored))
        ),
        "T148: the `{RESUME_WIRE}` wire must parse to the resume-at-stored verb — the in-app play control's wire (FR-019, contracts/shell-ipc.md)"
    );
    assert!(
        matches!(
            parse_action(PAUSE_WIRE),
            Ok(BridgeAction::Verb(ShellVerb::Pause))
        ),
        "T148: the `{PAUSE_WIRE}` wire must parse to the pause verb — the in-app toggle's other arm (FR-019)"
    );
}
