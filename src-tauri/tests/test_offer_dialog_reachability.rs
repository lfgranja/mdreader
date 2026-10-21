//! T144 — Phase 17 RED-FIRST contract test: the REACHABILITY of the
//! in-app resume-offer dialog (feature `004-ressoa-dogfood-build`).
//!
//! The finding (tasks.md F3, HIGH): `src-tauri/src/status_view.html`
//! opens the offer dialog ONLY when the MIRRORED transport reads
//! `stopped` (the `applyOffer` guard), and the mirrored transport is
//! fed ONLY by the mirror publish channel. The parked cold-start open
//! publishes NOTHING — `switch_to_document`
//! (`src-tauri/src/lib.rs`) tears the outgoing session down without
//! publishing — so the mirrored transport keeps its PREVIOUS value.
//! After a mid-document pause the mirror says `paused` ⇒ re-picking
//! the same DELIBERATELY PAUSED document parks the cold start, but
//! the guard CLOSES the dialog ⇒ the reader is never offered the
//! continue-vs-start-over choice, "start over" is unreachable, and
//! the shell still runs the FR-032c 3-second window and
//! auto-continues on silence.
//!
//! The discriminator already exists on the Rust side — a parked
//! cold-start open (`pending_open_exists`, `src-tauri/src/lib.rs`) —
//! but it is NOT published in the `offer_route` reply, so the surface
//! has no way to tell a cold-start reopen from a pause inside a
//! running document (both read `paused`).
//!
//! The contract this file pins (FR-009, `contracts/surface.md:28` —
//! "Resume: deliberate-pause reopen → 3 s visual dialog (continue vs
//! start-over; silence = continue from stored)"):
//!
//! 1. **The shell publishes a discriminator.** The `offer_route`
//!    reply must tell the reader that a cold-start open is parked.
//!    The route alone cannot express it: the route is `in_app_dialog`
//!    for a deliberate-pause reopen AND for a re-pick of a document
//!    that was paused mid-reading.
//! 2. **The surface reads that discriminator.** The published fact
//!    must reach the surface that renders the offer.
//! 3. **The open condition is gated on it.** The dialog's open
//!    condition must consult a fact the shell published about the
//!    cold start — NOT the mirrored transport, which cannot express
//!    the difference (a cold-start reopen and a pause inside a
//!    running document are both `paused`).
//! 4. **The negatives hold.** The dialog stays CLOSED for the
//!    `lockscreen_resume` route, and for a pause inside a RUNNING
//!    document — the offer is a cold-start reopen affordance, not a
//!    mid-document pause affordance — and the "start over" choice
//!    lives INSIDE the offer dialog, so opening the dialog is what
//!    makes the choice reachable.
//!
//! The mechanism (the surface is plain HTML with no JavaScript test
//! harness): a static contract assertion over the surface source —
//! the same mechanism `test_offer_dialog.rs` (T116) uses for the
//! surface and `test_noscope.rs` (T047) uses for its HTML guard —
//! plus a static assertion over the production shell source for the
//! reply site, the way the sibling suite reads the dispatcher. Every
//! assertion is an INVARIANT, never a literal snapshot: the surface
//! is free to name its elements, its parameters, and its published
//! key as it likes, as long as the offer can distinguish a cold-start
//! reopen from a mid-document pause and stays closed everywhere else.
//!
//! T144 is RED against the current code: the reply publishes no
//! discriminator, and the open condition branches only on the
//! mirrored transport.

use std::collections::BTreeSet;

use player::cli::ShellVerb;
use src_tauri::commands::{BridgeAction, parse_action};

/// The in-app surface — the single screen the reader reads on
/// (FR-019).
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// The shell source — the publisher of the offer shape the surface
/// consumes.
const SHELL_SOURCE: &str = include_str!("../src/lib.rs");

/// The surface's own fields — the mirrored transport, the mirror
/// state, and the offer context. Reading one of these is NOT a
/// discriminator: the mirrored transport in particular CANNOT tell a
/// cold-start reopen from a pause inside a running document (both
/// read `paused`), which is exactly the defect this test pins.
const SURFACE_FIELDS: [&str; 17] = [
    // the offer context
    "route",
    "windowMs",
    "timer",
    "shown",
    // the mirror state
    "title",
    "sectionTitle",
    "transport",
    "positionSecs",
    "progress",
    "progressLabel",
    "rate",
    "reason",
    "reasonExpiry",
    "durationKnownAtStart",
    // the offer vocabulary the shell already publishes
    "offer_route",
    "window_ms",
    "offer_window_ms",
];

