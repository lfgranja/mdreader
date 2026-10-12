//! T042 — US3 property: routing direction invariants
//! (FR-016 / FR-016a / FR-017 / FR-017a).
//!
//! tasks.md T042, verbatim: "Property test: routing
//! direction invariants in `crates/player/tests/prop_routing.rs`
//! (proptest: `DeviceDisappeared` → `Paused` with position
//! preserved, ≤1 s bound recorded either signal alone
//! sufficient; `DeviceAppeared` while playing → same
//! `position_secs`, no pause/restart/repeat; private→public
//! move ALWAYS pauses first even with simultaneous
//! appearance; each switch emits `output_device` record
//! event; FAIL before implementation)".
//!
//! Spec anchors (`specs/001-android-media-shell/spec.md`):
//! - FR-016: pause within one second when headphones, a
//!   wired headset, or a connected Bluetooth A2DP device
//!   are disconnected, so audio does not move to the
//!   loudspeaker. A Bluetooth disconnect carries exactly
//!   the same consequence as a wired removal: pause with
//!   the stored position preserved. Where a disappearance
//!   and an appearance coincide in one route change, the
//!   disappearance takes precedence: pause first, then
//!   re-evaluate the newly available device under FR-017a
//!   before resuming.
//! - FR-016a: the one-second pause is reached through TWO
//!   named mechanisms, each sufficient on its own —
//!   `ACTION_AUDIO_BECOMING_NOISY` (primary) and
//!   `AudioDeviceCallback.onAudioDevicesRemoved`
//!   (secondary). Each mechanism MUST reach the pause
//!   within one second of the removal being observable;
//!   neither waits for the other; no removal may pass
//!   unhandled because the platform delivered only one of
//!   them.
//! - FR-017: resume through a reconnected output device
//!   only when the user explicitly resumes.
//! - FR-017a: a new output device available while audio is
//!   already playing is followed automatically at the
//!   unchanged position — no pause, restart, or repeated
//!   phrase — EXCEPT a switch out of a private output to
//!   a public one, which still pauses first (privacy
//!   precedence). Private outputs: wired headsets, wired
//!   headphones, Bluetooth headsets (A2DP/SCO worn by the
//!   reader). Public outputs: built-in loudspeaker, car
//!   Bluetooth in shared-cabin use. The switch MUST be
//!   recorded as an output-device event (`output_device`,
//!   `diag::writer::DeviceEvent`) so a route change is
//!   distinguishable from a stall or a failure afterwards.
//!
//! Reference model: `RoutingTracker::apply` is checked
//! against an independent reference function
//! (`reference_decision` / `reference_latch_after`) over
//! arbitrary signal sequences with interleaved audible
//! ticks, driven through a simulated session (`SimSession`)
//! holding the behavioral rules R1–R6: removals and
//! `BecomingNoisy` pause within `ROUTING_PAUSE_BOUND_MS`
//! while playing (each mechanism alone sufficient);
//! appearances follow at the unchanged position unless the
//! move leaves a private output; every signal while not
//! playing is a no-op (never an auto-resume); no decision
//! moves the position; every switch records a
//! `DeviceEvent` whose direction and `device_id` echo the
//! signal; and `current_privacy()` latches exactly as the
//! model does.
//!
//! Trial legs: T043 (`tests/integration/test_routing_us3.rs`)
//! drives the platform trials (noisy + device-callback,
//! reconnect + explicit resume, coincide unplug/connect);
//! T044 (`tests/contract/test_removal_signals_us3.rs`) pins
//! the removal-signal contract. This file owns the
//! direction invariants over arbitrary sequences.
//!
//! Format note: every `prop_assert*!` message uses
//! explicit positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`) — proptest 1.11
//! builds the message through `concat!`, and rustc (1.98)
//! rejects implicit captures in macro-expanded format
//! strings (same convention as `prop_focus.rs`).
//!
//! Red state: `player::routing` does not exist yet — this
//! file is the T045 RED gate; it fails to compile against
//! `player::routing` until T046 implements the module.

use proptest::prelude::*;

use diag::writer::DeviceDirection;
use engine_core::closed::TransportState;
use player::routing::{
    OutputDevicePrivacy, ROUTING_PAUSE_BOUND_MS, RoutingDecision, RoutingSignal, RoutingTracker,
};

/// The closed device pool: each identifier with the output
/// class it really belongs to (FR-017a — wired headsets
/// and Bluetooth headsets are private outputs; the
/// loudspeaker and car Bluetooth are public).
const DEVICE_POOL: [(&str, OutputDevicePrivacy); 4] = [
    ("wired", OutputDevicePrivacy::Private),
    ("bt-headset", OutputDevicePrivacy::Private),
    ("car-bt", OutputDevicePrivacy::Public),
    ("speaker", OutputDevicePrivacy::Public),
];

