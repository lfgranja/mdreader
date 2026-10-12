//! T054 — US4 contract: shell-IPC verb/state mirror
//! (`contracts/shell-ipc.md`).
//!
//! Invariants under test, from
//! `specs/001-android-media-shell/tasks.md` T054:
//!
//! - C1 every cell of the closed (transport × verb)
//!   table is honoured or rejected — five states ×
//!   fifteen verbs, no catch-all arm (FR-020 / FR-044
//!   / gate 7);
//! - C2 the transition table's shape follows the
//!   shell-ipc contract: Stop is the universal
//!   teardown press, Complete is legal only from
//!   Playing, the resume/start-over/external-start
//!   offers exist only from Paused and Interrupted,
//!   live-session controls need a live session, and
//!   ChooseEngine is a selection that changes nothing
//!   (T020 / FR-005 / FR-018 / FR-019 / FR-018b /
//!   FR-032 / FR-032f);
//! - C3 the mirror payload carries the session's
//!   authoritative position only — the Rust module is
//!   the single source of truth and the mirror is a
//!   view, never a second clock (FR-003 / SC-035);
//! - C4 the mirror position reflects the Rust module
//!   and never moves while silent (FR-002 / FR-005);
//! - C5 the snapshot and the mirror serialize to the
//!   contract's snake_case wire keys — exactly the
//!   enumerated key sets, nothing added (contracts/
//!   shell-ipc.md case-mapping table).
//!
//! Notes:
//!
//! - The verb set is the closed fifteen: the thirteen
//!   pre-existing verbs plus `CycleVoiceNext` /
//!   `CycleVoicePrev` (FR-018b), which T057 adds to
//!   `ShellVerb` — so the (transport × verb) product
//!   this contract enumerates is 5 × 15.
//! - The voice verbs are live-session controls at the
//!   player level: legal from Playing / Paused /
//!   Interrupted, changing no transport, rate, or
//!   position; rejected from the torn-down states
//!   (Stopped / Completed). The voice pool itself is
//!   the voice-selection interface (FR-024) and the
//!   FR-038 re-anchor belongs to the session crate
//!   (T059) — the one-voice-per-press walk is the
//!   integration trial's (T053).
//! - The mirror construction (`mirror_of`) is the
//!   test-side model of the shell-IPC mirror seam the
//!   Kotlin service consumes: it derives from a
//!   `SessionSnapshot` plus the duration knowledge
//!   fixed at session start (T051), and never
//!   computes a position of its own.
//!
//! FAILS before `crates/player/src/rate.rs` (T057)
//! and `crates/player/src/surface.rs` (T061) exist:
//! the `player::rate` and `player::surface` modules
//! are unresolved, the `CycleVoiceNext` /
//! `CycleVoicePrev` verb variants do not exist yet,
//! and `SessionSnapshot` / `MirrorState` do not
//! implement `Serialize` — so this test target does
//! not build. That build failure is the required
//! pre-implementation red state (T056).

use std::error::Error;

use engine_core::closed::{PlaybackRate, TransportState};
use player::cli::{PlaybackSession, PlayerError, SessionSnapshot, ShellVerb};
use player::position::SKIP_STEP_SECS;
use player::rate::{LADDER, step_down, step_up};
use player::surface::{MirrorState, Progress};

/// Position comparison tolerance, in seconds — the same
/// epsilon form the focus-trial integration tests use,
/// so no strict float comparison ever appears (clippy
/// `float_cmp`).
const POS_EPS_SECS: f64 = 0.001;

/// The trial document's duration, in seconds — a known
/// duration, so the mirror resolves a determinate
/// progress for the whole session (determinacy is
/// fixed at session start, T051).
const DURATION_SECS: f64 = 120.0;

/// The stored position every live-state session holds
/// when a cell under test runs: a known position so
/// the retain/discard contract is asserted against a
/// real value, not only against 0.0 (FR-032 /
/// FR-032a).
const STORED_POSITION_SECS: f64 = 30.0;

/// The expected outcome of one (transport, verb) cell
/// of the closed transition table — the contract the
/// shell relies on (`contracts/shell-ipc.md`).
///
/// Every variant is explicit (Principle VII / FR-044):
/// a new verb adds a variant here and a cell in
/// [`expected`] before anything compiles — no
/// catch-all arm exists.
#[derive(Debug, Clone, Copy)]
enum Expected {
    /// Land the transport on a new state, retaining
    /// the stored position — the resume plays (`Play`,
    /// `ResumeAtStored`, `ExternalStart`) and the
    /// silence demands (`Pause`, `Interrupt`).
    Land {
        /// The transport state the press lands on.
        transport: TransportState,
    },
    /// Land the transport on a new state, discarding
    /// the stored position — `Stop`, `Complete`, and
    /// `StartOver` delete the stored entry
    /// (data-model.md `StoredPosition`:
    /// "stopped|completed delete the entry").
    LandDiscarding {
        /// The transport state the press lands on.
        transport: TransportState,
    },
    /// A live-session control (FR-018 / FR-019 /
    /// FR-018b): no transport change; the position
    /// and the rate follow the verb's closed effect.
    Control,
    /// A selection, not a transition (`ChooseEngine`):
    /// legal from every state, changing nothing
    /// (FR-005).
    NoOp,
    /// Rejected as an illegal transition (FR-032f):
    /// the session does not move (T019).
    Rejected,
}