/// The JavaScript globals — a read through one of them is the
/// platform, never a fact the shell published.
const JS_GLOBALS: [&str; 24] = [
    "window",
    "document",
    "console",
    "Math",
    "JSON",
    "Object",
    "Array",
    "Number",
    "String",
    "Boolean",
    "setTimeout",
    "clearTimeout",
    "setInterval",
    "clearInterval",
    "parseInt",
    "parseFloat",
    "isNaN",
    "NaN",
    "Infinity",
    "Promise",
    "Date",
    "Error",
    "Symbol",
    "globalThis",
];

/// The in-app offer route — the ONLY route that opens a dialog
/// (`contracts/surface.md`).
const IN_APP_ROUTE: &str = "in_app_dialog";

/// The offer-choice verb that starts over from the first spoken
/// position (FR-009).
const START_OVER_VERB: &str = "start_over";

/// The Rust predicate that reports a parked cold-start open — the
/// discriminator the surface needs (FR-009 / FR-032c).
const PENDING_FACT: [&str; 2] = ["pending_open_exists", "pending_open"];

/// The vocabulary of a published key that carries the pending
/// cold-start fact. The key name is the SHELL's choice; the fact it
/// carries is not.
const PENDING_KEY_VOCABULARY: [&str; 4] = ["pending", "parked", "cold", "reopen"];

// ---------------------------------------------------------------------------
// The surface's JavaScript — a scanner, not a snapshot
// ---------------------------------------------------------------------------

/// One JavaScript function declaration of the surface: its name, the
/// names of its parameters, and its body.
#[derive(Debug, Clone)]
struct JsFunction {
    /// The declared name.
    name: String,
    /// The declared parameter names, in order.
    params: Vec<String>,
    /// The body, braces included.
    body: String,
}

/// Whether a byte can appear inside a JavaScript identifier.
const fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

/// Whether a byte can START a JavaScript identifier.
const fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

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
            let mut depth = 0_u32;
            while index < bytes.len() {
                match bytes[index] {
                    b'\\' => index += 2,
                    b'`' if depth == 0 => return index + 1,
                    b'$' if depth == 0 && bytes.get(index + 1) == Some(&b'{') => {
                        depth += 1;
                        index += 2;
                    }
                    b'{' => {
                        depth += 1;
                        index += 1;
                    }
                    b'}' => {
                        depth = depth.saturating_sub(1);
                        index += 1;
                    }
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

/// The index of the `)` that closes the `(` at `open`; `None` when
/// the parentheses do not balance.
fn js_matching_paren(source: &str, open: usize) -> Option<usize> {
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

/// The parameter names of a JavaScript parameter list. A destructured
/// parameter has no name of its own and contributes none.
fn js_param_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for part in text.split(',') {
        let name: String = part
            .trim()
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || *character == '_' || *character == '$'
            })
            .collect();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names
}

/// The function declaration that starts at the `function` keyword at
/// `keyword`, with the offset just past its body.
fn js_function_at(source: &str, keyword: usize) -> Option<(JsFunction, usize)> {
    let bytes = source.as_bytes();
    let mut cursor = keyword + "function".len();
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if !bytes.get(cursor).is_some_and(|byte| is_ident_start(*byte)) {
        return None;
    }
    let name_start = cursor;
    while cursor < bytes.len() && is_ident_byte(bytes[cursor]) {
        cursor += 1;
    }
    let name = source.get(name_start..cursor)?.to_string();
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'(') {
        return None;
    }
    let params_end = js_matching_paren(source, cursor)?;
    let params = js_param_names(source.get(cursor + 1..params_end)?);
    let mut brace = params_end + 1;
    while brace < bytes.len() && bytes[brace].is_ascii_whitespace() {
        brace += 1;
    }
    if bytes.get(brace) != Some(&b'{') {
        return None;
    }
    let body_end = js_matching_brace(source, brace)?;
    let body = source.get(brace..=body_end)?.to_string();
    Some((JsFunction { name, params, body }, body_end + 1))
}