/// The closed device-identifier pool as a strategy.
fn device_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("wired".to_string()),
        Just("bt-headset".to_string()),
        Just("car-bt".to_string()),
        Just("speaker".to_string()),
    ]
}

/// Either output class, so generated signals carry matched
/// AND mismatched claims (a "wired" id claiming `Public`,
/// a "speaker" id claiming `Private`): the tracker must
/// enforce R2 by the claimed privacy, not by the id's
/// real class.
fn privacy_strategy() -> impl Strategy<Value = OutputDevicePrivacy> {
    prop_oneof![
        Just(OutputDevicePrivacy::Private),
        Just(OutputDevicePrivacy::Public),
    ]
}

/// The output class a pool device really belongs to.
fn matched_privacy(device_id: &str) -> OutputDevicePrivacy {
    DEVICE_POOL
        .iter()
        .find(|(id, _)| *id == device_id)
        .map_or(OutputDevicePrivacy::Public, |(_, privacy)| *privacy)
}

/// A removal signal with the device's real class attached.
fn disappeared_matched_strategy() -> impl Strategy<Value = RoutingSignal> {
    device_id_strategy().prop_map(|device_id| RoutingSignal::DeviceDisappeared {
        device_id,
        privacy: matched_privacy(&device_id),
    })
}

/// A removal signal with an arbitrary claimed class.
fn disappeared_strategy() -> impl Strategy<Value = RoutingSignal> {
    (device_id_strategy(), privacy_strategy())
        .prop_map(|(device_id, privacy)| RoutingSignal::DeviceDisappeared { device_id, privacy })
}

/// An appearance signal with an arbitrary claimed class.
fn appeared_strategy() -> impl Strategy<Value = RoutingSignal> {
    (device_id_strategy(), privacy_strategy())
        .prop_map(|(device_id, privacy)| RoutingSignal::DeviceAppeared { device_id, privacy })
}

/// A removal signal of either named mechanism (FR-016a).
fn removal_signal_strategy() -> impl Strategy<Value = RoutingSignal> {
    prop_oneof![Just(RoutingSignal::BecomingNoisy), disappeared_strategy(),]
}

/// An appearance signal claiming the public class — any
/// identifier, matched or mismatched.
fn public_appearance_strategy() -> impl Strategy<Value = RoutingSignal> {
    device_id_strategy().prop_map(|device_id| RoutingSignal::DeviceAppeared {
        device_id,
        privacy: OutputDevicePrivacy::Public,
    })
}

/// The closed routing-signal set (FR-016a / FR-017a).
fn routing_signal_strategy() -> impl Strategy<Value = RoutingSignal> {
    prop_oneof![
        Just(RoutingSignal::BecomingNoisy),
        disappeared_strategy(),
        appeared_strategy(),
    ]
}

/// An audible-progress tick (mirroring prop_lifecycle's
/// `on_audible_progress`): an exact binary fraction, so
/// every position comparison in this file is bit-exact.
fn audible_step_strategy() -> impl Strategy<Value = f64> {
    0.0..=5.0f64
}

/// One simulated step: a routing signal or an audible
/// tick.
#[derive(Clone, Debug)]
enum Step {
    /// Audible progress, in seconds.
    Audible {
        /// The seconds of progress.
        secs: f64,
    },
    /// A routing signal from the platform.
    Signal(RoutingSignal),
}

/// Generated steps, as any surface might receive them.
fn step_strategy() -> impl Strategy<Value = Step> {
    prop_oneof![
        audible_step_strategy().prop_map(|secs| Step::Audible { secs }),
        routing_signal_strategy().prop_map(Step::Signal),
    ]
}

/// Generated step sequences with interleaved ticks.
fn step_vec_strategy() -> impl Strategy<Value = Vec<Step>> {
    proptest::collection::vec(step_strategy(), 1..=48)
}

/// A generated batch guaranteed to hold at least one
/// removal and at least one public appearance — the
/// coincide case of FR-016 (unplugging wired while car
/// Bluetooth connects), in an arbitrary order with
/// arbitrary signals around the pair.
fn coincidence_signal_vec_strategy() -> impl Strategy<Value = Vec<RoutingSignal>> {
    (
        proptest::collection::vec(routing_signal_strategy(), 0..=4),
        removal_signal_strategy(),
        proptest::collection::vec(routing_signal_strategy(), 0..=4),
        public_appearance_strategy(),
        proptest::collection::vec(routing_signal_strategy(), 0..=4),
    )
        .prop_map(|(prefix, removal, middle, appearance, suffix)| {
            let mut signals = prefix;
            signals.push(removal);
            signals.extend(middle);
            signals.push(appearance);
            signals.extend(suffix);
            signals
        })
}

