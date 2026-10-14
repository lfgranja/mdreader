//! T038 — BLOCKING US4 purity property (Feature 004-ressoa-dogfood-build,
//! task T038).
//!
//! Requirement (tasks.md T038, verbatim): "Purity property in
//! `src-tauri/tests/test_purity.rs` (lock-screen + in-app snapshots across
//! playing/paused/section-change/rate-change/interruption ⇒ display-projection
//! keys ⊆ `{title, section_title, progress}` (full payload additionally
//! carries transport ∈ `{playing,paused,stopped}` + reason/reason_ttl_ms/
//! dismissal per contracts/surface.md Display-vs-payload; prohibited
//! anywhere: prose/time/rate-VALUE/voice); zero prose/time/rate-VALUE/voice
//! leaks — rate-step action permitted, no value display;
//! dismissal-refused-while-playing assert; zero sections ⇒ `section_title
//! == null` + indeterminate)"
//!
//! What this contract test pins, against
//! `specs/004-ressoa-dogfood-build/contracts/surface.md`:
//!
//! - **Display vs payload** (the contract's Display-vs-payload split): the
//!   DISPLAY projection of either surface is a subset of exactly
//!   `{title, section_title, progress}`. The full mirror payload
//!   additionally carries state and behavior — `transport` ∈
//!   `{playing, paused, stopped}` (STATE; `Interrupted`/`Completed` are
//!   end-kinds, never transport values, and the Stop CONTROL lives in-app
//!   only), `reason`, `reason_expiry`/`reason_ttl_ms`, `dismissal`,
//!   `position_secs`, `rate`, `fingerprint`. Those are state, not display
//!   leaks.
//! - **Prohibited anywhere**: document prose (body text — never the
//!   sentence being read nor any paragraph), elapsed/remaining time, the
//!   numeric rate VALUE, and voice identity. The rate-step ACTION
//!   (RateUp/RateDown presses) is permitted on both surfaces; only the
//!   value display is prohibited (D1 fix, FR-012 value-blind).
//! - **Dismissal refused while playing**: `Dismissal::NonDismissable` for
//!   every surface kind while audio plays — pausing is the only route to
//!   dismissal (FR-028e); the rest of the closed truth table is pinned
//!   alongside.
//! - **Zero sections**: a resolve over an empty heading list names no
//!   section title and stays indeterminate for the whole session
//!   (contracts/surface.md "zero sections → indeterminate + no title";
//!   a zero-section document marks no extent, so its total is unknown at
//!   start and FR-028a's whole-session indeterminate rule applies. The
//!   duration-known zero-heading cell — determinate with a zero total,
//!   freeze-then-drop — is pinned by T006 `prop_progress_fraction`).
//!
//! Bindings (all public surfaces; `src-tauri/src/lib.rs` is read-only
//! reference here — the `pub mod publish` seam's `publish_mirror` is the
//! T132 test-compatible payload builder):
//! - `src_tauri::status_view::render_status_view` — the in-app status
//!   view renderer (T065b), a pure function over its inputs;
//! - `src_tauri::publish::publish_mirror` — the full mirror payload the
//!   frontend channel carries;
//! - `player::surface::{SurfaceResolver, SurfaceSnapshot, Progress,
//!   Dismissal}` — the lock-screen surface projection (T061);
//! - `src_tauri::surface::{FullReason, ReasonExpiry, full_reason}` — the
//!   single source of every rendered reason statement (T065).
//!
//! This is the US4 snapshot/contract test rule 2 names (T038 by
//! T006 `prop_progress_fraction` for the determinacy subset, complemented
//! by T051 on-device purity inspections); it is BLOCKING for the T041–T044
//! implementation wave and must stay green across it.

use serde_json::{Map, Value};

use player::surface::{
    Dismissal, Heading, Progress, SurfaceResolver, SurfaceSnapshot, TRANSIENT_DISMISS_MS,
};
use player::transport::{PlaybackRate, TransportState};
use src_tauri::publish::publish_mirror;
use src_tauri::status_view::{StatusView, render_status_view};
use src_tauri::surface::{FullReason, ReasonExpiry, full_reason};

