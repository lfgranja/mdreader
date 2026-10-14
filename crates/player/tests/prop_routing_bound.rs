//! T024 — US2 property: the routing pause bound (FR-006 / FR-016a).
//!
//! `specs/004-ressoa-dogfood-build/tasks.md` T024, verbatim:
//! "Property `prop_routing_pause_bound` in
//! `crates/player/tests/prop_routing.rs` (invariants:
//! `BecomingNoisy`/`DeviceDisappeared` ⇒ pause decision within
//! normative ≤1000 ms per FR-006 (gating acceptance); 250 ms is an
//! informative local engineering target mirroring plan.md Performance
//! Goals, non-gating, logged only; never routes to speaker)".
//!
//! This file is the T024 BLOCKING test for the feature-004 dogfood
//! wave. The feature-001 direction suite (`tests/prop_routing.rs`,
//! T042) already owns the R1–R6 behavior of the routing machine —
//! the latch rules, the record events, the position immutability —
//! so this file binds ONLY to the three invariants T024 adds:
//!
//! 1. every `BecomingNoisy` / `DeviceDisappeared` signal reaching the
//!    machine while playing yields a [`RoutingDecision::Pause`] —
//!    never `Follow`, never `NoChange` — whose declared latency is
//!    inside the normative one-second bound of FR-006 (the gating
//!    acceptance half);
//! 2. that pause is never a route onto a public output: a removal
//!    never follows, and an appearance claiming the public class
//!    never follows out of a private (or not-yet-known) output
//!    (SC-002 — audio never moves to the loudspeaker unattended);
//! 3. the 250 ms figure is the informative local engineering target
//!    of `plan.md` Performance Goals ("Declared implementation
//!    target, informative and non-gating: routing decision 250 ms
//!    within the normative ≤1000 ms FR-006 bound") — recorded, never
//!    asserted, and never an acceptance gate.
//!
//! Timing model (documented, per the task): the routing machine is a
//! synchronous, pure state machine — the decision is computed from
//! the signal and the transport state with no clock read — so the
//! pause latency is a declared value carried on the decision
//! (`RoutingDecision::Pause { latency_ms }`) rather than a
//! measurement. At the pure-function level the ≤1000 ms bound is
//! therefore decidable and always satisfiable, and this file asserts
//! it on every removal of every generated sequence. The wall-clock
//! ≤1 s half (the surface acting on the decision within one second of
//! the removal being observable, per FR-016a) is attested on-device —
//! T051, 10/10 per interruption class including the "headphone ≤1 s"
//! leg — and is not decidable on the host.
//!
//! Surfaces under test (public only): `player::routing` —
//! `RoutingSignal` (3), `RoutingDecision` (3), `RoutingTracker`,
//! `OutputDevicePrivacy` (2), `ROUTING_PAUSE_BOUND_MS`.
//!
//! Note on file choice: tasks.md names `prop_routing.rs` as the T024
//! home. That path already carries the feature-001 T042 suite, and
//! the same-file serialize guard forbids batching same-file work into
//! one wave, so T024 lands in this sibling file; the two-way diff of
//! `scripts/expected-tests.txt` (verify.sh gate 2) must pin the new
//! `player::prop_routing_bound` target when this wave's gate is wired.
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest builds the message through `concat!`, and
//! rustc rejects implicit captures in macro-expanded format strings.

use proptest::prelude::*;

use diag::writer::DeviceDirection;
use engine_core::closed::TransportState;
use player::routing::ROUTING_PAUSE_BOUND_MS as SOURCE_ROUTING_PAUSE_BOUND_MS;
use player::routing::{OutputDevicePrivacy, RoutingDecision, RoutingSignal, RoutingTracker};

/// The normative FR-006 pause bound, in milliseconds: a device
/// removal MUST reach the pause within one second so audio never
/// moves to the loudspeaker. This is the GATING figure of T024
/// (SC-002), asserted on every removal of every generated sequence.
const ROUTING_PAUSE_BOUND_MS: u128 = 1_000;

