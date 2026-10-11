//! T020 — US1 session-lifecycle property tests (Phase 3, BLOCKING).
//!
//! tasks.md T020, verbatim: "Property test: session lifecycle
//! transitions + FR-020 transport-state reflection (proptest
//! over closed `TransportState`: allowed
//! `Playing→Paused|Interrupted|Stopped|Completed`,
//! `Paused→Playing`, `Interrupted→Playing` only via FR-023a,
//! FR-032c expiries or explicit action, any→`Stopped`;
//! transport controls MUST reflect actual state per FR-020 —
//! NEVER playing-while-silent; FAIL before implementation)".
//!
//! Spec anchors:
//! - data-model.md `PlaybackSession` transition table — the
//!   closed list this file's reference model implements
//!   exactly: `Playing → Paused` (pause, transient-silence
//!   demand, BECOMING_NOISY, device loss); `Playing →
//!   Interrupted` (permanent loss, engine failure pending
//!   offer); `Interrupted → Playing` (offer choice or
//!   fallback takeover, same position); `Paused → Playing`
//!   (resume); `any → Stopped` (Stop press); `Playing →
//!   Completed` (document end → clear stored position).
//!   "Never: `Interrupted → Playing` without user action
//!   except the FR-023a/FR-032c bounded expiries; never any
//!   transition that reports playing while silent (FR-005)."
//! - FR-020: "Transport controls MUST reflect actual playback
//!   state at all times, and MUST NOT display a playing
//!   state while playback is paused."
//! - FR-005: "The system MUST report a playing state while
//!   producing no audible output" is the failure mode; the
//!   system MUST NOT do it.
//! - FR-044 / Principle VII: the sets are closed — the
//!   reference transition function is an exhaustive match
//!   over the five states × five lifecycle verbs with no
//!   catch-all arm, so a state or verb added without a
//!   table entry cannot compile silently.
//! - T019 convention (`prop_transport_math.rs`): a press
//!   that is not a legal transition is an
//!   `IllegalTransition` the driver discards — the session
//!   must reject it, not move.
//!
//! The reference model: the five lifecycle verbs (`Play`,
//! `Pause`, `Stop`, `Complete`, `Interrupt`) over the closed
//! five-state set. Ten cells are legal transitions — the
//! table above plus the idempotent `Stopped → Stop →
//! Stopped` press T019 documents; the remaining fifteen
//! cells are illegal and must be rejected with
//! `PlayerError::IllegalTransition`, leaving both the
//! transport and the position untouched. `Stop` and
//! `Complete` discard the stored position (data-model.md:
//! stopped and completed delete the stored entry); every
//! other legal press retains it.
//!
//! Red state: the T027 dispatch currently applies every
//! verb from every state — `PlayerError::IllegalTransition`
//! is defined but never constructed — so the fifteen illegal
//! cells are red until transition validation lands (the T031
//! green gate). The ten legal cells hold today; the rejection
//! cells are the required failing assertions of this test.

use proptest::prelude::*;

use engine_core::closed::TransportState;
use player::cli::{PlaybackSession, PlayerError, ShellVerb};

/// Audible progress fed while driving a session into a state
/// — an exact binary fraction, so every position comparison
/// in this file is bit-exact (a tolerance would admit an
/// off-table position as conforming).
const AUDIBLE_PROGRESS_SECS: f64 = 10.0;

/// One press of the closed lifecycle-verb set this task
/// drives (tasks.md T020: `Play|Pause|Stop|Complete|
/// Interrupt`). Every surface — notification, lock screen,
/// CLI, IPC — issues exactly these verbs through the single
/// dispatch (FR-042).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleAction {
    /// Resume playback (or start it from the initial
    /// `Paused`).
    Play,
    /// Pause playback, retaining the position.
    Pause,
    /// End the session and discard the stored position.
    Stop,
    /// End the session at document end, discarding the
    /// stored position.
    Complete,
    /// Enter the interrupted state (focus loss, engine
    /// failure pending offer).
    Interrupt,
}

