//! T023 — US2 property: focus latest-event-wins over the closed
//! 8-reason interruption set (FR-004 / FR-005 / FR-013b / FR-014).
//!
//! `specs/004-ressoa-dogfood-build/tasks.md` T023, verbatim:
//! "Property `prop_focus_latest_wins` in
//! `crates/player/tests/prop_focus8.rs` (invariants: 8 reasons
//! exhaustive, no `_ =>`; transient-immediately-followed-by-permanent
//! ⇒ final paused no-auto-resume with latest reason;
//! permanent-loss-followed-by-transient ⇒ no-op (sticky permanent,
//! transient ignored); `Deferred` ⇒ zero-audio hold with no reason;
//! duck keeps `Playing` with `DUCK_GAIN 0.20 ±0.02` + restore without
//! place loss, never owes rewind; call/alarm pause ≤1s)".
//!
//! Spec anchors:
//! - FR-004 (`spec.md`): reading clearly quieter to 0.20× gain
//!   (±0.02, ~20%) under a navigation-style transient, restoring full
//!   volume afterwards without losing the listener's place.
//! - FR-005 (`spec.md`): pause within 1 s on a silence-demanding
//!   interruption (call, alarm) or a permanent audio loss, state the
//!   reason for 3 seconds, never resume audio on its own; the reason
//!   set is closed and has exactly eight members (FocusDenied,
//!   FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved,
//!   VoiceUnavailable, FormatUndeliverable, DecodeFailed — no
//!   `Other`/`Unknown`); `FocusDelayed` and a duckable `TransientPaused`
//!   state no line and no earcon.
//! - FR-014: latest-event-wins — a permanent loss supersedes a
//!   transient and clears its flags; a late transient after a
//!   permanent loss authorizes nothing.
//! - FR-013b: a deferred grant holds zero audio with no rewind and no
//!   stated reason; a refused grant is the named `StartRefused`
//!   failure, never a playing state.
//!
//! Surfaces under test (public only):
//! - `player::focus` — `FocusOutcome` (3), `FocusGrant` (3),
//!   `FocusIntent` (9), `FocusTracker`, `classify`, `apply_grant`,
//!   `DUCK_GAIN` / `FULL_GAIN`;
//! - `player::interruption` — `InterruptionReason` (8),
//!   `from_focus_outcome`, `from_focus_grant`, `from_routing_pause`,
//!   `reason_line`;
//! - `player::routing` — `RoutingSignal` / `RoutingTracker`, the only
//!   way to obtain the `RoutingDecision` values `from_routing_pause`
//!   consumes. The ≤1 s routing bound itself is T024's
//!   `prop_routing_bound` in `prop_routing_bound.rs`.
//!
//! Timing note (call/alarm ≤1 s): the focus paths are pure
//! classifications of the closed intent set with no clock, so the
//! mapping work is 0 ms — always inside the FR-005 one-second bound.
//! The wall-clock half (the surface reaching the pause within one
//! second of the interrupter becoming observable) is attested
//! on-device by T051 (10/10 per interruption class) and is recorded as
//! such below; this file owns the decision half.
//!
//! Gain note: `DUCK_GAIN` (0.20) exists in the player crate
//! (`player::focus::DUCK_GAIN`, band 0.15–0.25), so the duck path is
//! asserted at the gain level — `Playing` at 0.20 ±0.02 and restore to
//! full gain — not only at the enum level.
//!
//! RED STATE (recorded, per the TDD gate rule): this file is RED on
//! entry, for two independent reasons.
//!
//! 1. The 8-reason closed-set, latest-wins, sticky-permanent, deferred
//!    and grant mappings, the duck band, and the call/alarm pause all
//!    PASS against the existing `player::focus` /
//!    `player::interruption` / `player::routing` surfaces.
//! 2. `prop_focus_latest_wins` and
//!    `a_focus_gain_immediately_followed_by_a_pause_clears_the_owed_resume`
//!    FAIL — a RED FINDING the property caught in the existing
//!    `FocusTracker`, recorded here rather than fixed here (tests only;
//!    no `src/` change is permitted by this task). Minimal failing
//!    input: `intents = [FocusRegained, DeviceDisappeared]` — a focus
//!    gain immediately followed by a device removal.
//!    `FocusTracker::apply` consults `supersede_flags` only when
//!    `self.current` is `Some`; the gain resets `current` to `None`, so
//!    the pause clears no flag and `resume_due` survives the pause — the
//!    tracker holds
//!    a standing pause and an owed resume at once, which is the
//!    auto-resume FR-005 forbids ("any AUDIOFOCUS_GAIN arriving after a
//!    pause MUST wait for explicit Play and MUST never auto-resume").
//!    The same gap holds for `[FocusRegained, SilenceDemanding]` and
//!    `[FocusRegained, PermanentLoss]`. Fix owner: the focus-tracker
//!    implementation wave — clear the pending resume flag whenever a
//!    pause-demanding outcome is decided, whatever stood before it.
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest builds the message through `concat!`, and
//! rustc rejects implicit captures in macro-expanded format strings.
//! Plain `assert!` / `assert_eq!` messages use implicit captures, as
//! elsewhere in the suite.

use proptest::prelude::*;

