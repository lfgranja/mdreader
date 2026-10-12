//! T043 — US3 integration: noisy + device-callback trials
//! (FR-016 / FR-016a / FR-017 / FR-017a).
//!
//! Host-runnable replay of the two device fixtures
//! (`specs/001-android-media-shell/tests/device/fixtures/routing_noisy_removal.jsonl`
//! and `headphone_disconnect.jsonl`): every fixture step is
//! driven through the Rust-side routing tracker
//! (`player::routing::RoutingTracker`) and the transport
//! authority (`player::cli::PlaybackSession`), asserting
//! the outcome the fixture's `result` token records.
//!
//! Trial legs (from `specs/001-android-media-shell/tasks.md`
//! T043, verbatim): "wired pull via `BECOMING_NOISY` ≤1 s;
//! BT loss via `onAudioDevicesRemoved`-only ≤1 s; reconnect
//! + explicit resume via reconnected device; coincide
//! unplug-wired/connect-BT pauses first; replay against
//! `tests/device/fixtures/routing_noisy_removal.jsonl` +
//! `headphone_disconnect.jsonl`; FAIL before
//! implementation".
//!
//! - Wired pull (`BECOMING_NOISY`, the primary mechanism,
//!   FR-016a): a `Playing` session pauses within the
//!   one-second bound, the transport lands `Paused`, and
//!   the position is unchanged (FR-016) — audio never
//!   reaches the loudspeaker.
//! - BT loss (`onAudioDevicesRemoved` only, the secondary
//!   mechanism, FR-016a): no `BECOMING_NOISY` ever fires,
//!   yet the pause is still reached within the bound — the
//!   secondary mechanism alone suffices, and a Bluetooth
//!   disconnect carries exactly the wired removal's
//!   consequence.
//! - Reconnect + explicit resume (FR-017): the reconnected
//!   device's appearance while `Paused` yields `NoChange`
//!   — never an auto-resume — and only the explicit
//!   `Play` verb resumes, through the device the tracker
//!   latched.
//! - Coincident unplug-wired/connect-`car-BT` (privacy
//!   precedence, FR-016/FR-017a): the disappearance takes
//!   precedence — a pause within the bound first, never a
//!   `Follow` to the public car-Bluetooth output.
//! - Fixture replay: both fixtures step-by-step, asserting
//!   each recorded `result` token.
//!
//! Device-only gaps (traced, not asserted on host): the
//! actual audio routing on the device (that audio never
//! moves to the loudspeaker), the real
//! `ACTION_AUDIO_BECOMING_NOISY` and
//! `AudioDeviceCallback.onAudioDevicesRemoved` delivery
//! timing, and the wall-clock pause latency — the host
//! asserts the decision the routing state machine returns
//! and its declared `latency_ms` bound, which the surface
//! layer (T046) maps onto the platform callbacks.

use diag::writer::{DeviceDirection, DeviceEvent};
use engine_core::closed::TransportState;
use player::cli::{PlaybackSession, ShellVerb};
use player::routing::{
    OutputDevicePrivacy, ROUTING_PAUSE_BOUND_MS, RoutingDecision, RoutingSignal, RoutingTracker,
};

/// Audible progress fed while driving a session into
/// `Playing` — an exact binary fraction, so every
/// position comparison in this file is bit-exact (a
/// tolerance would admit a position the pause did not
/// preserve).
const AUDIBLE_PROGRESS_SECS: f64 = 10.0;

/// The wired headset the noisy-removal trial plays
/// through — a private output (FR-017a).
const WIRED_DEVICE_ID: &str = "wired";

/// The headphone the disconnect trial plays through — a
/// private output (FR-016).
const HEADPHONE_DEVICE_ID: &str = "headphone";

/// The Bluetooth headset the secondary-mechanism trial
/// plays through — a private output (FR-016a: a
/// Bluetooth disconnect carries exactly the wired
/// removal's consequence).
const BT_HEADSET_DEVICE_ID: &str = "bt-headset";

/// The device that reappears mid-trial in the
/// noisy-removal fixture — a private output (FR-017).
const BT_NEW_DEVICE_ID: &str = "bt_new";

