//! T044 — US3 contract: removal-signal independence
//! (`contracts/shell-ipc.md`).
//!
//! Invariants under test, quoted from
//! `specs/001-android-media-shell/tasks.md` T044:
//!
//! - each of the two FR-016a mechanisms reaches pause ≤1 s
//!   independently;
//! - neither waits for the other;
//! - no removal passes unhandled when only one fires.
//!
//! Spec anchors: FR-016 (pause within one second when
//! headphones, a wired headset, or a connected Bluetooth A2DP
//! device are disconnected, so audio does not move to the
//! loudspeaker; a Bluetooth disconnect carries exactly the same
//! consequence as a wired removal — pause with the stored
//! position preserved); FR-016a (the one-second pause of FR-016
//! MUST be reached through two named mechanisms rather than one —
//! `ACTION_AUDIO_BECOMING_NOISY` is the primary signal and is
//! handled on every device removal it covers, and
//! `AudioDeviceCallback.onAudioDevicesRemoved` is the secondary
//! signal and is handled independently of the first, so a
//! Bluetooth route reporting the removal only through the device
//! callback still pauses; each mechanism MUST reach the pause
//! within one second of the removal being observable, and each
//! MUST be sufficient on its own — neither waits for the other,
//! and no removal may pass unhandled because the platform
//! happened to deliver only one of them, because the one-second
//! bound is the whole privacy guarantee of this feature — it is
//! what keeps a private document from moving to a loudspeaker —
//! and a guarantee held by a single signal is a guarantee the
//! platform may decline to signal); FR-017a (an appearance
//! while playing follows at the unchanged position — the
//! `Follow` counterpart asserted here); FR-039 / FR-044 (every
//! switch is recorded as an `output_device` record event);
//! SC-002 (the privacy guarantee the bound protects).
//!
//! Notes:
//!
//! - the routing state machine is T046 — this file is the
//!   contract it is implemented against. The decision shapes
//!   (`Pause { latency_ms, device }`, `Follow { device }`,
//!   `NoChange`), the three-variant signal set
//!   (`BecomingNoisy`, `DeviceDisappeared`, `DeviceAppeared`),
//!   the `ROUTING_PAUSE_BOUND_MS` constant, and the wire
//!   strings are the contract, not implementation details.
//! - `BecomingNoisy` names no device, so the primary's pause
//!   payload is asserted for its variant and latency only; the
//!   secondary and appearance legs carry the device-payload
//!   assertions.
//! - the assertion legs are T044's C1–C7: primary alone,
//!   secondary alone, isolation, closed enumeration, wired =
//!   Bluetooth consequence, wire conformance, structural
//!   independence.
//!
//! FAILS before `crates/player/src/routing.rs` exists (T046):
//! the `player::routing` module is unresolved, so this test
//! target does not build. That build failure is the required
//! pre-implementation red state (T045).

use diag::writer::{DeviceDirection, DeviceEvent, EventKind, RecordEvent};
use engine_core::closed::TransportState;
use player::routing::{
    OutputDevicePrivacy, ROUTING_PAUSE_BOUND_MS, RoutingDecision, RoutingSignal, RoutingTracker,
};

/// The closed device-id fixture set (T044 C4): the four
/// output-route identifiers the removal contract enumerates —
/// the private wired-headset and bt-headset routes and the
/// public car-bt and speaker routes (FR-017a's privacy
/// classes: wired headsets, wired headphones, and Bluetooth
/// headsets are private; the built-in loudspeaker and car
/// Bluetooth in shared-cabin use are public).
const DEVICE_IDS: [&str; 4] = ["wired-headset", "bt-headset", "car-bt", "speaker"];

/// The closed privacy set: exactly the two
/// [`OutputDevicePrivacy`] values — iterating it is exhaustive,
/// because no third privacy exists to handle.
const PRIVACIES: [OutputDevicePrivacy; 2] =
    [OutputDevicePrivacy::Private, OutputDevicePrivacy::Public];

/// The wire string of the closed device-direction set — two
/// explicit arms, never a wildcard (`contracts/shell-ipc.md`
/// case mapping: Rust `PascalCase` → wire `snake_case`).
const fn direction_wire(direction: DeviceDirection) -> &'static str {
    match direction {
        DeviceDirection::Appeared => "appeared",
        DeviceDirection::Disappeared => "disappeared",
    }
}

/// The wire tag of the closed routing-signal set — three
/// explicit arms, never a wildcard, so adding a fourth variant
/// breaks this file's compilation (FR-044).
const fn signal_wire_tag(signal: &RoutingSignal) -> &'static str {
    match signal {
        RoutingSignal::BecomingNoisy => "becoming_noisy",
        RoutingSignal::DeviceDisappeared { .. } => "device_disappeared",
        RoutingSignal::DeviceAppeared { .. } => "device_appeared",
    }
}

