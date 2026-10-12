//! Player CLI dispatch (T027).
//!
//! Every surface routes through the same dispatch:
//! notification, lock screen, CLI, IPC — all call the same
//! backend function per Principle II.

use crate::position::PositionClock;
use crate::transport::{PlaybackRate, TransportState, rate_down_from, rate_up_from};
use thiserror::Error;
use tracing::{debug, instrument};

/// The player-side engine identifier (FR-023c).
const ENGINE_ID: &str = "rust-player";

/// Shell verb — the closed verb set that every surface maps to
/// (contracts/shell-ipc.md). Excludes read-only queries
/// (`position`, `state`) which are handled separately and are
/// not part of the closed-verb exhaustiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellVerb {
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// Stop — discards the stored position.
    Stop,
    /// Document completed.
    Complete,
    /// Interruption (e.g. focus loss).
    Interrupt,
    /// Rate up one step on the ladder.
    RateUp,
    /// Rate down one step on the ladder.
    RateDown,
    /// Skip forward ±5.0 s.
    SkipForward,
    /// Skip backward ±5.0 s.
    SkipBackward,
    /// Resume at stored position.
    ResumeAtStored,
    /// Start over from 0.0.
    StartOver,
    /// Choose engine by id.
    ChooseEngine,
    /// External start (for the externally-started-service trial).
    ExternalStart,
}

impl std::fmt::Display for ShellVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ShellVerb::Play => "play",
            ShellVerb::Pause => "pause",
            ShellVerb::Stop => "stop",
            ShellVerb::Complete => "complete",
            ShellVerb::Interrupt => "interrupt",
            ShellVerb::RateUp => "rate-up",
            ShellVerb::RateDown => "rate-down",
            ShellVerb::SkipForward => "skip-forward",
            ShellVerb::SkipBackward => "skip-backward",
            ShellVerb::ResumeAtStored => "resume-at-stored",
            ShellVerb::StartOver => "start-over",
            ShellVerb::ChooseEngine => "choose-engine",
            ShellVerb::ExternalStart => "external-start",
        };
        f.write_str(name)
    }
}

/// Helper for skip verbs — the skip actions are exactly
/// ±5.0 s, measured against the position at the moment the
/// control was pressed (FR-018).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellVariant {
    /// Skip forward variant.
    SkipForward,
    /// Skip backward variant.
    SkipBackward,
}

impl ShellVariant {
    /// Create a skip-forward verb.
    pub fn skip_forward() -> ShellVerb {
        ShellVerb::SkipForward
    }
    /// Create a skip-backward verb.
    pub fn skip_back() -> ShellVerb {
        ShellVerb::SkipBackward
    }
}

/// Snapshot of the session after a command.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSnapshot {
    /// Current transport state.
    pub transport: TransportState,
    /// Current position in seconds.
    pub position_secs: f64,
    /// Current playback rate.
    pub rate: PlaybackRate,
    /// Engine identifier.
    pub engine_id: &'static str,
}