/// Every `function` declaration of a JavaScript source.
fn js_functions(source: &str) -> Vec<JsFunction> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\'' | b'"' | b'`')
            || (byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')))
        {
            index = js_token_end(source, index);
            continue;
        }
        let parsed =
            if byte == b'f' && source.get(index..index + "function".len()) == Some("function") {
                js_function_at(source, index)
            } else {
                None
            };
        match parsed {
            Some((function, end)) => {
                found.push(function);
                index = end;
            }
            None => index += 1,
        }
    }
    found
}

/// The property accesses of a JavaScript fragment — the `Y` of every
/// `X.Y`, with the `X` it hangs off when there is one. A member read
/// `x.y` and a method call `x.y()` are the same shape to this
/// scanner.
fn js_property_accesses(source: &str) -> Vec<(Option<String>, String)> {
    let bytes = source.as_bytes();
    let mut accesses = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\'' | b'"' | b'`')
            || (byte == b'/' && matches!(bytes.get(index + 1), Some(b'/' | b'*')))
        {
            index = js_token_end(source, index);
            continue;
        }
        let access = if byte == b'.' {
            js_access_at(source, index)
        } else {
            None
        };
        match access {
            Some((object, property, end)) => {
                accesses.push((object, property));
                index = end;
            }
            None => index += 1,
        }
    }
    accesses
}

/// The property access that starts at the `.` at `dot`: the object it
/// hangs off, the property name, and the offset just past the name.
fn js_access_at(source: &str, dot: usize) -> Option<(Option<String>, String, usize)> {
    let bytes = source.as_bytes();
    let mut cursor = dot + 1;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if !bytes.get(cursor).is_some_and(|byte| is_ident_start(*byte)) {
        return None;
    }
    let name_start = cursor;
    while cursor < bytes.len() && is_ident_byte(bytes[cursor]) {
        cursor += 1;
    }
    let property = source.get(name_start..cursor)?.to_string();
    // The object: the identifier just before the dot.
    let mut back = dot;
    while back > 0 && bytes[back - 1].is_ascii_whitespace() {
        back -= 1;
    }
    let object = if back > 0 && is_ident_byte(bytes[back - 1]) {
        let start = back - 1;
        let mut floor = start;
        while floor > 0 && is_ident_byte(bytes[floor - 1]) {
            floor -= 1;
        }
        source.get(floor..back).map(str::to_string)
    } else {
        None
    };
    Some((object, property, cursor))
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

/// The keys of a JavaScript object literal — the identifiers that name
/// a member.
fn js_object_keys(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut keys = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        if !bytes.get(index).is_some_and(|byte| is_ident_start(*byte)) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && is_ident_byte(bytes[index]) {
            index += 1;
        }
        let name = text.get(start..index).unwrap_or_default().to_string();
        let mut cursor = index;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if bytes.get(cursor) == Some(&b':') && !keys.contains(&name) {
            keys.push(name);
        }
    }
    keys
}

// ---------------------------------------------------------------------------
// The surface — the offer decision
// ---------------------------------------------------------------------------

/// The surface's script — the only executable part of the in-app
/// screen.
fn surface_script() -> &'static str {
    const OPEN: &str = "<script type=\"module\">";
    const CLOSE: &str = "</script>";
    let start = STATUS_VIEW.find(OPEN).map_or(0, |at| at + OPEN.len());
    let end = STATUS_VIEW
        .get(start..)
        .and_then(|rest| rest.find(CLOSE))
        .map_or(STATUS_VIEW.len(), |offset| start + offset);
    STATUS_VIEW.get(start..end).unwrap_or(STATUS_VIEW)
}

/// The ids of the surface's DOM elements — the `els` map of the
/// in-app screen. A read of one of them is presentation, never a fact
/// the shell published.
fn surface_element_ids() -> BTreeSet<String> {
    let script = surface_script();
    let Some(declaration) = script.find("const els = {") else {
        return BTreeSet::new();
    };
    let open = declaration + "const els = ".len();
    let Some(end) = js_matching_brace(script, open) else {
        return BTreeSet::new();
    };
    let Some(text) = script.get(open..=end) else {
        return BTreeSet::new();
    };
    js_object_keys(text).into_iter().collect()
}