/// The display projection: the ONLY keys either surface may show
/// (contracts/surface.md Display-vs-payload — "display projection keys
/// ⊆ {title, section_title, progress}").
const DISPLAY_KEYS: [&str; 3] = ["title", "section_title", "progress"];

/// The permitted full-payload keys beyond the display projection —
/// state and behavior, never display leaks (contracts/surface.md
/// Display-vs-payload: the payload additionally carries transport,
/// reason/reason_ttl_ms and dismissal). The established mirror set
/// (T015 `test_surface_us1.rs`) plus the contract's `reason_ttl_ms`
/// name and the publish channel's `transient_reason` carrier.
const PERMITTED_PAYLOAD_KEYS: [&str; 9] = [
    "transport",
    "position_secs",
    "rate",
    "reason",
    "reason_expiry",
    "reason_ttl_ms",
    "dismissal",
    "fingerprint",
    "transient_reason",
];

/// The forbidden key fragments: document prose, elapsed/remaining
/// time, the numeric rate value, and voice identity — none may appear
/// in any payload key on either surface (contracts/surface.md
/// "Prohibited anywhere").
const FORBIDDEN_KEY_FRAGMENTS: [&str; 12] = [
    "prose",
    "body",
    "text",
    "elapsed",
    "remaining",
    "duration_secs",
    "position_secs_display",
    "rate_value",
    "speed",
    "voice",
    "engine_id",
    "stop",
];

/// Every rendered rate VALUE string — the FR-012 value-blind set. A
/// surface may carry a rate-step ACTION and the stored rung for
/// stepper orientation, but no rendered value string like `1.5x`.
const RATE_VALUE_STRINGS: [&str; 7] = [
    "0.75x", "1.0x", "1.25x", "1.5x", "1.75x", "2.0x", "4.0x",
];

/// Time-display words (both surface languages): no elapsed or
/// remaining time ever renders on a surface.
const TIME_DISPLAY_WORDS: [&str; 4] = ["elapsed", "remaining", "restante", "decorrido"];

/// The document title fixture: metadata title, never body prose
/// (FR-029 / FR-011).
const DOCUMENT_TITLE: &str = "Rota da Manhã";

/// The filename placeholder fixture — the stable title a title-less
/// document resolves to (FR-029).
const FILENAME_PLACEHOLDER: &str = "rota-da-manha.md";

/// A body-prose marker that must never reach any surface payload:
/// neither the sentence being read nor any paragraph (FR-028 series).
const BODY_PROSE_MARKER: &str = "frase do corpo do documento";

/// The all-zero test fingerprint.
const TEST_FINGERPRINT: [u8; 32] = [0u8; 32];

/// The closed `FullReason` set — all ten members (Principle VII /
/// FR-044), so the reason-property tests cover the whole set.
const ALL_FULL_REASONS: [FullReason; 10] = [
    FullReason::FocusSilenceDemanding,
    FullReason::FocusPermanentLoss,
    FullReason::EngineUnreachable,
    FullReason::EngineInitFailed,
    FullReason::EngineRateMismatch,
    FullReason::EngineChannelMismatch,
    FullReason::EngineDeviceRefused,
    FullReason::EngineDeviceLost,
    FullReason::EngineStartRefused,
    FullReason::NotificationDenied,
];

/// The seven failure reasons — none may quote document text (FR-037).
const FAILURE_REASONS: [FullReason; 7] = [
    FullReason::EngineUnreachable,
    FullReason::EngineInitFailed,
    FullReason::EngineRateMismatch,
    FullReason::EngineChannelMismatch,
    FullReason::EngineDeviceRefused,
    FullReason::EngineDeviceLost,
    FullReason::EngineStartRefused,
];