/// The closed transport set (FR-020) — every state, so
/// the loops below are the full product, never a
/// sample.
const ALL_TRANSPORTS: [TransportState; 5] = [
    TransportState::Playing,
    TransportState::Paused,
    TransportState::Interrupted,
    TransportState::Stopped,
    TransportState::Completed,
];

/// The closed verb set (`contracts/shell-ipc.md`) —
/// the thirteen pre-existing verbs plus the two
/// voice-cycle verbs (FR-018b). Adding a variant
/// breaks compilation here until every table below is
/// extended (FR-044 / gate 7).
const ALL_VERBS: [ShellVerb; 15] = [
    ShellVerb::Play,
    ShellVerb::Pause,
    ShellVerb::Stop,
    ShellVerb::Complete,
    ShellVerb::Interrupt,
    ShellVerb::RateUp,
    ShellVerb::RateDown,
    ShellVerb::SkipForward,
    ShellVerb::SkipBackward,
    ShellVerb::ResumeAtStored,
    ShellVerb::StartOver,
    ShellVerb::ChooseEngine,
    ShellVerb::ExternalStart,
    ShellVerb::CycleVoiceNext,
    ShellVerb::CycleVoicePrev,
];

/// The contract wire name of every closed verb
/// (`contracts/shell-ipc.md` case-mapping table: Rust
/// PascalCase → wire snake_case).
const WIRE_VERBS: [(ShellVerb, &str); 15] = [
    (ShellVerb::Play, "play"),
    (ShellVerb::Pause, "pause"),
    (ShellVerb::Stop, "stop"),
    (ShellVerb::Complete, "complete"),
    (ShellVerb::Interrupt, "interrupt"),
    (ShellVerb::RateUp, "rate-up"),
    (ShellVerb::RateDown, "rate-down"),
    (ShellVerb::SkipForward, "skip-forward"),
    (ShellVerb::SkipBackward, "skip-backward"),
    (ShellVerb::ResumeAtStored, "resume-at-stored"),
    (ShellVerb::StartOver, "start-over"),
    (ShellVerb::ChooseEngine, "choose-engine"),
    (ShellVerb::ExternalStart, "external-start"),
    (ShellVerb::CycleVoiceNext, "cycle-voice-next"),
    (ShellVerb::CycleVoicePrev, "cycle-voice-prev"),
];