/// The closed non-playing transport states (R3).
fn non_playing_state_strategy() -> impl Strategy<Value = TransportState> {
    prop_oneof![
        Just(TransportState::Paused),
        Just(TransportState::Interrupted),
        Just(TransportState::Stopped),
        Just(TransportState::Completed),
    ]
}

/// The kind of a routing decision — the closed
/// three-member set (FR-044: no `Other`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionKind {
    /// The FR-016 pause.
    Pause,
    /// The FR-017a follow.
    Follow,
    /// The FR-017 / R3 no-op.
    NoChange,
}

/// The decision kind, for kind-level assertions.
fn decision_kind(decision: &RoutingDecision) -> DecisionKind {
    match decision {
        RoutingDecision::Pause { .. } => DecisionKind::Pause,
        RoutingDecision::Follow { .. } => DecisionKind::Follow,
        RoutingDecision::NoChange => DecisionKind::NoChange,
    }
}

/// The output device the simulated session occupies — the
/// routing machine's latch (R6).
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeviceLatch {
    /// The device identifier.
    device_id: String,
    /// The claimed output class.
    privacy: OutputDevicePrivacy,
}

/// The simulated playback session the routing machine
/// drives (R4): the transport state, the position, and
/// the latched output device. Routing decisions only
/// pause or follow — the position never moves; audible
/// ticks advance it only while playing.
#[derive(Clone, Debug)]
struct SimSession {
    /// The transport state.
    transport: TransportState,
    /// The playback position, in seconds.
    position_secs: f64,
    /// The latched output device, or `None` before any
    /// device is latched.
    current: Option<DeviceLatch>,
}

impl SimSession {
    /// A session playing through a freshly latched device.
    fn playing_on(device_id: &str, privacy: OutputDevicePrivacy) -> Self {
        Self {
            transport: TransportState::Playing,
            position_secs: 0.0,
            current: Some(DeviceLatch {
                device_id: device_id.to_string(),
                privacy,
            }),
        }
    }

    /// A session in a given (non-playing) state, still
    /// holding its latched output.
    fn in_state(transport: TransportState) -> Self {
        Self {
            transport,
            position_secs: 0.0,
            current: Some(DeviceLatch {
                device_id: "wired".to_string(),
                privacy: OutputDevicePrivacy::Private,
            }),
        }
    }

    /// Audible progress: advances the position only while
    /// playing (mirroring prop_lifecycle's
    /// `on_audible_progress`).
    fn tick_audible(&mut self, secs: f64) {
        if self.transport == TransportState::Playing {
            self.position_secs += secs;
        }
    }

    /// Apply a routing decision (R4): a pause lands the
    /// transport on `Paused`, a follow keeps playing, a
    /// no-op changes nothing. The position never moves.
    fn apply_decision(&mut self, decision: &RoutingDecision) {
        match decision {
            RoutingDecision::Pause { .. } => {
                self.transport = TransportState::Paused;
            }
            RoutingDecision::Follow { .. } | RoutingDecision::NoChange => {}
        }
    }
}

/// The reference verdict for one signal — the routing
/// invariants of FR-016 / FR-016a / FR-017 / FR-017a
/// computed independently of the implementation under
/// test.
#[derive(Debug)]
enum ReferenceDecision {
    /// FR-016: pause within the one-second bound. The
    /// event names the removed route — for
    /// `BecomingNoisy`, the latched route the noise ended.
    Pause {
        /// The event direction the pause records (R5).
        direction: DeviceDirection,
        /// The route the pause event names.
        device_id: String,
    },
    /// FR-017a: follow the appearing device.
    Follow {
        /// The appearing device identifier.
        device_id: String,
        /// The appearing device's claimed class.
        privacy: OutputDevicePrivacy,
    },
    /// FR-017 / R3: the signal changes nothing.
    NoChange,
}

/// The reference routing function: the closed signal set
/// over the transport state and the latched output — no
/// catch-all arm (Principle VII / FR-044), so a new
/// signal or decision forces a table entry here before
/// anything can compile.
fn reference_decision(
    signal: &RoutingSignal,
    transport: TransportState,
    current: Option<&DeviceLatch>,
) -> ReferenceDecision {
    // R3 / FR-017: a signal while not playing changes
    // nothing — never an auto-resume.
    if transport != TransportState::Playing {
        return ReferenceDecision::NoChange;
    }
    match signal {
        // FR-016a primary: the route went noisy — pause
        // within the bound, naming the latched route.
        RoutingSignal::BecomingNoisy => ReferenceDecision::Pause {
            direction: DeviceDirection::Disappeared,
            device_id: current.map_or_else(|| "route".to_string(), |latch| latch.device_id.clone()),
        },
        // FR-016a secondary: a removal — pause within the
        // bound, whatever the removed device's class.
        RoutingSignal::DeviceDisappeared { device_id, .. } => ReferenceDecision::Pause {
            direction: DeviceDirection::Disappeared,
            device_id: device_id.clone(),
        },
        // FR-017a: follow the appearing device — except a
        // move out of a private output to a public one,
        // which pauses first (FR-016 privacy precedence).
        RoutingSignal::DeviceAppeared { device_id, privacy } => {
            let leaves_a_private_output = current
                .is_some_and(|latch| latch.privacy == OutputDevicePrivacy::Private)
                && *privacy == OutputDevicePrivacy::Public;
            if leaves_a_private_output {
                ReferenceDecision::Pause {
                    direction: DeviceDirection::Appeared,
                    device_id: device_id.clone(),
                }
            } else {
                ReferenceDecision::Follow {
                    device_id: device_id.clone(),
                    privacy: *privacy,
                }
            }
        }
    }
}

