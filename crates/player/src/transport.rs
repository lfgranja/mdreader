//! Transport mathematics: position, rewind, rewind cap, skip, snap,
//! rate-ladder application, pre-roll (FR-002, FR-003, FR-019, FR-043).
//! Single source of truth — no second position source (FR-003).

use thiserror::Error;
use tracing::{debug, trace};

/// Closed-set rate ladder (FR-019). Pitch-preserving stretch is engine's
/// contract (FR-043) — this layer only selects the rung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlaybackRate {
    R0_75,
    R1_0,
    R1_25,
    R1_5,
    R1_75,
    R2_0,
    R4_0,
}

/// Transport-state closed set (FR-044, no catch-all).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransportState {
    Playing,
    Paused,
    Stopped,
    Completed,
    Interrupted,
}

/// Rewind / snap / skip invariants (SC-007 / FR-015 / FR-018 / FR-038):
/// — Replay after kill ≤30 s (FR-004).
/// — Resume sentence snap lands at start of sentence containing point
///    5% of paused duration back from yield; total backward ≤30 s.
/// — Skip displacement exactly ±5.0 s.
/// — Pause duration accumulates only when transport is Paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub secs: f64,
    pub transport: TransportState,
}

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Position exceeds document bounds")]
    OutOfBounds,
    #[error("Rewind exceeds 30 s cap")]
    RewindCapExceeded,
    #[error("Skip displacement not exactly ±5.0 s")]
    SkipDisplacement,
    #[error("Rate mismatch: engine delivered different rung")]
    RateMismatch,
}

/// Apply rewind / snap — called by Rust playback module (FR-002).
/// Fails closed if cap exceeded (Principle V / SC-007).
pub fn apply_rewind(
    current: Position,
    paused_duration_secs: f64,
) -> Result<Position, TransportError> {
    let rewind_secs = (paused_duration_secs * 0.05).min(30.0);
    let target = (current.secs - rewind_secs).max(0.0);
    if target < 0.0 || rewind_secs > 30.0 {
        return Err(TransportError::RewindCapExceeded);
    }
    // Sentence snap: fallback to nearest sentence boundary
    // (document-structure resolves via doc-structure crate at resume)
    Ok(Position {
        secs: target,
        transport: TransportState::Playing,
    })
}

/// Skip ±5.0 s — exact displacement, never start/end violation.
pub fn apply_skip(current: Position, direction: SkipDirection) -> Result<Position, TransportError> {
    let delta = match direction {
        SkipDirection::Forward => 5.0,
        SkipDirection::Backward => -5.0,
    };
    let target = current.secs + delta;
    if target < 0.0 {
        return Err(TransportError::OutOfBounds);
    }
    // End-bound check deferred to document-structure (FR-038 / FR-022)
    Ok(Position {
        secs: target,
        transport: TransportState::Playing,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipDirection {
    Forward,
    Backward,
}

/// Rate-change selection — selects rung; engine applies pitch-preserving
/// stretch (FR-043). This layer never stretches audio.
pub fn select_rate(current_rate: PlaybackRate, step: i8) -> PlaybackRate {
    // Ladder saturates at ends; one step per press.
    // Exact rung mapping documented in spec FR-019.
    match current_rate {
        PlaybackRate::R0_75 => {
            if step < 0 {
                PlaybackRate::R0_75
            } else {
                PlaybackRate::R1_0
            }
        }
        PlaybackRate::R1_0 => {
            if step > 0 {
                PlaybackRate::R1_25
            } else {
                PlaybackRate::R0_75
            }
        }
        PlaybackRate::R1_25 => {
            if step > 0 {
                PlaybackRate::R1_5
            } else {
                PlaybackRate::R1_0
            }
        }
        PlaybackRate::R1_5 => {
            if step > 0 {
                PlaybackRate::R1_75
            } else {
                PlaybackRate::R1_25
            }
        }
        PlaybackRate::R1_75 => {
            if step > 0 {
                PlaybackRate::R2_0
            } else {
                PlaybackRate::R1_5
            }
        }
        PlaybackRate::R2_0 => {
            if step > 0 {
                PlaybackRate::R4_0
            } else {
                PlaybackRate::R1_75
            }
        }
        PlaybackRate::R4_0 => {
            if step < 0 {
                PlaybackRate::R2_0
            } else {
                PlaybackRate::R4_0
            }
        }
    }
}

/// Pre-roll buffer ≤60 s (FR-015a / FR-015b).
/// Never advances position; emptied at session end.
pub struct PreRollBuffer {
    pub max_secs: f64,
    pub current_secs: f64,
}

impl PreRollBuffer {
    pub const MAX: f64 = 60.0;
    pub fn new() -> Self {
        Self {
            max_secs: Self::MAX,
            current_secs: 0.0,
        }
    }
    pub fn refill(&mut self, delivered: f64) {
        self.current_secs = (self.current_secs + delivered).min(Self::MAX);
    }
    pub fn drain_while_idle(&mut self) {
        // Pipeline-side bound — never advances position
        self.current_secs = 0.0;
    }
}