/// The closed transition function's contract (FR-020):
/// the expected outcome of every verb press from every
/// transport state, as one exhaustive match over the
/// closed (state, verb) product — five states ×
/// fifteen verbs, no catch-all arm (Principle VII /
/// FR-044 / gate 7).
///
/// This is the contract twin of `player::cli`'s
/// `transition`: an illegal press is rejected with
/// [`PlayerError::IllegalTransition`] and the session
/// does not move — the T019 double-press convention
/// (a driver may press a verb twice; the second press
/// is discarded).
#[expect(
    clippy::match_same_arms,
    reason = "Principle VII / FR-044: the closed 5x15 transition table's illegal cells all construct the same rejection and its live-session control cells share one effect shape by design; enumerating every cell explicitly (rather than a catch-all arm) is what makes an added state or verb a compile error"
)]
fn expected(transport: TransportState, verb: ShellVerb) -> Expected {
    match (transport, verb) {
        // Playing: exactly {Paused, Interrupted,
        // Stopped, Completed} is reachable (tasks.md
        // T020) — a second Play is the T019 double
        // press, and no resume or start-over offer
        // exists while playing.
        (TransportState::Playing, ShellVerb::Play) => Expected::Rejected,
        (TransportState::Playing, ShellVerb::Pause) => Expected::Land {
            transport: TransportState::Paused,
        },
        (TransportState::Playing, ShellVerb::Stop) => Expected::LandDiscarding {
            transport: TransportState::Stopped,
        },
        (TransportState::Playing, ShellVerb::Complete) => Expected::LandDiscarding {
            transport: TransportState::Completed,
        },
        (TransportState::Playing, ShellVerb::Interrupt) => Expected::Land {
            transport: TransportState::Interrupted,
        },
        (TransportState::Playing, ShellVerb::RateUp) => Expected::Control,
        (TransportState::Playing, ShellVerb::RateDown) => Expected::Control,
        (TransportState::Playing, ShellVerb::SkipForward) => Expected::Control,
        (TransportState::Playing, ShellVerb::SkipBackward) => Expected::Control,
        (TransportState::Playing, ShellVerb::ResumeAtStored) => Expected::Rejected,
        (TransportState::Playing, ShellVerb::StartOver) => Expected::Rejected,
        (TransportState::Playing, ShellVerb::ChooseEngine) => Expected::NoOp,
        (TransportState::Playing, ShellVerb::ExternalStart) => Expected::Rejected,
        (TransportState::Playing, ShellVerb::CycleVoiceNext) => Expected::Control,
        (TransportState::Playing, ShellVerb::CycleVoicePrev) => Expected::Control,
        // Paused: resume (Play, ResumeAtStored,
        // ExternalStart) and the universal Stop. A
        // document ends from playback, not from a
        // pause, and an already-silent session cannot
        // be interrupted into a new state.
        (TransportState::Paused, ShellVerb::Play) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Paused, ShellVerb::Pause) => Expected::Rejected,
        (TransportState::Paused, ShellVerb::Stop) => Expected::LandDiscarding {
            transport: TransportState::Stopped,
        },
        (TransportState::Paused, ShellVerb::Complete) => Expected::Rejected,
        (TransportState::Paused, ShellVerb::Interrupt) => Expected::Rejected,
        (TransportState::Paused, ShellVerb::RateUp) => Expected::Control,
        (TransportState::Paused, ShellVerb::RateDown) => Expected::Control,
        (TransportState::Paused, ShellVerb::SkipForward) => Expected::Control,
        (TransportState::Paused, ShellVerb::SkipBackward) => Expected::Control,
        (TransportState::Paused, ShellVerb::ResumeAtStored) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Paused, ShellVerb::StartOver) => Expected::LandDiscarding {
            transport: TransportState::Playing,
        },
        (TransportState::Paused, ShellVerb::ChooseEngine) => Expected::NoOp,
        (TransportState::Paused, ShellVerb::ExternalStart) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Paused, ShellVerb::CycleVoiceNext) => Expected::Control,
        (TransportState::Paused, ShellVerb::CycleVoicePrev) => Expected::Control,
        // Interrupted: the same legality as Paused —
        // the explicit-action resume (Play,
        // ResumeAtStored, ExternalStart) and the
        // universal Stop; the FR-023a / FR-032c
        // expiries are the only automatic paths and
        // are not session verbs.
        (TransportState::Interrupted, ShellVerb::Play) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Interrupted, ShellVerb::Pause) => Expected::Rejected,
        (TransportState::Interrupted, ShellVerb::Stop) => Expected::LandDiscarding {
            transport: TransportState::Stopped,
        },
        (TransportState::Interrupted, ShellVerb::Complete) => Expected::Rejected,
        (TransportState::Interrupted, ShellVerb::Interrupt) => Expected::Rejected,
        (TransportState::Interrupted, ShellVerb::RateUp) => Expected::Control,
        (TransportState::Interrupted, ShellVerb::RateDown) => Expected::Control,
        (TransportState::Interrupted, ShellVerb::SkipForward) => Expected::Control,
        (TransportState::Interrupted, ShellVerb::SkipBackward) => Expected::Control,
        (TransportState::Interrupted, ShellVerb::ResumeAtStored) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Interrupted, ShellVerb::StartOver) => Expected::LandDiscarding {
            transport: TransportState::Playing,
        },
        (TransportState::Interrupted, ShellVerb::ChooseEngine) => Expected::NoOp,
        (TransportState::Interrupted, ShellVerb::ExternalStart) => Expected::Land {
            transport: TransportState::Playing,
        },
        (TransportState::Interrupted, ShellVerb::CycleVoiceNext) => Expected::Control,
        (TransportState::Interrupted, ShellVerb::CycleVoicePrev) => Expected::Control,
        // Stopped is terminal: only the idempotent
        // Stop press (T019) and the engine selection
        // are legal — the session is torn down
        // (FR-032f), so rate moves, lock-screen
        // skips, voice cycles, and every resume form
        // are rejected.
        (TransportState::Stopped, ShellVerb::Play) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::Pause) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::Stop) => Expected::LandDiscarding {
            transport: TransportState::Stopped,
        },
        (TransportState::Stopped, ShellVerb::Complete) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::Interrupt) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::RateUp) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::RateDown) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::SkipForward) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::SkipBackward) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::ResumeAtStored) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::StartOver) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::ChooseEngine) => Expected::NoOp,
        (TransportState::Stopped, ShellVerb::ExternalStart) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::CycleVoiceNext) => Expected::Rejected,
        (TransportState::Stopped, ShellVerb::CycleVoicePrev) => Expected::Rejected,
        // Completed is terminal except Stop (tasks.md
        // T020): the same torn-down legality as
        // Stopped, and the Stop press re-enters
        // Stopped, discarding the position a second
        // time.
        (TransportState::Completed, ShellVerb::Play) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::Pause) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::Stop) => Expected::LandDiscarding {
            transport: TransportState::Stopped,
        },
        (TransportState::Completed, ShellVerb::Complete) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::Interrupt) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::RateUp) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::RateDown) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::SkipForward) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::SkipBackward) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::ResumeAtStored) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::StartOver) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::ChooseEngine) => Expected::NoOp,
        (TransportState::Completed, ShellVerb::ExternalStart) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::CycleVoiceNext) => Expected::Rejected,
        (TransportState::Completed, ShellVerb::CycleVoicePrev) => Expected::Rejected,
    }
}