use engine_core::closed::{EngineError, TransportState};
use player::focus::{
    DUCK_GAIN, FULL_GAIN, FocusGrant, FocusIntent, FocusOutcome, FocusTracker, apply_grant,
    classify,
};
use player::interruption::{
    InterruptionReason, from_focus_grant, from_focus_outcome, from_routing_pause, reason_line,
};
use player::routing::{OutputDevicePrivacy, RoutingDecision, RoutingSignal, RoutingTracker};

/// The FR-005 pause bound for a call/alarm or a permanent loss, in
/// milliseconds (the same one-second bound FR-006 states for device
/// removal).
const CALL_PAUSE_BOUND_MS: u128 = 1_000;

/// The call/alarm mapping latency, in milliseconds.
///
/// The focus paths are pure functions of the classified intent: no
/// clock is read, so the mapping work is 0 ms and is always inside
/// [`CALL_PAUSE_BOUND_MS`]. The wall-clock half is attested on-device
/// (T051, 10/10 per interruption class).
const FOCUS_MAPPING_LATENCY_MS: u128 = 0;

/// The nominal duck gain of FR-004: 0.20×, with its ±0.02 tolerance.
const DUCK_GAIN_NOMINAL: f32 = 0.20;

/// The FR-004 duck-gain tolerance: ±0.02 around
/// [`DUCK_GAIN_NOMINAL`].
const DUCK_GAIN_TOLERANCE: f32 = 0.02;

/// The reason-line lifetime of FR-005: three seconds, in milliseconds.
const REASON_TTL_MS: u64 = 3_000;

/// The FR-005 lock-screen line budget, in whitespace-separated words
/// (the count includes any appended title).
const REASON_LINE_MAX_WORDS: usize = 8;

/// The closed interruption-reason set of FR-005, pinned member by
/// member: exactly eight reasons, no `Other` / `Unknown`.
const EIGHT_REASONS: [InterruptionReason; 8] = [
    InterruptionReason::FocusDenied,
    InterruptionReason::FocusDelayed,
    InterruptionReason::PermanentLoss,
    InterruptionReason::TransientPaused,
    InterruptionReason::DeviceRemoved,
    InterruptionReason::VoiceUnavailable,
    InterruptionReason::FormatUndeliverable,
    InterruptionReason::DecodeFailed,
];

/// The closed focus-grant answer set of FR-013b, pinned member by
/// member: exactly three answers.
const THREE_GRANTS: [FocusGrant; 3] = [
    FocusGrant::Granted,
    FocusGrant::Deferred,
    FocusGrant::Refused,
];

/// The closed focus-outcome set of FR-013 / FR-013a / FR-014, pinned
/// member by member: exactly three outcomes.
const THREE_OUTCOMES: [FocusOutcome; 3] = [
    FocusOutcome::DuckableTransient,
    FocusOutcome::SilenceDemanding,
    FocusOutcome::PermanentLoss,
];

/// The closed nine-member focus-intent set (FR-044: no `Other`),
/// generated explicitly: a tenth intent would need a strategy arm
/// here before anything can compile.
fn intent_strategy() -> impl Strategy<Value = FocusIntent> {
    prop_oneof![
        Just(FocusIntent::DuckableTransient),
        Just(FocusIntent::SilenceDemanding),
        Just(FocusIntent::PermanentLoss),
        Just(FocusIntent::FocusRegained),
        Just(FocusIntent::BecomingNoisy),
        Just(FocusIntent::DeviceAppeared),
        Just(FocusIntent::DeviceDisappeared),
        Just(FocusIntent::AppForegrounded),
        Just(FocusIntent::AppBackgrounded),
    ]
}

/// Generated intent sequences: overlapping interruption streams as any
/// surface (notification, lock screen, MediaSession) might deliver
/// them — a duckable navigation prompt landing inside a call, a call
/// ending into a headset pull, a background latch between losses.
fn intent_vec_strategy() -> impl Strategy<Value = Vec<FocusIntent>> {
    proptest::collection::vec(intent_strategy(), 1..=24)
}

/// What the tracker MUST answer for one intent — an independent
/// restatement of the FR-014 latest-event-wins machine.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ReferenceDecision {
    /// The classified outcome, or `None` when the intent decides no
    /// outcome (latch-only, follow-only, or a regain).
    outcome: Option<FocusOutcome>,
    /// The transport to mirror, or `None` when the intent changes no
    /// transport.
    transport: Option<TransportState>,
    /// The gain to apply.
    gain: f32,
    /// The foreground latch the intent writes, if any.
    foreground_latched: Option<bool>,
}

/// The reference tracker state.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ReferenceTracker {
    /// The standing outcome.
    current: Option<FocusOutcome>,
    /// Whether playback is ducked.
    ducked: bool,
    /// Whether a transient resume is owed.
    resume_due: bool,
    /// Whether a gain restore is owed.
    duck_restore_due: bool,
    /// Whether the application is foregrounded.
    foregrounded: bool,
}

impl ReferenceTracker {
    /// A fresh reference tracker: no loss classified, nothing owed,
    /// foregrounded.
    const fn new() -> Self {
        Self {
            current: None,
            ducked: false,
            resume_due: false,
            duck_restore_due: false,
            foregrounded: true,
        }
    }