impl LifecycleAction {
    /// The shell verb the action presses — the closed
    /// five-verb subset of `ShellVerb` (the full verb set
    /// also carries rate, skip, and resume verbs, which no
    /// lifecycle transition depends on).
    fn verb(self) -> ShellVerb {
        match self {
            LifecycleAction::Play => ShellVerb::Play,
            LifecycleAction::Pause => ShellVerb::Pause,
            LifecycleAction::Stop => ShellVerb::Stop,
            LifecycleAction::Complete => ShellVerb::Complete,
            LifecycleAction::Interrupt => ShellVerb::Interrupt,
        }
    }
}

/// The reference verdict for one verb press — the closed
/// transition function of data-model.md, computed
/// independently of the implementation under test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceVerdict {
    /// The press is legal: the transport lands on
    /// `transport`, and the stored position is discarded
    /// exactly when `resets_position` (`Stop` and
    /// `Complete` discard it; every other legal press
    /// retains it).
    Lands {
        /// The transport state the press must land on.
        transport: TransportState,
        /// Whether the press discards the stored position.
        resets_position: bool,
    },
    /// The press is illegal from this state: it must be
    /// rejected with [`PlayerError::IllegalTransition`]
    /// and leave both the transport and the position
    /// untouched (the T019 convention — a driver may
    /// press a verb twice; the second, no-op press is
    /// discarded).
    Rejected,
}

/// The reference transition function: the data-model.md
/// table as an exhaustive five-state × five-verb match
/// over the closed sets — no catch-all arm (Principle VII
/// / FR-044), so a new state or verb forces a table entry
/// here before anything can compile.
#[expect(
    clippy::match_same_arms,
    reason = "Principle VII / FR-044: the closed 5x5 transition table's fifteen illegal cells all share the Rejected verdict by design; enumerating them explicitly (rather than a catch-all arm) is what makes an added state or verb a compile error"
)]
fn reference_transition(current: TransportState, action: LifecycleAction) -> ReferenceVerdict {
    match (current, action) {
        // Playing: exactly {Paused, Interrupted, Stopped,
        // Completed} is reachable (tasks.md T020).
        (TransportState::Playing, LifecycleAction::Pause) => ReferenceVerdict::Lands {
            transport: TransportState::Paused,
            resets_position: false,
        },
        (TransportState::Playing, LifecycleAction::Interrupt) => ReferenceVerdict::Lands {
            transport: TransportState::Interrupted,
            resets_position: false,
        },
        (TransportState::Playing, LifecycleAction::Stop) => ReferenceVerdict::Lands {
            transport: TransportState::Stopped,
            resets_position: true,
        },
        (TransportState::Playing, LifecycleAction::Complete) => ReferenceVerdict::Lands {
            transport: TransportState::Completed,
            resets_position: true,
        },
        // A second Play while already playing is not a
        // transition (the T019 double-press convention).
        (TransportState::Playing, LifecycleAction::Play) => ReferenceVerdict::Rejected,
        // Paused: resume (Playing) and the universal Stop.
        (TransportState::Paused, LifecycleAction::Play) => ReferenceVerdict::Lands {
            transport: TransportState::Playing,
            resets_position: false,
        },
        (TransportState::Paused, LifecycleAction::Stop) => ReferenceVerdict::Lands {
            transport: TransportState::Stopped,
            resets_position: true,
        },
        // Already paused — no transition exists: a document
        // ends from playback, not from a pause, and an
        // already-silent session cannot be interrupted into
        // a new state.
        (TransportState::Paused, LifecycleAction::Pause) => ReferenceVerdict::Rejected,
        (TransportState::Paused, LifecycleAction::Complete) => ReferenceVerdict::Rejected,
        (TransportState::Paused, LifecycleAction::Interrupt) => ReferenceVerdict::Rejected,
        // Interrupted: Playing is reachable only through an
        // explicit action — the Play press (offer choice or
        // fallback takeover, data-model.md). The FR-023a /
        // FR-032c expiries are the only automatic paths and
        // are not session verbs; the session must reject
        // every other route, including a spontaneous resume.
        (TransportState::Interrupted, LifecycleAction::Play) => ReferenceVerdict::Lands {
            transport: TransportState::Playing,
            resets_position: false,
        },
        (TransportState::Interrupted, LifecycleAction::Stop) => ReferenceVerdict::Lands {
            transport: TransportState::Stopped,
            resets_position: true,
        },
        (TransportState::Interrupted, LifecycleAction::Pause) => ReferenceVerdict::Rejected,
        (TransportState::Interrupted, LifecycleAction::Complete) => ReferenceVerdict::Rejected,
        (TransportState::Interrupted, LifecycleAction::Interrupt) => ReferenceVerdict::Rejected,
        // Stopped is terminal: only the idempotent Stop
        // press (T019) is legal.
        (TransportState::Stopped, LifecycleAction::Stop) => ReferenceVerdict::Lands {
            transport: TransportState::Stopped,
            resets_position: true,
        },
        (TransportState::Stopped, LifecycleAction::Play) => ReferenceVerdict::Rejected,
        (TransportState::Stopped, LifecycleAction::Pause) => ReferenceVerdict::Rejected,
        (TransportState::Stopped, LifecycleAction::Complete) => ReferenceVerdict::Rejected,
        (TransportState::Stopped, LifecycleAction::Interrupt) => ReferenceVerdict::Rejected,
        // Completed is terminal except Stop (tasks.md T020).
        (TransportState::Completed, LifecycleAction::Stop) => ReferenceVerdict::Lands {
            transport: TransportState::Stopped,
            resets_position: true,
        },
        (TransportState::Completed, LifecycleAction::Play) => ReferenceVerdict::Rejected,
        (TransportState::Completed, LifecycleAction::Pause) => ReferenceVerdict::Rejected,
        (TransportState::Completed, LifecycleAction::Complete) => ReferenceVerdict::Rejected,
        (TransportState::Completed, LifecycleAction::Interrupt) => ReferenceVerdict::Rejected,
    }
}