/// The ordered heading fixture (the T061b provider shape): three
/// sections, the boundary crosses at 45.0, 90.0, and 150.0 s.
fn headings() -> Vec<Heading> {
    vec![
        Heading {
            position_secs: 45.0,
            title: "Parte I".to_string(),
        },
        Heading {
            position_secs: 90.0,
            title: "Parte II".to_string(),
        },
        Heading {
            position_secs: 150.0,
            title: "Parte III".to_string(),
        },
    ]
}

/// One modeled session state of the five the requirement names.
struct StateCase {
    /// The state's name, for failure messages.
    name: &'static str,
    /// The stored `last_rate` the view mirrors (SC-027 / FR-019b).
    last_rate: Option<PlaybackRate>,
    /// The session's authoritative transport (FR-020).
    transport: TransportState,
    /// The session's authoritative position (FR-002 / FR-003).
    position_secs: f64,
    /// The current stated reason with its expiry classification, if any.
    reason: Option<(FullReason, ReasonExpiry)>,
}

/// The five states: playing, paused, section-change, rate-change, and
/// interruption — the status-view renders modeled with different
/// transports/reasons, the lock-screen resolvers resolved at the
/// section-change positions.
fn state_cases() -> Vec<StateCase> {
    vec![
        StateCase {
            name: "playing",
            last_rate: Some(PlaybackRate::R1_0),
            transport: TransportState::Playing,
            position_secs: 50.0,
            reason: None,
        },
        StateCase {
            name: "paused",
            last_rate: Some(PlaybackRate::R1_0),
            transport: TransportState::Paused,
            position_secs: 100.0,
            reason: None,
        },
        StateCase {
            // The boundary cross at 90.0 s: the section title moves
            // from "Parte I" to "Parte II" (FR-028d).
            name: "section-change",
            last_rate: Some(PlaybackRate::R1_25),
            transport: TransportState::Playing,
            position_secs: 90.0,
            reason: None,
        },
        StateCase {
            // The rate stepped to the fastest rung; the stepper
            // orients by ladder position, never the value (FR-012).
            name: "rate-change",
            last_rate: Some(PlaybackRate::R4_0),
            transport: TransportState::Playing,
            position_secs: 50.0,
            reason: None,
        },
        StateCase {
            // A permanent focus loss: the pause reason expires 3 s
            // after the pause on both surfaces (FR-041 / FR-037a).
            name: "interruption",
            last_rate: Some(PlaybackRate::R1_75),
            transport: TransportState::Interrupted,
            position_secs: 95.0,
            reason: Some((FullReason::FocusPermanentLoss, ReasonExpiry::Focus3s)),
        },
    ]
}

/// The mirror-transport wire value a session state surfaces as:
/// the closed `{playing, paused, stopped}` STATE set —
/// `Interrupted`/`Completed` are end-kinds, never transport values
/// (contracts/surface.md), and the lock screen shows an interruption
/// as the paused state.
fn mirror_transport_wire(transport: TransportState) -> &'static str {
    match transport {
        TransportState::Playing => "playing",
        TransportState::Paused | TransportState::Interrupted => "paused",
        TransportState::Stopped | TransportState::Completed => "stopped",
    }
}

/// Assert one payload object's keys are pure: every key is either a
/// display-projection key (⊆ {title, section_title, progress}) or a
/// permitted full-payload state key, and no key carries a forbidden
/// fragment (prose/time/rate-value/voice identity).
fn assert_key_purity(label: &str, object: &Map<String, Value>) {
    for key in object.keys() {
        let display = DISPLAY_KEYS.contains(&key.as_str());
        let payload = PERMITTED_PAYLOAD_KEYS.contains(&key.as_str());
        assert!(
            display || payload,
            "{label}: payload key {key:?} is neither a display-projection key nor a permitted state key (contracts/surface.md Display-vs-payload)"
        );
        for fragment in FORBIDDEN_KEY_FRAGMENTS {
            assert!(
                !key.contains(fragment),
                "{label}: payload key {key:?} carries forbidden fragment {fragment:?} — prose/time/rate-value/voice prohibited anywhere"
            );
        }
    }
}