/// The latch after one signal: a follow latches the
/// followed device; a removal unlatches only the removed
/// device, and only when it was the current output;
/// `BecomingNoisy` unlatches (the route went noisy); a
/// privacy-precedence pause leaves the private output
/// latched — it remains the current output until the
/// reader resumes (FR-016).
fn reference_latch_after(
    signal: &RoutingSignal,
    decision: &ReferenceDecision,
    previous: Option<DeviceLatch>,
) -> Option<DeviceLatch> {
    match (signal, decision) {
        // A follow latches the followed device (R6).
        (_, ReferenceDecision::Follow { device_id, privacy }) => Some(DeviceLatch {
            device_id: device_id.clone(),
            privacy: *privacy,
        }),
        // A no-op, and a privacy-precedence pause, leave
        // the latch where it was.
        (RoutingSignal::DeviceAppeared { .. }, ReferenceDecision::Pause { .. })
        | (_, ReferenceDecision::NoChange) => previous,
        // The route went noisy: no device remains latched.
        (RoutingSignal::BecomingNoisy, ReferenceDecision::Pause { .. }) => None,
        // A removal unlatches only the removed device, and
        // only when it was the current output.
        (RoutingSignal::DeviceDisappeared { device_id, .. }, ReferenceDecision::Pause { .. }) => {
            match previous {
                Some(latch) if latch.device_id == *device_id => None,
                Some(latch) => Some(latch),
                None => None,
            }
        }
    }
}

