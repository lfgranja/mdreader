//! T116 — Phase 16 RED-FIRST contract test: the in-app resume
//! offer surface (feature `004-ressoa-dogfood-build`).
//!
//! The finding (tasks.md F2): `src-tauri/src/status_view.html`
//! has ZERO hits for `offer` — the single in-app screen has no
//! resume-offer dialog, no frontend code reads the
//! `offer_route` / `offer_window_ms` the shell publishes
//! (`src-tauri/src/lib.rs:2646-2668`, `:4238`), and no code
//! path anywhere sends the `resume_at_stored` / `start_over`
//! verbs (only the verb-name constants exist, in
//! `MediaMirror.kt:366,370`). The consequence: "start over" is
//! unreachable and the deliberate-pause offer resolves only
//! through the automatic 3-second silence rule (FR-009).
//!
//! The contract this file pins (FR-009, FR-011, FR-019 and
//! `specs/004-ressoa-dogfood-build/contracts/surface.md` —
//! "Resume: deliberate-pause reopen → 3 s visual dialog
//! (continue vs start-over; silence = continue from stored)"):
//!
//! 1. **The surface consumes the published offer shape.** The
//!    shell publishes the route key `offer_route` (the
//!    cold-start open payload and the `offer_route` command's
//!    reply) and the FR-032c window; the surface must read the
//!    route, name the in-app route token `in_app_dialog` — the
//!    only route that opens a dialog (the `lockscreen_resume`
//!    route opens NO in-app dialog, per the same contract), read
//!    the published window, and carry a semantic dialog
//!    element.
//! 2. **Exactly two choices.** The offer-choice verbs the
//!    surface sends are exactly `resume_at_stored` (continue
//!    from the stored position) and `start_over` (begin at the
//!    first spoken position) — and nothing beyond the
//!    transport/rate controls FR-019 and FR-012 name.
//! 3. **Purity.** The offer surface paints no document prose,
//!    no numeric rate value, no voice identity, and no
//!    elapsed/remaining time (FR-011 / FR-019).
//! 4. **The 3-second bound, owned by the shell.** The window
//!    the surface consumes is the shell's own 3-second offer
//!    window, and only the deliberate pause routes to the in-app
//!    dialog — the unexpected end waits paused for the
//!    lock-screen resume control (FR-009, FR-031).
//!
//! The mechanism (the surface is plain HTML with no JavaScript
//! test harness): a static contract assertion over the surface
//! source — the same mechanism `test_noscope.rs` uses for its
//! HTML guard — plus the Rust-side assertions over the
//! production dispatcher (`commands::parse_action`), the offer
//! machine (`session::offer`), and the shell's published offer
//! shape. Every assertion is an INVARIANT, never a literal HTML
//! snapshot: the surface is free to name its elements as it
//! likes, as long as it handles the offer shape, offers exactly
//! the two closed choices, and paints nothing prohibited.

use std::collections::BTreeSet;

use engine_core::closed::SessionEnd;
use player::cli::ShellVerb;
use session::offer::{ColdStart, ExpiryOutcome, OfferMachine, OpenOffer, cold_start};
use session::store::StoredPosition;
use src_tauri::commands::{BridgeAction, parse_action};

/// The in-app surface — the single screen the reader reads on
/// (FR-019).
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// The shell source — the publisher of the offer shape the
/// surface must consume.
const SHELL_SOURCE: &str = include_str!("../src/lib.rs");

/// The transport and rate control verbs the in-app screen owns:
/// FR-019's transport controls (play, pause, skip-forward/back,
/// and the dedicated Stop) plus FR-012's value-blind speed
/// stepper. They are the surface's own vocabulary, never the
/// offer's.
const CONTROL_VERBS: [&str; 7] = [
    "play",
    "pause",
    "stop",
    "skip_forward5s",
    "skip_back5s",
    "rate_up",
    "rate_down",
];

/// The two closed offer-choice verbs (FR-009): continue from
/// the stored position, and start over from the first spoken
/// position. The set is closed — a third choice does not exist.
const OFFER_CHOICE_VERBS: [&str; 2] = ["resume_at_stored", "start_over"];

/// The numeric rate values the surface may never paint (FR-012
/// value-blind; the same convention `test_purity.rs` uses).
const RATE_VALUE_STRINGS: [&str; 7] = ["0.75x", "1.0x", "1.25x", "1.5x", "1.75x", "2.0x", "4.0x"];

