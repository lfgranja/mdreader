//! T118 — RED-FIRST contract test: the lock-screen explicit resume
//! control (Feature 004-ressoa-dogfood-build / task T118).
//!
//! Requirement (`specs/004-ressoa-dogfood-build/spec.md` FR-009,
//! verbatim): "after an unexpected end (system kill, power loss,
//! permanent audio loss, or engine failure) it MUST wait paused with
//! an explicit resume control on the lock-screen surface instead" —
//! and `contracts/surface.md:22`, the normative restatement: "After
//! kill/power-loss: wait paused + explicit resume control on this
//! surface (no visual dialog)".
//!
//! The finding this test pins (`tasks.md` F3):
//!
//! * `MediaMirror.kt:1205-1232` — the surface's action set carries
//!   only the speed and voice controls; no control of the surface
//!   reaches Rust as the stored-point resume verb.
//! * `MediaMirror.kt:723-735` — the resume broadcast is classified
//!   as the plain-play verb, so the control that exists never asks
//!   for the stored point.
//! * `MediaMirror.kt:696-712` — the session callback carries no
//!   resume route at all.
//! * `src-tauri/src/lib.rs:2714-2718` — the parked-open resolution
//!   admits the stored-resume and start-over verbs only, never the
//!   plain-play verb, so a media-session Play pressed on a parked
//!   open never reaches the stored point.
//!
//! MECHANISM. The Kotlin half of the contract is not host-runnable
//! (the mirror needs the Android platform), so this file reads the
//! mirror's SOURCE and asserts the invariant over it — the same
//! discipline `scripts/check-shell-purity.sh` and
//! `src-tauri/tests/test_noscope.rs` already use. The assertions are
//! INVARIANTS, never snapshots: they ask "does the lock-screen
//! control vocabulary reach Rust as the stored-point resume?" and
//! "does the bridge admit the plain-play verb to the parked-open
//! resolution?", never "does the file still look exactly like
//! this?". A declaration proves nothing: a verb constant that is
//! only DECLARED is not a control, so the assertions read the
//! forwarding routes and the resolution sites, not the constant
//! table.
//!
//! The two behavioural guards at the end pin the halves that must
//! NOT move while the control is added: the published lock-screen
//! payload keeps its no-Stop-control / no-prohibited-field shape
//! (FR-002 / FR-011), and an unexpected end keeps the reading
//! WAITING — no offer opens and nothing begins on its own
//! (FR-031 / FR-032d).
//!
//! SHELL-VOCABULARY NOTE. This file is deliberately outside the
//! shell-purity gate's vocabulary allowlist, so it names no
//! shell-vocabulary word anywhere — not even in prose, not even in
//! a comment. The wording below is the reason: the lock-screen
//! surface's own vocabulary is spoken of as "the state", "the
//! stored point", "the speed control" and "the voice control"
//! instead. Keep it that way: the gate greps the WHOLE file, and a
//! word introduced here fails the gate for every other task.

use std::error::Error;
use std::path::{Path, PathBuf};

use engine_core::closed::{SessionEnd, TransportState};
use player::surface::{Dismissal, Progress, SurfaceSnapshot};
use serde_json::Value;
use session::offer::{ExpiryOutcome, OfferMachine, OpenOffer};
use session::store::StoredPosition;
use src_tauri::publish::build_kotlin_mirror_json;

/// The Kotlin lock-screen mirror — the file that owns the surface
/// and every control route the platform can deliver.
const KOTLIN_MIRROR: &str = "android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt";

/// The Rust bridge — the file that owns the parked-open resolution
/// and the verb classification.
const RUST_BRIDGE: &str = "src-tauri/src/lib.rs";

/// The Kotlin constant that names the stored-point resume verb.
const STORED_RESUME_CONST: &str = "VERB_RESUME_AT_STORED";

/// The wire spelling of the stored-point resume verb
/// (`contracts/shell-ipc.md`): the string the bridge classifies
/// back into the stored-point resume.
const STORED_RESUME_WIRE: &str = "resume_at_stored";