/// The closed lifecycle-action strategy: uniform over the
/// five verbs (tasks.md T020).
fn lifecycle_action_strategy() -> impl Strategy<Value = LifecycleAction> {
    prop_oneof![
        Just(LifecycleAction::Play),
        Just(LifecycleAction::Pause),
        Just(LifecycleAction::Stop),
        Just(LifecycleAction::Complete),
        Just(LifecycleAction::Interrupt),
    ]
}

/// Generated verb sequences, as any surface might issue
/// them — notification, lock screen, CLI, or IPC (all
/// route through the same dispatch, FR-042).
fn lifecycle_action_vec_strategy() -> impl Strategy<Value = Vec<LifecycleAction>> {
    proptest::collection::vec(lifecycle_action_strategy(), 1..=48)
}

/// The closed transport-state set (FR-044) — exactly the
/// five variants, so iterating it exercises every cell of
/// the transition table.
const CLOSED_STATES: [TransportState; 5] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Stopped,
    TransportState::Completed,
    TransportState::Interrupted,
];

/// The closed lifecycle-action set (tasks.md T020).
const CLOSED_ACTIONS: [LifecycleAction; 5] = [
    LifecycleAction::Play,
    LifecycleAction::Pause,
    LifecycleAction::Stop,
    LifecycleAction::Complete,
    LifecycleAction::Interrupt,
];

/// A fresh session driven deterministically into `state` —
/// the only legal route through the table (`Paused` is the
/// FR-019b default; `Interrupted` passes through `Playing`
/// and is fed audible progress so the retained position is
/// observable; `Stopped` and `Completed` are terminal and
/// hold no progress).
fn session_in(state: TransportState) -> PlaybackSession {
    let mut session = PlaybackSession::new();
    match state {
        TransportState::Paused => {
            // The FR-019b default: a new session opens
            // Paused at position 0.0.
        }
        TransportState::Playing => {
            let _playing = session.execute(ShellVerb::Play);
            session.on_audible_progress(AUDIBLE_PROGRESS_SECS);
        }
        TransportState::Interrupted => {
            let _playing = session.execute(ShellVerb::Play);
            session.on_audible_progress(AUDIBLE_PROGRESS_SECS);
            let _interrupted = session.execute(ShellVerb::Interrupt);
        }
        TransportState::Stopped => {
            let _stopped = session.execute(ShellVerb::Stop);
        }
        TransportState::Completed => {
            let _completed = session.execute(ShellVerb::Complete);
        }
    }
    session
}