/// The car Bluetooth output — a public output in
/// shared-cabin use (FR-017a): never a follow target.
const CAR_BT_DEVICE_ID: &str = "car-bt";

/// The built-in loudspeaker — a public output (FR-017a):
/// never a follow target.
const SPEAKER_DEVICE_ID: &str = "speaker";

/// The reconnected wired device of the headphone
/// fixture — a private output (FR-017).
const WIRED_ORIGINAL_DEVICE_ID: &str = "wired_original";

/// Embedded device fixture: the wired-pull
/// (`BECOMING_NOISY`) trial — noisy removal, BT
/// reappear, BT-only removal.
const FIXTURE_ROUTING_NOISY_REMOVAL: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/routing_noisy_removal.jsonl"
);

/// Embedded device fixture: the headphone-disconnect
/// trial — pull with speaker risk, then reconnect.
const FIXTURE_HEADPHONE_DISCONNECT: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/headphone_disconnect.jsonl"
);

/// A session driven `Playing` with audible progress —
/// the state every trial leg starts from (FR-002: the
/// position advances only while audio is audible).
fn playing_session() -> PlaybackSession {
    let mut session = PlaybackSession::new();
    let played = session.execute(ShellVerb::Play);
    assert!(
        played.is_ok(),
        "Play must succeed from the initial Paused state (FR-019b)"
    );
    session.on_audible_progress(AUDIBLE_PROGRESS_SECS);
    session
}

/// A routing tracker latched on `device_id` as its
/// current output — the appearance that connected the
/// session's private output before playback began.
fn tracker_on(device_id: &str) -> RoutingTracker {
    let mut tracker = RoutingTracker::new();
    let _latched = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: device_id.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        TransportState::Paused,
    );
    tracker
}

/// Fixture step 1 of the noisy-removal trial
/// (`BECOMING_NOISY → pause_within_1000ms`): the
/// primary mechanism (FR-016a) reaches the pause within
/// the one-second bound on its own — the transport lands
/// `Paused` with the position unchanged (FR-016), and
/// the pause is recorded as the wired output's
/// disappearance (FR-017a).
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertion pins the pause-preserves-position contract (FR-016); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn a_wired_pull_via_becoming_noisy_pauses_within_the_bound() {
    let mut session = playing_session();
    let mut tracker = tracker_on(WIRED_DEVICE_ID);
    let transport = session.snapshot().transport;
    assert_eq!(
        transport,
        TransportState::Playing,
        "the trial starts from a Playing session (FR-005)"
    );
    let decision = tracker.apply(RoutingSignal::BecomingNoisy, transport);
    match &decision {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the BECOMING_NOISY pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: WIRED_DEVICE_ID.to_string(),
                },
                "the pause must record the wired output's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&decision, RoutingDecision::Pause { .. }),
                "a wired pull must pause, never follow another output (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&decision, RoutingDecision::Pause { .. }),
                "the primary mechanism must reach the pause on its own (FR-016a)"
            );
        }
    }
    // The surface layer acts on the decision: the
    // transport lands Paused and the position is
    // unchanged (FR-016).
    let paused = session.execute(ShellVerb::Pause);
    assert!(paused.is_ok(), "Pause must succeed from the Playing state");
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "the wired pull must pause the transport (FR-016)"
    );
    assert_eq!(
        snapshot.position_secs, AUDIBLE_PROGRESS_SECS,
        "the pause must preserve the last spoken position (FR-016)"
    );
}