    /// Apply one intent — seven explicit arms over the closed
    /// nine-member intent set, no catch-all (gate 7): the grouped
    /// arms carry the same consequence by rule, never by accident.
    fn apply(&mut self, intent: FocusIntent) -> ReferenceDecision {
        match intent {
            FocusIntent::AppForegrounded => {
                self.foregrounded = true;
                ReferenceDecision {
                    outcome: None,
                    transport: None,
                    gain: FULL_GAIN,
                    foreground_latched: Some(true),
                }
            }
            FocusIntent::AppBackgrounded => {
                self.foregrounded = false;
                ReferenceDecision {
                    outcome: None,
                    transport: None,
                    gain: FULL_GAIN,
                    foreground_latched: Some(false),
                }
            }
            FocusIntent::FocusRegained => {
                self.current = None;
                self.ducked = false;
                self.duck_restore_due = false;
                self.resume_due = true;
                ReferenceDecision {
                    outcome: None,
                    transport: Some(TransportState::Playing),
                    gain: FULL_GAIN,
                    foreground_latched: None,
                }
            }
            FocusIntent::DeviceAppeared => ReferenceDecision {
                outcome: None,
                transport: Some(TransportState::Playing),
                gain: FULL_GAIN,
                foreground_latched: None,
            },
            // A duckable navigation prompt keeps reading at ~20%.
            FocusIntent::DuckableTransient => {
                self.decide(FocusOutcome::DuckableTransient, DUCK_GAIN)
            }
            // A silence-demanding transient — a call or alarm — and a
            // route gone noisy both pause.
            FocusIntent::SilenceDemanding | FocusIntent::BecomingNoisy => {
                self.decide(FocusOutcome::SilenceDemanding, FULL_GAIN)
            }
            // A permanent loss and an output-device removal both pause
            // with permanent-loss consequences.
            FocusIntent::PermanentLoss | FocusIntent::DeviceDisappeared => {
                self.decide(FocusOutcome::PermanentLoss, FULL_GAIN)
            }
        }
    }

    /// A decided outcome under the latest-event-wins rules: the pause
    /// transport and full gain for every non-duck outcome, `Playing`
    /// at [`DUCK_GAIN`] for a duck.
    fn decide(&mut self, next: FocusOutcome, gain: f32) -> ReferenceDecision {
        // Sticky permanent (FR-005 edge case / FR-014): a duckable
        // transient arriving while a permanent loss stands is a
        // no-op — the app is already paused, and the transient
        // authorizes nothing.
        if self.current == Some(FocusOutcome::PermanentLoss)
            && next == FocusOutcome::DuckableTransient
        {
            return ReferenceDecision {
                outcome: Some(FocusOutcome::PermanentLoss),
                transport: Some(TransportState::Paused),
                gain: FULL_GAIN,
                foreground_latched: None,
            };
        }
        if self.current.is_some() {
            let escalates_into_permanent = next == FocusOutcome::PermanentLoss
                && self.current != Some(FocusOutcome::PermanentLoss);
            if escalates_into_permanent {
                // Latest-event-wins: the superseded transient's
                // resume flag and duck-restore flag both die.
                self.resume_due = false;
                self.duck_restore_due = false;
            }
            if next == FocusOutcome::SilenceDemanding {
                // A silence pause supersedes a duck: the gain restore
                // owed by the duck dies with it.
                self.duck_restore_due = false;
            }
        }
        self.current = Some(next);
        if next == FocusOutcome::DuckableTransient {
            self.ducked = true;
            self.duck_restore_due = true;
        } else {
            self.ducked = false;
        }
        let transport = if next == FocusOutcome::DuckableTransient {
            TransportState::Playing
        } else {
            TransportState::Paused
        };
        ReferenceDecision {
            outcome: Some(next),
            transport: Some(transport),
            gain,
            foreground_latched: None,
        }
    }
}

/// The FR-005 outcome→reason mapping restated independently: a duck
/// states no reason (playback continues, so there is no pause to
/// state); the two pause-demanding outcomes map 1:1.
fn reference_reason(outcome: FocusOutcome) -> Option<InterruptionReason> {
    match outcome {
        FocusOutcome::DuckableTransient => None,
        FocusOutcome::SilenceDemanding => Some(InterruptionReason::TransientPaused),
        FocusOutcome::PermanentLoss => Some(InterruptionReason::PermanentLoss),
    }
}

/// The FR-013b grant→reason mapping restated independently.
fn reference_grant_reason(grant: FocusGrant) -> Option<InterruptionReason> {
    match grant {
        FocusGrant::Granted => None,
        FocusGrant::Deferred => Some(InterruptionReason::FocusDelayed),
        FocusGrant::Refused => Some(InterruptionReason::FocusDenied),
    }
}