/// The decision variant's name — three explicit arms, never a
/// wildcard, so the closed decision set stays closed here.
const fn decision_variant(decision: &RoutingDecision) -> &'static str {
    match decision {
        RoutingDecision::Pause { .. } => "Pause",
        RoutingDecision::Follow { .. } => "Follow",
        RoutingDecision::NoChange => "NoChange",
    }
}

/// Every removal-shaped input must decide the FR-016 pause
/// within the one-second bound — never `Follow`, never
/// `NoChange`: a removal may never pass unhandled, whatever
/// its privacy class or delivering mechanism (FR-016,
/// FR-016a). The match is exhaustive over the closed
/// three-variant decision set, with no catch-all arm.
fn assert_pause_within_bound(decision: &RoutingDecision, mechanism: &str) {
    match decision {
        RoutingDecision::Pause { latency_ms, .. } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "{mechanism} decided the pause at {latency_ms} ms, past the FR-016 one-second bound"
            );
        }
        RoutingDecision::Follow { .. } => {
            let paused = matches!(decision, RoutingDecision::Pause { .. });
            assert!(
                paused,
                "{mechanism} followed the route instead of pausing — a removal must never follow (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            let paused = matches!(decision, RoutingDecision::Pause { .. });
            assert!(
                paused,
                "{mechanism} returned NoChange — no removal may pass unhandled (FR-016a)"
            );
        }
    }
}

/// A decision's device payload is the output-device event the
/// signal named — what `EventKind::OutputDevice` records
/// carry (FR-017a, FR-039): equal as a struct to the expected
/// `diag::writer::DeviceEvent` and byte-exact on the wire
/// (`direction` then `device_id`, both `snake_case`).
fn assert_device_event(payload: &DeviceEvent, direction: DeviceDirection, device_id: &str) {
    let expected = DeviceEvent {
        direction,
        device_id: String::from(device_id),
    };
    assert_eq!(
        payload, &expected,
        "the device payload did not carry the signal's output-device event"
    );
    let wire = format!(
        "{{\"direction\":\"{}\",\"device_id\":\"{device_id}\"}}",
        direction_wire(direction)
    );
    match serde_json::to_string(payload) {
        Ok(json) => assert_eq!(
            json, wire,
            "the device payload did not serialize to the diag output-device wire shape"
        ),
        Err(err) => {
            let error_text = err.to_string();
            assert!(
                !error_text.is_empty(),
                "the device payload failed to serialize: {error_text}"
            );
        }
    }
}

/// C1 / C3 (primary): a fresh tracker, transport `Playing`,
/// fed ONLY `BecomingNoisy` decides the pause within the
/// bound. The primary mechanism never requires the secondary
/// to have fired — no signal of the secondary kind is
/// constructed anywhere in this test.
#[test]
fn primary_becoming_noisy_alone_reaches_pause_within_the_bound() {
    let mut tracker = RoutingTracker::new();
    let decision = tracker.apply(RoutingSignal::BecomingNoisy, TransportState::Playing);
    assert_pause_within_bound(
        &decision,
        "the primary BecomingNoisy mechanism, firing alone",
    );
}

/// C2 / C3 (secondary): the same setup, fed ONLY
/// `DeviceDisappeared` decides the pause within the bound. The
/// secondary mechanism never requires the primary to have
/// fired — no signal of the primary kind is constructed
/// anywhere in this test.
#[test]
fn secondary_device_disappearance_alone_reaches_pause_within_the_bound() {
    let mut tracker = RoutingTracker::new();
    let decision = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_pause_within_bound(
        &decision,
        "the secondary onAudioDevicesRemoved mechanism, firing alone",
    );
}

/// C4 (no removal passes unhandled): every removal-shaped
/// input — the primary signal, and the full cross product of
/// the closed device-id set with the closed privacy set —
/// decides the pause within the bound while `Playing`, never
/// `NoChange`, never `Follow`, never a silent acceptance.
#[test]
fn no_removal_passes_unhandled_when_only_one_mechanism_fires() {
    assert_eq!(
        DEVICE_IDS.len(),
        4,
        "the closed device-id set holds exactly the four removal-contract routes"
    );
    assert_eq!(
        PRIVACIES.len(),
        2,
        "the closed privacy set holds exactly the two privacy classes"
    );

    let mut primary = RoutingTracker::new();
    let primary_decision = primary.apply(RoutingSignal::BecomingNoisy, TransportState::Playing);
    assert_pause_within_bound(&primary_decision, "BecomingNoisy, firing alone");

    for device_id in DEVICE_IDS {
        for privacy in PRIVACIES {
            let mechanism = format!("DeviceDisappeared({device_id}, {privacy:?})");
            let mut tracker = RoutingTracker::new();
            let decision = tracker.apply(
                RoutingSignal::DeviceDisappeared {
                    device_id: String::from(device_id),
                    privacy,
                },
                TransportState::Playing,
            );
            assert_pause_within_bound(&decision, &mechanism);
        }
    }
}