/// The secondary mechanism alone (FR-016a): a Bluetooth
/// headset loss delivered only through
/// `AudioDeviceCallback.onAudioDevicesRemoved` — no
/// `BECOMING_NOISY` ever fires — still reaches the
/// pause within the one-second bound, with the position
/// preserved (FR-016: a Bluetooth disconnect carries
/// exactly the wired removal's consequence).
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertion pins the pause-preserves-position contract (FR-016); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn a_bluetooth_loss_via_on_audio_devices_removed_only_pauses_within_the_bound() {
    let mut session = playing_session();
    let mut tracker = tracker_on(BT_HEADSET_DEVICE_ID);
    let transport = session.snapshot().transport;
    // The removal arrives through the secondary
    // mechanism only: no BECOMING_NOISY is ever
    // delivered (FR-016a).
    let decision = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: BT_HEADSET_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        transport,
    );
    match &decision {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the onAudioDevicesRemoved pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: BT_HEADSET_DEVICE_ID.to_string(),
                },
                "the pause must record the Bluetooth headset's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&decision, RoutingDecision::Pause { .. }),
                "a Bluetooth loss must pause, never follow another output (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&decision, RoutingDecision::Pause { .. }),
                "the secondary mechanism must reach the pause on its own — no BECOMING_NOISY is delivered (FR-016a)"
            );
        }
    }
    let paused = session.execute(ShellVerb::Pause);
    assert!(paused.is_ok(), "Pause must succeed from the Playing state");
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "the Bluetooth loss must pause the transport (FR-016)"
    );
    assert_eq!(
        snapshot.position_secs, AUDIBLE_PROGRESS_SECS,
        "the pause must preserve the last spoken position (FR-016)"
    );
}

/// Reconnect + explicit resume (FR-017): after the
/// pause, the reconnected device's appearance yields
/// `NoChange` — never an auto-resume — and only the
/// explicit `Play` verb resumes, through the device the
/// tracker latched.
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertion pins the resume-at-same-position contract (FR-017); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn a_reconnected_device_never_auto_resumes_and_the_explicit_play_resumes_through_it() {
    let mut session = playing_session();
    let mut tracker = tracker_on(WIRED_DEVICE_ID);
    // The wired pull pauses the session (FR-016).
    let transport = session.snapshot().transport;
    let _pulled = tracker.apply(RoutingSignal::BecomingNoisy, transport);
    let paused = session.execute(ShellVerb::Pause);
    assert!(paused.is_ok(), "Pause must succeed from the Playing state");
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "the wired pull must pause the transport (FR-016)"
    );
    // The reconnected device appears while Paused: the
    // decision is NoChange — the resume is explicit
    // only (FR-017), never automatic.
    let reconnected = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: WIRED_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.snapshot().transport,
    );
    match &reconnected {
        RoutingDecision::NoChange => {}
        RoutingDecision::Pause { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must not pause anything — the session is already paused (FR-017)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must never auto-resume playback — the resume is explicit only (FR-017)"
            );
        }
    }
    // The tracker has latched the reconnected device:
    // the explicit resume happens through it (FR-017).
    assert_eq!(
        tracker.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the tracker must latch the reconnected device (FR-017)"
    );
    let resumed = session.execute(ShellVerb::Play);
    assert!(
        resumed.is_ok(),
        "the explicit Play must succeed from the Paused state"
    );
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Playing,
        "the explicit Play must resume the transport (FR-017)"
    );
    assert_eq!(
        snapshot.position_secs, AUDIBLE_PROGRESS_SECS,
        "the explicit resume must continue at the same position (FR-017)"
    );
}

/// Coincident unplug-wired/connect-`car-BT` (privacy
/// precedence, FR-016/FR-017a): where a disappearance
/// and an appearance coincide in one route change, the
/// disappearance takes precedence — the pause lands
/// within the bound first, and the public car-Bluetooth
/// appearance is never followed.
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertion pins the pause-preserves-position contract (FR-016); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn a_coincident_wired_pull_and_car_bt_appearance_pauses_first_and_never_follows_the_public_output()
{
    let mut session = playing_session();
    let mut tracker = tracker_on(WIRED_DEVICE_ID);
    // The disappearance is applied first: the wired
    // output is pulled while Playing (FR-016).
    let transport = session.snapshot().transport;
    let pulled = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: WIRED_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        transport,
    );
    match &pulled {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the coincident-route pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: WIRED_DEVICE_ID.to_string(),
                },
                "the pause must record the wired output's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "the disappearance takes precedence — pause first, never follow the coincident appearance (FR-016, FR-017a)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "the disappearance takes precedence and must reach the pause within the bound (FR-016, FR-016a)"
            );
        }
    }
    let paused = session.execute(ShellVerb::Pause);
    assert!(paused.is_ok(), "Pause must succeed from the Playing state");
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "the coincident route change must pause first (FR-016)"
    );
    // The coincident car-BT appearance is then
    // re-evaluated under FR-017a — and never followed:
    // a switch out of a private output to a public one
    // stays paused, and following a newly available
    // device never permits audio to move to a public
    // output.
    let car_bt = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: CAR_BT_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        session.snapshot().transport,
    );
    match &car_bt {
        RoutingDecision::Pause { latency_ms, .. } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "any pause the tracker returns must respect the one-second bound (FR-016)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                !matches!(&car_bt, RoutingDecision::Follow { .. }),
                "the public car-BT appearance must never be followed — audio must not move to a public output (FR-017a)"
            );
        }
        RoutingDecision::NoChange => {}
    }
}