/// Errors raised by the playback session (the position
/// clock's own errors live in [`crate::position::PlayerError`]).
#[derive(Debug, Error)]
pub enum PlayerError {
    /// The requested skip would move the position below 0.0.
    #[error("skip backward would go below floor 0.0")]
    SkipBelowFloor,
    /// The requested progress amount is negative.
    #[error("progress must be non-negative, got {0}")]
    NegativeProgress(f64),
    /// Illegal transport transition.
    #[error("illegal transition from {current} via {verb}")]
    IllegalTransition {
        /// The current transport state.
        current: TransportState,
        /// The verb that was attempted.
        verb: ShellVerb,
    },
    /// The engine is not available.
    #[error("engine {0} is not available")]
    EngineNotAvailable(&'static str),
}

/// The effect a legal verb press has on the session —
/// the output of the closed transition function
/// [`transition`].
///
/// Every variant is explicit (Principle VII / FR-044): a
/// new verb adds a variant here and a cell in
/// [`transition`] before anything compiles — no
/// catch-all arm exists.
#[derive(Debug, Clone, Copy)]
enum Transition {
    /// Land the transport on a new state, retaining the
    /// stored position — the resume plays (`Play`,
    /// `ResumeAtStored`, `ExternalStart`) and the
    /// silence demands (`Pause`, `Interrupt`).
    Land {
        /// The transport state the press lands on.
        transport: TransportState,
    },
    /// Land the transport on a new state, discarding the
    /// stored position — `Stop`, `Complete`, and
    /// `StartOver` delete the stored entry
    /// (data-model.md `StoredPosition`: "stopped|
    /// completed delete the entry").
    LandDiscarding {
        /// The transport state the press lands on.
        transport: TransportState,
    },
    /// Move the playback rate one rung up the closed
    /// ladder (FR-019) — a live-session control; no
    /// transport change.
    RateUp,
    /// Move the playback rate one rung down the closed
    /// ladder (FR-019) — a live-session control; no
    /// transport change.
    RateDown,
    /// Skip forward exactly 5.0 s (FR-018) — a
    /// live-session control; no transport change.
    SkipForward,
    /// Skip backward exactly 5.0 s, clamped at the 0.0
    /// floor (FR-018) — a live-session control; no
    /// transport change.
    SkipBackward,
    /// Select the engine — a selection, not a
    /// transition: it changes no transport or position
    /// state, so it is legal from every state and can
    /// never report playing while silent (FR-005).
    ChooseEngine,
}

/// The closed transition function (data-model.md
/// `PlaybackSession`): the effect of every verb press
/// from every transport state, as one exhaustive match
/// over the closed (state, verb) product — five states
/// × thirteen verbs, no catch-all arm (Principle VII /
/// FR-044 / gate 7). A new state or verb is a compile
/// error here until it has a table entry, so the sets
/// cannot drift.
///
/// An illegal press is rejected with
/// [`PlayerError::IllegalTransition`] and the session
/// does not move — the T019 double-press convention (a
/// driver may press a verb twice; the second press is
/// discarded).
///
/// # Errors
///
/// [`PlayerError::IllegalTransition`] when the verb is
/// not a legal transition from `transport` — never any
/// other error, and never a silent acceptance.
#[expect(
    clippy::match_same_arms,
    reason = "Principle VII / FR-044: the closed 5x13 transition table's illegal cells all construct the same IllegalTransition and its legal control cells share effect constructors by design; enumerating every cell explicitly (rather than a catch-all arm) is what makes an added state or verb a compile error"
)]
fn transition(transport: TransportState, verb: ShellVerb) -> Result<Transition, PlayerError> {
    match (transport, verb) {
        // Playing: exactly {Paused, Interrupted, Stopped,
        // Completed} is reachable (tasks.md T020) — a
        // second Play is the T019 double press, and no
        // resume or start-over offer exists while
        // playing.
        (TransportState::Playing, ShellVerb::Play) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Playing, ShellVerb::Pause) => Ok(Transition::Land {
            transport: TransportState::Paused,
        }),
        (TransportState::Playing, ShellVerb::Stop) => Ok(Transition::LandDiscarding {
            transport: TransportState::Stopped,
        }),
        (TransportState::Playing, ShellVerb::Complete) => Ok(Transition::LandDiscarding {
            transport: TransportState::Completed,
        }),
        (TransportState::Playing, ShellVerb::Interrupt) => Ok(Transition::Land {
            transport: TransportState::Interrupted,
        }),
        (TransportState::Playing, ShellVerb::RateUp) => Ok(Transition::RateUp),
        (TransportState::Playing, ShellVerb::RateDown) => Ok(Transition::RateDown),
        (TransportState::Playing, ShellVerb::SkipForward) => Ok(Transition::SkipForward),
        (TransportState::Playing, ShellVerb::SkipBackward) => Ok(Transition::SkipBackward),
        (TransportState::Playing, ShellVerb::ResumeAtStored) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Playing, ShellVerb::StartOver) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Playing, ShellVerb::ChooseEngine) => Ok(Transition::ChooseEngine),
        (TransportState::Playing, ShellVerb::ExternalStart) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        // Paused: resume (Play, ResumeAtStored,
        // ExternalStart) and the universal Stop. A
        // document ends from playback, not from a pause,
        // and an already-silent session cannot be
        // interrupted into a new state.
        (TransportState::Paused, ShellVerb::Play) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        (TransportState::Paused, ShellVerb::Pause) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Paused, ShellVerb::Stop) => Ok(Transition::LandDiscarding {
            transport: TransportState::Stopped,
        }),
        (TransportState::Paused, ShellVerb::Complete) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Paused, ShellVerb::Interrupt) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Paused, ShellVerb::RateUp) => Ok(Transition::RateUp),
        (TransportState::Paused, ShellVerb::RateDown) => Ok(Transition::RateDown),
        (TransportState::Paused, ShellVerb::SkipForward) => Ok(Transition::SkipForward),
        (TransportState::Paused, ShellVerb::SkipBackward) => Ok(Transition::SkipBackward),
        (TransportState::Paused, ShellVerb::ResumeAtStored) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        (TransportState::Paused, ShellVerb::StartOver) => Ok(Transition::LandDiscarding {
            transport: TransportState::Playing,
        }),
        (TransportState::Paused, ShellVerb::ChooseEngine) => Ok(Transition::ChooseEngine),
        (TransportState::Paused, ShellVerb::ExternalStart) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        // Interrupted: the same legality as Paused — the
        // explicit-action resume (Play, ResumeAtStored,
        // ExternalStart) and the universal Stop; the
        // FR-023a / FR-032c expiries are the only
        // automatic paths and are not session verbs.
        (TransportState::Interrupted, ShellVerb::Play) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        (TransportState::Interrupted, ShellVerb::Pause) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Interrupted, ShellVerb::Stop) => Ok(Transition::LandDiscarding {
            transport: TransportState::Stopped,
        }),
        (TransportState::Interrupted, ShellVerb::Complete) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Interrupted, ShellVerb::Interrupt) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Interrupted, ShellVerb::RateUp) => Ok(Transition::RateUp),
        (TransportState::Interrupted, ShellVerb::RateDown) => Ok(Transition::RateDown),
        (TransportState::Interrupted, ShellVerb::SkipForward) => Ok(Transition::SkipForward),
        (TransportState::Interrupted, ShellVerb::SkipBackward) => Ok(Transition::SkipBackward),
        (TransportState::Interrupted, ShellVerb::ResumeAtStored) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        (TransportState::Interrupted, ShellVerb::StartOver) => Ok(Transition::LandDiscarding {
            transport: TransportState::Playing,
        }),
        (TransportState::Interrupted, ShellVerb::ChooseEngine) => Ok(Transition::ChooseEngine),
        (TransportState::Interrupted, ShellVerb::ExternalStart) => Ok(Transition::Land {
            transport: TransportState::Playing,
        }),
        // Stopped is terminal: only the idempotent Stop
        // press (T019) and the engine selection are
        // legal — the session is torn down (FR-032f),
        // so rate moves, lock-screen skips, and every
        // resume form are rejected.
        (TransportState::Stopped, ShellVerb::Play) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::Pause) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::Stop) => Ok(Transition::LandDiscarding {
            transport: TransportState::Stopped,
        }),
        (TransportState::Stopped, ShellVerb::Complete) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::Interrupt) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::RateUp) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::RateDown) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::SkipForward) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::SkipBackward) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::ResumeAtStored) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Stopped, ShellVerb::StartOver) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Stopped, ShellVerb::ChooseEngine) => Ok(Transition::ChooseEngine),
        (TransportState::Stopped, ShellVerb::ExternalStart) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        // Completed is terminal except Stop (tasks.md
        // T020): the same torn-down legality as
        // Stopped, and the Stop press re-enters
        // Stopped, discarding the position a second
        // time.
        (TransportState::Completed, ShellVerb::Play) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::Pause) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::Stop) => Ok(Transition::LandDiscarding {
            transport: TransportState::Stopped,
        }),
        (TransportState::Completed, ShellVerb::Complete) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::Interrupt) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::RateUp) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::RateDown) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::SkipForward) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Completed, ShellVerb::SkipBackward) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Completed, ShellVerb::ResumeAtStored) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
        (TransportState::Completed, ShellVerb::StartOver) => Err(PlayerError::IllegalTransition {
            current: transport,
            verb,
        }),
        (TransportState::Completed, ShellVerb::ChooseEngine) => Ok(Transition::ChooseEngine),
        (TransportState::Completed, ShellVerb::ExternalStart) => {
            Err(PlayerError::IllegalTransition {
                current: transport,
                verb,
            })
        }
    }
}