/// The plain-play verb — the control the media session delivers for
/// its own Play affordance.
const PLAIN_PLAY_CONST: &str = "VERB_PLAY";

/// The verbs the bridge may name in its closed verb set, as the
/// Rust-side enum members.
const RUST_VERB_ENUM: &str = "ShellVerb";

/// The closed state set the lock-screen surface renders.
const CLOSED_STATES: [TransportState; 5] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Stopped,
    TransportState::Completed,
    TransportState::Interrupted,
];

/// The display projection the lock-screen surface renders: the
/// title, the section title and the progress indication — and
/// nothing else (`contracts/surface.md`).
const DISPLAY_PROJECTION: [&str; 3] = ["title", "sectionTitle", "progress"];

/// The key fragments the lock-screen payload may never carry:
/// document prose, elapsed or remaining time, a numeric speed
/// value, the voice identity, the engine identity, and the Stop
/// control (FR-002 / FR-011 / FR-012 / FR-019).
const PROHIBITED_KEY_FRAGMENTS: [&str; 10] = [
    "prose",
    "body",
    "text",
    "elapsed",
    "remaining",
    "duration",
    "speed",
    "voice",
    "engine",
    "stop",
];

/// The repository root: the WORKSPACE manifest's parent — the
/// ancestor whose manifest declares `[workspace]`, not merely any
/// manifest (this crate's own manifest would otherwise match first
/// and resolve the root to `src-tauri/`).
fn repo_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && std::fs::read_to_string(candidate.join("Cargo.toml"))
                    .is_ok_and(|text| text.contains("[workspace]"))
        })
        .map(Path::to_path_buf)
        .ok_or("the workspace root carries the manifest declaring [workspace]")?)
}

/// One repository source file, read whole.
fn read_source(relative: &str) -> Result<String, Box<dyn Error>> {
    Ok(std::fs::read_to_string(repo_root()?.join(relative))?)
}

/// The source with every comment removed, the line structure kept.
///
/// The contract assertions read CODE, never prose: a doc comment
/// that names a verb proves nothing about the route that carries
/// it, and a comment that names a state proves nothing about the
/// payload that publishes it. Line comments (`//`) and block
/// comments are dropped; a comment marker inside a string literal
/// is not a comment and is kept, so the scanner tracks the string
/// state.
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

/// The index of the parenthesis closing the one at `open`, or
/// `None` when the source is unbalanced. A parenthesis inside a
/// string literal never closes a call.
const fn closing_paren(source: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
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
        } else if ch == '"' {
            in_string = true;
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

/// Every argument region of a `name(` call site: the text between
/// the opening parenthesis and its matching close, in source order.
fn call_regions(source: &[char], name: &str) -> Vec<String> {
    let needle: Vec<char> = name.chars().collect();
    let mut regions = Vec::new();
    let mut index = 0usize;
    while index + needle.len() <= source.len() {
        if source[index..index + needle.len()] == needle[..] {
            let mut open = index + needle.len();
            while open < source.len() && source[open].is_whitespace() {
                open += 1;
            }
            if open < source.len()
                && source[open] == '('
                && let Some(close) = closing_paren(source, open)
            {
                regions.push(source[open + 1..close].iter().collect::<String>());
                index = close + 1;
                continue;
            }
        }
        index += 1;
    }
    regions
}

/// The identifiers and the string literals of a text, in order.
///
/// A string literal is yielded with its quotes stripped, so a wire
/// verb and the constant that names it are comparable.
fn words_and_literals(text: &str) -> Vec<String> {
    let source: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index];
        if ch == '"' {
            let mut literal = String::new();
            index += 1;
            while index < source.len() && source[index] != '"' {
                if source[index] == '\\' && index + 1 < source.len() {
                    index += 1;
                }
                literal.push(source[index]);
                index += 1;
            }
            index += 1;
            tokens.push(literal);
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' {
            let mut word = String::new();
            while index < source.len() && (source[index].is_alphanumeric() || source[index] == '_')
            {
                word.push(source[index]);
                index += 1;
            }
            tokens.push(word);
            continue;
        }
        index += 1;
    }
    tokens
}