/// FR-017a's absolute privacy invariant: while audio is
/// playing on a private output, a newly available
/// public output is never followed — following it would
/// move audio to a public output, which the contract
/// forbids.
#[test]
fn a_public_appearance_is_never_followed_while_playing_on_a_private_output() {
    let mut session = playing_session();
    let mut tracker = tracker_on(WIRED_DEVICE_ID);
    let transport = session.snapshot().transport;
    let speaker = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: SPEAKER_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        transport,
    );
    match &speaker {
        RoutingDecision::Pause { latency_ms, .. } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "any pause the tracker returns must respect the one-second bound (FR-016)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                !matches!(&speaker, RoutingDecision::Follow { .. }),
                "following a newly available device must never move audio to a public output (FR-017a)"
            );
        }
        RoutingDecision::NoChange => {}
    }
    // The session is the transport authority: a tracker
    // decision never moves it on its own (FR-005,
    // FR-020) — the surface layer applies the pause.
    assert_eq!(
        session.snapshot().transport,
        TransportState::Playing,
        "a routing decision alone must not move the transport (FR-005, FR-020)"
    );
}

/// Full replay of `routing_noisy_removal.jsonl`: every
/// fixture step, in fixture order, driven through the
/// routing tracker and the session, with the outcome
/// each `result` token records — the primary mechanism
/// pauses within the bound, the reconnected device
/// continues at the same position only through the
/// explicit verb, and the secondary mechanism alone
/// pauses again within the bound.
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertions pin the pause-preserves-position and resume-at-same-position contracts (FR-016, FR-017); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn the_routing_noisy_removal_fixture_replays_every_recorded_result() {
    let lines: Vec<&str> = FIXTURE_ROUTING_NOISY_REMOVAL.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "routing_noisy_removal.jsonl must carry exactly the 3 recorded trial steps"
    );
    assert!(
        lines[0].contains("BECOMING_NOISY")
            && lines[0].contains("wired")
            && lines[0].contains("pause_within_1000ms"),
        "fixture step 1 must be BECOMING_NOISY on the wired route → pause_within_1000ms"
    );
    assert!(
        lines[1].contains("bt_new")
            && lines[1].contains("position_preserved")
            && lines[1].contains("continue_same_position"),
        "fixture step 2 must be DEVICE_APPEARED bt_new → continue_same_position"
    );
    assert!(
        lines[2].contains("onAudioDevicesRemoved") && lines[2].contains("pause_within_1000ms"),
        "fixture step 3 must be onAudioDevicesRemoved on the bt-only route → pause_within_1000ms"
    );

    // Step 1 (ts 0): the wired pull arrives as
    // BECOMING_NOISY and pauses within the bound
    // (pause_within_1000ms).
    let mut session = playing_session();
    let mut tracker = tracker_on(WIRED_DEVICE_ID);
    let transport = session.snapshot().transport;
    let pulled = tracker.apply(RoutingSignal::BecomingNoisy, transport);
    match &pulled {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the BECOMING_NOISY pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: WIRED_DEVICE_ID.to_string(),
                },
                "the pause must record the wired output's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "a wired pull must pause, never follow another output (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "the primary mechanism must reach the pause on its own (FR-016a)"
            );
        }
    }
    let first_pause = session.execute(ShellVerb::Pause);
    assert!(
        first_pause.is_ok(),
        "Pause must succeed from the Playing state"
    );
    let at_pause = session.snapshot();
    assert_eq!(
        at_pause.transport,
        TransportState::Paused,
        "the noisy removal must pause the transport (FR-016)"
    );
    assert_eq!(
        at_pause.position_secs, AUDIBLE_PROGRESS_SECS,
        "the pause must preserve the position (fixture step 1, position_preserved)"
    );

    // Step 2 (ts 800): bt_new appears while Paused —
    // the decision is NoChange and the position is
    // preserved; the explicit Play continues at the
    // same position (continue_same_position, FR-017).
    let reconnected = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: BT_NEW_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.snapshot().transport,
    );
    match &reconnected {
        RoutingDecision::NoChange => {}
        RoutingDecision::Pause { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must not pause anything — the session is already paused (FR-017)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must never auto-resume playback — the resume is explicit only (FR-017)"
            );
        }
    }
    assert_eq!(
        tracker.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the tracker must latch the reconnected device (FR-017)"
    );
    let resumed = session.execute(ShellVerb::Play);
    assert!(
        resumed.is_ok(),
        "the explicit Play must succeed from the Paused state"
    );
    let at_resume = session.snapshot();
    assert_eq!(
        at_resume.transport,
        TransportState::Playing,
        "the explicit resume must continue playback (fixture step 2, continue_same_position)"
    );
    assert_eq!(
        at_resume.position_secs, AUDIBLE_PROGRESS_SECS,
        "the resume must continue at the same position (fixture step 2, continue_same_position)"
    );

    // Step 3 (ts 1500): the BT-only route is removed
    // through onAudioDevicesRemoved alone — the
    // secondary mechanism reaches the pause within the
    // bound (pause_within_1000ms).
    let transport = session.snapshot().transport;
    let bt_lost = tracker.apply(
        RoutingSignal::DeviceDisappeared {
            device_id: BT_NEW_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        transport,
    );
    match &bt_lost {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the onAudioDevicesRemoved pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: BT_NEW_DEVICE_ID.to_string(),
                },
                "the pause must record the Bluetooth output's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&bt_lost, RoutingDecision::Pause { .. }),
                "a Bluetooth loss must pause, never follow another output (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&bt_lost, RoutingDecision::Pause { .. }),
                "the secondary mechanism must reach the pause on its own (FR-016a)"
            );
        }
    }
    let second_pause = session.execute(ShellVerb::Pause);
    assert!(
        second_pause.is_ok(),
        "Pause must succeed from the Playing state"
    );
    let at_second_pause = session.snapshot();
    assert_eq!(
        at_second_pause.transport,
        TransportState::Paused,
        "the BT-only removal must pause the transport (FR-016)"
    );
    assert_eq!(
        at_second_pause.position_secs, AUDIBLE_PROGRESS_SECS,
        "the second pause must preserve the position (FR-016)"
    );
}