/// The per-step verdict: `None` when the decision conforms
/// to the reference, or a failure detail naming the
/// violated invariant.
fn routing_mismatch(decision: &RoutingDecision, reference: &ReferenceDecision) -> Option<String> {
    match (decision, reference) {
        (
            RoutingDecision::Pause { latency_ms, device },
            ReferenceDecision::Pause {
                direction,
                device_id,
            },
        ) => {
            if *latency_ms > ROUTING_PAUSE_BOUND_MS {
                Some(format!(
                    "the pause latency {latency_ms} ms exceeded the one-second bound (FR-016a, SC-002)"
                ))
            } else if device.direction != *direction {
                Some(format!(
                    "the pause event direction {:?} did not match the signal direction {direction:?} (R5)",
                    device.direction
                ))
            } else if device.device_id != *device_id {
                Some(format!(
                    "the pause event named {} instead of the removed route {device_id} (R5)",
                    device.device_id
                ))
            } else {
                None
            }
        }
        (RoutingDecision::Follow { device }, ReferenceDecision::Follow { device_id, .. }) => {
            if device.direction != DeviceDirection::Appeared {
                Some(format!(
                    "the follow event direction {:?} is not an appearance (R5)",
                    device.direction
                ))
            } else if device.device_id != *device_id {
                Some(format!(
                    "the follow event named {} instead of the appearing device {device_id} (R5)",
                    device.device_id
                ))
            } else {
                None
            }
        }
        (RoutingDecision::NoChange, ReferenceDecision::NoChange) => None,
        (RoutingDecision::Pause { .. }, ReferenceDecision::Follow { .. }) => Some(String::from(
            "the decision paused where the reference follows (FR-017a)",
        )),
        (RoutingDecision::Pause { .. }, ReferenceDecision::NoChange) => Some(String::from(
            "the decision paused where the reference changes nothing (FR-017)",
        )),
        (RoutingDecision::Follow { .. }, ReferenceDecision::Pause { .. }) => Some(String::from(
            "the decision followed where the reference pauses (FR-016)",
        )),
        (RoutingDecision::Follow { .. }, ReferenceDecision::NoChange) => Some(String::from(
            "the decision followed where the reference changes nothing (FR-017)",
        )),
        (RoutingDecision::NoChange, ReferenceDecision::Pause { .. }) => Some(String::from(
            "the decision changed nothing where the reference pauses (FR-016a)",
        )),
        (RoutingDecision::NoChange, ReferenceDecision::Follow { .. }) => Some(String::from(
            "the decision changed nothing where the reference follows (FR-017a)",
        )),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// FR-016a (primary mechanism): a sequence containing
    /// ONLY `BecomingNoisy` — no device-callback removal
    /// anywhere — still reaches the pause within the
    /// one-second bound. Each mechanism is sufficient on
    /// its own: neither waits for the other.
    #[test]
    fn a_becoming_noisy_sequence_alone_reaches_the_pause_within_the_bound(
        signals in proptest::collection::vec(Just(RoutingSignal::BecomingNoisy), 1..=16)
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
        // Latch the private route the way a session opens:
        // the appearing headset is followed.
        let opening = tracker.apply(
            RoutingSignal::DeviceAppeared {
                device_id: "wired".to_string(),
                privacy: OutputDevicePrivacy::Private,
            },
            session.transport,
        );
        prop_assert_eq!(
            decision_kind(&opening),
            DecisionKind::Follow,
            "the opening appearance must follow the wired headset (FR-017a)"
        );
        session.apply_decision(&opening);
        for signal in signals {
            let position_before = session.position_secs;
            let reference = reference_decision(&signal, session.transport, session.current.as_ref());
            let previous_latch = session.current.take();
            let latched = reference_latch_after(&signal, &reference, previous_latch);
            let decision = tracker.apply(signal, session.transport);
            let mismatch = routing_mismatch(&decision, &reference);
            prop_assert!(
                mismatch.is_none(),
                "the routing decision {:?} violated the reference verdict {:?}: {}",
                decision,
                reference,
                mismatch.as_deref().unwrap_or("")
            );
            session.apply_decision(&decision);
            session.current = latched;
            prop_assert_eq!(
                session.position_secs,
                position_before,
                "a routing decision moved the position (R4)"
            );
        }
    }

    /// FR-016a (secondary mechanism): a sequence containing
    /// ONLY `DeviceDisappeared` removals — no
    /// `ACTION_AUDIO_BECOMING_NOISY` anywhere — still
    /// reaches the pause within the one-second bound, so a
    /// Bluetooth route reporting the loss only through the
    /// device callback still pauses (FR-016: a Bluetooth
    /// disconnect carries exactly the same consequence as
    /// a wired removal).
    #[test]
    fn a_device_disappeared_sequence_alone_reaches_the_pause_within_the_bound(
        signals in proptest::collection::vec(disappeared_matched_strategy(), 1..=16)
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
        for signal in signals {
            let position_before = session.position_secs;
            let reference = reference_decision(&signal, session.transport, session.current.as_ref());
            let previous_latch = session.current.take();
            let latched = reference_latch_after(&signal, &reference, previous_latch);
            let decision = tracker.apply(signal, session.transport);
            let mismatch = routing_mismatch(&decision, &reference);
            prop_assert!(
                mismatch.is_none(),
                "the routing decision {:?} violated the reference verdict {:?}: {}",
                decision,
                reference,
                mismatch.as_deref().unwrap_or("")
            );
            session.apply_decision(&decision);
            session.current = latched;
            prop_assert_eq!(
                session.position_secs,
                position_before,
                "a routing decision moved the position (R4)"
            );
        }
    }

    /// FR-017a: while playing, an appearance follows at the
    /// unchanged position — no pause, restart, or repeat —
    /// except a move out of a private output to a public
    /// one, which pauses first (FR-016 privacy
    /// precedence). The privacy is the appearing device's
    /// CLAIMED class, so a mismatched identifier (a
    /// "wired" id claiming `Public`) is enforced by the
    /// claim, not by the id's real class.
    #[test]
    fn an_appearance_sequence_while_playing_follows_or_pauses_by_claimed_privacy(
        signals in proptest::collection::vec(appeared_strategy(), 1..=16)
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
        for signal in signals {
            let position_before = session.position_secs;
            let reference = reference_decision(&signal, session.transport, session.current.as_ref());
            let previous_latch = session.current.take();
            let latched = reference_latch_after(&signal, &reference, previous_latch);
            let decision = tracker.apply(signal, session.transport);
            let mismatch = routing_mismatch(&decision, &reference);
            prop_assert!(
                mismatch.is_none(),
                "the routing decision {:?} violated the reference verdict {:?}: {}",
                decision,
                reference,
                mismatch.as_deref().unwrap_or("")
            );
            session.apply_decision(&decision);
            session.current = latched;
            prop_assert_eq!(
                session.position_secs,
                position_before,
                "a routing decision moved the position (R4)"
            );
        }
    }

    /// R1–R6 together: over arbitrary signal sequences with
    /// interleaved audible ticks, from a session playing
    /// through a latched private output — removals and
    /// `BecomingNoisy` pause within the bound, appearances
    /// follow or pause by claimed privacy, no signal
    /// moves the position, every switch records the right
    /// event, and the latch follows the model.
    #[test]
    fn every_signal_sequence_holds_the_routing_invariants(
        steps in step_vec_strategy()
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
        for step in steps {
            match step {
                Step::Audible { secs } => {
                    let position_before = session.position_secs;
                    session.tick_audible(secs);
                    if session.transport == TransportState::Playing {
                        prop_assert_eq!(
                            session.position_secs,
                            position_before + secs,
                            "an audible tick while playing must advance the position"
                        );
                    } else {
                        prop_assert_eq!(
                            session.position_secs,
                            position_before,
                            "an audible tick while silent must not advance the position (FR-005)"
                        );
                    }
                }
                Step::Signal(signal) => {
                    let transport_before = session.transport;
                    let position_before = session.position_secs;
                    let reference = reference_decision(&signal, transport_before, session.current.as_ref());
                    let previous_latch = session.current.take();
                    let latched = reference_latch_after(&signal, &reference, previous_latch);
                    let decision = tracker.apply(signal, transport_before);
                    let mismatch = routing_mismatch(&decision, &reference);
                    prop_assert!(
                        mismatch.is_none(),
                        "the routing decision {:?} violated the reference verdict {:?}: {}",
                        decision,
                        reference,
                        mismatch.as_deref().unwrap_or("")
                    );
                    session.apply_decision(&decision);
                    session.current = latched;
                    prop_assert_eq!(
                        session.position_secs,
                        position_before,
                        "a routing decision moved the position (R4)"
                    );
                    match &decision {
                        RoutingDecision::Pause { .. } => prop_assert_eq!(
                            session.transport,
                            TransportState::Paused,
                            "a pause decision must land the transport on Paused"
                        ),
                        RoutingDecision::Follow { .. } | RoutingDecision::NoChange => prop_assert_eq!(
                            session.transport,
                            transport_before,
                            "a follow or no-op decision must leave the transport where the signal found it"
                        ),
                    }
                    prop_assert_eq!(
                        tracker.current_privacy(),
                        session.current.as_ref().map(|latch| latch.privacy),
                        "the tracker latched a different privacy than the session model (R6)"
                    );
                }
            }
        }
    }

    /// FR-016 precedence: a private→public move pauses
    /// first even when the disappearance and the
    /// appearance arrive in one generated batch. Whichever
    /// order the batch delivers them, the pause fires —
    /// from the removal (the disappearance wins), or from
    /// privacy precedence on the appearance — and no
    /// public device is ever followed while a private
    /// output is occupied.
    #[test]
    fn a_private_to_public_coincidence_batch_pauses_first_and_never_follows_public(
        signals in coincidence_signal_vec_strategy()
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
        let mut pause_count = 0u64;
        for signal in signals {
            let position_before = session.position_secs;
            let reference = reference_decision(&signal, session.transport, session.current.as_ref());
            let previous_latch = session.current.take();
            let latched = reference_latch_after(&signal, &reference, previous_latch);
            let decision = tracker.apply(signal, session.transport);
            let mismatch = routing_mismatch(&decision, &reference);
            prop_assert!(
                mismatch.is_none(),
                "the routing decision {:?} violated the reference verdict {:?}: {}",
                decision,
                reference,
                mismatch.as_deref().unwrap_or("")
            );
            if matches!(&decision, RoutingDecision::Pause { .. }) {
                pause_count += 1;
            }
            if let (RoutingDecision::Follow { .. }, ReferenceDecision::Follow { privacy, .. }) =
                (&decision, &reference)
            {
                prop_assert_eq!(
                    *privacy,
                    OutputDevicePrivacy::Private,
                    "a public device was followed out of a private output (FR-016, FR-017a)"
                );
            }
            session.apply_decision(&decision);
            session.current = latched;
            prop_assert_eq!(
                session.position_secs,
                position_before,
                "a routing decision moved the position (R4)"
            );
        }
        prop_assert!(
            pause_count >= 1,
            "the coincidence batch must pause at least once (FR-016)"
        );
        prop_assert_eq!(
            session.transport,
            TransportState::Paused,
            "the coincidence batch must end paused — audio never moves to the public device unattended"
        );
    }

    /// R3 / FR-017: while not playing, every signal is a
    /// no-op — never an auto-resume, and never a latch
    /// change: an already-silent session absorbs removals
    /// and appearances alike.
    #[test]
    fn every_non_playing_state_absorbs_every_signal(
        (state, signal) in (non_playing_state_strategy(), routing_signal_strategy())
    ) {
        let mut tracker = RoutingTracker::new();
        let mut session = SimSession::in_state(state);
        let position_before = session.position_secs;
        let reference = reference_decision(&signal, session.transport, session.current.as_ref());
        let previous_latch = session.current.take();
        let latched = reference_latch_after(&signal, &reference, previous_latch);
        let decision = tracker.apply(signal, session.transport);
        let mismatch = routing_mismatch(&decision, &reference);
        prop_assert!(
            mismatch.is_none(),
            "the routing decision {:?} violated the reference verdict {:?}: {}",
            decision,
            reference,
            mismatch.as_deref().unwrap_or("")
        );
        session.apply_decision(&decision);
        session.current = latched;
        prop_assert_eq!(
            session.transport,
            state,
            "a signal while not playing must not move the transport (FR-017)"
        );
        prop_assert_eq!(
            session.position_secs,
            position_before,
            "a signal while not playing must not move the position (R4)"
        );
        prop_assert_eq!(
            tracker.current_privacy(),
            session.current.as_ref().map(|latch| latch.privacy),
            "a signal while not playing must not change the latched privacy (R6)"
        );
    }
}

/// A wired pull (FR-016): while playing through a wired
/// headset, `DeviceDisappeared` pauses within the
/// one-second bound, preserves the position, records the
/// removed route, and unlatches the output.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (42.5 s); == is exact and a tolerance would admit a routing decision that moved the position (R4)"
)]
#[test]
fn wired_pull_pauses_within_the_bound_and_preserves_position() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
    session.tick_audible(42.5);
    let decision = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::Pause,
        "a wired pull must pause (FR-016): the decision was {:?}",
        decision
    );
    if let RoutingDecision::Pause { latency_ms, device } = &decision {
        assert!(
            *latency_ms <= ROUTING_PAUSE_BOUND_MS,
            "the wired pull paused in {latency_ms} ms, over the one-second bound (FR-016a, SC-002)"
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
    session.apply_decision(&decision);
    assert_eq!(
        session.transport,
        TransportState::Paused,
        "the wired pull must pause playback"
    );
    assert_eq!(
        session.position_secs, 42.5,
        "the wired pull must preserve the position (FR-016)"
    );
    assert_eq!(
        tracker.current_privacy(),
        None,
        "the removed wired headset is no longer the current output (R6)"
    );
}

/// A Bluetooth headset loss that the platform reports ONLY
/// through `AudioDeviceCallback.onAudioDevicesRemoved` —
/// no `ACTION_AUDIO_BECOMING_NOISY` — still pauses
/// within the one-second bound (FR-016a: the secondary
/// mechanism is sufficient on its own), with the stored
/// position preserved (FR-016: a Bluetooth disconnect
/// carries exactly the same consequence as a wired
/// removal).
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (31.75 s); == is exact and a tolerance would admit a routing decision that moved the position (R4)"
)]
#[test]
fn bt_headset_loss_reported_only_through_the_device_callback_pauses_within_the_bound() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("bt-headset", OutputDevicePrivacy::Private);
    session.tick_audible(31.75);
    let decision = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "bt-headset".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::Pause,
        "a Bluetooth loss reported only through the device callback must pause (FR-016a): the decision was {:?}",
        decision
    );
    if let RoutingDecision::Pause { latency_ms, device } = &decision {
        assert!(
            *latency_ms <= ROUTING_PAUSE_BOUND_MS,
            "the Bluetooth loss paused in {latency_ms} ms, over the one-second bound (FR-016a, SC-002)"
        );
        assert_eq!(
            device.direction,
            DeviceDirection::Disappeared,
            "the Bluetooth loss records a disappearance (R5)"
        );
        assert_eq!(
            device.device_id, "bt-headset",
            "the Bluetooth loss records the removed route (R5)"
        );
    }
    session.apply_decision(&decision);
    assert_eq!(
        session.transport,
        TransportState::Paused,
        "the Bluetooth loss must pause playback"
    );
    assert_eq!(
        session.position_secs, 31.75,
        "the Bluetooth loss must preserve the position (FR-016)"
    );
    assert_eq!(
        tracker.current_privacy(),
        None,
        "the removed Bluetooth headset is no longer the current output (R6)"
    );
}