/// The position a verb press leaves the session at,
/// given the position before the press — the closed
/// position contract of every verb (FR-018). Only the
/// skip presses move the position; every other verb
/// leaves it exactly where it was.
#[expect(
    clippy::match_same_arms,
    reason = "Principle VII / FR-044: every verb that is not a skip press leaves the position exactly where it was — the identical arms ARE the contract; enumerating every verb explicitly (rather than a catch-all arm) is what makes an added verb a compile error"
)]
fn expected_position_after(verb: ShellVerb, before: f64) -> f64 {
    match verb {
        ShellVerb::SkipForward => before + SKIP_STEP_SECS,
        ShellVerb::SkipBackward => (before - SKIP_STEP_SECS).max(0.0),
        ShellVerb::Play => before,
        ShellVerb::Pause => before,
        ShellVerb::Stop => before,
        ShellVerb::Complete => before,
        ShellVerb::Interrupt => before,
        ShellVerb::RateUp => before,
        ShellVerb::RateDown => before,
        ShellVerb::ResumeAtStored => before,
        ShellVerb::StartOver => before,
        ShellVerb::ChooseEngine => before,
        ShellVerb::ExternalStart => before,
        ShellVerb::CycleVoiceNext => before,
        ShellVerb::CycleVoicePrev => before,
    }
}

/// The rate a verb press leaves the session at, given
/// the rate before the press — the closed rate
/// contract of every verb (FR-019). Only the rate
/// presses move the rate, exactly one ladder rung per
/// press, saturating at both ends; every other verb
/// leaves it exactly where it was.
#[expect(
    clippy::match_same_arms,
    reason = "Principle VII / FR-044: every verb that is not a rate press changes no rate — the identical arms ARE the contract; enumerating every verb explicitly (rather than a catch-all arm) is what makes an added verb a compile error"
)]
fn expected_rate_after(verb: ShellVerb, before: PlaybackRate) -> Option<PlaybackRate> {
    match verb {
        ShellVerb::RateUp => Some(step_up(before)),
        ShellVerb::RateDown => Some(step_down(before)),
        ShellVerb::Play => None,
        ShellVerb::Pause => None,
        ShellVerb::Stop => None,
        ShellVerb::Complete => None,
        ShellVerb::Interrupt => None,
        ShellVerb::SkipForward => None,
        ShellVerb::SkipBackward => None,
        ShellVerb::ResumeAtStored => None,
        ShellVerb::StartOver => None,
        ShellVerb::ChooseEngine => None,
        ShellVerb::ExternalStart => None,
        ShellVerb::CycleVoiceNext => None,
        ShellVerb::CycleVoicePrev => None,
    }
}

/// Drive a fresh session into `transport` through
/// legal presses only — the only routes the closed
/// table allows — holding a known stored position in
/// the live states (`Paused` is the FR-019b default;
/// `Interrupted` passes through `Playing`; `Stopped`
/// is the universal Stop press; `Completed` is a
/// document end, so it passes through `Playing` —
/// `Complete` is legal only from `Playing`).
fn session_in(transport: TransportState) -> PlaybackSession {
    let mut session = PlaybackSession::new();
    match transport {
        TransportState::Paused => {
            let _played = session.execute(ShellVerb::Play);
            session.on_audible_progress(STORED_POSITION_SECS);
            let _paused = session.execute(ShellVerb::Pause);
        }
        TransportState::Playing => {
            let _played = session.execute(ShellVerb::Play);
            session.on_audible_progress(STORED_POSITION_SECS);
        }
        TransportState::Interrupted => {
            let _played = session.execute(ShellVerb::Play);
            session.on_audible_progress(STORED_POSITION_SECS);
            let _interrupted = session.execute(ShellVerb::Interrupt);
        }
        TransportState::Stopped => {
            let _played = session.execute(ShellVerb::Play);
            session.on_audible_progress(STORED_POSITION_SECS);
            let _stopped = session.execute(ShellVerb::Stop);
        }
        TransportState::Completed => {
            let _played = session.execute(ShellVerb::Play);
            session.on_audible_progress(STORED_POSITION_SECS);
            let _completed = session.execute(ShellVerb::Complete);
        }
    }
    session
}

/// Build the shell-IPC mirror of a session snapshot
/// (the mirror seam the Kotlin service consumes,
/// `contracts/shell-ipc.md`).
///
/// Test-side model of the mirror construction: the
/// mirror derives from the session's authoritative
/// snapshot plus the duration knowledge fixed at
/// session start (T051) — determinate progress when
/// the duration was known, indeterminate otherwise —
/// and never computes a position of its own (SC-035).
fn mirror_of(snapshot: &SessionSnapshot, duration_secs: Option<f64>) -> MirrorState {
    let progress = match duration_secs {
        Some(duration) => Progress::Determinate {
            position_secs: snapshot.position_secs,
            duration_secs: duration,
        },
        None => Progress::Indeterminate,
    };
    MirrorState {
        transport: snapshot.transport,
        position_secs: snapshot.position_secs,
        progress,
    }
}