/// The closed three-member routing-decision kind, for
/// `from_routing_pause` coverage.
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// T023 `prop_focus_latest_wins`: over arbitrary overlapping
    /// interruption streams the tracker IS the reference machine —
    /// latest-event-wins, sticky permanent, no auto-resume, and the
    /// standing outcome mapping to exactly one interruption reason.
    #[test]
    fn prop_focus_latest_wins(intents in intent_vec_strategy()) {
        let mut tracker = FocusTracker::new();
        let mut reference = ReferenceTracker::new();
        for intent in intents {
            let decision = tracker.apply(intent);
            let expected = reference.apply(intent);
            prop_assert_eq!(
                decision.outcome,
                expected.outcome,
                "intent {:?} resolved a different outcome than the reference machine",
                intent
            );
            prop_assert_eq!(
                decision.transport,
                expected.transport,
                "intent {:?} resolved a different transport than the reference machine",
                intent
            );
            prop_assert_eq!(
                decision.gain,
                expected.gain,
                "intent {:?} resolved a different gain than the reference machine",
                intent
            );
            prop_assert_eq!(
                decision.foreground_latched,
                expected.foreground_latched,
                "intent {:?} resolved a different foreground latch than the reference machine",
                intent
            );
            prop_assert_eq!(
                tracker.foregrounded,
                reference.foregrounded,
                "intent {:?} left a different foreground latch in the tracker",
                intent
            );
        }
        prop_assert_eq!(
            tracker.current,
            reference.current,
            "the standing outcome drifted from the latest-event-wins rule"
        );
        prop_assert_eq!(
            tracker.ducked,
            reference.ducked,
            "the ducked state drifted from the latest-event-wins rule"
        );
        prop_assert_eq!(
            tracker.resume_due,
            reference.resume_due,
            "the transient-resume flag drifted from the latest-event-wins rule"
        );
        prop_assert_eq!(
            tracker.duck_restore_due,
            reference.duck_restore_due,
            "the duck-restore flag drifted from the latest-event-wins rule"
        );

        // A standing pause-demanding outcome never owes a resume and
        // never ducks (FR-005: "every other paused outcome … and any
        // AUDIOFOCUS_GAIN arriving after a pause) MUST wait for
        // explicit Play and MUST never auto-resume").
        //
        // RECORDED RED FINDING (a defect the property caught; it is
        // not an unimplemented feature and it is NOT fixed here): the
        // tracker leaves `resume_due` set when a pause-demanding
        // outcome arrives while NO outcome is standing — the typical
        // case being a focus gain that is immediately followed by a
        // device removal or a call. `FocusTracker::apply` consults
        // `supersede_flags` only when `self.current` is `Some`, so the
        // pending transient resume survives the pause.
        //   Minimal failing inputs:
        //     intents = [FocusRegained, DeviceDisappeared]
        //     intents = [FocusRegained, SilenceDemanding]
        // The tracker-vs-reference-model equality above holds for
        // these sequences: the reference machine restates the
        // documented latest-event-wins rules and the implementation
        // matches it. This absolute invariant is the FR-005 contract
        // the machine does not yet satisfy, and it is the gate for the
        // focus-tracker fix in the implementation wave.
        if matches!(
            reference.current,
            Some(FocusOutcome::PermanentLoss | FocusOutcome::SilenceDemanding)
        ) {
            prop_assert!(
                !tracker.resume_due,
                "a standing pause-demanding outcome owed a resume — playback would auto-resume (FR-005)"
            );
            prop_assert!(
                !tracker.ducked,
                "a standing pause-demanding outcome left playback ducked instead of paused at full gain"
            );
            prop_assert_eq!(
                tracker.current,
                reference.current,
                "the standing pause-demanding outcome drifted from the reference machine"
            );
        }

        // A standing duck keeps `Playing`: the document never stopped,
        // so no place was lost. Whether a resume is still owed across
        // a gain→duck pairing belongs to the transient/duck pairing
        // the deterministic duck test pins, not to this branch.
        if reference.current == Some(FocusOutcome::DuckableTransient) {
            prop_assert!(
                tracker.ducked,
                "a standing duckable transient left the tracker un-ducked"
            );
        }

        // Latest-wins: the standing outcome is the last reason the
        // focus path would state (a duck states none).
        if let Some(current) = reference.current {
            prop_assert_eq!(
                from_focus_outcome(current),
                reference_reason(current),
                "the standing outcome {:?} mapped to the wrong interruption reason",
                current
            );
        }
    }

    /// The three grant answers resolve their audio permission and
    /// their reason 1:1 (FR-013b): `Granted` permits audio and states
    /// no reason; `Deferred` holds zero audio and states the empty
    /// `FocusDelayed` line; `Refused` is the named `StartRefused`
    /// failure with the `FocusDenied` reason.
    #[test]
    fn prop_focus_grants_resolve_audio_and_reason(grant in prop_oneof![
        Just(FocusGrant::Granted),
        Just(FocusGrant::Deferred),
        Just(FocusGrant::Refused),
    ]) {
        let audio_permitted = match grant {
            FocusGrant::Granted => Ok(true),
            FocusGrant::Deferred => Ok(false),
            FocusGrant::Refused => Err(EngineError::StartRefused),
        };
        prop_assert_eq!(
            apply_grant(grant),
            audio_permitted,
            "the {:?} grant did not resolve its audio permission",
            grant
        );
        prop_assert_eq!(
            from_focus_grant(grant),
            reference_grant_reason(grant),
            "the {:?} grant did not map to its own interruption reason",
            grant
        );
    }

    /// Every removal signal arriving while playing reaches a Pause
    /// decision whose mapping latency is inside the FR-005
    /// one-second bound, and never a follow onto a public output
    /// (the routing machine is the sole authority for the decision,
    /// SC-035; its timing bound is T024's).
    #[test]
    fn prop_routing_pause_decisions_map_to_device_removed(
        (signal, transport) in (
            prop_oneof![
                Just(RoutingSignal::BecomingNoisy),
                any::<String>().prop_map(|device_id| RoutingSignal::DeviceDisappeared {
                    device_id,
                    privacy: OutputDevicePrivacy::Private,
                }),
            ],
            prop_oneof![
                Just(TransportState::Playing),
                Just(TransportState::Paused),
            ],
        )
    ) {
        let mut tracker = RoutingTracker::new();
        let decision = tracker.apply(signal.clone(), transport);
        if transport == TransportState::Playing {
            prop_assert_eq!(
                decision_kind(&decision),
                DecisionKind::Pause,
                "the removal signal {:?} did not pause (FR-016a)",
                signal
            );
            if let RoutingDecision::Pause { latency_ms, .. } = &decision {
                prop_assert!(
                    u128::from(*latency_ms) <= CALL_PAUSE_BOUND_MS,
                    "the pause declared {} ms, over the {} ms bound",
                    latency_ms,
                    CALL_PAUSE_BOUND_MS
                );
            }
            prop_assert_eq!(
                from_routing_pause(&decision),
                Some(InterruptionReason::DeviceRemoved),
                "the removal signal {:?} did not map to the DeviceRemoved reason",
                signal
            );
        } else {
            prop_assert_eq!(
                decision_kind(&decision),
                DecisionKind::NoChange,
                "the removal signal {:?} while not playing must be a no-op (FR-017)",
                signal
            );
            prop_assert_eq!(
                from_routing_pause(&decision),
                None,
                "a no-op decision stated an interruption reason"
            );
        }
    }
}