/// The informative local engineering target, in milliseconds,
/// mirroring the plan.md Performance Goals ("routing decision 250 ms
/// within the normative ≤1000 ms FR-006 bound").
///
/// NON-GATING: this figure is recorded (logged) against every
/// measured pause and never asserted — only
/// [`ROUTING_PAUSE_BOUND_MS`] gates acceptance. It is kept as a
/// named constant so the engineering target is visible in the code
/// instead of living only in prose.
const ROUTING_LOCAL_TARGET_MS: u128 = 250;

/// The closed device-identifier pool (FR-016a / FR-017a).
fn device_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("wired".to_string()),
        Just("bt-headset".to_string()),
        Just("car-bt".to_string()),
        Just("speaker".to_string()),
    ]
}

/// Either output class, so generated signals carry matched AND
/// mismatched privacy claims (a "wired" id claiming `Public`, a
/// "speaker" id claiming `Private`): the machine enforces the
/// CLAIMED class, never the identifier (R2 of the feature-001
/// direction suite).
fn privacy_strategy() -> impl Strategy<Value = OutputDevicePrivacy> {
    prop_oneof![
        Just(OutputDevicePrivacy::Private),
        Just(OutputDevicePrivacy::Public),
    ]
}

/// A removal signal of either named FR-016a mechanism: the primary
/// `ACTION_AUDIO_BECOMING_NOISY` and the secondary
/// `AudioDeviceCallback.onAudioDevicesRemoved` — each sufficient on
/// its own, neither waiting for the other.
fn removal_signal_strategy() -> impl Strategy<Value = RoutingSignal> {
    prop_oneof![
        Just(RoutingSignal::BecomingNoisy),
        (device_id_strategy(), privacy_strategy()).prop_map(|(device_id, privacy)| {
            RoutingSignal::DeviceDisappeared { device_id, privacy }
        }),
    ]
}

/// An appearance signal with an arbitrary claimed class.
fn appeared_signal_strategy() -> impl Strategy<Value = RoutingSignal> {
    (device_id_strategy(), privacy_strategy())
        .prop_map(|(device_id, privacy)| RoutingSignal::DeviceAppeared { device_id, privacy })
}

/// The closed routing-signal set (FR-016a / FR-017a) — three explicit
/// arms, no catch-all: a fourth signal would need a strategy arm
/// here before anything can compile.
fn routing_signal_strategy() -> impl Strategy<Value = RoutingSignal> {
    prop_oneof![
        Just(RoutingSignal::BecomingNoisy),
        (device_id_strategy(), privacy_strategy()).prop_map(|(device_id, privacy)| {
            RoutingSignal::DeviceDisappeared { device_id, privacy }
        }),
        appeared_signal_strategy(),
    ]
}

/// Generated sequences that always contain at least one removal —
/// the FR-016a pause is the invariant under test, so every case
/// must exercise it (or an earlier pause that already did).
fn pause_bound_signal_vec_strategy() -> impl Strategy<Value = Vec<RoutingSignal>> {
    (
        proptest::collection::vec(routing_signal_strategy(), 0..=8),
        removal_signal_strategy(),
        proptest::collection::vec(routing_signal_strategy(), 0..=8),
    )
        .prop_map(|(prefix, removal, suffix)| {
            let mut signals = prefix;
            signals.push(removal);
            signals.extend(suffix);
            signals
        })
}

/// The closed three-member routing-decision kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecisionKind {
    /// The FR-016 pause.
    Pause,
    /// The FR-017a follow.
    Follow,
    /// The FR-017 / R3 no-op.
    NoChange,
}

/// The decision kind — three explicit arms, no catch-all.
fn decision_kind(decision: &RoutingDecision) -> DecisionKind {
    match decision {
        RoutingDecision::Pause { .. } => DecisionKind::Pause,
        RoutingDecision::Follow { .. } => DecisionKind::Follow,
        RoutingDecision::NoChange => DecisionKind::NoChange,
    }
}

/// Whether a signal is one of the two named FR-016a removal
/// mechanisms.
fn is_removal_signal(signal: &RoutingSignal) -> bool {
    matches!(
        signal,
        RoutingSignal::BecomingNoisy | RoutingSignal::DeviceDisappeared { .. }
    )
}