/// The single dispatch every surface routes through (FR-042).
///
/// The session owns the position clock and the transport state.
/// Position advances only on audible playback (never while silent),
/// and the transport state follows the closed transition function
/// per data-model.md `PlaybackSession`.
#[derive(Debug)]
pub struct PlaybackSession {
    /// The authoritative position clock (FR-002).
    clock: PositionClock,
    /// Current transport state.
    transport: TransportState,
    /// Current playback rate.
    rate: PlaybackRate,
}

impl PlaybackSession {
    /// Create a new session at position 0.0, rate R1_0,
    /// transport `Paused` (FR-019b default).
    pub fn new() -> Self {
        Self {
            clock: PositionClock::new(0.0),
            transport: TransportState::Paused,
            rate: PlaybackRate::R1_0,
        }
    }

    /// Snapshot of the current session state.
    #[must_use]
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            transport: self.transport,
            position_secs: self.clock.position_secs(),
            rate: self.rate,
            engine_id: ENGINE_ID,
        }
    }

    /// Execute a shell verb, returning the new snapshot or an error.
    ///
    /// The closed transition function decides every (state, verb)
    /// cell first (FR-020): an illegal press is rejected with the
    /// transport and the position untouched, so the controls always
    /// reflect the actual state — and a playing state is never
    /// reported while the session is silent (FR-005).
    #[instrument(skip(self))]
    pub fn execute(&mut self, verb: ShellVerb) -> Result<SessionSnapshot, PlayerError> {
        let effect = transition(self.transport, verb)?;
        match effect {
            Transition::Land { transport } => {
                self.transport = transport;
                debug!(verb = %verb, transport = ?transport, "transport landed, position retained");
            }
            Transition::LandDiscarding { transport } => {
                self.transport = transport;
                self.clock = PositionClock::new(0.0);
                debug!(verb = %verb, transport = ?transport, "transport landed, stored position discarded");
            }
            Transition::RateUp => {
                self.rate = rate_up_from(self.rate);
                debug!(rate = ?self.rate, "rate up one step");
            }
            Transition::RateDown => {
                self.rate = rate_down_from(self.rate);
                debug!(rate = ?self.rate, "rate down one step");
            }
            Transition::SkipForward => {
                let position = self.clock.skip_forward();
                debug!(position, "skip forward");
            }
            Transition::SkipBackward => {
                let position = self.clock.skip_back();
                debug!(position, "skip backward");
            }
            Transition::ChooseEngine => {
                // Placeholder: engine selection is handled by the
                // fallback selector (T075). No position change.
                debug!("choose engine");
            }
        }
        Ok(self.snapshot())
    }

    /// Feed audible progress to the session (FR-002).
    ///
    /// Position advances only while `Playing` — never while
    /// silent (FR-005). Pre-roll never advances position.
    #[instrument(skip(self))]
    pub fn on_audible_progress(&mut self, elapsed_secs: f64) {
        if self.transport != TransportState::Playing {
            debug!(
                transport = ?self.transport,
                "audible progress ignored — not playing"
            );
            return;
        }
        if elapsed_secs < 0.0 {
            debug!("negative progress ignored");
            return;
        }
        let _ = self.clock.on_audible_progress(elapsed_secs);
    }
}