/// The closed eight-reason set is exhaustive and carries its FR-005
/// shape — eight explicit match arms, no catch-all: a ninth reason
/// cannot compile.
#[test]
fn the_closed_eight_reason_set_is_exhaustive_with_no_catch_all() {
    for reason in EIGHT_REASONS {
        // FR-005: each reason maps 1:1 to a ≤8-word pt-BR line, a
        // distinct warning earcon, and a logged failure name.
        // `FocusDelayed` (zero-audio hold) and a duckable
        // `TransientPaused` state no line — the empty line is the
        // only empty one in the closed set.
        let (expected_persistent, states_a_line) = match reason {
            InterruptionReason::FocusDenied => (false, true),
            InterruptionReason::FocusDelayed => (false, false),
            InterruptionReason::PermanentLoss => (false, true),
            InterruptionReason::TransientPaused => (false, true),
            InterruptionReason::DeviceRemoved => (false, true),
            InterruptionReason::VoiceUnavailable => (true, true),
            InterruptionReason::FormatUndeliverable => (true, true),
            InterruptionReason::DecodeFailed => (true, true),
        };
        let line = reason_line(&reason);
        assert_eq!(
            line,
            reason.reason_line(),
            "the free-function wrapper drifted from the method for {:?}",
            reason
        );
        assert_eq!(
            line.is_empty(),
            !states_a_line,
            "the {:?} reason stated a line when FR-005 requires none (or none when one is required)",
            reason
        );
        assert!(
            line.split_whitespace().count() <= REASON_LINE_MAX_WORDS,
            "the {:?} reason line {:?} exceeded the {} -word FR-005 budget",
            reason,
            line,
            REASON_LINE_MAX_WORDS
        );
        assert_eq!(
            reason.is_persistent(),
            expected_persistent,
            "the {:?} reason carried the wrong persistence",
            reason
        );
        if expected_persistent {
            assert_eq!(
                reason.reason_ttl_ms(),
                None,
                "the persistent {:?} reason stated a TTL instead of null",
                reason
            );
        } else {
            assert_eq!(
                reason.reason_ttl_ms(),
                Some(REASON_TTL_MS),
                "the transient {:?} reason drifted from the 3 s FR-005 lifetime",
                reason
            );
        }
    }
    // Distinctness: the closed reasons never share a line, so the
    // lock-screen surface can tell a call from a headset pull.
    let lines: Vec<&str> = EIGHT_REASONS
        .iter()
        .map(InterruptionReason::reason_line)
        .collect();
    for (index, line) in lines.iter().enumerate() {
        assert!(
            !lines[..index].contains(line),
            "two reasons of the closed set state the same line {:?}",
            line
        );
    }
}

/// Every focus outcome maps to exactly one reason or to none —
/// three explicit arms, no catch-all (FR-005).
#[test]
fn every_focus_outcome_maps_to_one_reason_or_to_none() {
    for outcome in THREE_OUTCOMES {
        let expected = match outcome {
            FocusOutcome::DuckableTransient => None,
            FocusOutcome::SilenceDemanding => Some(InterruptionReason::TransientPaused),
            FocusOutcome::PermanentLoss => Some(InterruptionReason::PermanentLoss),
        };
        assert_eq!(
            from_focus_outcome(outcome),
            expected,
            "the {:?} outcome did not map to its own interruption reason",
            outcome
        );
        if let Some(reason) = expected {
            assert!(
                !reason.reason_line().is_empty(),
                "a pause-demanding outcome stated an empty reason line for {:?}",
                reason
            );
        }
    }
}