/// The names of the surface's own functions — a call to one of them
/// is the surface's own machinery, never a published fact.
fn surface_function_names(functions: &[JsFunction]) -> BTreeSet<String> {
    functions
        .iter()
        .map(|function| function.name.clone())
        .collect()
}

/// The surface's offer dialog opener — the function that opens the
/// dialog (`showModal`).
fn offer_opener(functions: &[JsFunction]) -> Option<&JsFunction> {
    functions
        .iter()
        .find(|function| function.body.contains("showModal"))
}

/// The surface's offer decision — the function(s) that decide to open
/// the dialog. A decision folded into the opener itself counts as the
/// decision.
fn offer_deciders<'a>(functions: &'a [JsFunction], opener: &'a JsFunction) -> Vec<&'a JsFunction> {
    let call = format!("{}(", opener.name);
    let deciders: Vec<&JsFunction> = functions
        .iter()
        .filter(|function| function.body.contains(&call))
        .collect();
    if deciders.is_empty() {
        return vec![opener];
    }
    deciders
}

/// The surface's offer closer — the function that closes the dialog.
fn offer_closer(functions: &[JsFunction]) -> Option<&JsFunction> {
    functions
        .iter()
        .find(|function| function.body.contains(".close("))
}

/// The region the offer decision is made in: the decider's own body
/// plus the bodies of the helpers it calls that paint nothing. A
/// helper that touches a DOM element is presentation, not decision.
fn offer_decision_region(functions: &[JsFunction], deciders: &[&JsFunction]) -> String {
    let mut region = String::new();
    let mut visited: Vec<&str> = Vec::new();
    let mut queue: Vec<&JsFunction> = deciders.to_vec();
    while let Some(function) = queue.pop() {
        if visited.iter().any(|name| *name == function.name) {
            continue;
        }
        visited.push(function.name.as_str());
        region.push_str(&function.body);
        region.push('\n');
        for callee in functions {
            if visited.iter().any(|name| *name == callee.name) {
                continue;
            }
            let call = format!("{}(", callee.name);
            if !function.body.contains(&call) || callee.body.contains("els.") {
                continue;
            }
            queue.push(callee);
        }
    }
    region
}

/// The parameter names the offer decision reads.
fn offer_decision_params(deciders: &[&JsFunction]) -> Vec<String> {
    let mut params: Vec<String> = Vec::new();
    for decider in deciders {
        for param in &decider.params {
            if !params.contains(param) {
                params.push(param.clone());
            }
        }
    }
    params
}

/// The signals the offer decision reads that are NOT the surface's own
/// state — the facts the shell published (or must publish) that the
/// decision may use to tell a cold-start reopen from a pause inside a
/// running document.
fn decision_signals(
    region: &str,
    params: &[String],
    excluded: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut signals: BTreeSet<String> = BTreeSet::new();
    for (object, property) in js_property_accesses(region) {
        if excluded.contains(&property) {
            continue;
        }
        if object
            .as_deref()
            .is_some_and(|name| JS_GLOBALS.contains(&name))
        {
            continue;
        }
        let _ignored = signals.insert(property);
    }
    for param in params {
        if !excluded.contains(param) {
            let _ignored = signals.insert(param.clone());
        }
    }
    signals
}

/// Whether the region TESTS a signal — a negation or a comparison
/// applied to it. A signal that is merely read is not a gate.
fn is_tested(region: &str, signal: &str) -> bool {
    let bytes = region.as_bytes();
    let mut search = 0_usize;
    while let Some(offset) = region.get(search..).and_then(|rest| rest.find(signal)) {
        let at = search + offset;
        search = at + signal.len();
        if bytes
            .get(at.saturating_sub(1))
            .is_some_and(|byte| is_ident_byte(*byte))
        {
            continue;
        }
        if bytes
            .get(at + signal.len())
            .is_some_and(|byte| is_ident_byte(*byte))
        {
            continue;
        }
        let window_start = at.saturating_sub(24);
        let before = region.get(window_start..at).unwrap_or("");
        if let Some(bang) = before.rfind('!') {
            let between = &before[bang + 1..];
            let only_names = between.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || character == '_'
                    || character == '$'
                    || character == '.'
                    || character.is_whitespace()
            });
            if only_names {
                return true;
            }
        }
        let after = region.get(at + signal.len()..).unwrap_or("");
        let trimmed = after.trim_start();
        if trimmed.starts_with("==")
            || trimmed.starts_with("!=")
            || trimmed.starts_with("&&")
            || trimmed.starts_with("||")
            || trimmed.starts_with('?')
        {
            return true;
        }
    }
    false
}