/// Assert one rendered payload carries no prohibited VALUE: no rate
/// value string (FR-012 value-blind), no body prose (FR-028 series),
/// and no time-display word.
fn assert_no_leaked_values(label: &str, rendered: &str) {
    for value in RATE_VALUE_STRINGS {
        assert!(
            !rendered.contains(value),
            "{label}: rendered payload leaks the rate value {value:?} — rate-step action permitted, value display prohibited (FR-012)"
        );
    }
    assert!(
        !rendered.contains(BODY_PROSE_MARKER),
        "{label}: rendered payload leaks document prose ({BODY_PROSE_MARKER:?}) — never the sentence being read nor any paragraph"
    );
    for word in TIME_DISPLAY_WORDS {
        assert!(
            !rendered.contains(word),
            "{label}: rendered payload leaks a time display ({word:?}) — elapsed/remaining time prohibited on both surfaces"
        );
    }
}

/// The in-app status view for one state, as its JSON object form.
fn status_view_object(state: &StateCase) -> Result<Value, Box<dyn std::error::Error>> {
    let view: StatusView = render_status_view(
        state.last_rate,
        state.transport,
        state.position_secs,
        DOCUMENT_TITLE,
        state.reason,
        TEST_FINGERPRINT,
    );
    Ok(serde_json::to_value(&view)?)
}

/// The lock-screen surface snapshot for one state, as its JSON object
/// form, resolved through the real `SurfaceResolver` over the heading
/// fixture.
fn surface_snapshot_object(
    resolver: &SurfaceResolver,
    state: &StateCase,
) -> Result<Value, Box<dyn std::error::Error>> {
    let snapshot: SurfaceSnapshot = resolver.resolve(state.position_secs, &headings());
    Ok(serde_json::to_value(&snapshot)?)
}

/// The full mirror payload for one state through the publish seam.
fn publish_payload_object(state: &StateCase) -> Result<Value, Box<dyn std::error::Error>> {
    let section_title = snapshot_section_title(state.position_secs);
    let payload = publish_mirror(
        mirror_transport_wire(state.transport),
        state.position_secs,
        section_title,
        true,
        state.reason.map(|(reason, _)| full_reason(&reason, DOCUMENT_TITLE)),
    );
    Ok(payload)
}

/// The most-recent-passed section title at `position_secs` over the
/// heading fixture — the same pure rule the resolver applies.
fn snapshot_section_title(position_secs: f64) -> Option<String> {
    headings()
        .into_iter()
        .rev()
        .find(|heading| heading.position_secs <= position_secs)
        .map(|heading| heading.title)
}

#[test]
fn the_five_states_keep_the_display_projection_and_payload_pure()
-> Result<(), Box<dyn std::error::Error>> {
    let resolver =
        SurfaceResolver::new(Some(DOCUMENT_TITLE.to_string()), FILENAME_PLACEHOLDER.to_string(), true);
    for state in state_cases() {
        let label = state.name;

        // --- the in-app status view (T065b) ---
        let view_value = status_view_object(&state)?;
        let view_object = view_value
            .as_object()
            .ok_or("the status view is not a JSON object")?;
        assert_key_purity(label, view_object);
        assert_no_leaked_values(label, &serde_json::to_string(&view_value)?);

        // --- the lock-screen surface snapshot (T061) ---
        let surface_value = surface_snapshot_object(&resolver, &state)?;
        let surface_object = surface_value
            .as_object()
            .ok_or("the surface snapshot is not a JSON object")?;
        assert_key_purity(label, surface_object);
        // The DISPLAY projection is exactly the subset
        // {title, section_title, progress}: the lock-screen snapshot
        // carries no payload-state key at all.
        for key in surface_object.keys() {
            assert!(
                DISPLAY_KEYS.contains(&key.as_str()),
                "{label}: the lock-screen display projection carries {key:?} outside {{title, section_title, progress}}"
            );
        }
        assert_no_leaked_values(label, &serde_json::to_string(&surface_value)?);

        // --- the full mirror payload (publish channel) ---
        let payload_value = publish_payload_object(&state)?;
        let payload_object = payload_value
            .as_object()
            .ok_or("the mirror payload is not a JSON object")?;
        assert_key_purity(label, payload_object);
        assert_no_leaked_values(label, &serde_json::to_string(&payload_value)?);
        // The payload's transport is a STATE from the closed
        // {playing, paused, stopped} set — never an end-kind, never
        // a control verb.
        let wire = payload_object
            .get("transport")
            .and_then(Value::as_str)
            .ok_or("the mirror payload carries no transport")?;
        assert!(
            matches!(wire, "playing" | "paused" | "stopped"),
            "{label}: the mirror payload carries transport {wire:?} outside the closed {{playing,paused,stopped}} STATE set"
        );
    }
    Ok(())
}