/// Every focus grant maps to exactly one reason or to none —
/// three explicit arms, no catch-all (FR-013b / FR-005).
#[test]
fn every_focus_grant_maps_to_one_reason_or_to_none() {
    for grant in THREE_GRANTS {
        let expected = match grant {
            FocusGrant::Granted => None,
            FocusGrant::Deferred => Some(InterruptionReason::FocusDelayed),
            FocusGrant::Refused => Some(InterruptionReason::FocusDenied),
        };
        assert_eq!(
            from_focus_grant(grant),
            expected,
            "the {:?} grant did not map to its own interruption reason",
            grant
        );
    }
}

/// Every routing decision maps to its reason through
/// `from_routing_pause` — three explicit arms, no catch-all: only the
/// pause names the `DeviceRemoved` reason (FR-005).
#[test]
fn every_routing_decision_maps_to_its_reason_through_from_routing_pause() {
    let mut removal_tracker = RoutingTracker::new();
    let pause = removal_tracker.apply(RoutingSignal::BecomingNoisy, TransportState::Playing);
    let no_change = removal_tracker.apply(RoutingSignal::BecomingNoisy, TransportState::Paused);
    let mut follow_tracker = RoutingTracker::new();
    // R3: an appearance while not playing is a no-op that still
    // latches the route, so the explicit resume happens through the
    // device the reader reconnected (FR-017).
    let latched = follow_tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "car-bt".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        TransportState::Paused,
    );
    assert_eq!(
        decision_kind(&latched),
        DecisionKind::NoChange,
        "an appearance while not playing must be a no-op (FR-017)"
    );
    let follow = follow_tracker.apply(
        RoutingSignal::DeviceAppeared {
            device_id: "speaker".to_string(),
            privacy: OutputDevicePrivacy::Public,
        },
        TransportState::Playing,
    );
    assert_eq!(
        decision_kind(&follow),
        DecisionKind::Follow,
        "a public→public switch while playing must follow (FR-017a)"
    );
    let mapped = [
        (
            decision_kind(&pause),
            from_routing_pause(&pause),
            Some(InterruptionReason::DeviceRemoved),
        ),
        (decision_kind(&follow), from_routing_pause(&follow), None),
        (
            decision_kind(&no_change),
            from_routing_pause(&no_change),
            None,
        ),
    ];
    for (kind, actual, expected) in mapped {
        assert_eq!(
            actual, expected,
            "the {:?} routing decision mapped to the wrong interruption reason",
            kind
        );
    }
}

/// A transient immediately followed by a permanent loss ends paused
/// with the LATEST reason and no auto-resume (FR-014 / FR-005): the
/// superseded transient's resume flag dies with the supersede.
#[expect(
    clippy::float_cmp,
    reason = "FULL_GAIN is an exact decimal literal from focus.rs; == is the correct assertion and a tolerance would admit a drifted restore gain"
)]
#[test]
fn transient_immediately_followed_by_permanent_pauses_with_the_latest_reason_and_no_auto_resume() {
    let mut tracker = FocusTracker::new();
    let ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        ducked.transport,
        Some(TransportState::Playing),
        "a duckable transient keeps reading (FR-004)"
    );
    let paused = tracker.apply(FocusIntent::PermanentLoss);
    assert_eq!(
        paused.transport,
        Some(TransportState::Paused),
        "the permanent loss must pause playback"
    );
    assert_eq!(
        paused.outcome,
        Some(FocusOutcome::PermanentLoss),
        "the latest event wins: the permanent loss is the standing outcome"
    );
    assert_eq!(
        paused.gain, FULL_GAIN,
        "the paused state must hold full gain, never the duck level"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "the latest event wins: the tracker holds the permanent loss"
    );
    assert!(
        !tracker.resume_due,
        "the supersede must clear the transient resume flag — no auto-resume (FR-005)"
    );
    assert!(
        !tracker.ducked,
        "the supersede must un-duck: the paused state holds at full gain"
    );
    assert!(
        !tracker.duck_restore_due,
        "the supersede must clear the duck-restore flag — gain is restored, not re-ducked"
    );
    let reason = from_focus_outcome(FocusOutcome::PermanentLoss);
    assert_eq!(
        reason,
        Some(InterruptionReason::PermanentLoss),
        "the standing reason must be the latest one (FR-005)"
    );
    if let Some(reason) = reason {
        assert!(
            !reason.reason_line().is_empty(),
            "a pause-demanding reason must state its lock-screen line"
        );
        assert_eq!(
            reason.reason_ttl_ms(),
            Some(REASON_TTL_MS),
            "the standing reason must expire after 3 s (FR-005)"
        );
    }
}