/// The coincidence case of FR-016, disappearance first:
/// unplugging wired while a public device connects. The
/// pause fires from the removal — the disappearance
/// takes precedence — and the simultaneous appearance
/// changes nothing while paused: no follow into the
/// public device, position preserved.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (12.5 s); == is exact and a tolerance would admit a routing decision that moved the position (R4)"
)]
#[test]
fn coincidence_with_disappearance_first_pauses_before_the_appearance() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
    session.tick_audible(12.5);
    let removal = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&removal),
        DecisionKind::Pause,
        "the disappearance must pause first (FR-016): the decision was {:?}",
        removal
    );
    if let RoutingDecision::Pause { latency_ms, .. } = &removal {
        assert!(
            *latency_ms <= ROUTING_PAUSE_BOUND_MS,
            "the coincidence pause fired in {latency_ms} ms, over the one-second bound (FR-016a, SC-002)"
        );
    }
    session.apply_decision(&removal);
    let appearance = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&appearance),
        DecisionKind::NoChange,
        "the appearance must not resume playback after the disappearance paused it (FR-017): the decision was {:?}",
        appearance
    );
    session.apply_decision(&appearance);
    assert_eq!(
        session.transport,
        TransportState::Paused,
        "the coincidence must end paused — the disappearance took precedence"
    );
    assert_eq!(
        session.position_secs, 12.5,
        "the coincidence must preserve the position (FR-016)"
    );
}