#[test]
fn the_interruption_reason_states_the_pause_statement_and_expires_in_three_seconds()
-> Result<(), Box<dyn std::error::Error>> {
    // The interruption state renders the specification-fixed Pause
    // Reason through the single T065 getter, naming the document
    // title and nothing else from the document (FR-041), and the
    // focus regime expires 3 s after the pause on both surfaces
    // (FR-037a).
    let state = state_cases()
        .into_iter()
        .find(|state| state.name == "interruption")
        .ok_or("the five-state set lost its interruption member")?;
    let view = render_status_view(
        state.last_rate,
        state.transport,
        state.position_secs,
        DOCUMENT_TITLE,
        state.reason,
        TEST_FINGERPRINT,
    );
    let statement = view.reason.ok_or("the interruption state states no reason")?;
    assert_eq!(statement, full_reason(&FullReason::FocusPermanentLoss, DOCUMENT_TITLE));
    assert_eq!(view.reason_expiry, Some(ReasonExpiry::Focus3s));
    // It names the title (allowed) and never the body prose.
    assert!(statement.contains(DOCUMENT_TITLE));
    assert!(!statement.contains(BODY_PROSE_MARKER));
    Ok(())
}

#[test]
fn no_reason_statement_ever_quotes_document_prose() -> Result<(), Box<dyn std::error::Error>> {
    // FR-037: a rendered reason never quotes document content. The
    // failure and denial statements take no document text at all; the
    // focus-loss arms name the title and nothing else.
    for reason in ALL_FULL_REASONS {
        let view = render_status_view(
            Some(PlaybackRate::R1_0),
            TransportState::Paused,
            33.25,
            DOCUMENT_TITLE,
            Some((reason, reason.expiry())),
            TEST_FINGERPRINT,
        );
        let statement = view.reason.ok_or("a stated reason rendered nothing")?;
        assert!(
            !statement.contains(BODY_PROSE_MARKER),
            "the {reason:?} statement quotes document prose"
        );
        if FAILURE_REASONS.contains(&reason) || reason == FullReason::NotificationDenied {
            assert!(
                !statement.contains(DOCUMENT_TITLE),
                "the {reason:?} statement quotes the document — only the focus-loss arms may name the title"
            );
        }
        assert_no_leaked_values("reason", &serde_json::to_string(&view)?);
    }
    Ok(())
}

#[test]
fn the_section_title_is_the_most_recent_passed_heading_never_forward_looking()
-> Result<(), Box<dyn std::error::Error>> {
    let resolver = SurfaceResolver::new(
        Some(DOCUMENT_TITLE.to_string()),
        FILENAME_PLACEHOLDER.to_string(),
        true,
    );
    // Before the first heading: no section title — never a
    // forward-looking heading (FR-028c).
    let before = resolver.resolve(10.0, &headings());
    assert_eq!(before.section_title, None);
    // At the boundary (at-or-before): the new section, and k steps.
    let at_boundary = resolver.resolve(90.0, &headings());
    assert_eq!(at_boundary.section_title.as_deref(), Some("Parte II"));
    // Just before the boundary: still the previous section.
    let approaching = resolver.resolve(89.75, &headings());
    assert_eq!(approaching.section_title.as_deref(), Some("Parte I"));
    // Past the last heading: the last section title stands.
    let past_end = resolver.resolve(180.0, &headings());
    assert_eq!(past_end.section_title.as_deref(), Some("Parte III"));
    // The section-change display projection: exactly the three keys,
    // with the most-recent-passed title.
    let value = serde_json::to_value(&at_boundary)?;
    let object = value.as_object().ok_or("the snapshot is not an object")?;
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["progress", "section_title", "title"]);
    assert_eq!(object.get("title").and_then(Value::as_str), Some(DOCUMENT_TITLE));
    Ok(())
}