/// Whether an appearance claims the public output class (the
/// loudspeaker or car Bluetooth in shared-cabin use).
fn claims_public(signal: &RoutingSignal) -> bool {
    matches!(
        signal,
        RoutingSignal::DeviceAppeared {
            privacy: OutputDevicePrivacy::Public,
            ..
        }
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// T024 `prop_routing_pause_bound`: every pause the machine
    /// reaches while playing — a removal by either named FR-016a
    /// mechanism, or a private→public switch by privacy precedence —
    /// carries a declared latency inside the normative ≤1000 ms
    /// FR-006 bound (gating); a removal never yields a follow; the
    /// 250 ms local target is recorded only (non-gating); and no
    /// decision ever routes audio onto a public output out of a
    /// private one.
    #[test]
    fn prop_routing_pause_bound(signals in pause_bound_signal_vec_strategy()) {
        let mut tracker = RoutingTracker::new();
        // Open the session the way a reader connects a wired
        // headset: the private route is latched and playback runs.
        let opening = tracker.apply(
            RoutingSignal::DeviceAppeared {
                device_id: "wired".to_string(),
                privacy: OutputDevicePrivacy::Private,
            },
            TransportState::Playing,
        );
        prop_assert_eq!(
            decision_kind(&opening),
            DecisionKind::Follow,
            "the opening appearance must follow the private headset (FR-017a)"
        );
        let mut transport = TransportState::Playing;
        let mut measured_pauses = 0u64;
        for signal in signals {
            let privacy_before = tracker.current_privacy();
            let decision = tracker.apply(signal.clone(), transport);
            let kind = decision_kind(&decision);

            // FR-006 (gating): EVERY pause the machine reaches while
            // playing — a removal by either named mechanism, or a
            // private→public switch by privacy precedence — carries a
            // declared latency inside the normative one-second bound.
            if let RoutingDecision::Pause { latency_ms, device } = &decision {
                prop_assert!(
                    u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
                    "the pause declared {} ms, over the normative {} ms FR-006 bound (gating)",
                    latency_ms,
                    ROUTING_PAUSE_BOUND_MS
                );
                if is_removal_signal(&signal) {
                    prop_assert_eq!(
                        device.direction,
                        DeviceDirection::Disappeared,
                        "the removal pause must record a disappearance (R5)"
                    );
                }
                // Informative, non-gating (plan.md Performance Goals):
                // the local engineering target is recorded against the
                // measured pause and never asserted.
                tracing::debug!(
                    latency_ms = *latency_ms,
                    "routing pause latency recorded; the {} ms local engineering target is informative and non-gating, only the {} ms FR-006 bound gates",
                    ROUTING_LOCAL_TARGET_MS,
                    ROUTING_PAUSE_BOUND_MS
                );
                if transport == TransportState::Playing {
                    measured_pauses += 1;
                }
            }

            if is_removal_signal(&signal) {
                if transport == TransportState::Playing {
                    // FR-006 (gating): a removal pauses — never a
                    // follow onto another output, never a no-op that
                    // leaves audio running.
                    prop_assert_eq!(
                        kind,
                        DecisionKind::Pause,
                        "the removal signal {:?} did not pause (FR-006 / FR-016a): the decision was {:?}",
                        signal,
                        decision
                    );
                } else {
                    // FR-017: while not playing, a removal is a no-op
                    // — never an auto-resume, never a transport change.
                    prop_assert_eq!(
                        kind,
                        DecisionKind::NoChange,
                        "the removal signal {:?} while not playing must be a no-op (FR-017): the decision was {:?}",
                        signal,
                        decision
                    );
                }
            }
            // Never routes to speaker (SC-002): a follow is never a
            // switch onto a public output out of a private — or
            // not-yet-known — output. Where the latch is already
            // public, FR-017a follows a public→public switch by rule;
            // the privacy guarantee concerns leaving a private output.
            let left_private_or_unknown =
                matches!(privacy_before, Some(OutputDevicePrivacy::Private) | None);
            prop_assert!(
                !(kind == DecisionKind::Follow && claims_public(&signal) && left_private_or_unknown),
                "a public appearance was followed out of a {:?} output — audio would move to the loudspeaker (SC-002): the decision was {:?}",
                privacy_before,
                decision
            );
            if kind == DecisionKind::Pause {
                transport = TransportState::Paused;
            }
        }
        prop_assert!(
            measured_pauses >= 1,
            "the generated sequence never reached a pause decision — the FR-006 bound was never exercised"
        );
        prop_assert_eq!(
            transport,
            TransportState::Paused,
            "a sequence containing a removal must end paused — audio never plays on past a removal (FR-006)"
        );
    }

    /// Either named FR-016a mechanism alone is sufficient (neither
    /// waits for the other): a `BecomingNoisy`-only stream and a
    /// `DeviceDisappeared`-only stream both reach the pause within
    /// the normative bound.
    #[test]
    fn either_removal_mechanism_alone_pauses_within_the_bound(
        mechanism in prop_oneof![
            Just("becoming_noisy"),
            Just("device_disappeared"),
        ]
    ) {
        let mut tracker = RoutingTracker::new();
        let mut transport = TransportState::Playing;
        let mut measured = 0u64;
        for round in 0..8u64 {
            let signal = if mechanism == "becoming_noisy" {
                RoutingSignal::BecomingNoisy
            } else {
                RoutingSignal::DeviceDisappeared {
                    device_id: format!("bt-headset-{round}"),
                    privacy: OutputDevicePrivacy::Private,
                }
            };
            let decision = tracker.apply(signal, transport);
            if transport == TransportState::Playing {
                prop_assert_eq!(
                    decision_kind(&decision),
                    DecisionKind::Pause,
                    "the {} mechanism did not pause in round {} (FR-016a)",
                    mechanism,
                    round
                );
                if let RoutingDecision::Pause { latency_ms, .. } = &decision {
                    prop_assert!(
                        u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
                        "the {} mechanism declared {} ms in round {}, over the {} ms bound",
                        mechanism,
                        latency_ms,
                        round,
                        ROUTING_PAUSE_BOUND_MS
                    );
                }
                measured += 1;
                transport = TransportState::Paused;
            }
        }
        prop_assert_eq!(
            measured, 1,
            "the removal must pause exactly once per playing span; {} pauses were measured",
            measured
        );
    }
}

/// A wired pull (FR-006): while playing through a private headset,
/// `DeviceDisappeared` pauses within the normative one-second bound,
/// records the removed route, and never follows onto another output.
#[test]
fn device_disappeared_pauses_within_the_normative_bound() {
    let mut tracker = RoutingTracker::new();
    let opening = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_kind(&opening),
        DecisionKind::Follow,
        "the opening appearance must follow the private headset (FR-017a)"
    );
    let decision = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::Pause,
        "a wired pull must pause (FR-006): the decision was {:?}",
        decision
    );
    if let RoutingDecision::Pause { latency_ms, device } = &decision {
        assert!(
            u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
            "the wired pull paused in {} ms, over the normative {} ms FR-006 bound (gating)",
            latency_ms,
            ROUTING_PAUSE_BOUND_MS
        );
        assert_eq!(
            device.direction,
            DeviceDirection::Disappeared,
            "the wired pull records a disappearance (R5)"
        );
        assert_eq!(
            device.device_id, "wired",
            "the wired pull records the removed route (R5)"
        );
    }
    assert_eq!(
        tracker.current_privacy(),
        None,
        "the removed wired headset is no longer the current output (R6)"
    );
}