/// The position a session driven into `state` holds: the
/// audible progress fed while passing through `Playing`
/// (retained by `Paused`, `Playing`, and `Interrupted`),
/// or `0.0` where the session never held or discarded it.
fn position_in(state: TransportState) -> f64 {
    match state {
        TransportState::Playing | TransportState::Interrupted => AUDIBLE_PROGRESS_SECS,
        TransportState::Paused | TransportState::Stopped | TransportState::Completed => 0.0,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_driven_sequence_tracks_the_reference_transition_table(
        actions in lifecycle_action_vec_strategy()
    ) {
        // FR-020: after every verb press the transport must
        // equal the state the transition table predicts —
        // the reference function above computes that
        // prediction independently of the implementation,
        // and the session must agree at every step. FR-005:
        // within a sequence that never fed audible progress,
        // `Playing` may be reported only by an explicit Play
        // press from a not-playing state — never on its own.
        let mut session = PlaybackSession::new();
        let mut before = session.snapshot();
        prop_assert_eq!(
            before.transport,
            TransportState::Paused,
            "a new session must open Paused (FR-019b), never Playing"
        );
        prop_assert_eq!(
            before.position_secs,
            0.0,
            "a new session must open at position 0.0"
        );
        for action in &actions {
            let current = before.transport;
            let verdict = reference_transition(current, *action);
            let result = session.execute(action.verb());
            let after = session.snapshot();
            match verdict {
                ReferenceVerdict::Lands {
                    transport,
                    resets_position,
                } => {
                    prop_assert!(
                        result.is_ok(),
                        "a legal {action:?} press from {current} was rejected"
                    );
                    prop_assert_eq!(
                        after.transport,
                        transport,
                        "a legal {:?} press from {} must land the transport on {}",
                        action,
                        current,
                        transport
                    );
                    if resets_position {
                        prop_assert_eq!(
                            after.position_secs,
                            0.0,
                            "a legal {:?} press must discard the stored position",
                            action
                        );
                    } else {
                        prop_assert_eq!(
                            after.position_secs,
                            before.position_secs,
                            "a legal {:?} press must retain the position",
                            action
                        );
                    }
                    // FR-005: within this no-progress sequence,
                    // Playing is reportable only by an explicit
                    // Play press from a not-playing state.
                    if transport == TransportState::Playing {
                        prop_assert!(
                            matches!(*action, LifecycleAction::Play),
                            "the transport reported Playing from {current} without an explicit Play press — a playing-while-silent report (FR-005, FR-020)"
                        );
                    }
                }
                ReferenceVerdict::Rejected => {
                    prop_assert!(
                        matches!(result, Err(PlayerError::IllegalTransition { .. })),
                        "an illegal {action:?} press from {current} was accepted — the transition table forbids it"
                    );
                    prop_assert_eq!(
                        after.transport,
                        current,
                        "an illegal {:?} press moved the transport from {}",
                        action,
                        current
                    );
                    prop_assert_eq!(
                        after.position_secs,
                        before.position_secs,
                        "an illegal {:?} press moved the position",
                        action
                    );
                }
            }
            before = after;
        }
    }
}

/// FR-005: "The system MUST NOT report a playing state
/// while producing no audible output." A session that has
/// never been played and has never received audible
/// progress reports the FR-019b default `Paused` — never
/// `Playing`.
#[test]
fn a_session_that_never_received_audible_progress_never_reports_playing() {
    let session = PlaybackSession::new();
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "a session that never played must report Paused, not Playing"
    );
    assert_ne!(
        snapshot.transport,
        TransportState::Playing,
        "a silent session reported Playing (FR-005)"
    );
}

/// FR-020: the controls reflect the actual state — a
/// paused session reports `Paused` with the position it
/// had — and FR-005: audible progress arriving while
/// paused neither advances the position nor resurrects a
/// `Playing` report.
#[expect(
    clippy::float_cmp,
    reason = "the retained-position assertions compare exact binary fractions (12.5 and 5.0 s); == is exact and a tolerance would admit a clock that advances while the session is silent (FR-005)"
)]
#[test]
fn a_paused_session_reports_paused_with_the_position_it_had() {
    let mut session = PlaybackSession::new();
    let played = session.execute(ShellVerb::Play);
    assert!(
        played.is_ok(),
        "Play must succeed from the initial Paused state"
    );
    session.on_audible_progress(12.5);
    let paused = session.execute(ShellVerb::Pause);
    assert!(paused.is_ok(), "Pause must succeed from Playing");
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "a paused session must report Paused (FR-020)"
    );
    assert_eq!(
        snapshot.position_secs, 12.5,
        "a paused session must retain the position it had"
    );
    // Progress while paused is silent: it must not advance
    // the position, and the transport must stay Paused.
    session.on_audible_progress(5.0);
    let after = session.snapshot();
    assert_eq!(
        after.transport,
        TransportState::Paused,
        "audible progress while paused must not resume playback (FR-005)"
    );
    assert_eq!(
        after.position_secs, 12.5,
        "audible progress while paused must not advance the position"
    );
}