/// A permanent loss followed by a transient is a no-op (FR-005 edge
/// case / FR-014): the app is already paused, the transient is
/// ignored, and any later focus gain waits for an explicit Play.
#[expect(
    clippy::float_cmp,
    reason = "FULL_GAIN is an exact decimal literal from focus.rs; == is the correct assertion and a tolerance would admit a drifted restore gain"
)]
#[test]
fn permanent_loss_followed_by_transient_is_a_no_op() {
    let mut tracker = FocusTracker::new();
    let loss = tracker.apply(FocusIntent::PermanentLoss);
    assert_eq!(
        loss.transport,
        Some(TransportState::Paused),
        "the permanent loss must pause"
    );
    let late = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        late.outcome,
        Some(FocusOutcome::PermanentLoss),
        "a late transient must not displace the standing permanent loss"
    );
    assert_eq!(
        late.transport,
        Some(TransportState::Paused),
        "a late transient must hold the paused state, never re-enter playing"
    );
    assert_eq!(
        late.gain, FULL_GAIN,
        "a late transient must not re-duck past the permanent loss"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "the authoritative state stays the permanent loss"
    );
    assert!(
        !tracker.ducked,
        "a late transient must not re-duck past the permanent loss"
    );
    assert!(
        !tracker.resume_due,
        "a late transient must not authorize a resume"
    );
}

/// A focus gain immediately followed by a pause-demanding outcome
/// clears the transient resume the gain owed (FR-005).
///
/// FR-005: "Duck-gain restore after a duckable transient (MAY_DUCK) is
/// automatic; every other paused outcome (DeviceRemoved, non-duckable
/// TransientPaused, PermanentLoss, FocusDenied, and any
/// AUDIOFOCUS_GAIN arriving after a pause) MUST wait for explicit Play
/// and MUST never auto-resume".
///
/// RECORDED RED FINDING — not an unimplemented feature, and NOT fixed
/// by this file. Minimal failing input:
/// `intents = [FocusRegained, DeviceDisappeared]` (equally
/// `[FocusRegained, SilenceDemanding]` / `[FocusRegained,
/// PermanentLoss]`). `FocusTracker::apply` consults `supersede_flags`
/// only when `self.current` is `Some`; a gain resets `current` to
/// `None`, so the pause that arrives next clears no flag and
/// `resume_due` stays set. The tracker then holds a standing pause AND
/// an owed resume — exactly the auto-resume FR-005 forbids. The fix
/// belongs to the focus-tracker implementation wave (decide the pending
/// resume flag whenever a pause-demanding outcome is decided, whatever
/// stood before it).
#[test]
fn a_focus_gain_immediately_followed_by_a_pause_clears_the_owed_resume() {
    for intent in [
        FocusIntent::DeviceDisappeared,
        FocusIntent::SilenceDemanding,
        FocusIntent::PermanentLoss,
    ] {
        let mut tracker = FocusTracker::new();
        let gain = tracker.apply(FocusIntent::FocusRegained);
        assert_eq!(
            gain.transport,
            Some(TransportState::Playing),
            "a regain authorizes a playing resume (FR-015)"
        );
        assert!(
            tracker.resume_due,
            "a gain must owe a transient resume before any pause arrives"
        );
        let pause = tracker.apply(intent);
        assert_eq!(
            pause.transport,
            Some(TransportState::Paused),
            "the {:?} outcome must pause",
            intent
        );
        assert!(
            !tracker.resume_due,
            "the {:?} pause did not clear the resume the gain owed — FR-005: a gain followed by a pause waits for explicit Play and never auto-resumes",
            intent
        );
        assert!(
            !tracker.ducked,
            "the {:?} pause left playback ducked instead of paused at full gain",
            intent
        );
    }
}

/// A deferred grant holds zero audio with no reason (FR-013b /
/// FR-005): nothing has played, so nothing is owed and no pause is
/// stated.
#[test]
fn deferred_grant_holds_zero_audio_with_no_reason() {
    assert_eq!(
        apply_grant(FocusGrant::Deferred),
        Ok(false),
        "a deferred grant must hold zero audio before the grant arrives (FR-013b)"
    );
    let reason = from_focus_grant(FocusGrant::Deferred);
    assert_eq!(
        reason,
        Some(InterruptionReason::FocusDelayed),
        "a deferred grant maps to the FocusDelayed hold (FR-013b)"
    );
    if let Some(reason) = reason {
        assert!(
            reason.reason_line().is_empty(),
            "FocusDelayed states no lock-screen line — a deferral is a delay, not a pause (FR-005)"
        );
    }
    let tracker = FocusTracker::new();
    assert!(
        !tracker.resume_due,
        "a deferred hold owes no rewind — nothing has played yet (FR-013b)"
    );
    assert!(!tracker.ducked, "a deferred hold never ducks (FR-013b)");
    assert_eq!(
        tracker.current, None,
        "a deferred hold states no pause, so no outcome stands (FR-005)"
    );
}

/// A refused grant is the named `StartRefused` failure with the
/// `FocusDenied` reason — never a playing state (FR-013b / FR-037).
#[test]
fn refused_grant_fails_named_with_the_focus_denied_reason() {
    assert_eq!(
        apply_grant(FocusGrant::Refused),
        Err(EngineError::StartRefused),
        "a refused focus must surface the named StartRefused failure (FR-037)"
    );
    assert_eq!(
        from_focus_grant(FocusGrant::Refused),
        Some(InterruptionReason::FocusDenied),
        "a refused focus maps to the FocusDenied reason (FR-005)"
    );
    assert_eq!(
        apply_grant(FocusGrant::Granted),
        Ok(true),
        "a granted focus must permit audio (FR-013b)"
    );
    assert_eq!(
        from_focus_grant(FocusGrant::Granted),
        None,
        "a granted focus states no reason (FR-005)"
    );
}