/// The coincidence case of FR-016, appearance first:
/// the public device connects before the wired headset
/// is unplugged. Privacy precedence pauses on the
/// appearance itself (FR-016 / FR-017a) — a switch out
/// of a private output to a public one still pauses
/// first — and the removal then changes nothing while
/// paused: position preserved, no follow.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (12.5 s); == is exact and a tolerance would admit a routing decision that moved the position (R4)"
)]
#[test]
fn coincidence_with_appearance_first_pauses_by_privacy_precedence() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
    session.tick_audible(12.5);
    let appearance = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&appearance),
        DecisionKind::Pause,
        "a private→public appearance must pause first by privacy precedence (FR-016, FR-017a): the decision was {:?}",
        appearance
    );
    if let RoutingDecision::Pause { latency_ms, .. } = &appearance {
        assert!(
            *latency_ms <= ROUTING_PAUSE_BOUND_MS,
            "the privacy-precedence pause fired in {latency_ms} ms, over the one-second bound (FR-016a, SC-002)"
        );
    }
    session.apply_decision(&appearance);
    let removal = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&removal),
        DecisionKind::NoChange,
        "the removal must change nothing once privacy precedence paused the session (FR-017): the decision was {:?}",
        removal
    );
    session.apply_decision(&removal);
    assert_eq!(
        session.transport,
        TransportState::Paused,
        "the coincidence must end paused — privacy precedence took the first pause"
    );
    assert_eq!(
        session.position_secs, 12.5,
        "the coincidence must preserve the position (FR-016)"
    );
}