/// The route itself going noisy (FR-016a primary): `BecomingNoisy`
/// pauses within the normative one-second bound and names the route
/// that went noisy, before any output device is latched.
#[test]
fn becoming_noisy_pauses_within_the_normative_bound() {
    let mut tracker = RoutingTracker::new();
    let decision = tracker.apply(RoutingSignal::BecomingNoisy, TransportState::Playing);
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::Pause,
        "a BecomingNoisy signal must pause (FR-006 / FR-016a): the decision was {:?}",
        decision
    );
    if let RoutingDecision::Pause { latency_ms, device } = &decision {
        assert!(
            u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
            "the noisy route paused in {} ms, over the normative {} ms FR-006 bound (gating)",
            latency_ms,
            ROUTING_PAUSE_BOUND_MS
        );
        assert_eq!(
            device.direction,
            DeviceDirection::Disappeared,
            "the noisy route records a disappearance (R5)"
        );
    }
}

/// The figures match FR-006 and the plan's own declaration verbatim:
/// a normative 1000 ms bound (which the crate's declared constant
/// must agree with) and an informative 250 ms local target that sits
/// inside it.
#[test]
fn the_bounds_match_the_fr006_figures_verbatim() {
    assert_eq!(
        ROUTING_PAUSE_BOUND_MS, 1_000,
        "the normative one-second bound drifted from 1000 ms (FR-006 / SC-002)"
    );
    assert_eq!(
        u128::from(SOURCE_ROUTING_PAUSE_BOUND_MS),
        ROUTING_PAUSE_BOUND_MS,
        "the crate's declared ROUTING_PAUSE_BOUND_MS drifted from the normative figure"
    );
    assert_eq!(
        SOURCE_ROUTING_PAUSE_BOUND_MS, 1_000,
        "the crate's declared bound drifted from 1000 ms (FR-006 / SC-002)"
    );
    assert!(
        ROUTING_LOCAL_TARGET_MS <= ROUTING_PAUSE_BOUND_MS,
        "the informative local engineering target {} ms must sit inside the normative {} ms bound",
        ROUTING_LOCAL_TARGET_MS,
        ROUTING_PAUSE_BOUND_MS
    );
}