/// The identifiers no painted line may carry (FR-011 /
/// FR-019): the numeric rate value, a reading-time field, a
/// voice identity, and a document-prose field. The list is
/// matched EXACTLY, so a ladder position (`rateIndex`) and a
/// control element (`btnRateUp`) are not rate values.
const PROHIBITED_PAINT_IDENTIFIERS: [&str; 16] = [
    "rate",
    "positionSecs",
    "position_secs",
    "elapsed",
    "remaining",
    "durationSecs",
    "duration_secs",
    "voice",
    "voiceId",
    "engine",
    "engineId",
    "prose",
    "paragraph",
    "bodyText",
    "documentText",
    "docText",
];

/// The time words no painted line may carry (pt-BR and
/// English): an elapsed or remaining reading time.
const PROHIBITED_PAINT_WORDS: [&str; 5] = [
    "elapsed",
    "remaining",
    "restante",
    "decorrido",
    "transcorrido",
];

/// Every string literal in the source — single-quoted,
/// double-quoted, and template literals (an embedded `${…}`
/// expression is scanned in turn, so a verb inside one is
/// still found).
fn string_literals(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    collect_literals(source, &mut found);
    found
}

/// Collect the string literals of one source fragment.
fn collect_literals(source: &str, found: &mut Vec<String>) {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'\'' | b'"' => {
                let start = index + 1;
                let mut end = start;
                while end < bytes.len() {
                    if bytes[end] == b'\\' {
                        end += 2;
                        continue;
                    }
                    if bytes[end] == byte {
                        break;
                    }
                    end += 1;
                }
                let end = end.min(bytes.len());
                found.push(source[start..end].to_string());
                index = end + 1;
            }
            b'`' => {
                let start = index + 1;
                let mut end = start;
                let mut depth = 0_u32;
                let mut expression_start = end;
                while end < bytes.len() {
                    let inner = bytes[end];
                    if inner == b'\\' {
                        end += 2;
                        continue;
                    }
                    if depth == 0 {
                        if inner == b'`' {
                            break;
                        }
                        if inner == b'$' && bytes.get(end + 1) == Some(&b'{') {
                            depth = 1;
                            expression_start = end + 2;
                            end += 2;
                            continue;
                        }
                        end += 1;
                        continue;
                    }
                    match inner {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                collect_literals(&source[expression_start..end], found);
                            }
                        }
                        _ => {}
                    }
                    end += 1;
                }
                let end = end.min(bytes.len());
                found.push(source[start..end].to_string());
                index = end + 1;
            }
            _ => index += 1,
        }
    }
}

/// The shell-IPC verbs the surface can send — every string
/// literal the production dispatcher (`commands::parse_action`)
/// classifies as a session verb. The dispatcher's closed wire
/// set is the single source of the vocabulary; the test never
/// re-declares it.
fn sent_verbs(source: &str) -> BTreeSet<String> {
    let mut verbs = BTreeSet::new();
    for literal in string_literals(source) {
        if matches!(parse_action(&literal), Ok(BridgeAction::Verb(_))) {
            let _ = verbs.insert(literal);
        }
    }
    verbs
}

/// The JavaScript identifier tokens of a fragment — so
/// `state.rate` yields `state` and `rate`, while `btnRateUp`
/// stays one token.
fn identifiers(fragment: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut start: Option<usize> = None;
    for (index, character) in fragment.char_indices() {
        if character.is_ascii_alphanumeric() || character == '_' || character == '$' {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(begin) = start.take() {
            found.push(&fragment[begin..index]);
        }
    }
    if let Some(begin) = start {
        found.push(&fragment[begin..]);
    }
    found
}

/// The lines that PAINT user-visible text: a `textContent` or
/// `innerHTML` assignment, or an `aria-label` (a label is
/// painted too).
fn painting_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| {
            line.contains(".textContent")
                || line.contains(".innerHTML")
                || line.contains("aria-label")
        })
        .collect()
}

/// The stored offset the deliberate-pause offer continues from.
const STORED_OFFSET_SECS: f64 = 42.0;

/// The position-comparison margin — the stored offset is copied
/// verbatim, so the comparison is exact up to the margin
/// (`clippy::float_cmp` is denied workspace-wide; the same
/// epsilon form `test_shell_ipc_us4.rs` uses).
const POSITION_EPS_SECS: f64 = 1e-9;

/// A stored reading position with the given end kind — the row
/// the offer decision is resolved from.
const fn stored_row(offset_secs: f64, end_kind: SessionEnd) -> StoredPosition {
    StoredPosition {
        offset_secs,
        first_yield_ms: 0,
        end_kind,
    }
}