/// FR-017a: a public→public switch while playing follows
/// automatically at the unchanged position — no pause,
/// restart, or repeated phrase — and latches the followed
/// device.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (7.25 s); == is exact and a tolerance would admit a routing decision that moved the position (R4)"
)]
#[test]
fn public_to_public_appearance_follows_at_the_unchanged_position() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("car-bt", OutputDevicePrivacy::Public);
    session.tick_audible(7.25);
    let decision = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::Follow,
        "a public→public appearance must follow (FR-017a): the decision was {:?}",
        decision
    );
    if let RoutingDecision::Follow { device } = &decision {
        assert_eq!(
            device.direction,
            DeviceDirection::Appeared,
            "the switch records an appearance (R5)"
        );
        assert_eq!(
            device.device_id, "speaker",
            "the switch records the appearing device (R5)"
        );
    }
    session.apply_decision(&decision);
    assert_eq!(
        session.transport,
        TransportState::Playing,
        "the follow must keep playing — no pause (FR-017a)"
    );
    assert_eq!(
        session.position_secs, 7.25,
        "the follow must keep the unchanged position — no restart, no repeat (FR-017a)"
    );
    assert_eq!(
        tracker.current_privacy(),
        Some(OutputDevicePrivacy::Public),
        "the follow must latch the followed device (R6)"
    );
}

/// FR-017: resume through a reconnected output device only
/// when the user explicitly resumes — an appearance while
/// paused is a no-op, never an auto-resume, and the
/// position is retained for the explicit resume.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (30.0 s); == is exact and a tolerance would admit an automatic resume (FR-017)"
)]
#[test]
fn an_appearance_while_paused_changes_nothing() {
    let mut tracker = RoutingTracker::new();
    let mut session = SimSession::playing_on("wired", OutputDevicePrivacy::Private);
    session.tick_audible(30.0);
    let removal = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&removal),
        DecisionKind::Pause,
        "the removal must pause (FR-016): the decision was {:?}",
        removal
    );
    session.apply_decision(&removal);
    let decision = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "wired".to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.transport,
    );
    assert_eq!(
        decision_kind(&decision),
        DecisionKind::NoChange,
        "an appearance while paused must not auto-resume (FR-017): the decision was {:?}",
        decision
    );
    session.apply_decision(&decision);
    assert_eq!(
        session.transport,
        TransportState::Paused,
        "an appearance while paused must leave the transport alone (FR-017)"
    );
    assert_eq!(
        session.position_secs, 30.0,
        "the reconnected-device resume is owed to the explicit user resume, not the appearance (FR-017)"
    );
}

/// The one-second bound of FR-016, in milliseconds
/// (SC-002).
#[test]
fn routing_pause_bound_matches_sc002_figure_verbatim() {
    assert_eq!(
        ROUTING_PAUSE_BOUND_MS, 1000,
        "the one-second bound drifted from 1000 ms (SC-002)"
    );
}
