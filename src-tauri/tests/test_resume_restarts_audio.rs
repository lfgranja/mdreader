//! T140 — RED-FIRST property test: the explicit-resume audio path
//! (Feature 004-ressoa-dogfood-build / task T140).
//!
//! Requirement (`specs/004-ressoa-dogfood-build/spec.md`):
//!
//! * FR-001 — "The system MUST continue reading aloud with the screen
//!   off and the device locked until the user pauses or the document
//!   ends."
//! * FR-005 — a pause-demanding interruption "MUST wait for explicit
//!   Play and MUST never auto-resume"; a playing state is never
//!   reported while the session is silent.
//! * FR-007 — the system "MUST resume after an interruption at or
//!   just before the point of interruption".
//!
//! The finding this test pins (`tasks.md` F1, CRITICAL):
//!
//! * `src-tauri/src/lib.rs` — the resume-class handler executes the
//!   session transition but NEVER calls the backend's `play()`, while
//!   the pause arm DOES call the backend's `pause()`. The asymmetry
//!   IS the defect.
//! * The only caller of the backend's `play()` in the workspace is
//!   the synthesis path (`crates/player/src/backend.rs`: `play()`
//!   starts the cpal stream, `pause()` pauses it, and the sample
//!   queue SURVIVES the pause).
//!
//! Consequence: after ANY pause (mid-document, call/alarm
//! interruption, headphone removal) the reader's Play moves the
//! transport to `Playing` and publishes it, but NO AUDIO IS EMITTED:
//! the lock screen shows the play glyph and the app is silent.
//! Before T119 this path refused to start (`EmptyDocument`) and the
//! transport honestly stayed `Paused`; T119 converted "still paused"
//! into a false "playing".
//!
//! MECHANISM. The backend's `play()` is NOT observable at runtime
//! from a `src-tauri` integration test: the backend field of the
//! application state is private, the backend is a concrete type (not
//! an injectable trait), the backend's stream state has no getter,
//! and the resume handler itself is private to the bridge module. So
//! this file reads the PRODUCTION SOURCE and asserts the invariant
//! over it — the same discipline
//! `src-tauri/tests/test_lockscreen_resume_control.rs` and
//! `crates/player/tests/test_earcon_mirror.rs` already use.
//!
//! The source reading is a MODEL, not a snapshot: the closed
//! transport transition table is parsed out of the session crate, the
//! bridge's call graph is walked, and the assertions are
//! INVARIANTS — "a path that lands the transport on `Playing` starts
//! the backend's stream" and "the parked cold-start resolution is
//! not the live-resume path" — never "the file still looks exactly
//! like this".
//!
//! The model resolves the resume path precisely enough to tell the
//! three shapes apart:
//!
//! * The SYNTHESIS path owns the document synthesis: it BUILDS a
//!   reading and starts the backend as part of that build. It is the
//!   path a fresh `Play` takes, and it is NOT the live-resume path.
//! * The PARKED cold-start resolution is a different path again: it
//!   consumes a parked open and re-synthesises. It is excluded from
//!   the live-resume closure, or the defect would hide inside it.
//! * The LIVE-RESUME path is the one that moves an existing session
//!   from a silent state back to `Playing`. It must start the
//!   ALREADY-OPEN backend — the queue and the position clock survive
//!   the pause (`crates/player/src/backend.rs`), so restarting the
//!   stream is what re-anchors the audio at the resume point. A
//!   resume that reaches the backend only THROUGH the synthesis would
//!   rebuild the reading from the document instead: the audio would
//!   restart from the buffer head, not at the resume point (FR-007),
//!   and the property would still fail.
//!
//! The runtime half of the contract IS reachable, and it is asserted
//! here too: the session's own resume transition re-anchors at the
//! stored position (FR-007) and the published lock-screen payload
//! carries exactly the session's transport (FR-005), so the surface
//! can only ever report a playing state the session actually holds.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use engine_core::closed::TransportState;
use player::cli::{PlaybackSession, ShellVerb};
use player::surface::{Dismissal, Progress, SurfaceSnapshot};
use serde_json::Value;
use src_tauri::publish::build_kotlin_mirror_json;