#[test]
fn determinacy_is_fixed_at_session_start_and_never_flips() -> Result<(), Box<dyn std::error::Error>> {
    // FR-028a: a session that knew its total at start resolves
    // determinate at EVERY position (freeze-then-drop); one that did
    // not stays indeterminate for the whole session. The five states
    // hold one determinacy each — never a flip, never an estimate.
    let known = SurfaceResolver::new(
        Some(DOCUMENT_TITLE.to_string()),
        FILENAME_PLACEHOLDER.to_string(),
        true,
    );
    let unknown = SurfaceResolver::new(
        Some(DOCUMENT_TITLE.to_string()),
        FILENAME_PLACEHOLDER.to_string(),
        false,
    );
    for state in state_cases() {
        let known_snapshot = known.resolve(state.position_secs, &headings());
        let unknown_snapshot = unknown.resolve(state.position_secs, &headings());
        assert!(
            matches!(known_snapshot.progress, Progress::Determinate { .. }),
            "{}: a duration-known session must resolve determinate at every position",
            state.name
        );
        assert!(
            matches!(unknown_snapshot.progress, Progress::Indeterminate),
            "{}: a duration-unknown session must stay indeterminate for the whole session",
            state.name
        );
        // The determinate total is the marked document extent (the
        // greatest heading position: 150), never an estimate; the
        // consumed offset is the resolve position's whole units.
        let Progress::Determinate {
            consumed_units,
            total_units,
            k,
            n,
        } = known_snapshot.progress
        else {
            return Err("a duration-known session produced no determinate progress".into());
        };
        assert_eq!(total_units, 150, "the marked extent is the greatest heading position");
        assert_eq!(n, 3, "the heading list marks three sections");
        assert_eq!(k, expected_section_index(state.position_secs));
        assert_eq!(consumed_units, expected_consumed_units(state.position_secs));
    }
    Ok(())
}

/// The expected 1-based section index at `position_secs` over the
/// heading fixture (0 before the first heading).
fn expected_section_index(position_secs: f64) -> u32 {
    let headings = headings();
    headings
        .iter()
        .rposition(|heading| heading.position_secs <= position_secs)
        .map_or(0, |index| (index + 1) as u32)
}

/// The expected consumed content units at `position_secs`: the
/// resolve position's whole seconds (the fixture positions are
/// whole-second values; the resolver truncates, FR-028a).
fn expected_consumed_units(position_secs: f64) -> u64 {
    position_secs.floor() as u64
}

#[test]
fn dismissal_is_refused_while_playing_and_the_closed_truth_table_holds() {
    let resolver = SurfaceResolver::new(
        Some(DOCUMENT_TITLE.to_string()),
        FILENAME_PLACEHOLDER.to_string(),
        true,
    );
    // Dismissal refused while playing, for EVERY surface kind: the
    // playing state never becomes dismissable — pausing is the only
    // route to dismissal while audio plays (FR-028e).
    for transient in [true, false] {
        assert_eq!(
            resolver.dismissal(TransportState::Playing, transient),
            Dismissal::NonDismissable,
            "dismissal must be refused while playing (transient = {transient})"
        );
    }
    // The transient window is exactly 3 s.
    assert_eq!(TRANSIENT_DISMISS_MS, 3_000);
    // The rest of the closed truth table: a live non-playing state
    // splits on the surface kind; a torn-down state has nothing left
    // to keep.
    for transport in [TransportState::Paused, TransportState::Interrupted] {
        assert_eq!(
            resolver.dismissal(transport, true),
            Dismissal::AutoDismiss {
                after_ms: TRANSIENT_DISMISS_MS,
            },
            "a transient {transport:?} surface auto-dismisses at 3 s"
        );
        assert_eq!(
            resolver.dismissal(transport, false),
            Dismissal::Dismissable,
            "a resident {transport:?} surface is dismissable, never auto-dismissed"
        );
    }
    for transport in [TransportState::Stopped, TransportState::Completed] {
        for transient in [true, false] {
            assert_eq!(
                resolver.dismissal(transport, transient),
                Dismissal::Dismissable,
                "a torn-down {transport:?} surface is dismissable (transient = {transient})"
            );
        }
    }
}

