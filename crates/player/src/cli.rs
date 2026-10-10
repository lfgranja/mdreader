//! Player CLI dispatch (T027).
//!
//! Every surface routes through the same dispatch:
//! notification, lock screen, CLI, IPC — all call the same
//! backend function per Principle II.

use crate::engine_core::closed::{PlaybackRate, TransportState};
use crate::position::{
    PlayerError, PositionClock, REWIND_CAP_SECS, REWIND_FRACTION, SKIP_STEP_SECS,
};
use crate::transport::{rate_down_from, rate_up_from};
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

/// Errors raised by the playback session.
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
    #[instrument(skip(self))]
    pub fn execute(&mut self, verb: ShellVerb) -> Result<SessionSnapshot, PlayerError> {
        match verb {
            ShellVerb::Play => {
                self.transport = TransportState::Playing;
                debug!("play pressed -> Playing");
            }
            ShellVerb::Pause => {
                self.transport = TransportState::Paused;
                debug!("pause pressed -> Paused");
            }
            ShellVerb::Stop => {
                self.transport = TransportState::Stopped;
                self.clock = PositionClock::new(0.0);
                debug!("stop pressed -> Stopped, position reset");
            }
            ShellVerb::Complete => {
                self.transport = TransportState::Completed;
                self.clock = PositionClock::new(0.0);
                debug!("complete -> Completed, position reset");
            }
            ShellVerb::Interrupt => {
                self.transport = TransportState::Interrupted;
                debug!("interrupt -> Interrupted");
            }
            ShellVerb::RateUp => {
                self.rate = rate_up_from(self.rate);
                debug!(rate = ?self.rate, "rate up one step");
            }
            ShellVerb::RateDown => {
                self.rate = rate_down_from(self.rate);
                debug!(rate = ?self.rate, "rate down one step");
            }
            ShellVerb::SkipForward => {
                self.clock.skip_forward();
                debug!(position = self.clock.position_secs(), "skip forward");
            }
            ShellVerb::SkipBackward => {
                self.clock.skip_back();
                debug!(position = self.clock.position_secs(), "skip backward");
            }
            ShellVerb::ResumeAtStored => {
                self.transport = TransportState::Playing;
                debug!(position = self.clock.position_secs(), "resume at stored");
            }
            ShellVerb::StartOver => {
                self.clock = PositionClock::new(0.0);
                self.transport = TransportState::Playing;
                debug!("start over -> position 0.0, Playing");
            }
            ShellVerb::ChooseEngine => {
                // Placeholder: engine selection is handled by the
                // fallback selector (T075). No position change.
                debug!("choose engine");
            }
            ShellVerb::ExternalStart => {
                // External start: same Rust code path, mirror-only.
                self.transport = TransportState::Playing;
                debug!("external start -> Playing");
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
        let snap = session.execute(ShellVerb::Play).unwrap();
        assert_eq!(snap.transport, TransportState::Playing);
    }

    #[test]
    fn pause_transitions_to_paused() {
        let mut session = PlaybackSession::new();
        session.execute(ShellVerb::Play).unwrap();
        let snap = session.execute(ShellVerb::Pause).unwrap();
        assert_eq!(snap.transport, TransportState::Paused);
    }

    #[test]
    fn stop_resets_position() {
        let mut session = PlaybackSession::new();
        session.execute(ShellVerb::Play).unwrap();
        session.on_audible_progress(10.0);
        session.execute(ShellVerb::Stop).unwrap();
        assert_eq!(session.snapshot().position_secs, 0.0);
    }

    #[test]
    fn audible_progress_advances_only_while_playing() {
        let mut session = PlaybackSession::new();
        // Not playing — progress ignored.
        session.on_audible_progress(5.0);
        assert_eq!(session.snapshot().position_secs, 0.0);

        // Now playing — progress advances.
        session.execute(ShellVerb::Play).unwrap();
        session.on_audible_progress(5.0);
        assert_eq!(session.snapshot().position_secs, 5.0);
    }

    #[test]
    fn skip_forward_moves_five_seconds() {
        let mut session = PlaybackSession::new();
        session.execute(ShellVerb::Play).unwrap();
        session.execute(ShellVerb::SkipForward).unwrap();
        assert_eq!(session.snapshot().position_secs, SKIP_STEP_SECS);
    }

    #[test]
    fn skip_back_clamps_at_zero() {
        let mut session = PlaybackSession::new();
        session.execute(ShellVerb::SkipBackward).unwrap();
        assert_eq!(session.snapshot().position_secs, 0.0);
    }

    #[test]
    fn rate_up_saturates_at_r4_0() {
        let mut session = PlaybackSession::new();
        // Walk to R4_0
        for _ in 0..10 {
            session.execute(ShellVerb::RateUp).unwrap();
        }
        assert_eq!(session.snapshot().rate, PlaybackRate::R4_0);
    }

    #[test]
    fn rate_down_saturates_at_r0_75() {
        let mut session = PlaybackSession::new();
        for _ in 0..10 {
            session.execute(ShellVerb::RateDown).unwrap();
        }
        assert_eq!(session.snapshot().rate, PlaybackRate::R0_75);
    }

    #[test]
    fn external_start_is_playing() {
        let mut session = PlaybackSession::new();
        session.execute(ShellVerb::ExternalStart).unwrap();
        assert_eq!(session.snapshot().transport, TransportState::Playing);
    }
}