/// The verbs the lock-screen mirror forwards to Rust: every verb
/// constant and every verb wire literal named inside a forwarding
/// call's argument region.
///
/// The forwarding call is the mirror's ONLY way to reach Rust
/// (SC-035), so the region of a forwarding call is the control
/// route itself. A verb constant that is only DECLARED — never
/// named inside a forwarding route — is not a control and is not
/// counted: that distinction is the whole finding.
fn forwarded_verbs(kotlin: &str) -> Vec<String> {
    let source: Vec<char> = kotlin.chars().collect();
    let mut verbs: Vec<String> = Vec::new();
    for region in call_regions(&source, "forwardVerb") {
        for token in words_and_literals(&region) {
            let is_verb_constant = token.starts_with("VERB_");
            let is_verb_wire = token.contains('_') && !token.starts_with("ACTION_");
            if (is_verb_constant || is_verb_wire) && !verbs.contains(&token) {
                verbs.push(token);
            }
        }
    }
    verbs.sort();
    verbs
}

/// The production part of a Rust source: everything before its
/// first inline test module.
///
/// The contract reads PRODUCTION code — a test module may name any
/// verb it likes, and a test module's mention of the plain-play
/// verb is not a route the lock screen can take.
fn production_prefix(text: &str) -> &str {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("mod ") && trimmed.contains("_tests") && trimmed.ends_with('{') {
            return &text[..offset];
        }
        offset += line.len();
    }
    text
}

/// The verbs the bridge admits to the parked-open resolution.
///
/// The parked-open resolution is the code that consumes a parked
/// cold start (FR-009 / FR-032d). The assertion reads the verbs
/// named in the neighbourhood of that resolution — the guard that
/// decides whether a verb resolves the parked open — so the
/// invariant is "the plain-play verb reaches the parked resolution",
/// not a snapshot of the bridge's shape.
fn parked_admitted_verbs(lib_rs: &str) -> Vec<String> {
    /// The names that mark the parked-open resolution.
    const PARKED_MARKERS: [&str; 6] = [
        "pending_open_exists",
        "resolve_parked_choice",
        "resolve_parked_open",
        "take_parked_open",
        "repark_cold_start",
        "park_cold_start",
    ];

    /// How many lines around a resolution site count as its
    /// neighbourhood: the guard and the arm that consults it sit
    /// within a few lines of each other.
    const NEIGHBOURHOOD_LINES: usize = 20;

    let production = production_prefix(lib_rs);
    let lines: Vec<&str> = production.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut verbs: Vec<String> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !PARKED_MARKERS.iter().any(|marker| line.contains(marker)) {
            continue;
        }
        let first = index.saturating_sub(NEIGHBOURHOOD_LINES);
        let last = (index + NEIGHBOURHOOD_LINES).min(lines.len().saturating_sub(1));
        for neighbour in &lines[first..=last] {
            for member in path_members(neighbour, RUST_VERB_ENUM) {
                if !verbs.contains(&member) {
                    verbs.push(member);
                }
            }
        }
    }
    verbs.sort();
    verbs
}

/// The members of a `Path::Member` pattern named in a text.
fn path_members(text: &str, path: &str) -> Vec<String> {
    let mut members = Vec::new();
    let source: Vec<char> = text.chars().collect();
    let needle: Vec<char> = path.chars().collect();
    let mut index = 0usize;
    while index + needle.len() <= source.len() {
        if source[index..index + needle.len()] == needle[..] {
            let mut cursor = index + needle.len();
            if cursor + 1 < source.len() && source[cursor] == ':' && source[cursor + 1] == ':' {
                cursor += 2;
                let mut member = String::new();
                while cursor < source.len()
                    && (source[cursor].is_alphanumeric() || source[cursor] == '_')
                {
                    member.push(source[cursor]);
                    cursor += 1;
                }
                if !member.is_empty() {
                    members.push(member);
                }
                index = cursor;
                continue;
            }
        }
        index += 1;
    }
    members
}