/// The id of the control that dispatches the given offer-choice verb.
fn control_bound_to_verb(script: &str, verb: &str) -> Option<String> {
    let needle = format!("sendAction('{verb}')");
    let at = script.find(&needle)?;
    let head = script.get(..at)?;
    let bound = head.rfind("els.")?;
    let rest = head.get(bound + "els.".len()..)?;
    let end = rest.find(".addEventListener")?;
    rest.get(..end).map(str::to_string)
}

/// The offer dialog element of the surface — the `<dialog>` block.
fn offer_dialog_markup() -> Option<&'static str> {
    let start = STATUS_VIEW.find("<dialog")?;
    let end = STATUS_VIEW.get(start..)?.find("</dialog>")?;
    STATUS_VIEW.get(start..start + end + "</dialog>".len())
}

// ---------------------------------------------------------------------------
// The shell — the reply site
// ---------------------------------------------------------------------------

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
                    .is_none_or(|byte| !is_ident_byte(*byte))
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

/// The production part of the shell source — everything before the
/// first test module. A test body is the shell's OWN evidence of the
/// contract, not the production behaviour, so the reply site never
/// reads one.
fn production_source(source: &str) -> &str {
    let first_module = ["#[cfg(test)]\nmod ", "#[cfg(test)]\npub(crate) mod "]
        .iter()
        .filter_map(|needle| source.find(needle))
        .min();
    first_module.map_or(source, |offset| source.get(..offset).unwrap_or(source))
}

/// The reply site: the body of the `offer_route` command, plus the
/// bodies of the production functions that assemble the offer's wire
/// shape.
fn offer_reply_site(source: &str) -> String {
    let production = production_source(source);
    let mut site = rust_fn_body(production, "offer_route").unwrap_or_default();
    for name in ["offer_reply", "offer_payload", "offer_shape"] {
        if let Some(body) = rust_fn_body(production, name) {
            site.push('\n');
            site.push_str(&body);
        }
    }
    site
}

/// The published keys of a reply site: every string literal, with the
/// source text that FOLLOWS it — the value the literal names.
fn published_keys(region: &str) -> Vec<(String, String)> {
    let bytes = region.as_bytes();
    let mut published = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            let end = rust_token_end(region, index);
            let key = region
                .get(index + 1..end.saturating_sub(1))
                .unwrap_or_default()
                .to_string();
            let value_end = rust_value_end(region, end);
            let value = region.get(end..value_end).unwrap_or_default().to_string();
            published.push((key, value));
            index = end;
            continue;
        }
        if byte == b'\'' && rust_char_literal_len(bytes, index).is_some() {
            index = rust_token_end(region, index);
            continue;
        }
        index += 1;
    }
    published
}

/// The end of the value expression that starts at `start`: the next
/// `,`, `}` or `;` at the top level of it.
fn rust_value_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut index = start;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || (byte == b'\'' && rust_char_literal_len(bytes, index).is_some()) {
            index = rust_token_end(source, index);
            continue;
        }
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth == 0 {
                    return index;
                }
                depth -= 1;
            }
            b',' | b';' if depth == 0 => return index,
            _ => {}
        }
        index += 1;
    }
    bytes.len()
}

// ---------------------------------------------------------------------------
// The contract
// ---------------------------------------------------------------------------

/// The key the shell publishes to carry the pending-cold-start fact,
/// with the value it publishes it under.
///
/// The key NAME is the shell's choice — the fact it carries is not.
fn published_pending_key(site: &str) -> Option<(String, String)> {
    published_keys(site).into_iter().find(|(key, value)| {
        !matches!(
            key.as_str(),
            "offer_route" | "window_ms" | "offer_window_ms" | "none"
        ) && PENDING_KEY_VOCABULARY
            .iter()
            .any(|word| key.to_lowercase().contains(word) || value.to_lowercase().contains(word))
    })
}

/// Whether the reply site consults the production pending-cold-start
/// fact — the discriminator the shell already owns.
fn consults_the_pending_fact(site: &str) -> bool {
    PENDING_FACT.iter().any(|fact| site.contains(fact))
}

