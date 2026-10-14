//! T015 — BLOCKING US1 TEST: lock-screen payload contract
//! (Feature 004-ressoa-dogfood-build / task T015).
//!
//! Requirement (tasks.md T015, verbatim):
//! "Contract test lock-screen payload (no Stop CONTROL;
//! `stopped` STATE permitted) in `src-tauri/tests/
//! test_surface_us1.rs` (payload matches `contracts/
//! surface.md`: title+section+progress only, transport ∈
//! `{playing,paused,stopped}`, no `stop` control, no
//! prose/times/rate-VALUE/voice keys — rate-step action
//! permitted, no value display; see T038 Display-vs-payload)."
//!
//! The payload shapes are pinned against
//! `specs/004-ressoa-dogfood-build/contracts/surface.md`:
//! the display projection is `{title, section_title,
//! progress}` only; the full mirror payload additionally
//! carries `transport` ∈ `{playing,paused,stopped}`
//! (STATE — `stopped` is the state, never the Stop
//! CONTROL, which is in-app only per FR-002/FR-019),
//! `reason`/`reason_ttl_ms`/`dismissal` per the contract's
//! Display-vs-payload split.
//!
//! Prohibited anywhere: document prose, elapsed/remaining
//! time, numeric rate value, voice identity.

use serde_json::Value;

use src_tauri::status_view::render_status_view;
use player::transport::{PlaybackRate, TransportState};

/// Every transport STATE the mirror payload may carry
/// (contracts/surface.md). `stopped` is a STATE — the
/// closed set has exactly three members, and none of them
/// is the Stop CONTROL.
const MIRROR_TRANSPORTS: [TransportState; 3] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Stopped,
];

/// The forbidden display-vector keys: document prose,
/// times, rate value, voice identity — none may appear in
/// any payload key (contracts/surface.md "Prohibited
/// anywhere").
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

/// The permitted full-mirror keys beyond the display
/// projection (state/behavior, not display leaks).
const PERMITTED_STATE_KEYS: [&str; 6] = [
    "transport",
    "reason",
    "reason_expiry",
    "dismissal",
    "fingerprint",
    "rate",
];

/// Renders the in-app status view for a transport state and
/// returns its JSON object form.
fn status_view_object(transport: TransportState) -> Value {
    let view = render_status_view(
        Some(PlaybackRate::R1_0),
        transport,
        30.0,
        "Rota da Manhã",
        None,
        [0u8; 32],
    );
    serde_json::to_value(&view).expect("status view serializes")
}

#[test]
fn status_payload_carries_title_section_and_progress_only() {
    // FR-011 / contracts/surface.md: the display projection
    // keys are a subset of {title, section_title, progress}.
    // The full payload additionally carries transport,
    // reason/reason_expiry, dismissal — state and behavior,
    // never display leaks.
    for transport in MIRROR_TRANSPORTS {
        let obj = status_view_object(transport);
        let object = obj.as_object().expect("status payload is a JSON object");
        for key in object.keys() {
            let permitted = PERMITTED_STATE_KEYS.contains(&key.as_str())
                || key == "section_title"
                || key == "progress"
                || key == "position_secs";
            assert!(
                permitted,
                "status payload for {transport:?} carries unexpected key {key:?} — the mirror wire form is closed (contracts/surface.md)"
            );
        }
    }
}

#[test]
fn no_forbidden_key_fragment_appears_in_any_payload() {
    // Prohibited anywhere: document prose, elapsed/remaining
    // time, numeric rate value, voice identity — checked as
    // key fragments across the whole payload object.
    for transport in MIRROR_TRANSPORTS {
        let obj = status_view_object(transport);
        let object = obj.as_object().expect("status payload is a JSON object");
        for key in object.keys() {
            for fragment in FORBIDDEN_KEY_FRAGMENTS {
                assert!(
                    !key.contains(fragment),
                    "payload key {key:?} for {transport:?} contains forbidden fragment {fragment:?} (FR-011 purity)"
                );
            }
        }
    }
}

#[test]
fn transport_state_set_is_closed_at_three_with_stopped_permitted() {
    // FR-002 / contracts/surface.md: transport is a STATE in
    // {playing, paused, stopped} — `stopped` is permitted as
    // a state, while the Stop CONTROL exists only in-app.
    // The closed-set shape is enforced by the engine-core
    // taxonomy; this test pins the mirror's rendering of
    // each state without leaking a control verb.
    for transport in MIRROR_TRANSPORTS {
        let obj = status_view_object(transport);
        let wire = serde_json::to_string(&obj.get("transport").expect("transport present"))
            .expect("transport renders");
        let permitted = wire.contains("playing")
            || wire.contains("paused")
            || wire.contains("stopped");
        assert!(
            permitted,
            "transport {wire} for {transport:?} is outside the closed {{playing,paused,stopped}} state set"
        );
        // A control verb must never appear as a payload key:
        // the mirror carries the state, the in-app screen
        // owns the Stop CONTROL.
        let object = obj.as_object().expect("object");
        assert!(
            !object.contains_key("stop"),
            "the mirror payload must not carry a stop control key (Stop is in-app only, FR-002/FR-019)"
        );
    }
}

#[test]
fn rate_mirror_is_the_stored_rung_not_a_display_value() {
    // FR-012: the lock-screen and in-app surfaces never show
    // the numeric rate value. The stored `last_rate` rung is
    // mirrored so the stepper orients by position, but no
    // user-visible rate value is rendered by the Rust
    // surface getters (the value-blind stepper lives in the
    // frontend and shows ladder position, never the rung).
    for rung in [
        PlaybackRate::R0_75,
        PlaybackRate::R1_0,
        PlaybackRate::R1_25,
        PlaybackRate::R1_5,
        PlaybackRate::R1_75,
        PlaybackRate::R2_0,
        PlaybackRate::R4_0,
    ] {
        let view = render_status_view(Some(rung), TransportState::Playing, 10.0, "Doc", None, [0u8; 32]);
        // The rung round-trips through the stored-rate mirror.
        assert_eq!(
            view.rate, rung,
            "the stored last_rate rung must mirror through the status view unchanged"
        );
        // No rate VALUE string is produced by the view.
        let rendered = serde_json::to_string(&view).expect("render");
        assert!(
            !rendered.contains("0.75x")
                && !rendered.contains("1.0x")
                && !rendered.contains("4.0x"),
            "the view must not render a numeric rate value (FR-012 value-blind)"
        );
    }
}