/// A surface the lock screen renders: the title, the section title
/// and the determinate progress indication.
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

/// Whether a rendered payload carries a numeric speed value (the
/// ladder rung painted on the surface).
fn carries_a_numeric_speed_value(payload: &str) -> bool {
    let source: Vec<char> = payload.chars().collect();
    source.iter().enumerate().any(|(index, ch)| {
        ch.is_ascii_digit()
            && source
                .get(index + 1)
                .is_some_and(|next| *next == 'x' || *next == 'X')
    })
}

/// Every key of a published payload, at any depth.
///
/// The prohibition is on the FIELDS the payload carries, so the
/// walk descends: a nested object may not smuggle a prohibited
/// field past a top-level-only check.
fn all_keys(value: &Value) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(object) = value.as_object() {
        for (key, child) in object {
            keys.push(key.clone());
            keys.extend(all_keys(child));
        }
    }
    keys
}

/// T118 RED: the lock-screen surface MUST expose an explicit resume
/// control wired to the stored-point resume (FR-009;
/// `contracts/surface.md:22`).
///
/// The control must REACH Rust as the stored-point resume: the
/// mirror's forwarding routes are the only channel the platform
/// has, so a verb constant that is merely declared proves nothing.
/// Today the forwarded vocabulary carries the plain-play verb and
/// nothing that resumes at the stored point, so the assertion
/// fails.
#[test]
fn the_lock_screen_forwards_the_stored_point_resume_verb() -> Result<(), Box<dyn Error>> {
    let kotlin = strip_comments(&read_source(KOTLIN_MIRROR)?);
    let forwarded = forwarded_verbs(&kotlin);
    let carries = forwarded
        .iter()
        .any(|verb| verb.as_str() == STORED_RESUME_CONST || verb.as_str() == STORED_RESUME_WIRE);
    assert!(
        carries,
        "the lock-screen surface MUST expose an explicit resume control wired to the stored-point resume \
         (FR-009; contracts/surface.md:22) — the verbs the mirror forwards to Rust are {forwarded:?}, and \
         none of them is {STORED_RESUME_CONST} / {STORED_RESUME_WIRE:?}. A declared-but-unforwarded \
         constant is not a control: the resume affordance must REACH Rust as the stored-point resume. \
         {PLAIN_PLAY_CONST} is a different verb — it starts a fresh reading, it does not continue at the \
         stored point."
    );
    Ok(())
}

/// T118 RED: the bridge MUST admit the plain-play verb to the
/// parked-open resolution (FR-009 / FR-031 / FR-032d).
///
/// A parked cold start is a document waiting for the reader's
/// explicit resume: its only stored point is the row the offer
/// opened from. A media-session Play pressed on that surface must
/// therefore resolve the parked open AT THE STORED POINT. Today the
/// resolution admits the stored-resume and start-over verbs only,
/// so the plain-play verb starts a fresh reading from the first
/// spoken point — and the assertion fails.
#[test]
fn the_bridge_admits_the_plain_play_verb_to_the_parked_open_resolution()
-> Result<(), Box<dyn Error>> {
    let lib_rs = strip_comments(&read_source(RUST_BRIDGE)?);
    let admitted = parked_admitted_verbs(&lib_rs);
    assert!(
        admitted.iter().any(|verb| verb == "Play"),
        "the bridge MUST admit the plain-play verb to the parked-open resolution (FR-009 / FR-031 / \
         FR-032d) — the verbs the parked resolution admits are {admitted:?}, and the plain-play verb is \
         not among them, so a media-session Play pressed on a parked cold start never resolves the parked \
         open at the stored point. The admitted set grows either by naming the plain-play verb in the \
         resolution's own guard, or by consulting the parked state inside the plain-play arm."
    );
    Ok(())
}