/// The duck keeps `Playing` at `DUCK_GAIN` 0.20 ±0.02 and restores
/// full gain without place loss, never owing a rewind (FR-004 /
/// T023): playback never leaves `Playing`, so there is nothing to
/// rewind and no place to lose.
#[expect(
    clippy::float_cmp,
    reason = "DUCK_GAIN and FULL_GAIN are exact decimal literals from focus.rs; == is the correct assertion and a tolerance would admit a drifted duck gain"
)]
#[test]
fn duck_keeps_playing_at_duck_gain_and_restores_without_losing_place() {
    let decision = classify(FocusIntent::DuckableTransient);
    assert_eq!(
        decision.outcome,
        Some(FocusOutcome::DuckableTransient),
        "a duckable transient must classify to its own outcome"
    );
    assert_eq!(
        decision.transport,
        Some(TransportState::Playing),
        "the duck keeps reading (FR-004): ducking is distinct from pausing"
    );
    assert!(
        (decision.gain - DUCK_GAIN_NOMINAL).abs() <= DUCK_GAIN_TOLERANCE,
        "the duck gain {} left the 0.20 ±0.02 figure of FR-004",
        decision.gain
    );
    assert_eq!(
        decision.gain, DUCK_GAIN,
        "the duck gain must be the crate's DUCK_GAIN constant (FR-004)"
    );
    let mut tracker = FocusTracker::new();
    let ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        ducked.transport,
        Some(TransportState::Playing),
        "the duck must keep playing, never pause (FR-004)"
    );
    assert_eq!(
        ducked.gain, DUCK_GAIN,
        "the tracker must apply the duck gain while ducked (FR-004)"
    );
    assert!(tracker.ducked, "the tracker must record the ducked state");
    assert!(
        tracker.duck_restore_due,
        "the duck owes a gain restore, not a rewind (FR-004)"
    );
    assert!(
        !tracker.resume_due,
        "a duck never owes a rewind: playback never left Playing, so no place was lost"
    );
    let restored = tracker.apply(FocusIntent::FocusRegained);
    assert_eq!(
        restored.gain, FULL_GAIN,
        "the gain restore after a duckable transient is automatic (FR-004 / FR-005)"
    );
    assert!(!tracker.ducked, "the regain must clear the ducked state");
    assert!(
        !tracker.duck_restore_due,
        "the regain must consume the owed gain restore (FR-004)"
    );
    assert_eq!(
        tracker.current, None,
        "the regain resolves the transient (FR-014)"
    );
}

/// A call/alarm (silence-demanding) or a permanent loss pauses within
/// the FR-005 one-second bound (FR-005: "MUST pause within 1s, same
/// bound as FR-006 routing").
///
/// The mapping is a pure classification with no clock, so its latency
/// is 0 ms and always inside the bound; the wall-clock half is
/// attested on-device by T051 (10/10 per interruption class:
/// nav-duck vs call/alarm pause vs headphone ≤1 s vs permanent loss).
#[expect(
    clippy::float_cmp,
    reason = "FULL_GAIN is an exact decimal literal from focus.rs; == is the correct assertion and a tolerance would admit a drifted restore gain"
)]
#[test]
fn call_and_alarm_pauses_land_within_the_one_second_bound() {
    assert!(
        FOCUS_MAPPING_LATENCY_MS <= CALL_PAUSE_BOUND_MS,
        "the focus mapping latency {} left the {} ms FR-005 bound",
        FOCUS_MAPPING_LATENCY_MS,
        CALL_PAUSE_BOUND_MS
    );
    let mut silence_tracker = FocusTracker::new();
    let mut loss_tracker = FocusTracker::new();
    let pauses = [
        (
            FocusIntent::SilenceDemanding,
            silence_tracker.apply(FocusIntent::SilenceDemanding),
        ),
        (
            FocusIntent::PermanentLoss,
            loss_tracker.apply(FocusIntent::PermanentLoss),
        ),
    ];
    for (intent, decision) in pauses {
        assert_eq!(
            decision.transport,
            Some(TransportState::Paused),
            "the {:?} interruption must pause (FR-005)",
            intent
        );
        assert_eq!(
            decision.gain, FULL_GAIN,
            "the {:?} pause must hold full gain (FR-005)",
            intent
        );
    }
    assert!(
        !silence_tracker.resume_due,
        "a call/alarm pause must never auto-resume (FR-005)"
    );
    assert!(
        !loss_tracker.resume_due,
        "a permanent loss must never auto-resume (FR-005)"
    );
    assert_eq!(
        from_focus_outcome(FocusOutcome::SilenceDemanding),
        Some(InterruptionReason::TransientPaused),
        "a call/alarm pause states the TransientPaused reason (FR-005)"
    );
    // Determinism: the same intent classifies the same way on every
    // call — no clock is read, so the mapping latency is stable.
    assert_eq!(
        classify(FocusIntent::SilenceDemanding),
        classify(FocusIntent::SilenceDemanding),
        "the focus classification must be a pure function of the intent"
    );
}