/// T144 RED-FIRST: the `offer_route` reply must carry the pending
/// cold-start fact.
///
/// Today the reply publishes the route and the window and nothing
/// else: the surface cannot tell a cold-start reopen from a pause
/// inside a running document, because the only difference between
/// them lives in the shell's parked cold-start open and the reply
/// never mentions it.
#[test]
fn the_offer_route_reply_publishes_the_pending_cold_start_fact() {
    let site = offer_reply_site(SHELL_SOURCE);
    assert!(
        !site.is_empty(),
        "T144: the shell must expose the `offer_route` command whose reply the surface reads (FR-009)"
    );
    let published = published_pending_key(&site);
    assert!(
        consults_the_pending_fact(&site),
        "T144: the `offer_route` reply must be built from the pending cold-start fact (`{}`) — the shell already owns the discriminator (a parked cold-start open), and the route alone cannot tell a deliberate-pause reopen from a pause inside a running document (both read `paused`), so the reader is never offered the continue-vs-start-over choice and `start over` is unreachable (FR-009, `contracts/surface.md:28`, Principle III); the reply site publishes: {:?}",
        PENDING_FACT[0],
        published_key_names(&site),
    );
    assert!(
        published.is_some(),
        "T144: the `offer_route` reply must PUBLISH the pending cold-start fact under a key the surface can read — a fact the shell keeps to itself cannot reach the surface that renders the offer (FR-009, `contracts/surface.md:28`); the reply site publishes: {:?}",
        published_key_names(&site),
    );
}

/// The keys the reply site publishes — the failure message's context.
fn published_key_names(site: &str) -> Vec<String> {
    let mut names: Vec<String> = published_keys(site)
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    names.sort();
    names.dedup();
    names
}

/// T144 RED-FIRST: the fact the shell publishes must REACH the
/// surface that renders the offer.
///
/// A discriminator the surface never reads cannot discriminate
/// anything: the dialog would still open (or stay closed) on the
/// mirrored transport alone.
#[test]
fn the_offer_dialog_reads_the_published_cold_start_fact() {
    let site = offer_reply_site(SHELL_SOURCE);
    let published = published_pending_key(&site);
    assert!(
        published.is_some(),
        "T144: the `offer_route` reply must publish the pending cold-start fact before the surface can read it (FR-009) — the reply site publishes: {:?}",
        published_key_names(&site)
    );
    let Some((key, _)) = published else {
        return;
    };
    let script = surface_script();
    let read_as_property = js_property_accesses(script)
        .into_iter()
        .any(|(_, property)| property == key);
    let read_as_literal = js_string_literals(script).contains(&key);
    assert!(
        read_as_property || read_as_literal,
        "T144: the surface must read the published cold-start fact `{key}` — the shell publishes it, so the surface that renders the offer must consume it (FR-009, `contracts/surface.md:28`)"
    );
}

/// The surface's offer dialog opener — the function that opens the
/// dialog — asserted to exist.
fn asserted_offer_opener(functions: &[JsFunction]) -> Option<&JsFunction> {
    let opener = offer_opener(functions);
    assert!(
        opener.is_some(),
        "T144: the in-app surface must carry a dialog the offer opens (`showModal`) — FR-009 requires a VISUAL dialog on the single in-app screen"
    );
    opener
}