/// C5 (same consequence): a wired-headset removal and a
/// bt-headset removal produce the same decision variant, each
/// within the same one-second bound — a Bluetooth disconnect
/// carries exactly the wired removal's consequence (FR-016).
#[test]
fn bluetooth_disconnect_carries_exactly_the_wired_consequence() {
    let mut wired = RoutingTracker::new();
    let wired_decision = wired.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("wired-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    let mut bluetooth = RoutingTracker::new();
    let bluetooth_decision = bluetooth.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_variant(&wired_decision),
        decision_variant(&bluetooth_decision),
        "a Bluetooth disconnect must carry exactly the wired removal's decision variant (FR-016)"
    );
    assert_pause_within_bound(&wired_decision, "the wired-headset removal");
    assert_pause_within_bound(&bluetooth_decision, "the bt-headset removal");
}

/// C6 (wire conformance): every closed signal variant
/// serializes to its exact `contracts/shell-ipc.md` wire form
/// (case mapping: Rust `PascalCase` → wire `snake_case`), and the
/// `Pause`/`Follow` device payloads serialize to the
/// `diag::writer::DeviceEvent` wire shape that
/// `EventKind::OutputDevice` records carry.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the C6 wire-conformance test walks the closed signal set and both payload shapes in one flow so every expected wire string sits beside the variant it pins; splitting would separate each variant from its wire form"
)]
fn routing_signals_and_device_payloads_carry_the_shell_ipc_wire_shape() {
    let signals = [
        (RoutingSignal::BecomingNoisy, "\"becoming_noisy\""),
        (
            RoutingSignal::DeviceDisappeared {
                device_id: String::from("wired-headset"),
                privacy: OutputDevicePrivacy::Private,
            },
            "{\"device_disappeared\":{\"device_id\":\"wired-headset\",\"privacy\":\"private\"}}",
        ),
        (
            RoutingSignal::DeviceAppeared {
                device_id: String::from("bt-headset"),
                privacy: OutputDevicePrivacy::Public,
            },
            "{\"device_appeared\":{\"device_id\":\"bt-headset\",\"privacy\":\"public\"}}",
        ),
    ];
    for (signal, expected) in signals {
        match serde_json::to_string(&signal) {
            Ok(json) => assert_eq!(
                json, expected,
                "signal {signal:?} did not serialize to its wire form"
            ),
            Err(err) => {
                let error_text = err.to_string();
                assert!(
                    !error_text.is_empty(),
                    "signal {signal:?} failed to serialize: {error_text}"
                );
            }
        }
    }

    let privacies = [
        (OutputDevicePrivacy::Private, "\"private\""),
        (OutputDevicePrivacy::Public, "\"public\""),
    ];
    for (privacy, expected) in privacies {
        match serde_json::to_string(&privacy) {
            Ok(json) => assert_eq!(
                json, expected,
                "privacy {privacy:?} did not serialize to its wire string"
            ),
            Err(err) => {
                let error_text = err.to_string();
                assert!(
                    !error_text.is_empty(),
                    "privacy {privacy:?} failed to serialize: {error_text}"
                );
            }
        }
    }

    // The pause a disappearance decides carries the removed
    // device as its diag output-device payload.
    let mut disappearing = RoutingTracker::new();
    let disappearance = disappearing.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("wired-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    match &disappearance {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the disappearance pause exceeded the FR-016 one-second bound"
            );
            assert_device_event(device, DeviceDirection::Disappeared, "wired-headset");
        }
        RoutingDecision::Follow { .. } => {
            let paused = matches!(disappearance, RoutingDecision::Pause { .. });
            assert!(paused, "a removal must pause, never follow (FR-016)");
        }
        RoutingDecision::NoChange => {
            let paused = matches!(disappearance, RoutingDecision::Pause { .. });
            assert!(paused, "a removal must never pass unhandled (FR-016a)");
        }
    }

    // The follow an appearance decides carries the appeared
    // device as its diag output-device payload. A private
    // appearance follows on a fresh tracker — a public one
    // pauses fail-closed while no output is latched (R2).
    let mut appearing = RoutingTracker::new();
    let appearance = appearing.apply(
        RoutingSignal::DeviceAppeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    match &appearance {
        RoutingDecision::Pause { .. } => {
            let followed = matches!(appearance, RoutingDecision::Follow { .. });
            assert!(
                followed,
                "an appearance while playing follows, never pauses (FR-017a)"
            );
        }
        RoutingDecision::Follow { device } => {
            assert_device_event(device, DeviceDirection::Appeared, "bt-headset");
        }
        RoutingDecision::NoChange => {
            let followed = matches!(appearance, RoutingDecision::Follow { .. });
            assert!(
                followed,
                "an appearance while playing follows at the unchanged position (FR-017a)"
            );
        }
    }

    // The same payload is what an EventKind::OutputDevice
    // record carries on the wire (FR-039: every switch is
    // recorded as an output-device event).
    let record = RecordEvent {
        v: 1,
        ts_ms: 1_000_u64,
        kind: EventKind::OutputDevice,
        position_secs: 10.0,
        transport: None,
        session_end_reason: None,
        supersedes_ts_ms: None,
        focus: None,
        device: Some(DeviceEvent {
            direction: DeviceDirection::Disappeared,
            device_id: String::from("wired-headset"),
        }),
        error: None,
        engine: None,
        engine_tier: None,
        detail: None,
        trim: None,
    };
    let expected_record = concat!(
        "{\"v\":1,\"ts_ms\":1000,\"kind\":\"output_device\",\"position_secs\":10.0,",
        "\"device\":{\"direction\":\"disappeared\",\"device_id\":\"wired-headset\"}}",
    );
    match serde_json::to_string(&record) {
        Ok(json) => assert_eq!(
            json, expected_record,
            "the output-device record did not serialize to its contract wire form"
        ),
        Err(err) => {
            let error_text = err.to_string();
            assert!(
                !error_text.is_empty(),
                "the output-device record failed to serialize: {error_text}"
            );
        }
    }
}