/// Assert one (transport, verb) cell against the
/// contract: the outcome the closed table predicts,
/// with the effect it owes — the landed transport,
/// the retained or discarded stored position, the
/// one-rung rate move, and the closed position
/// effect.
fn assert_cell(
    transport: TransportState,
    verb: ShellVerb,
    before: &SessionSnapshot,
    result: &Result<SessionSnapshot, PlayerError>,
) {
    match expected(transport, verb) {
        Expected::Land {
            transport: landed,
        } => {
            assert!(
                matches!(result, Ok(snap) if snap.transport == landed),
                "a {verb} press from {transport:?} must land on {landed:?} (FR-020)"
            );
            if let Ok(snap) = result {
                assert_eq!(
                    snap.rate, before.rate,
                    "a {verb} press changes no rate"
                );
                assert!(
                    (snap.position_secs - before.position_secs).abs() <= POS_EPS_SECS,
                    "a {verb} press retains the stored position (FR-032a)"
                );
            }
        }
        Expected::LandDiscarding {
            transport: landed,
        } => {
            assert!(
                matches!(result, Ok(snap) if snap.transport == landed),
                "a {verb} press from {transport:?} must land on {landed:?} (FR-020)"
            );
            if let Ok(snap) = result {
                assert_eq!(
                    snap.rate, before.rate,
                    "a {verb} press changes no rate"
                );
                assert!(
                    snap.position_secs.abs() <= POS_EPS_SECS,
                    "a {verb} press discards the stored position (FR-032)"
                );
            }
        }
        Expected::Control => {
            assert!(
                matches!(result, Ok(snap) if snap.transport == transport),
                "a {verb} press from {transport:?} must change no transport (FR-018 / FR-019 / FR-018b)"
            );
            if let Ok(snap) = result {
                match expected_rate_after(verb, before.rate) {
                    Some(rate) => assert_eq!(
                        snap.rate, rate,
                        "a {verb} press moves the rate exactly one rung (FR-019)"
                    ),
                    None => assert_eq!(
                        snap.rate, before.rate,
                        "a {verb} press changes no rate"
                    ),
                }
                let expected_position = expected_position_after(verb, before.position_secs);
                assert!(
                    (snap.position_secs - expected_position).abs() <= POS_EPS_SECS,
                    "a {verb} press must leave the position at {expected_position} s (FR-018 / FR-019 / FR-018b)"
                );
            }
        }
        Expected::NoOp => {
            assert!(
                matches!(result, Ok(snap) if snap.transport == transport),
                "a {verb} press from {transport:?} must change no transport (FR-005)"
            );
            if let Ok(snap) = result {
                assert_eq!(
                    snap.rate, before.rate,
                    "a {verb} press changes no rate (FR-005)"
                );
                assert!(
                    (snap.position_secs - before.position_secs).abs() <= POS_EPS_SECS,
                    "a {verb} press changes no position (FR-005)"
                );
            }
        }
        Expected::Rejected => {
            assert!(
                matches!(
                    result,
                    Err(PlayerError::IllegalTransition {
                        current,
                        verb: rejected_verb,
                    }) if *current == transport && *rejected_verb == verb
                ),
                "a {verb} press from {transport:?} must be rejected as an illegal transition (FR-032f)"
            );
        }
    }
}

/// C1 — every cell of the closed verb table is
/// honoured or rejected (FR-020 / FR-044): the full
/// (transport × verb) product — five states ×
/// fifteen verbs — with each cell asserted against
/// the contract twin of the transition function, and
/// with the session's own post-press state asserted
/// to equal the returned snapshot (or, for a
/// rejected press, to equal the pre-press snapshot —
/// the T019 convention).
#[test]
fn every_cell_of_the_closed_verb_table_is_honoured_or_rejected() {
    for transport in ALL_TRANSPORTS {
        for verb in ALL_VERBS {
            let mut session = session_in(transport);
            let before = session.snapshot();
            let result = session.execute(verb);
            assert_cell(transport, verb, &before, &result);
            let after = session.snapshot();
            if let Ok(snap) = &result {
                assert_eq!(
                    after, *snap,
                    "the dispatch returns the session's own snapshot (FR-042)"
                );
            } else {
                assert_eq!(
                    after, before,
                    "a rejected press leaves the session exactly where it was (T019)"
                );
            }
        }
    }
}