/// Never routes to speaker (SC-002): while playing through a private
/// output — or before any output is latched — an appearance claiming
/// the public class pauses first and is never followed. Audio never
/// moves to the loudspeaker unattended.
#[test]
fn a_public_appearance_is_never_followed_out_of_a_private_output() {
    let mut tracker = RoutingTracker::new();
    // Fail-closed while no output is latched: an unknown output is
    // treated as private, because the privacy guarantee is the point
    // of the bound (R2 of the feature-001 direction suite).
    let unlatched = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_kind(&unlatched),
        DecisionKind::Pause,
        "a public appearance with no latched output must pause fail-closed (SC-002)"
    );
    if let RoutingDecision::Pause { latency_ms, .. } = &unlatched {
        assert!(
            u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
            "the fail-closed pause declared {} ms, over the normative {} ms bound",
            latency_ms,
            ROUTING_PAUSE_BOUND_MS
        );
    }
    let mut private_tracker = RoutingTracker::new();
    let latched = private_tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Paused,
    );
    assert_eq!(
        decision_kind(&latched),
        DecisionKind::NoChange,
        "an appearance while not playing is a no-op that latches the route (FR-017)"
    );
    let switch = private_tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_kind(&switch),
        DecisionKind::Pause,
        "a private→public switch must pause first — never follow onto the loudspeaker (FR-016 precedence, SC-002)"
    );
    if let RoutingDecision::Pause { latency_ms, device } = &switch {
        assert!(
            u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
            "the privacy-precedence pause declared {} ms, over the normative {} ms bound",
            latency_ms,
            ROUTING_PAUSE_BOUND_MS
        );
        assert_eq!(
            device.direction,
            DeviceDirection::Appeared,
            "the privacy-precedence pause records the appearing route (R5)"
        );
    }
    assert_eq!(
        private_tracker.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the private output stays latched until the reader resumes (FR-016)"
    );
}

/// The routing machine is a pure state machine: the same signal at
/// the same transport state from the same latch decides the same way
/// on every call, because no clock is read. The pause latency is
/// therefore a declared value rather than a measurement, the
/// ≤1000 ms bound is decidable here, and the wall-clock ≤1 s half of
/// FR-006 is attested on-device (T051, the "headphone ≤1 s" leg of
/// the 10/10 per interruption class).
#[test]
fn the_routing_decision_is_pure_so_the_bound_is_decidable_without_a_clock() {
    let mut first = RoutingTracker::new();
    let mut second = RoutingTracker::new();
    let signal = RoutingSignal::BecomingNoisy;
    let first_decision = first.apply(signal.clone(), TransportState::Playing);
    let second_decision = second.apply(signal, TransportState::Playing);
    assert_eq!(
        first_decision, second_decision,
        "the routing decision must be a pure function of the signal and the transport state — a differing answer would mean a clock is read and the bound is not decidable"
    );
    if let RoutingDecision::Pause { latency_ms, .. } = &first_decision {
        assert!(
            u128::from(*latency_ms) <= ROUTING_PAUSE_BOUND_MS,
            "the declared pause latency {} ms must sit inside the normative {} ms bound",
            latency_ms,
            ROUTING_PAUSE_BOUND_MS
        );
    }
}