#[test]
fn zero_sections_resolve_no_section_title_and_indeterminate_progress()
-> Result<(), Box<dyn std::error::Error>> {
    // Zero sections: the resolve names no section title and the whole
    // session stays indeterminate — a heading-less document marks no
    // extent, so its total content offset is unknown at start
    // (contracts/surface.md "zero sections → indeterminate + no
    // title"; FR-028a).
    let resolver = SurfaceResolver::new(
        Some(DOCUMENT_TITLE.to_string()),
        FILENAME_PLACEHOLDER.to_string(),
        false,
    );
    let snapshot = resolver.resolve(30.0, &[]);
    assert_eq!(
        snapshot.section_title, None,
        "a zero-sections resolve must name no section title"
    );
    assert!(
        matches!(snapshot.progress, Progress::Indeterminate),
        "a zero-sections session must resolve indeterminate progress"
    );
    // The wire form omits the section key (the absent form of the
    // contract's `null`) and tags the progress indeterminate.
    let value = serde_json::to_value(&snapshot)?;
    let object = value.as_object().ok_or("the snapshot is not an object")?;
    assert_eq!(object.get("section_title"), None, "the null section title serializes as the absent key");
    assert_eq!(
        object.get("progress").and_then(Value::as_str),
        Some("indeterminate"),
        "the progress wire form is the indeterminate tag"
    );
    assert_eq!(
        object.get("title").and_then(Value::as_str),
        Some(DOCUMENT_TITLE),
        "the title stays the metadata title (FR-029)"
    );
    Ok(())
}

#[test]
fn the_rate_step_is_value_blind_on_both_surfaces() -> Result<(), Box<dyn std::error::Error>> {
    // FR-012 / D1: the rate-step affordance is permitted without a
    // value display on either surface. The stored rung mirrors so the
    // stepper orients by ladder position; no rendered surface carries
    // the numeric value.
    for rung in [
        PlaybackRate::R0_75,
        PlaybackRate::R1_0,
        PlaybackRate::R1_25,
        PlaybackRate::R1_5,
        PlaybackRate::R1_75,
        PlaybackRate::R2_0,
        PlaybackRate::R4_0,
    ] {
        let view = render_status_view(
            Some(rung),
            TransportState::Playing,
            12.5,
            DOCUMENT_TITLE,
            None,
            TEST_FINGERPRINT,
        );
        assert_eq!(view.rate, rung, "the stored rung mirrors unchanged");
        assert_no_leaked_values("rate-step", &serde_json::to_string(&view)?);
        // The lock-screen projection never carries a rate key at all:
        // its display set is exactly {title, section_title, progress}.
        let resolver = SurfaceResolver::new(
            Some(DOCUMENT_TITLE.to_string()),
            FILENAME_PLACEHOLDER.to_string(),
            true,
        );
        let snapshot = resolver.resolve(12.5, &headings());
        assert_no_leaked_values("rate-step", &serde_json::to_string(&snapshot)?);
        let value = serde_json::to_value(&snapshot)?;
        let object = value.as_object().ok_or("the snapshot is not an object")?;
        assert!(
            !object.contains_key("rate"),
            "the lock-screen display projection must not carry a rate key"
        );
    }
    Ok(())
}