/// C2 — the transition table's shape follows the
/// shell-ipc contract (T020 / FR-005 / FR-018 /
/// FR-019 / FR-018b / FR-032 / FR-032f): the
/// structural invariants that hold across the whole
/// closed product.
///
/// The closed sets themselves are enforced by
/// compilation: [`expected`],
/// [`expected_position_after`], and
/// [`expected_rate_after`] match the full
/// (transport × verb) product with no wildcard arm,
/// so adding a state or a verb breaks this test's
/// compilation until every table here is extended
/// (FR-044 / gate 7).
#[test]
fn transition_table_shape_follows_the_shell_ipc_contract() -> Result<(), Box<dyn Error>> {
    // The closed verb set: exactly the fifteen
    // variants the shell-ipc contract enumerates —
    // the thirteen pre-existing verbs plus the two
    // voice-cycle verbs (FR-018b).
    assert_eq!(
        ALL_VERBS.len(),
        15,
        "the closed verb set holds exactly the 15 contract verbs (FR-044)"
    );
    assert_eq!(
        ALL_TRANSPORTS.len(),
        5,
        "the closed transport set holds exactly the 5 states (FR-020)"
    );

    // Every verb carries its contract wire name
    // (`contracts/shell-ipc.md` case-mapping table:
    // Rust PascalCase → wire snake_case).
    for (verb, wire) in WIRE_VERBS {
        assert_eq!(
            verb.to_string(),
            wire,
            "the verb must carry its contract wire name"
        );
    }

    // Stop is the universal teardown press: legal
    // from every transport, landing on Stopped and
    // discarding the stored position (FR-032f).
    for transport in ALL_TRANSPORTS {
        let mut session = session_in(transport);
        let _stopped = session.execute(ShellVerb::Stop)?;
        let snap = session.snapshot();
        assert_eq!(
            snap.transport,
            TransportState::Stopped,
            "a Stop press is legal from every transport (FR-032f)"
        );
        assert!(
            snap.position_secs.abs() <= POS_EPS_SECS,
            "a Stop press discards the stored position (FR-032)"
        );
    }

    // Complete is legal only from Playing — a
    // document ends from playback, never from a
    // silence state (T020).
    for transport in ALL_TRANSPORTS {
        let mut session = session_in(transport);
        let result = session.execute(ShellVerb::Complete);
        if transport == TransportState::Playing {
            let completed = result?;
            assert_eq!(
                completed.transport,
                TransportState::Completed,
                "a Complete press from Playing lands on Completed (T020)"
            );
        } else {
            assert!(
                matches!(
                    &result,
                    Err(PlayerError::IllegalTransition { current, verb: rejected })
                        if *current == transport && *rejected == ShellVerb::Complete
                ),
                "a Complete press from {transport:?} must be rejected (T020)"
            );
        }
    }

    // The resume, start-over, and external-start
    // offers exist only where an offer exists
    // (Paused, Interrupted) and share the Play
    // legality; from the other states they are
    // rejected (FR-032 / FR-032f).
    for transport in ALL_TRANSPORTS {
        for verb in [
            ShellVerb::Play,
            ShellVerb::ResumeAtStored,
            ShellVerb::StartOver,
            ShellVerb::ExternalStart,
        ] {
            let mut session = session_in(transport);
            let result = session.execute(verb);
            if transport == TransportState::Paused
                || transport == TransportState::Interrupted
            {
                let resumed = result?;
                assert_eq!(
                    resumed.transport,
                    TransportState::Playing,
                    "a {verb} press from {transport:?} must resume at Playing (FR-032)"
                );
            } else {
                assert!(
                    matches!(
                        &result,
                        Err(PlayerError::IllegalTransition { current, verb: rejected })
                            if *current == transport && *rejected == verb
                    ),
                    "a {verb} press from {transport:?} must be rejected (FR-032f)"
                );
            }
        }
    }

    // Live-session controls — the rate moves, the
    // lock-screen skips, and the voice cycle — are
    // legal from the live states and rejected once
    // the session is torn down (FR-018 / FR-019 /
    // FR-018b / FR-032f).
    for transport in ALL_TRANSPORTS {
        for verb in [
            ShellVerb::RateUp,
            ShellVerb::RateDown,
            ShellVerb::SkipForward,
            ShellVerb::SkipBackward,
            ShellVerb::CycleVoiceNext,
            ShellVerb::CycleVoicePrev,
        ] {
            let mut session = session_in(transport);
            let result = session.execute(verb);
            if transport == TransportState::Playing
                || transport == TransportState::Paused
                || transport == TransportState::Interrupted
            {
                let _controlled = result?;
            } else {
                assert!(
                    matches!(
                        &result,
                        Err(PlayerError::IllegalTransition { current, verb: rejected })
                            if *current == transport && *rejected == verb
                    ),
                    "a {verb} press from {transport:?} must be rejected once the session is torn down (FR-032f)"
                );
            }
        }
    }

    // ChooseEngine is a selection, not a transition:
    // legal from every state, changing nothing
    // (FR-005).
    for transport in ALL_TRANSPORTS {
        let mut session = session_in(transport);
        let before = session.snapshot();
        let _selected = session.execute(ShellVerb::ChooseEngine)?;
        assert_eq!(
            session.snapshot(),
            before,
            "a ChooseEngine press changes no transport, no position, and no rate (FR-005)"
        );
    }

    // The rate ladder: every rung of the closed
    // 7-rung ladder in order, saturating at both
    // ends (FR-019).
    let mut session = PlaybackSession::new();
    let _played = session.execute(ShellVerb::Play)?;
    for &rung in LADDER.iter().skip(1) {
        let _stepped = session.execute(ShellVerb::RateUp)?;
        assert_eq!(
            session.snapshot().rate,
            rung,
            "a rate-up press must walk the ladder in order (FR-019)"
        );
    }
    let _saturated = session.execute(ShellVerb::RateUp)?;
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R4_0,
        "a rate-up press at the fastest rung saturates (FR-019)"
    );
    for &rung in LADDER.iter().rev().skip(1) {
        let _stepped = session.execute(ShellVerb::RateDown)?;
        assert_eq!(
            session.snapshot().rate,
            rung,
            "a rate-down press must walk the ladder back in order (FR-019)"
        );
    }
    let _saturated = session.execute(ShellVerb::RateDown)?;
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R0_75,
        "a rate-down press at the slowest rung saturates (FR-019)"
    );

    // The voice cycle is a live-session control that
    // changes no transport, no rate, and no position
    // at the player level (FR-018b) — the
    // one-voice-per-press walk is the integration
    // trial's (T053).
    let mut session = PlaybackSession::new();
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(STORED_POSITION_SECS);
    let before = session.snapshot();
    let _next = session.execute(ShellVerb::CycleVoiceNext)?;
    let after = session.snapshot();
    assert_eq!(
        after.transport, before.transport,
        "a next-voice press changes no transport (FR-018b)"
    );
    assert_eq!(
        after.rate, before.rate,
        "a next-voice press changes no rate (FR-018b)"
    );
    assert!(
        (after.position_secs - before.position_secs).abs() <= POS_EPS_SECS,
        "a next-voice press changes no position at the player level (FR-018b)"
    );
    Ok(())
}