impl Default for PlaybackSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::SKIP_STEP_SECS;

    #[test]
    fn new_session_defaults_to_paused_at_zero() {
        let session = PlaybackSession::new();
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Paused);
        assert_eq!(snap.position_secs, 0.0);
        assert_eq!(snap.rate, PlaybackRate::R1_0);
    }

    #[test]
    fn play_transitions_to_playing() {
        let mut session = PlaybackSession::new();
        let result = session.execute(ShellVerb::Play);
        assert!(matches!(result, Ok(ref snap) if snap.transport == TransportState::Playing));
    }

    #[test]
    fn pause_transitions_to_paused() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        let result = session.execute(ShellVerb::Pause);
        assert!(matches!(result, Ok(ref snap) if snap.transport == TransportState::Paused));
    }

    #[test]
    fn stop_resets_position() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(10.0);
        assert!(session.execute(ShellVerb::Stop).is_ok());
        assert_eq!(session.snapshot().position_secs, 0.0);
    }

    #[test]
    fn audible_progress_advances_only_while_playing() {
        let mut session = PlaybackSession::new();
        // Not playing — progress ignored.
        session.on_audible_progress(5.0);
        assert_eq!(session.snapshot().position_secs, 0.0);

        // Now playing — progress advances.
        assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(5.0);
        assert_eq!(session.snapshot().position_secs, 5.0);
    }

    #[test]
    fn skip_forward_moves_five_seconds() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        assert!(session.execute(ShellVerb::SkipForward).is_ok());
        assert_eq!(session.snapshot().position_secs, SKIP_STEP_SECS);
    }

    #[test]
    fn skip_back_clamps_at_zero() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::SkipBackward).is_ok());
        assert_eq!(session.snapshot().position_secs, 0.0);
    }

    #[test]
    fn rate_up_saturates_at_r4_0() {
        let mut session = PlaybackSession::new();
        // Walk to R4_0
        for _ in 0..10 {
            assert!(session.execute(ShellVerb::RateUp).is_ok());
        }
        assert_eq!(session.snapshot().rate, PlaybackRate::R4_0);
    }

    #[test]
    fn rate_down_saturates_at_r0_75() {
        let mut session = PlaybackSession::new();
        for _ in 0..10 {
            assert!(session.execute(ShellVerb::RateDown).is_ok());
        }
        assert_eq!(session.snapshot().rate, PlaybackRate::R0_75);
    }

    #[test]
    fn external_start_is_playing() {
        let mut session = PlaybackSession::new();
        let result = session.execute(ShellVerb::ExternalStart);
        assert!(matches!(result, Ok(ref snap) if snap.transport == TransportState::Playing));
    }

    /// Drive a fresh session into `state` through legal
    /// presses only — the only routes the closed table
    /// allows (`Paused` is the FR-019b default;
    /// `Interrupted` passes through `Playing`; `Stopped`
    /// is the universal Stop press; `Completed` is a
    /// document end, so it passes through `Playing` —
    /// `Complete` is legal only from `Playing`).
    fn session_in(state: TransportState) -> PlaybackSession {
        let mut session = PlaybackSession::new();
        match state {
            TransportState::Paused => {}
            TransportState::Playing => {
                let _played = session.execute(ShellVerb::Play);
            }
            TransportState::Interrupted => {
                let _played = session.execute(ShellVerb::Play);
                let _interrupted = session.execute(ShellVerb::Interrupt);
            }
            TransportState::Stopped => {
                let _stopped = session.execute(ShellVerb::Stop);
            }
            TransportState::Completed => {
                let _played = session.execute(ShellVerb::Play);
                let _completed = session.execute(ShellVerb::Complete);
            }
        }
        session
    }

    /// Assert the press was rejected as the illegal
    /// transition the closed table predicts — the error
    /// must name the state and the verb, and the
    /// session must not move.
    fn assert_rejected(
        result: Result<SessionSnapshot, PlayerError>,
        current: TransportState,
        verb: ShellVerb,
    ) {
        assert!(
            matches!(
                &result,
                Err(PlayerError::IllegalTransition {
                    current: rejected_current,
                    verb: rejected_verb,
                }) if *rejected_current == current && *rejected_verb == verb
            ),
            "a {verb} press from {current:?} must be rejected as an illegal transition, got {result:?}"
        );
    }

    #[test]
    fn play_from_stopped_is_rejected() {
        let mut session = session_in(TransportState::Stopped);
        assert_rejected(
            session.execute(ShellVerb::Play),
            TransportState::Stopped,
            ShellVerb::Play,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Stopped);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn play_from_completed_is_rejected() {
        let mut session = session_in(TransportState::Completed);
        assert_rejected(
            session.execute(ShellVerb::Play),
            TransportState::Completed,
            ShellVerb::Play,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Completed);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn pause_from_paused_is_rejected() {
        let mut session = PlaybackSession::new();
        assert_rejected(
            session.execute(ShellVerb::Pause),
            TransportState::Paused,
            ShellVerb::Pause,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Paused);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn complete_from_paused_is_rejected() {
        let mut session = PlaybackSession::new();
        assert_rejected(
            session.execute(ShellVerb::Complete),
            TransportState::Paused,
            ShellVerb::Complete,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Paused);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn resume_at_stored_from_stopped_is_rejected() {
        let mut session = session_in(TransportState::Stopped);
        assert_rejected(
            session.execute(ShellVerb::ResumeAtStored),
            TransportState::Stopped,
            ShellVerb::ResumeAtStored,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Stopped);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn skip_backward_from_completed_is_rejected() {
        let mut session = session_in(TransportState::Completed);
        assert_rejected(
            session.execute(ShellVerb::SkipBackward),
            TransportState::Completed,
            ShellVerb::SkipBackward,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Completed);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn double_play_from_playing_is_rejected() {
        let mut session = session_in(TransportState::Playing);
        assert_rejected(
            session.execute(ShellVerb::Play),
            TransportState::Playing,
            ShellVerb::Play,
        );
        let snap = session.snapshot();
        assert_eq!(snap.transport, TransportState::Playing);
        assert_eq!(snap.position_secs, 0.0);
    }

    #[test]
    fn rejected_verbs_leave_state_and_position_untouched() {
        // A paused session holding a retained position:
        // every illegal press from Paused must leave
        // both the transport and the position exactly
        // where they were (the T019 convention).
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(12.5);
        assert!(session.execute(ShellVerb::Pause).is_ok());
        for verb in [ShellVerb::Pause, ShellVerb::Complete, ShellVerb::Interrupt] {
            assert_rejected(session.execute(verb), TransportState::Paused, verb);
            let snap = session.snapshot();
            assert_eq!(snap.transport, TransportState::Paused);
            assert_eq!(snap.position_secs, 12.5);
        }
    }

    #[test]
    fn start_over_from_paused_discards_the_stored_position() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(20.0);
        assert!(session.execute(ShellVerb::Pause).is_ok());
        let started_over = session.execute(ShellVerb::StartOver);
        assert!(
            matches!(&started_over, Ok(snap) if snap.transport == TransportState::Playing),
            "StartOver from Paused must resume at 0.0"
        );
        assert_eq!(session.snapshot().position_secs, 0.0);
    }

    #[test]
    fn resume_at_stored_from_interrupted_retains_the_position() {
        let mut session = PlaybackSession::new();
        assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(40.0);
        assert!(session.execute(ShellVerb::Interrupt).is_ok());
        let resumed = session.execute(ShellVerb::ResumeAtStored);
        assert!(
            matches!(&resumed, Ok(snap) if snap.transport == TransportState::Playing),
            "ResumeAtStored from Interrupted must resume"
        );
        assert_eq!(session.snapshot().position_secs, 40.0);
    }

    #[test]
    fn control_verbs_need_a_live_session() {
        // Rate moves and lock-screen skips are
        // live-session controls: legal while the session
        // is alive (Playing, Paused, Interrupted) and
        // rejected once it is torn down (Stopped,
        // Completed — FR-032f).
        for state in [
            TransportState::Playing,
            TransportState::Paused,
            TransportState::Interrupted,
        ] {
            for verb in [
                ShellVerb::RateUp,
                ShellVerb::RateDown,
                ShellVerb::SkipForward,
                ShellVerb::SkipBackward,
            ] {
                let mut session = session_in(state);
                assert!(
                    session.execute(verb).is_ok(),
                    "a {verb} press from {state:?} must be a live-session control"
                );
            }
        }
        for state in [TransportState::Stopped, TransportState::Completed] {
            for verb in [
                ShellVerb::RateUp,
                ShellVerb::RateDown,
                ShellVerb::SkipForward,
                ShellVerb::SkipBackward,
            ] {
                let mut session = session_in(state);
                assert_rejected(session.execute(verb), state, verb);
            }
        }
    }

    #[test]
    fn resume_and_start_over_offers_exist_only_where_an_offer_exists() {
        // The resume and start-over offers exist only
        // where an offer can exist (Paused, Interrupted);
        // ExternalStart shares the Play legality.
        for state in [TransportState::Paused, TransportState::Interrupted] {
            for verb in [
                ShellVerb::ResumeAtStored,
                ShellVerb::StartOver,
                ShellVerb::ExternalStart,
            ] {
                let mut session = session_in(state);
                assert!(
                    session.execute(verb).is_ok(),
                    "a {verb} press from {state:?} must be a legal offer choice"
                );
            }
        }
        for state in [
            TransportState::Playing,
            TransportState::Stopped,
            TransportState::Completed,
        ] {
            for verb in [
                ShellVerb::ResumeAtStored,
                ShellVerb::StartOver,
                ShellVerb::ExternalStart,
            ] {
                let mut session = session_in(state);
                assert_rejected(session.execute(verb), state, verb);
            }
        }
    }

    #[test]
    fn choose_engine_is_a_no_op_from_every_state() {
        // A selection, not a transition: legal from every
        // state and changes no transport or position
        // state (FR-005).
        for state in [
            TransportState::Playing,
            TransportState::Paused,
            TransportState::Interrupted,
            TransportState::Stopped,
            TransportState::Completed,
        ] {
            let mut session = session_in(state);
            let before = session.snapshot();
            assert!(
                session.execute(ShellVerb::ChooseEngine).is_ok(),
                "ChooseEngine must be a no-op from {state:?}"
            );
            assert_eq!(session.snapshot(), before);
        }
    }
}