/// The Rust bridge — the file that owns the verb dispatch and every
/// backend action.
const BRIDGE: &str = include_str!("../src/lib.rs");

/// The session crate's verb dispatch — the file that owns the closed
/// transport transition table.
const SESSION_CLI: &str = include_str!("../../crates/player/src/cli.rs");

/// The names that mark the parked cold-start resolution: the branch
/// that consumes a parked open and re-synthesises the document. The
/// live-resume path MUST NOT be conflated with it — the parked
/// resolution is a different path with a different mechanism (it
/// re-synthesises through the synthesis path, which starts the
/// backend as part of the synthesis).
const PARKED_MARKERS: [&str; 7] = [
    "park_cold_start",
    "pending_open_exists",
    "repark_cold_start",
    "resolve_parked_choice",
    "resolve_parked_open",
    "start_parked_reading",
    "take_parked_open",
];

/// The call that synthesises the document's audio: the synthesis
/// path, and the path that starts the backend as part of the
/// synthesis.
const SYNTHESIS_CALL: &str = "adapter_synthesis::synth::synthesize";

/// The backend's start call: the method call that starts the output
/// stream. The bridge's only such call is the backend's own `play()`
/// (`crates/player/src/backend.rs`).
const BACKEND_START_CALL: &str = ".play(";

/// The backend's pause call: the method call that pauses the output
/// stream while the sample queue survives.
const BACKEND_PAUSE_CALL: &str = ".pause(";

/// The marker that opens a cell of the closed transport transition
/// table.
const CELL_MARKER: &str = "(TransportState::";

/// The closed transport states the session can hold.
const CLOSED_STATES: [TransportState; 5] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Stopped,
    TransportState::Completed,
    TransportState::Interrupted,
];

/// The production part of a Rust source: everything before its
/// first inline test module.
///
/// The contract reads PRODUCTION code: a test module may name any
/// verb it likes, and a test module's mention of the backend's
/// `play()` is not a route the reader can take.
fn production_prefix(text: &str) -> &str {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        let opens_a_test_module = trimmed.starts_with("mod ")
            && (trimmed.contains("_tests") || trimmed.contains(" tests"))
            && trimmed.ends_with('{');
        if opens_a_test_module {
            return &text[..offset];
        }
        offset += line.len();
    }
    text
}

/// The source with every comment removed, the line structure kept.
///
/// The contract assertions read CODE, never prose: a doc comment
/// that names a verb proves nothing about the route that carries it,
/// and a comment that names a state proves nothing about the payload
/// that publishes it. A comment marker inside a string literal is not
/// a comment and is kept, so the scanner tracks the string state.
fn strip_comments(text: &str) -> String {
    let source: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let ch = source[index];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += 1;
            continue;
        }
        if ch == '/' && index + 1 < source.len() {
            let next = source[index + 1];
            if next == '/' {
                while index < source.len() && source[index] != '\n' {
                    index += 1;
                }
                continue;
            }
            if next == '*' {
                index += 2;
                let mut depth = 1usize;
                while index < source.len() && depth > 0 {
                    if source[index] == '/' && index + 1 < source.len() && source[index + 1] == '*'
                    {
                        depth += 1;
                        index += 2;
                        continue;
                    }
                    if source[index] == '*' && index + 1 < source.len() && source[index + 1] == '/'
                    {
                        depth -= 1;
                        index += 2;
                        continue;
                    }
                    if source[index] == '\n' {
                        out.push('\n');
                    }
                    index += 1;
                }
                continue;
            }
        }
        out.push(ch);
        index += 1;
    }
    out
}