/// C3 — the mirror payload carries the session's
/// authoritative position only (FR-003 / SC-035):
/// the Rust module is the single source of truth for
/// playback position, and the mirror the shell
/// consumes is a view of the `SessionSnapshot` —
/// never a second clock.
#[test]
fn mirror_payload_carries_the_sessions_authoritative_position_only()
-> Result<(), Box<dyn Error>> {
    // A session at a known position.
    let mut session = PlaybackSession::new();
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(STORED_POSITION_SECS);
    let _paused = session.execute(ShellVerb::Pause)?;
    let snapshot = session.snapshot();

    // The mirror carries the session's authoritative
    // transport and position — and nothing the
    // snapshot does not carry.
    let mirror = mirror_of(&snapshot, Some(DURATION_SECS));
    assert_eq!(
        mirror.transport, snapshot.transport,
        "the mirror carries the session's transport (SC-035)"
    );
    assert!(
        (mirror.position_secs - snapshot.position_secs).abs() <= POS_EPS_SECS,
        "the mirror carries the session's authoritative position only (FR-003)"
    );

    // The progress is determinate when the duration
    // is known at session start (T051), carrying the
    // same position and the duration.
    let Progress::Determinate {
        position_secs,
        duration_secs,
    } = mirror.progress
    else {
        return Err(
            "a session with a known duration resolves a determinate progress (T051)".into(),
        );
    };
    assert!(
        (position_secs - snapshot.position_secs).abs() <= POS_EPS_SECS,
        "the determinate progress carries the session's position (FR-003)"
    );
    assert!(
        (duration_secs - DURATION_SECS).abs() <= POS_EPS_SECS,
        "the determinate progress carries the session's duration (T051)"
    );

    // The mirror is derived, never a second clock:
    // rebuilding it from an unchanged snapshot yields
    // an equal mirror — no amount of time passing
    // moves it (SC-035).
    let rebuilt = mirror_of(&snapshot, Some(DURATION_SECS));
    assert_eq!(
        mirror, rebuilt,
        "the mirror is a pure view of the snapshot — never a second clock (SC-035)"
    );

    // Without a duration at session start the
    // progress is indeterminate for the whole session
    // (T051) — the transport and position are still
    // the session's own.
    let unknown = mirror_of(&snapshot, None);
    assert_eq!(
        unknown.progress, Progress::Indeterminate,
        "a session without a duration at start resolves an indeterminate progress (T051)"
    );
    assert_eq!(
        unknown.transport, snapshot.transport,
        "the indeterminate mirror carries the session's transport (SC-035)"
    );
    assert!(
        (unknown.position_secs - snapshot.position_secs).abs() <= POS_EPS_SECS,
        "the indeterminate mirror carries the session's position (FR-003)"
    );
    Ok(())
}