/// The lock screen keeps its shape while the resume control is
/// added (FR-002 / FR-011 / FR-012 / FR-019).
///
/// The published payload carries the display projection and the
/// state, and NOTHING else: no document prose, no elapsed or
/// remaining time, no numeric speed value, no voice identity, and
/// no Stop CONTROL (the `stopped` STATE is a state, the Stop
/// CONTROL is in-app only). The mirror declares no stop
/// notification action and no stop session callback.
#[test]
fn the_lock_screen_keeps_no_stop_control_and_no_prohibited_field() -> Result<(), Box<dyn Error>> {
    for state in CLOSED_STATES {
        let payload =
            build_kotlin_mirror_json(state, 12.5, &surface(), None, None, &Dismissal::Dismissable);
        let parsed: Value = serde_json::from_str(&payload)?;
        let object = parsed
            .as_object()
            .ok_or("the published lock-screen payload is a JSON object")?;
        for key in all_keys(&parsed) {
            for fragment in PROHIBITED_KEY_FRAGMENTS {
                assert!(
                    !key.contains(fragment),
                    "the published lock-screen payload for {state:?} carries the key {key:?}, which holds \
                     the prohibited fragment {fragment:?} (FR-011 / FR-002 / FR-012 / FR-019)"
                );
            }
        }
        for key in DISPLAY_PROJECTION {
            assert!(
                object.contains_key(key),
                "the published lock-screen payload for {state:?} is missing the display projection key \
                 {key:?} (contracts/surface.md)"
            );
        }
        assert!(
            !carries_a_numeric_speed_value(&payload),
            "the published lock-screen payload for {state:?} paints a numeric speed value (FR-012): {payload}"
        );
    }
    let kotlin = strip_comments(&read_source(KOTLIN_MIRROR)?);
    assert!(
        !kotlin.contains("ACTION_STOP"),
        "the lock-screen mirror declares no Stop notification action — the Stop CONTROL exists in-app only \
         (FR-002 / FR-019)"
    );
    assert!(
        !kotlin.contains("onStop"),
        "the lock-screen mirror's session callback handles no Stop control — the Stop CONTROL exists \
         in-app only (FR-002 / FR-019)"
    );
    Ok(())
}

/// An unexpected end leaves the reading WAITING (FR-031 / FR-032d).
///
/// The offer machine opens no offer for an unexpected end, and its
/// expiry begins nothing: the reader's explicit resume is the only
/// start. The deliberate-pause form is the contrast — its 3-second
/// silence rule continues from the stored point — and the start-over
/// choice is the only route to the first spoken point.
#[test]
fn an_unexpected_end_leaves_the_reading_waiting_for_an_explicit_resume() {
    let mut machine = OfferMachine::new();
    let unexpected = StoredPosition {
        offset_secs: 12.5,
        first_yield_ms: 250,
        end_kind: SessionEnd::Interrupted,
    };
    let open = machine.resolve_open(Some(unexpected));
    assert!(
        matches!(open, OpenOffer::ExplicitResume),
        "an unexpected end MUST leave the explicit-resume requirement standing, not open an offer \
         (FR-031 / FR-032d): the open decision was {open:?}"
    );
    assert!(
        machine.continue_offer().is_none(),
        "an unexpected end opens no visible offer — the lock screen carries the resume control instead \
         (FR-032d / contracts/surface.md:22)"
    );
    let expiry = machine.expire();
    assert!(
        matches!(expiry, ExpiryOutcome::ExplicitResume),
        "the window elapsing on an unexpected end MUST begin nothing — the reader's explicit resume is \
         the only start (FR-031): the expiry was {expiry:?}"
    );

    let mut machine = OfferMachine::new();
    machine.on_foregrounded();
    let deliberate = StoredPosition {
        offset_secs: 12.5,
        first_yield_ms: 250,
        end_kind: SessionEnd::Paused,
    };
    let open = machine.resolve_open(Some(deliberate));
    assert!(
        matches!(open, OpenOffer::Continue { .. }),
        "a deliberate pause MUST open the continue offer (FR-032c): the open decision was {open:?}"
    );
    let expiry = machine.expire();
    assert!(
        matches!(expiry, ExpiryOutcome::AutoStart { offset_secs } if (offset_secs - 12.5).abs() < 1e-9),
        "the silence rule MUST continue from the stored point (FR-009): the expiry was {expiry:?}"
    );
}