/// `Interrupted → Playing` is reachable only through an
/// explicit action or the FR-023a / FR-032c expiries —
/// which are not session verbs. The automatic path
/// (audible progress arriving while interrupted) must be
/// rejected: the session stays `Interrupted`, the position
/// does not advance, and the transport never reports
/// `Playing` (data-model.md "Never" clause, FR-005).
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (0.0 and 5.0 s); == is exact and a tolerance would admit an automatic resume from Interrupted"
)]
#[test]
fn an_interrupted_session_never_self_resumes() {
    let mut session = PlaybackSession::new();
    let _playing = session.execute(ShellVerb::Play);
    let interrupted = session.execute(ShellVerb::Interrupt);
    assert!(interrupted.is_ok(), "Interrupt must succeed from Playing");
    assert_eq!(
        session.snapshot().transport,
        TransportState::Interrupted,
        "Interrupt from Playing must land Interrupted"
    );
    // The automatic-transition probe: engine audio arrives
    // while Interrupted. It must not resume playback.
    session.on_audible_progress(5.0);
    let after = session.snapshot();
    assert_eq!(
        after.transport,
        TransportState::Interrupted,
        "audible progress while Interrupted must not self-resume to Playing"
    );
    assert_eq!(
        after.position_secs, 0.0,
        "audible progress while Interrupted must not advance the position"
    );
    // The only in-session path back to Playing is the
    // explicit Play press (offer choice or fallback
    // takeover, data-model.md).
    let resumed = session.execute(ShellVerb::Play);
    assert!(
        resumed.is_ok(),
        "an explicit Play must resume an Interrupted session"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Playing,
        "an explicit Play press must land Interrupted to Playing"
    );
}

/// The closed transition table, cell by cell: every one of
/// the five states × five lifecycle verbs is driven from a
/// session placed in that state and compared against the
/// reference verdict — the closed-set exhaustiveness check
/// (Principle VII / FR-044). A disagreeing cell is an
/// FR-020 defect: the transport controls would reflect a
/// state the transition table does not allow.
#[expect(
    clippy::float_cmp,
    reason = "the position assertions compare exact binary fractions (0.0 and 10.0 s, the reset and retained positions); == is exact and a tolerance would admit a position the table does not predict"
)]
#[test]
fn every_state_verb_cell_matches_the_reference_table() {
    for start in CLOSED_STATES {
        for action in CLOSED_ACTIONS {
            let mut session = session_in(start);
            let verdict = reference_transition(start, action);
            let result = session.execute(action.verb());
            match verdict {
                ReferenceVerdict::Lands {
                    transport,
                    resets_position,
                } => {
                    assert!(
                        result.is_ok(),
                        "a legal {action:?} press from {start} must succeed"
                    );
                    let after = session.snapshot();
                    assert_eq!(
                        after.transport, transport,
                        "a legal {action:?} press from {start} must land the transport on {transport}"
                    );
                    let expected_position = if resets_position {
                        0.0
                    } else {
                        position_in(start)
                    };
                    assert_eq!(
                        after.position_secs, expected_position,
                        "a legal {action:?} press from {start} must hold position {expected_position}"
                    );
                }
                ReferenceVerdict::Rejected => {
                    assert!(
                        matches!(result, Err(PlayerError::IllegalTransition { .. })),
                        "an illegal {action:?} press from {start} must be rejected"
                    );
                    let after = session.snapshot();
                    assert_eq!(
                        after.transport, start,
                        "an illegal {action:?} press from {start} must not move the transport"
                    );
                    assert_eq!(
                        after.position_secs,
                        position_in(start),
                        "an illegal {action:?} press from {start} must not move the position"
                    );
                }
            }
        }
    }
}