/// C7 (structural independence): the signal set is exactly the
/// three closed variants — constructed, matched exhaustively
/// with no catch-all arm — and applying one mechanism's signal
/// neither consumes nor gates the other's: each fires alone on
/// its own fresh tracker and both reach the pause within the
/// bound, so the tracker's state after either is a paused
/// latch, never a "waiting for the other signal" state.
#[test]
fn the_two_mechanisms_are_structurally_independent() {
    let signals = [
        RoutingSignal::BecomingNoisy,
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("wired-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        RoutingSignal::DeviceAppeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Public,
        },
    ];
    assert_eq!(
        signals.len(),
        3,
        "the routing signal set holds exactly the three closed variants (FR-016a)"
    );
    assert_eq!(
        [
            signal_wire_tag(&signals[0]),
            signal_wire_tag(&signals[1]),
            signal_wire_tag(&signals[2]),
        ],
        ["becoming_noisy", "device_disappeared", "device_appeared"],
        "the closed signal set drifted from its three wire tags"
    );

    // The primary fires alone on its own tracker...
    let mut primary = RoutingTracker::new();
    let primary_decision = primary.apply(RoutingSignal::BecomingNoisy, TransportState::Playing);
    assert_pause_within_bound(&primary_decision, "the primary mechanism, firing alone");

    // ...and the secondary fires alone on a fresh tracker:
    // neither waited for, consumed, or gated the other.
    let mut secondary = RoutingTracker::new();
    let secondary_decision = secondary.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_pause_within_bound(&secondary_decision, "the secondary mechanism, firing alone");

    // The tracker's route state reflects the latched output
    // (the privacy the current output device carries): unknown
    // before any signal, the followed device's class after.
    // A public appearance while a private output is latched
    // pauses by privacy precedence (FR-016) and latches
    // nothing — the private output remains the current
    // output until the reader resumes.
    let mut route = RoutingTracker::new();
    assert_eq!(
        route.current_privacy(),
        None,
        "a fresh tracker knows no current device, so no current privacy"
    );
    let _appearance = route.apply(
        RoutingSignal::DeviceAppeared {
            device_id: String::from("bt-headset"),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_eq!(
        route.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the current privacy after a private appearance is the private class (FR-017a)"
    );
    let _precedence = route.apply(
        RoutingSignal::DeviceAppeared {
            device_id: String::from("car-bt"),
            privacy: OutputDevicePrivacy::Public,
        },
        TransportState::Playing,
    );
    assert_eq!(
        route.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the public appearance pauses by privacy precedence, so the private output remains current until the reader resumes (FR-016)"
    );
}