/// T116 RED-FIRST: the single in-app screen must CONSUME the
/// offer shape the shell publishes — the route key, the in-app
/// route token, the window, and a semantic dialog element.
///
/// Today the surface has zero `offer` handling: no dialog
/// element, no route read, no window read. The assertion fails
/// until T117 renders the offer.
#[test]
fn the_in_app_surface_consumes_the_published_offer_shape() {
    assert!(
        STATUS_VIEW.contains("offer_route"),
        "T116: the in-app surface must read the published offer route key `offer_route` — the shell publishes it on the cold-start open payload (`lib.rs`) and in the `offer_route` command's reply, and the surface is the only surface that renders the offer (FR-009)"
    );
    assert!(
        STATUS_VIEW.contains("in_app_dialog"),
        "T116: the in-app surface must name the in-app offer route `in_app_dialog` — it is the only route that opens the visual dialog (the `lockscreen_resume` route opens NO in-app dialog, per `contracts/surface.md`)"
    );
    assert!(
        STATUS_VIEW.contains("offer_window_ms") || STATUS_VIEW.contains("window_ms"),
        "T116: the in-app surface must read the published offer window (`offer_window_ms` on the cold-start open payload, `window_ms` in the `offer_route` command's reply) — the offer's 3-second bound is the shell's, never one the surface invents (FR-009, FR-032c)"
    );
    assert!(
        STATUS_VIEW.contains("role=\"dialog\"") || STATUS_VIEW.contains("<dialog"),
        "T116: the in-app surface must carry a semantic dialog element for the offer (`role=\"dialog\"` or a `<dialog>` element) — FR-009 requires a VISUAL dialog on the single in-app screen"
    );
}

/// T116 RED-FIRST: the offer dialog offers EXACTLY TWO choices —
/// continue (`resume_at_stored`) and start over (`start_over`) —
/// and the surface sends no other verb beyond the transport and
/// rate controls it already owns.
///
/// The set is derived from the production dispatcher: every
/// string literal the surface sends that `parse_action` accepts
/// as a session verb. A third offer choice, a voice verb, or an
/// engine verb fails here.
#[test]
fn the_offer_dialog_offers_exactly_two_choices() {
    let sent = sent_verbs(STATUS_VIEW);
    let choices: BTreeSet<&str> = sent
        .iter()
        .map(String::as_str)
        .filter(|verb| !CONTROL_VERBS.contains(verb))
        .collect();
    assert_eq!(
        choices,
        BTreeSet::from(OFFER_CHOICE_VERBS),
        "T116: the in-app offer dialog must offer exactly TWO choices — continue (`resume_at_stored`, from the stored position) and start over (`start_over`, from the first spoken position) — and no other verb beyond the transport/rate controls (FR-009, `contracts/surface.md` \"Resume:\")"
    );
    for choice in OFFER_CHOICE_VERBS {
        assert!(
            choices.contains(choice),
            "T116: the offer choice `{choice}` must be reachable from the surface — the in-app screen dispatches it through the existing `sendAction` → `run_mobile_plugin` path (FR-009)"
        );
    }
}

/// T116: the offer surface paints NOTHING prohibited — no
/// document prose, no numeric rate value, no voice identity, and
/// no elapsed/remaining time (FR-011 / FR-019). The guard holds
/// today and must still hold once the dialog lands.
#[test]
fn the_offer_surface_paints_no_prohibited_field() {
    for value in RATE_VALUE_STRINGS {
        assert!(
            !STATUS_VIEW.contains(value),
            "T116: the in-app surface must never paint a numeric rate value (`{value}`) — the stepper orients by ladder position alone (FR-011, FR-012, FR-019)"
        );
    }
    for line in painting_lines(STATUS_VIEW) {
        for identifier in identifiers(line) {
            assert!(
                !PROHIBITED_PAINT_IDENTIFIERS.contains(&identifier),
                "T116: a painted line may not carry `{identifier}` — the in-app screen shows the title, the section title, the progress and the transport state, never a rate value, a voice identity, a reading time, or document prose (FR-011, FR-019); the offending line is: {line}"
            );
        }
        for word in PROHIBITED_PAINT_WORDS {
            assert!(
                !line.contains(word),
                "T116: a painted line may not carry the time word `{word}` — no elapsed/remaining time reaches a surface (FR-011, FR-019); the offending line is: {line}"
            );
        }
    }
    assert!(
        !STATUS_VIEW.contains("innerText"),
        "T116: the in-app screen never reads painted text — it is a read-only mirror of the Rust snapshot, and reading text back is the only way document prose could reach it (FR-019)"
    );
    assert!(
        !STATUS_VIEW.contains("fetch("),
        "T116: the in-app screen never loads the document body — the surface renders the Rust snapshot's display projection only, never document prose (FR-019)"
    );
}