/// Full replay of `headphone_disconnect.jsonl`: the pull
/// pauses first and the private-to-speaker switch is
/// false — the loudspeaker is never followed — then the
/// reconnect resumes at the same position only through
/// the explicit verb (FR-016 / FR-016a / FR-017 /
/// FR-017a).
#[expect(
    clippy::float_cmp,
    reason = "the retained position 10.0 is an exact binary fraction and the assertions pin the pause-preserves-position and resume-at-same-position contracts (FR-016, FR-017); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn the_headphone_disconnect_fixture_replays_every_recorded_result() {
    let lines: Vec<&str> = FIXTURE_HEADPHONE_DISCONNECT.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "headphone_disconnect.jsonl must carry exactly the 2 recorded trial steps"
    );
    assert!(
        lines[0].contains("HEADPHONE_PULL")
            && lines[0].contains("speaker_risk")
            && lines[0].contains("pause_then_private_to_speaker_false"),
        "fixture step 1 must be HEADPHONE_PULL → pause_then_private_to_speaker_false"
    );
    assert!(
        lines[1].contains("RECONNECT")
            && lines[1].contains("wired_original")
            && lines[1].contains("resume_at_same_position_explicit"),
        "fixture step 2 must be RECONNECT wired_original → resume_at_same_position_explicit"
    );

    // Step 1 (ts 0): the headphone pull pauses within
    // the bound through the primary mechanism
    // (pause_then_private_to_speaker_false, FR-016a).
    let mut session = playing_session();
    let mut tracker = tracker_on(HEADPHONE_DEVICE_ID);
    let transport = session.snapshot().transport;
    let pulled = tracker.apply(RoutingSignal::BecomingNoisy, transport);
    match &pulled {
        RoutingDecision::Pause { latency_ms, device } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "the headphone-pull pause must land within {ROUTING_PAUSE_BOUND_MS} ms, got {latency_ms} ms (FR-016, FR-016a)"
            );
            assert_eq!(
                device,
                &DeviceEvent {
                    direction: DeviceDirection::Disappeared,
                    device_id: HEADPHONE_DEVICE_ID.to_string(),
                },
                "the pause must record the headphone's disappearance as the output_device event (FR-017a)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "a headphone pull must pause, never follow another output (FR-016)"
            );
        }
        RoutingDecision::NoChange => {
            assert!(
                matches!(&pulled, RoutingDecision::Pause { .. }),
                "the primary mechanism must reach the pause on its own (FR-016a)"
            );
        }
    }
    let pause = session.execute(ShellVerb::Pause);
    assert!(pause.is_ok(), "Pause must succeed from the Playing state");
    let at_pause = session.snapshot();
    assert_eq!(
        at_pause.transport,
        TransportState::Paused,
        "the headphone pull must pause the transport (FR-016)"
    );
    assert_eq!(
        at_pause.position_secs, AUDIBLE_PROGRESS_SECS,
        "the pause must preserve the position (FR-016)"
    );
    // The speaker risk is false: the built-in
    // loudspeaker appearing after the pull is never
    // followed — the private-to-speaker switch does not
    // happen (pause_then_private_to_speaker_false,
    // FR-017a).
    let speaker = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: SPEAKER_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        session.snapshot().transport,
    );
    match &speaker {
        RoutingDecision::Pause { latency_ms, .. } => {
            assert!(
                *latency_ms <= ROUTING_PAUSE_BOUND_MS,
                "any pause the tracker returns must respect the one-second bound (FR-016)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                !matches!(&speaker, RoutingDecision::Follow { .. }),
                "the private-to-speaker switch must be false — audio must never move to the loudspeaker (FR-017a)"
            );
        }
        RoutingDecision::NoChange => {}
    }

    // Step 2 (ts 350): the original wired device
    // reconnects — the appearance yields NoChange
    // (never an auto-resume), and the explicit Play
    // resumes at the same position
    // (resume_at_same_position_explicit, FR-017).
    let reconnected = tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: WIRED_ORIGINAL_DEVICE_ID.to_string(),
            privacy: OutputDevicePrivacy::Private,
        },
        session.snapshot().transport,
    );
    match &reconnected {
        RoutingDecision::NoChange => {}
        RoutingDecision::Pause { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must not pause anything — the session is already paused (FR-017)"
            );
        }
        RoutingDecision::Follow { .. } => {
            assert!(
                matches!(&reconnected, RoutingDecision::NoChange),
                "an appearance while Paused must never auto-resume playback — the resume is explicit only (FR-017)"
            );
        }
    }
    assert_eq!(
        tracker.current_privacy(),
        Some(OutputDevicePrivacy::Private),
        "the tracker must latch the reconnected device (FR-017)"
    );
    let resumed = session.execute(ShellVerb::Play);
    assert!(
        resumed.is_ok(),
        "the explicit Play must succeed from the Paused state"
    );
    let at_resume = session.snapshot();
    assert_eq!(
        at_resume.transport,
        TransportState::Playing,
        "the explicit resume must continue playback (fixture step 2, resume_at_same_position_explicit)"
    );
    assert_eq!(
        at_resume.position_secs, AUDIBLE_PROGRESS_SECS,
        "the explicit resume must continue at the same position (fixture step 2, resume_at_same_position_explicit)"
    );
}