/// Whether a character may appear inside an identifier.
fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// The index of the closing brace of the one at `open`, or `None`
/// when the source is unbalanced. A brace inside a string literal
/// never closes a block, and a lifetime never opens a character
/// literal.
fn matching_brace(source: &[char], open: usize) -> Option<usize> {
    let mut depth = 0i64;
    let mut index = open;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let ch = source[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => {
                index = skip_character_or_lifetime(source, index);
                continue;
            }
            '{' => depth += 1,
            '}' => {
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

/// The index just past the character literal or lifetime that starts
/// at `at`.
fn skip_character_or_lifetime(source: &[char], at: usize) -> usize {
    let mut cursor = at + 1;
    if cursor < source.len() && source[cursor] == '\\' {
        cursor += 1;
        while cursor < source.len() && source[cursor] != '\'' {
            cursor += 1;
        }
        return cursor + 1;
    }
    let name_start = cursor;
    while cursor < source.len() && is_ident_char(source[cursor]) {
        cursor += 1;
    }
    if cursor > name_start {
        if cursor < source.len() && source[cursor] == '\'' {
            return cursor + 1; // a character literal
        }
        return cursor; // a lifetime
    }
    at + 1
}

/// The index of the brace that opens a function body, or `None` when
/// the declaration has no body.
const fn body_open_brace(source: &[char], from: usize) -> Option<usize> {
    let mut index = from;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let ch = source[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => return Some(index),
            ';' => return None,
            _ => {}
        }
        index += 1;
    }
    None
}

/// Every function of a source, with its body text.
///
/// The scan is textual and deliberately shallow: it recognises the
/// `fn` keyword, reads the name, and takes the balanced braces of
/// the body. It is a model of the call graph, not a parser.
fn functions_of(source: &str) -> BTreeMap<String, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut bodies: BTreeMap<String, String> = BTreeMap::new();
    let mut index = 0usize;
    while index + 2 < chars.len() {
        if chars[index] != 'f' || chars[index + 1] != 'n' {
            index += 1;
            continue;
        }
        let starts_a_word = index == 0 || !is_ident_char(chars[index - 1]);
        if !starts_a_word || (index + 2 < chars.len() && !chars[index + 2].is_whitespace()) {
            index += 1;
            continue;
        }
        let mut cursor = index + 2;
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < chars.len() && is_ident_char(chars[cursor]) {
            cursor += 1;
        }
        if cursor == name_start {
            index += 2;
            continue;
        }
        let name: String = chars[name_start..cursor].iter().collect();
        let Some(open) = body_open_brace(&chars, cursor) else {
            index = cursor;
            continue;
        };
        let Some(close) = matching_brace(&chars, open) else {
            index = cursor;
            continue;
        };
        let body: String = chars[open + 1..close].iter().collect();
        let _ignored = bodies.entry(name).or_insert(body);
        index = close + 1;
    }
    bodies
}

/// The index just past the closing parenthesis of the call that opens
/// at `open`, or `None` when the parentheses are unbalanced.
const fn closing_paren(source: &[char], open: usize) -> Option<usize> {
    let mut depth = 0i64;
    let mut index = open;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let ch = source[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
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

/// The functions a body calls, by name.
fn called_functions(body: &str) -> BTreeSet<String> {
    let chars: Vec<char> = body.chars().collect();
    let mut names = BTreeSet::new();
    let mut index = 0usize;
    while index < chars.len() {
        if !is_ident_char(chars[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && is_ident_char(chars[index]) {
            index += 1;
        }
        let name: String = chars[start..index].iter().collect();
        let mut cursor = index;
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor < chars.len() && chars[cursor] == '(' {
            let _ignored = names.insert(name);
        }
    }
    names
}

/// Every `ShellVerb::` member a text names.
fn shell_verbs(text: &str) -> BTreeSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let needle: Vec<char> = "ShellVerb::".chars().collect();
    let mut verbs = BTreeSet::new();
    let mut index = 0usize;
    while index + needle.len() <= chars.len() {
        if chars[index..index + needle.len()] == needle[..] {
            let mut cursor = index + needle.len();
            let name_start = cursor;
            while cursor < chars.len() && is_ident_char(chars[cursor]) {
                cursor += 1;
            }
            if cursor > name_start {
                let _ignored = verbs.insert(chars[name_start..cursor].iter().collect::<String>());
            }
            index = cursor;
            continue;
        }
        index += 1;
    }
    verbs
}

/// The verbs a match arm's pattern names, for the arm whose `=>` is
/// at `arrow`.
fn pattern_verbs(source: &[char], arrow: usize) -> BTreeSet<String> {
    let mut start = 0usize;
    let mut index = arrow;
    while index > 0 {
        index -= 1;
        if matches!(source[index], ',' | '{' | ';' | '}') {
            start = index + 1;
            break;
        }
    }
    let pattern: String = source[start..arrow].iter().collect();
    shell_verbs(&pattern)
}

/// The index of the first occurrence of an ASCII needle at or after
/// `from`.
fn find_ascii(source: &[char], needle: &str, from: usize) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    let mut index = from;
    while index + needle.len() <= source.len() {
        if source[index..index + needle.len()] == needle[..] {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// The index of the last occurrence of an ASCII needle before
/// `before`.
fn rfind_ascii(source: &[char], needle: &str, before: usize) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    let mut index = before.min(source.len());
    while index > 0 {
        index -= 1;
        if index + needle.len() <= source.len() && source[index..index + needle.len()] == needle[..]
        {
            return Some(index);
        }
    }
    None
}

/// Whether the call that opens at `call_open` sits inside the block
/// the arm's `=>` opens.
fn call_sits_in_the_arm_body(source: &[char], arrow: usize, call_open: usize) -> bool {
    let Some(open) = find_ascii(source, "{", arrow) else {
        return false;
    };
    let mut depth = 0i64;
    let mut index = open;
    while index < call_open {
        match source[index] {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    depth >= 1
}

/// The verbs a match arm names, for the arm that encloses the call
/// whose opening parenthesis is at `call_open`.
///
/// The arm is found from its own pattern: the closest preceding
/// `ShellVerb::` names the arm's last pattern member, and the arm's
/// own `=>` is the next one after that member. The call must sit
/// inside the block that `=>` opens, so a call that is not the arm
/// body's own statement yields no verb — the conservative answer.
fn arm_verbs(source: &[char], call_open: usize) -> BTreeSet<String> {
    let Some(member) = rfind_ascii(source, "ShellVerb::", call_open) else {
        return BTreeSet::new();
    };
    let Some(arrow) = find_ascii(source, "=>", member) else {
        return BTreeSet::new();
    };
    if !call_sits_in_the_arm_body(source, arrow, call_open) {
        return BTreeSet::new();
    }
    pattern_verbs(source, arrow)
}

/// The bridge's analysed production code: the source characters and
/// the function map.
struct Bridge {
    /// The production source, character-indexed.
    chars: Vec<char>,
    /// Every production function and its body.
    functions: BTreeMap<String, String>,
}

impl Bridge {
    /// The bridge's production source, comments stripped.
    fn read() -> Self {
        let stripped = strip_comments(BRIDGE);
        let source = production_prefix(&stripped);
        Self {
            chars: source.chars().collect(),
            functions: functions_of(source),
        }
    }

    /// Whether a name belongs to the parked cold-start resolution.
    fn is_parked(name: &str) -> bool {
        PARKED_MARKERS.iter().any(|marker| name.contains(marker))
    }

    /// The functions reachable from `name` without passing through the
    /// parked cold-start resolution.
    ///
    /// The parked resolution is a DIFFERENT path with a different
    /// mechanism: it re-synthesises the document and starts the
    /// backend as part of that synthesis. Folding it into the live
    /// resume path would hide exactly the defect this test pins.
    fn live_closure(&self, name: &str) -> BTreeSet<String> {
        let mut reached = BTreeSet::new();
        let _ignored = reached.insert(name.to_string());
        let mut pending = vec![name.to_string()];
        while let Some(current) = pending.pop() {
            let Some(body) = self.functions.get(&current) else {
                continue;
            };
            for callee in called_functions(body) {
                if !self.functions.contains_key(&callee) || Self::is_parked(&callee) {
                    continue;
                }
                if reached.insert(callee.clone()) {
                    pending.push(callee);
                }
            }
        }
        reached
    }

    /// The bodies of a closure, concatenated.
    fn bodies_of(&self, closure: &BTreeSet<String>) -> String {
        let mut text = String::new();
        for name in closure {
            if let Some(body) = self.functions.get(name) {
                text.push_str(body);
                text.push('\n');
            }
        }
        text
    }
}

/// The verbs the closed transport transition table says can land the
/// transport on `Playing`.
///
/// The table is the session crate's own: the set is read out of the
/// production source, never written down twice.
fn playing_verbs() -> BTreeSet<String> {
    let stripped = strip_comments(SESSION_CLI);
    let source = production_prefix(&stripped);
    let functions = functions_of(source);
    let Some(table) = functions.get("transition") else {
        return BTreeSet::new();
    };
    let mut verbs = BTreeSet::new();
    let mut cursor = 0usize;
    while let Some(offset) = table[cursor..].find(CELL_MARKER) {
        let start = cursor + offset;
        let after = start + CELL_MARKER.len();
        let end = table[after..]
            .find(CELL_MARKER)
            .map_or(table.len(), |index| after + index);
        let cell = &table[start..end];
        if cell.contains("transport: TransportState::Playing") {
            verbs.extend(shell_verbs(cell));
        }
        cursor = end;
    }
    verbs
}

/// The verbs the bridge's match arms route to a function.
fn routed_verbs(bridge: &Bridge, name: &str) -> BTreeSet<String> {
    let mut verbs = BTreeSet::new();
    let needle: Vec<char> = name.chars().collect();
    let mut index = 0usize;
    while index + needle.len() <= bridge.chars.len() {
        if bridge.chars[index..index + needle.len()] == needle[..] {
            let preceded_by_a_word = index > 0
                && is_ident_char(bridge.chars[index - 1])
                && bridge.chars[index - 1] != ':';
            let mut cursor = index + needle.len();
            while cursor < bridge.chars.len() && bridge.chars[cursor].is_whitespace() {
                cursor += 1;
            }
            if !preceded_by_a_word && cursor < bridge.chars.len() && bridge.chars[cursor] == '(' {
                let Some(close) = closing_paren(&bridge.chars, cursor) else {
                    index = cursor;
                    continue;
                };
                let argument: String = bridge.chars[cursor + 1..close].iter().collect();
                if !argument.trim().starts_with("ShellVerb::") {
                    verbs.extend(arm_verbs(&bridge.chars, cursor));
                }
                index = close + 1;
                continue;
            }
        }
        index += 1;
    }
    verbs
}

/// The verbs a bridge function can execute.
///
/// A function that executes a literal verb owns that verb. A
/// function that executes a CALLER-SUPPLIED verb owns the verbs it
/// names in its own body plus the verbs its callers route to it.
fn executed_verbs(bridge: &Bridge, name: &str) -> BTreeSet<String> {
    let Some(body) = bridge.functions.get(name) else {
        return BTreeSet::new();
    };
    let mut verbs = BTreeSet::new();
    let mut generic = false;
    let chars: Vec<char> = body.chars().collect();
    let needle: Vec<char> = ".execute(".chars().collect();
    let mut index = 0usize;
    while index + needle.len() <= chars.len() {
        if chars[index..index + needle.len()] == needle[..] {
            let open = index + needle.len() - 1;
            if let Some(close) = closing_paren(&chars, open) {
                let argument: String = chars[open + 1..close].iter().collect();
                let trimmed = argument.trim();
                if let Some(rest) = trimmed.strip_prefix("ShellVerb::") {
                    let member: String = rest.chars().take_while(|ch| is_ident_char(*ch)).collect();
                    if !member.is_empty() {
                        let _ignored = verbs.insert(member);
                    }
                } else {
                    generic = true;
                }
                index = close + 1;
                continue;
            }
        }
        index += 1;
    }
    if generic {
        verbs.extend(shell_verbs(body));
        verbs.extend(routed_verbs(bridge, name));
    }
    verbs
}

/// The bridge functions that can land the transport on `Playing`,
/// with the playing-class verbs each of them owns.
fn playing_executors(bridge: &Bridge) -> BTreeMap<String, BTreeSet<String>> {
    let playing = playing_verbs();
    let mut executors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for name in bridge.functions.keys() {
        let owned = executed_verbs(bridge, name);
        let landing: BTreeSet<String> = owned.intersection(&playing).cloned().collect();
        if landing.is_empty() {
            continue;
        }
        let _ignored = executors.insert(name.clone(), landing);
    }
    executors
}

/// Whether a closure synthesises the document's audio.
fn closure_synthesises(bridge: &Bridge, closure: &BTreeSet<String>) -> bool {
    bridge.bodies_of(closure).contains(SYNTHESIS_CALL)
}

/// Whether a closure starts the backend's output stream.
fn closure_starts_the_backend(bridge: &Bridge, closure: &BTreeSet<String>) -> bool {
    bridge.bodies_of(closure).contains(BACKEND_START_CALL)
}

/// The functions that own the document synthesis: the ones whose own
/// body calls the synthesis adapter.
fn synthesis_owners(bridge: &Bridge) -> BTreeSet<String> {
    let mut owners = BTreeSet::new();
    for (name, body) in &bridge.functions {
        if body.contains(SYNTHESIS_CALL) {
            let _ignored = owners.insert(name.clone());
        }
    }
    owners
}

/// The functions reachable from the synthesis owners, the synthesis
/// path's own machinery: the queueing, the trimming and the backend
/// start that the synthesis performs.
fn synthesis_closure(bridge: &Bridge) -> BTreeSet<String> {
    let mut reached = BTreeSet::new();
    let mut pending: Vec<String> = synthesis_owners(bridge).into_iter().collect();
    while let Some(current) = pending.pop() {
        let Some(body) = bridge.functions.get(&current) else {
            continue;
        };
        for callee in called_functions(body) {
            if !bridge.functions.contains_key(&callee) {
                continue;
            }
            if reached.insert(callee.clone()) {
                pending.push(callee);
            }
        }
    }
    reached
}

/// The bridge functions that call a function directly.
fn direct_callers(bridge: &Bridge, name: &str) -> BTreeSet<String> {
    let mut callers = BTreeSet::new();
    for (candidate, body) in &bridge.functions {
        if candidate == name {
            continue;
        }
        if called_functions(body).contains(name) {
            let _ignored = callers.insert(candidate.clone());
        }
    }
    callers
}

/// Whether a function owns the document synthesis: it calls the
/// synthesis adapter itself, so it BUILDS a reading and starts the
/// backend as part of that build.
fn owns_the_synthesis(bridge: &Bridge, name: &str) -> bool {
    let Some(body) = bridge.functions.get(name) else {
        return false;
    };
    let owners = synthesis_owners(bridge);
    called_functions(body)
        .iter()
        .any(|callee| owners.contains(callee))
}

/// The path a verb press actually takes: the handler's own reachable
/// code plus the code of the functions that dispatch to it.
///
/// The dispatch arm counts because the fix belongs to the PRESS, not
/// to one function: a resume that starts the backend from the arm
/// that routes the resume verb is the same fix as one that starts it
/// inside the handler.
fn press_path(bridge: &Bridge, handler: &str) -> BTreeSet<String> {
    let mut path = bridge.live_closure(handler);
    for caller in direct_callers(bridge, handler) {
        path.extend(bridge.live_closure(&caller));
    }
    path
}

/// Whether the bridge pauses the backend's output stream.
fn bridge_pauses_the_backend(bridge: &Bridge) -> bool {
    let everything: BTreeSet<String> = bridge.functions.keys().cloned().collect();
    bridge.bodies_of(&everything).contains(BACKEND_PAUSE_CALL)
}

/// The strategy the playing-class verbs are drawn from.
fn playing_verb_strategy() -> impl Strategy<Value = String> {
    let mut verbs: Vec<String> = playing_verbs().into_iter().collect();
    if verbs.is_empty() {
        verbs.push(String::from("<no playing verb was derived>"));
    }
    proptest::sample::select(verbs)
}

/// The surface the lock-screen payload renders.
fn surface() -> SurfaceSnapshot {
    SurfaceSnapshot {
        title: String::from("Rota da Manhã"),
        section_title: Some(String::from("Capítulo 2 — A chegada")),
        progress: Progress::Determinate {
            consumed_units: 12,
            total_units: 60,
            k: 2,
            n: 5,
        },
    }
}

// T140 RED: a resume after a pause MUST restart the backend's
// output stream (FR-001 / FR-005 / FR-007).
//
// The invariant: every bridge path that can land the transport on
// `Playing` MUST start the backend's stream on that same path — and
// a path that does NOT synthesise the document (a resume of a
// running reading) MUST NOT rely on the synthesis to start it,
// because the synthesis is what would restart the reading from the
// buffer head instead of re-anchoring it at the resume point.
//
// Today the resume-class handler executes the session transition and
// publishes it, but never calls the backend's `play()`: the
// transport reads `Playing` while the session is silent, and the
// lock screen shows a play glyph over a silent app.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn prop_a_resume_restarts_the_backend_stream(verb in playing_verb_strategy()) {
        let bridge = Bridge::read();
        let executors = playing_executors(&bridge);
        let handlers: Vec<&String> = executors
            .iter()
            .filter(|(_, landing)| landing.contains(&verb))
            .map(|(name, _)| name)
            .collect();
        prop_assert!(
            !handlers.is_empty(),
            "the bridge MUST have a path for the playing-class verb {:?} — the closed transport \
             transition table says it can land the transport on Playing",
            verb
        );
        for handler in handlers {
            if owns_the_synthesis(&bridge, handler) {
                // The synthesis path owns the document synthesis: it
                // BUILDS a reading and starts the backend as part of
                // that build. It is not the live-resume path, and a
                // resume must not borrow it — the synthesis rebuilds
                // the reading from the document instead of
                // re-anchoring the paused one at the resume point.
                continue;
            }
            let path = press_path(&bridge, handler);
            let synthesis = synthesis_closure(&bridge);
            let without_synthesis: BTreeSet<String> =
                path.difference(&synthesis).cloned().collect();
            prop_assert!(
                closure_starts_the_backend(&bridge, &without_synthesis),
                "a resume MUST restart the backend's output stream (FR-001 / FR-005 / FR-007): the \
                 bridge path {:?} lands the transport on Playing for the verb {:?} but never calls \
                 the backend's play(), so the reader's Play moves the transport to Playing and \
                 publishes it while the session stays SILENT — the lock screen shows the play \
                 glyph and the app emits no audio. The pause path calls the backend's pause() on \
                 the same bridge; the resume path MUST call the backend's play() exactly as it \
                 calls pause().",
                handler,
                verb
            );
        }
    }
}

// T140: a resume after a pause re-anchors at the stored position,
// not at the buffer head (FR-007).
//
// The session is the sole position authority, so the session's own
// resume transition is the runtime half of the re-anchoring
// contract: the resume keeps the position it was paused at, and
// only the deliberate start-over discards it.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_d_a_resume_re_anchors_at_the_stored_position(position in 0.0f64..3_600.0) {
        let mut session = PlaybackSession::new();
        let started = session.execute(ShellVerb::Play);
        prop_assert!(started.is_ok(), "a fresh session MUST start playing");

        session.on_audible_progress(position);
        prop_assert!(
            (session.snapshot().position_secs - position).abs() < 1e-9,
            "the audible clock MUST advance the position to {:?} s, but the session sits at {} s",
            position,
            session.snapshot().position_secs
        );

        let paused = session.execute(ShellVerb::Pause);
        prop_assert!(paused.is_ok(), "a playing session MUST accept the pause");
        prop_assert_eq!(session.snapshot().transport, TransportState::Paused);

        let resumed = session.execute(ShellVerb::ResumeAtStored);
        prop_assert!(resumed.is_ok(), "a paused session MUST accept the resume");
        prop_assert_eq!(
            session.snapshot().transport,
            TransportState::Playing,
            "the resume MUST land the transport on Playing"
        );
        prop_assert!(
            (session.snapshot().position_secs - position).abs() < 1e-9,
            "the resume MUST re-anchor at the stored position, not at the buffer head (FR-007): \
             the session resumed at {} s, but it paused at {:?} s",
            session.snapshot().position_secs,
            position
        );
    }
}

// T140: the published surface reports exactly the session's
// transport (FR-005).
//
// The lock-screen payload carries the transport the bridge
// publishes, and the bridge publishes the session's transport. So
// the surface can only ever report a playing state the session
// actually holds — and the property above then forces the bridge
// to have started the backend before it lands on that state.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_e_the_published_surface_reports_the_session_transport(
        state in proptest::sample::select(CLOSED_STATES.to_vec()),
        position in 0.0f64..86_400.0,
    ) {
        let payload = build_kotlin_mirror_json(
            state,
            position,
            &surface(),
            None,
            None,
            &Dismissal::Dismissable,
        );
        let parsed: Option<Value> = serde_json::from_str(&payload).ok();
        prop_assert!(parsed.is_some(), "the published lock-screen payload MUST be valid JSON");
        let parsed = parsed.unwrap_or(Value::Null);
        let expected = state.to_string();
        prop_assert_eq!(
            parsed.get("transport").and_then(Value::as_str),
            Some(expected.as_str()),
            "the published lock-screen payload MUST carry the session's transport verbatim \
             (FR-005): the session holds {:?} and the payload says {:?}",
            state,
            parsed.get("transport")
        );
    }
}

/// T140 control: the synthesis path starts the backend (the
/// non-vacuity guard for the property above).
///
/// If the source scan could not find the backend's start call
/// anywhere in the bridge, the failing property above would be
/// failing for the wrong reason. The synthesis path — the path that
/// builds a reading from the document — starts the backend, and this
/// proves the scan sees it.
#[test]
fn prop_b_the_synthesis_path_starts_the_backend() {
    let bridge = Bridge::read();
    let mut started = false;
    for name in bridge.functions.keys() {
        let closure = bridge.live_closure(name);
        if closure_synthesises(&bridge, &closure) && closure_starts_the_backend(&bridge, &closure) {
            started = true;
            break;
        }
    }
    assert!(
        started,
        "the bridge's synthesis path MUST start the backend's output stream — the scan that \
         checks the resume path must be able to SEE a backend start call, otherwise its verdict \
         on the resume path means nothing"
    );
}

/// T140 control: the bridge pauses the backend (the other half of
/// the asymmetry the finding names).
///
/// The pause path tells the backend to pause. The resume path must
/// tell it to play.
#[test]
fn prop_c_the_pause_path_stops_the_backend() {
    let bridge = Bridge::read();
    assert!(
        bridge_pauses_the_backend(&bridge),
        "the bridge MUST pause the backend's output stream — the pause path is the control that \
         makes the resume path's missing play() a defect and not a design"
    );
}