/// T116: the Rust side of the contract — the two offer-choice
/// wires resolve to the two closed offer verbs, the shell
/// publishes the offer shape the surface consumes, and the
/// window it publishes is the FR-032c 3-second bound.
#[test]
fn the_shell_publishes_the_offer_the_surface_consumes() {
    assert!(
        matches!(
            parse_action("resume_at_stored"),
            Ok(BridgeAction::Verb(ShellVerb::ResumeAtStored))
        ),
        "the wire `resume_at_stored` must parse to the ResumeAtStored verb — the continue choice (contracts/shell-ipc.md)"
    );
    assert!(
        matches!(
            parse_action("start_over"),
            Ok(BridgeAction::Verb(ShellVerb::StartOver))
        ),
        "the wire `start_over` must parse to the StartOver verb — the start-over choice (contracts/shell-ipc.md)"
    );
    for alias in [
        "resume",
        "continue",
        "start-over",
        "restart",
        "startFromStored",
    ] {
        assert!(
            parse_action(alias).is_err(),
            "the wire set is closed: `{alias}` is not an offer choice — only `resume_at_stored` and `start_over` are (contracts/shell-ipc.md)"
        );
    }
    assert!(
        SHELL_SOURCE.contains("commands::offer_route"),
        "the shell must expose the `offer_route` command the surface reads — the route-to-surface command (FR-009, FR-044)"
    );
    assert!(
        SHELL_SOURCE.contains("\"offer_route\""),
        "the shell must publish the offer route key `offer_route` — the surface's only way to learn that an offer is open (FR-009)"
    );
    assert!(
        SHELL_SOURCE.contains("\"offer_window_ms\""),
        "the shell must publish the offer window key `offer_window_ms` on the cold-start open payload — the surface's only way to learn the window (FR-009, FR-032c)"
    );
    let three_second_window = SHELL_SOURCE.lines().any(|line| {
        let normalized = line.replace('_', "");
        normalized.contains("const") && normalized.contains("= 3000") && normalized.contains("MS")
    });
    assert!(
        three_second_window,
        "the shell's offer window must be the FR-032c 3-second bound (`OFFER_WINDOW_MS = 3000`) — the contract's bound, never another duration (FR-009, `contracts/surface.md`)"
    );
}

/// T116: only the deliberate pause routes to the in-app dialog.
/// A deliberate pause continues from the stored position and
/// opens the offer; an unexpected end opens NO in-app dialog
/// (the lock-screen resume control is the only way back, FR-009
/// / FR-031); no stored position opens nothing.
#[test]
fn only_the_deliberate_pause_routes_to_the_in_app_dialog() {
    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(Some(stored_row(STORED_OFFSET_SECS, SessionEnd::Paused)));
    assert!(
        matches!(open, OpenOffer::Continue { offset_secs, .. } if (offset_secs - STORED_OFFSET_SECS).abs() <= POSITION_EPS_SECS),
        "a deliberate pause opens the continue offer at the STORED position — the continue choice resumes from there (FR-009)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::AwaitOffer { .. }),
        "a deliberate pause routes to the in-app 3-second dialog — no audio begins until the offer resolves (FR-009, FR-032c)"
    );

    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(Some(stored_row(
        STORED_OFFSET_SECS,
        SessionEnd::Interrupted,
    )));
    assert!(
        matches!(open, OpenOffer::ExplicitResume),
        "an unexpected end opens NO offer — the reader's explicit resume is the only start (FR-009, FR-031, FR-032d)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::AwaitExplicitResume),
        "an unexpected end routes to the lock-screen resume control, never to the in-app dialog (`contracts/surface.md`: no visual dialog after kill/power-loss)"
    );

    let mut machine = OfferMachine::new();
    let open = machine.resolve_open(None);
    assert!(
        matches!(open, OpenOffer::NoOffer),
        "no stored position opens no offer (FR-010, FR-036a)"
    );
    assert!(
        matches!(cold_start(open), ColdStart::StartNow),
        "no stored position starts the reading at the first spoken position, with no dialog (FR-010)"
    );

    let mut machine = OfferMachine::new();
    let _ = machine.resolve_open(Some(stored_row(STORED_OFFSET_SECS, SessionEnd::Paused)));
    assert!(
        matches!(machine.start_over(), ExpiryOutcome::AutoStart { offset_secs } if offset_secs.abs() <= POSITION_EPS_SECS),
        "the start-over choice begins at the first spoken position — it discards the stored entry (FR-009, FR-032c)"
    );
}