/// T144 RED-FIRST: the dialog's open condition must be able to tell a
/// cold-start reopen from a pause inside a running document.
///
/// Today the open condition branches only on the MIRRORED transport
/// (`stopped`). The mirrored transport cannot express the difference:
/// after a mid-document pause it reads `paused`, and the parked
/// cold-start open publishes nothing, so it still reads `paused` when
/// the reader re-picks the deliberately paused document. The
/// consequence is the defect: the dialog is closed exactly when the
/// offer must be presented, and "start over" is unreachable.
#[test]
fn the_offer_open_condition_is_not_the_mirrored_transport_alone() {
    let script = surface_script();
    let functions = js_functions(script);
    let opener = asserted_offer_opener(&functions);
    let Some(opener) = opener else {
        return;
    };
    let deciders = offer_deciders(&functions, opener);
    let region = offer_decision_region(&functions, &deciders);
    let params = offer_decision_params(&deciders);
    let mut excluded: BTreeSet<String> = SURFACE_FIELDS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    excluded.extend(surface_element_ids());
    excluded.extend(surface_function_names(&functions));
    let signals = decision_signals(&region, &params, &excluded);
    let mut reads: Vec<String> = js_property_accesses(&region)
        .into_iter()
        .map(|(_, property)| property)
        .collect();
    reads.extend(params);
    reads.sort();
    reads.dedup();
    assert!(
        !signals.is_empty(),
        "T144: the offer-open condition must consult a fact the shell published about the cold start — today it branches only on the MIRRORED transport, which cannot tell a deliberate-pause reopen from a pause inside a running document (both read `paused`), so the dialog is closed exactly when the offer must be presented (FR-009, `contracts/surface.md:28`, Principle III); the offer decision reads only: {reads:?}"
    );
    assert!(
        signals.iter().any(|signal| is_tested(&region, signal)),
        "T144: the cold-start fact the offer-open condition reads must GATE the dialog (a negation or a comparison applied to it) — a fact that is merely read leaves the offer decided by the mirrored transport (FR-009); the decision region reads: {signals:?}"
    );
}

/// T144: the negative invariants hold — the dialog stays CLOSED for
/// the `lockscreen_resume` route, and for a pause inside a RUNNING
/// document.
///
/// The offer is a cold-start reopen affordance: the lock-screen route
/// opens NO in-app dialog (`contracts/surface.md`), and a pause
/// inside a running document is not a reopen at all.
#[test]
fn the_offer_dialog_stays_closed_for_the_lock_screen_route() {
    let script = surface_script();
    let functions = js_functions(script);
    let opener = asserted_offer_opener(&functions);
    let Some(opener) = opener else {
        return;
    };
    let closer = offer_closer(&functions);
    assert!(
        closer.is_some(),
        "T144: the in-app surface must carry a dialog the offer closes — the offer episode ends when the dialog closes (FR-009)"
    );
    let Some(closer) = closer else {
        return;
    };
    let deciders = offer_deciders(&functions, opener);
    let region = offer_decision_region(&functions, &deciders);
    assert!(
        js_string_literals(&region)
            .iter()
            .any(|literal| literal == IN_APP_ROUTE),
        "T144: the offer-open condition must name the in-app route `{IN_APP_ROUTE}` — it is the ONLY route that opens a dialog; the `lockscreen_resume` route opens NO in-app dialog (`contracts/surface.md`)"
    );
    assert!(
        region.contains(&format!("{}(", closer.name)),
        "T144: the offer-open condition must close the dialog for every route that is not the in-app one — the `lockscreen_resume` route opens NO in-app dialog (`contracts/surface.md`)"
    );
}

/// T144: the "start over" choice lives INSIDE the offer dialog, and
/// the shell accepts the verb — so opening the dialog is what makes
/// the choice reachable (FR-009).
///
/// The reachability of the CHOICE is the defect F3 broke: the choice
/// existed and dispatched, but the dialog that carries it never
/// opened for a deliberate-pause reopen, so the reader was left with
/// the automatic 3-second silence rule alone.
#[test]
fn the_start_over_choice_lives_inside_the_offer_dialog() {
    let script = surface_script();
    let control = control_bound_to_verb(script, START_OVER_VERB);
    assert!(
        control.is_some(),
        "T144: the in-app surface must dispatch the `{START_OVER_VERB}` choice — it is one of the two closed offer choices (FR-009)"
    );
    let Some(control) = control else {
        return;
    };
    let dialog = offer_dialog_markup();
    assert!(
        dialog.is_some(),
        "T144: the in-app surface must carry a semantic dialog element for the offer (FR-009)"
    );
    let Some(dialog) = dialog else {
        return;
    };
    assert!(
        dialog.contains(&format!("id=\"{control}\"")),
        "T144: the `{START_OVER_VERB}` choice must live INSIDE the offer dialog — the dialog is the affordance that makes the choice reachable (FR-009); the control `{control}` is outside it"
    );
    assert!(
        matches!(
            parse_action(START_OVER_VERB),
            Ok(BridgeAction::Verb(ShellVerb::StartOver))
        ),
        "T144: the `{START_OVER_VERB}` choice must be a verb the shell dispatches — an unreachable choice is not a choice (FR-009, `contracts/shell-ipc.md`)"
    );
}