/// C4 — the mirror position reflects the Rust module
/// and never moves while silent (FR-002 / FR-005):
/// position advances only through audible progress
/// while playing, the lock-screen skips are
/// live-session controls, a pause retains the stored
/// position, and a rejected press moves nothing.
#[test]
fn mirror_position_reflects_the_rust_module_and_never_moves_while_silent()
-> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::new();

    // Silent: the session is Paused — position never
    // moves while silent, and the mirror reports
    // exactly that (FR-005).
    session.on_audible_progress(5.0);
    let silent = mirror_of(&session.snapshot(), Some(DURATION_SECS));
    assert!(
        silent.position_secs.abs() <= POS_EPS_SECS,
        "position never moves while the session is silent (FR-005)"
    );
    assert_eq!(
        silent.transport,
        TransportState::Paused,
        "a fresh session is Paused (FR-019b)"
    );

    // Playing: position advances only through audible
    // progress — the Rust module's sole advance path
    // (FR-002) — and the mirror follows every step.
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(5.0);
    let playing = mirror_of(&session.snapshot(), Some(DURATION_SECS));
    assert!(
        (playing.position_secs - 5.0).abs() <= POS_EPS_SECS,
        "audible progress must advance the position the mirror reflects (FR-002)"
    );
    assert_eq!(
        playing.transport,
        TransportState::Playing,
        "the mirror reflects the session's transport (SC-035)"
    );

    // A skip press is a live-session control
    // (FR-018): it moves the stored position and the
    // mirror reflects the Rust module's position,
    // never its own.
    let _skipped = session.execute(ShellVerb::SkipForward)?;
    let skipped = mirror_of(&session.snapshot(), Some(DURATION_SECS));
    assert!(
        (skipped.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "a skip-forward press must advance the position the mirror reflects (FR-018)"
    );

    // Paused: the stored position is retained
    // (FR-032a) and no further progress moves it
    // (FR-005).
    let _paused = session.execute(ShellVerb::Pause)?;
    session.on_audible_progress(5.0);
    let paused = mirror_of(&session.snapshot(), Some(DURATION_SECS));
    assert!(
        (paused.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "a pause retains the stored position the mirror reflects (FR-032a)"
    );
    assert_eq!(
        paused.transport,
        TransportState::Paused,
        "the mirror reflects the session's transport (SC-035)"
    );

    // A rejected press — the T019 double press — is
    // discarded and the position stays exactly where
    // it was; the mirror is unchanged too.
    let rejected = session.execute(ShellVerb::Pause);
    assert!(
        matches!(&rejected, Err(PlayerError::IllegalTransition { .. })),
        "a double Pause press must be rejected (T019)"
    );
    let unchanged = mirror_of(&session.snapshot(), Some(DURATION_SECS));
    assert!(
        (unchanged.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "a rejected press leaves the position the mirror reflects unchanged (T019)"
    );
    Ok(())
}

/// C5 — the snapshot and the mirror serialize to the
/// contract's snake_case wire keys (contracts/
/// shell-ipc.md case-mapping table): exactly the
/// enumerated key sets, nothing added, nothing
/// renamed.
#[test]
fn snapshot_and_mirror_serialize_to_snake_case_wire_keys() -> Result<(), Box<dyn Error>> {
    // A session at a known state.
    let mut session = PlaybackSession::new();
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(STORED_POSITION_SECS);
    let _paused = session.execute(ShellVerb::Pause)?;
    let snapshot = session.snapshot();

    // The session snapshot's wire form: exactly the
    // four contract keys, snake_case.
    let wire = serde_json::to_string(&snapshot)?;
    let value: serde_json::Value = serde_json::from_str(&wire)?;
    let object = value
        .as_object()
        .ok_or("the snapshot wire form must be a JSON object")?;
    assert_eq!(
        object.len(),
        4,
        "the snapshot wire form carries exactly the four contract keys (contracts/shell-ipc.md)"
    );
    assert_eq!(
        object.get("transport"),
        Some(&serde_json::json!("paused")),
        "the snapshot wire form carries the transport under its contract key"
    );
    assert_eq!(
        object.get("position_secs"),
        Some(&serde_json::json!(STORED_POSITION_SECS)),
        "the snapshot wire form carries the position under its contract key"
    );
    assert_eq!(
        object.get("rate"),
        Some(&serde_json::json!(1.0)),
        "the snapshot wire form carries the rate as its JSON number (contracts/shell-ipc.md)"
    );
    assert_eq!(
        object.get("engine_id"),
        Some(&serde_json::json!("rust-player")),
        "the snapshot wire form carries the engine identifier under its contract key"
    );

    // The mirror's wire form: exactly the three
    // contract keys.
    let mirror = mirror_of(&snapshot, Some(DURATION_SECS));
    let mirror_wire = serde_json::to_string(&mirror)?;
    let mirror_value: serde_json::Value = serde_json::from_str(&mirror_wire)?;
    let mirror_object = mirror_value
        .as_object()
        .ok_or("the mirror wire form must be a JSON object")?;
    assert_eq!(
        mirror_object.len(),
        3,
        "the mirror wire form carries exactly the three contract keys (contracts/shell-ipc.md)"
    );
    assert_eq!(
        mirror_object.get("transport"),
        Some(&serde_json::json!("paused")),
        "the mirror wire form carries the transport under its contract key"
    );
    assert_eq!(
        mirror_object.get("position_secs"),
        Some(&serde_json::json!(STORED_POSITION_SECS)),
        "the mirror wire form carries the session's authoritative position under its contract key (FR-003)"
    );
    let progress = mirror_object
        .get("progress")
        .ok_or("the mirror wire form must carry the progress")?;
    assert_eq!(
        progress,
        &serde_json::json!({
            "determinate": {
                "position_secs": STORED_POSITION_SECS,
                "duration_secs": DURATION_SECS,
            }
        }),
        "a determinate progress serializes as an externally-tagged object (contracts/shell-ipc.md)"
    );

    // The indeterminate progress serializes as its
    // bare wire string.
    let indeterminate = mirror_of(&snapshot, None);
    let indeterminate_wire = serde_json::to_string(&indeterminate)?;
    let indeterminate_value: serde_json::Value =
        serde_json::from_str(&indeterminate_wire)?;
    let indeterminate_object = indeterminate_value
        .as_object()
        .ok_or("the indeterminate mirror wire form must be a JSON object")?;
    assert_eq!(
        indeterminate_object.get("progress"),
        Some(&serde_json::json!("indeterminate")),
        "an indeterminate progress serializes as its bare wire string (contracts/shell-ipc.md)"
    );
    Ok(())
}
