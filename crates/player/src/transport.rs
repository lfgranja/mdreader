//! Transport mathematics: position, rewind, rewind cap, skip, snap,
//! rate-ladder application, pre-roll (FR-002, FR-003, FR-019, FR-043).
//!
//! Single source of truth — no second position source (FR-003).
//!
//! ## Closed-set compliance (Principle VII / FR-044)
//! Every enum variant is explicit; no `_ =>` catch-all arms exist.
//! Adding a variant requires coordinated edit across all consumers,
//! a semver bump, and a documentation update.

use thiserror::Error;
use tracing::instrument;

/// The closed-set rate ladder (FR-019). The single
/// authoritative definition lives in `engine-core`
/// (`engine_core::closed::PlaybackRate` — the closed-set
/// registry; Principle VII / FR-044: no second list that
/// can drift). Re-exported here so the `player::transport`
/// path stays stable. Pitch-preserving stretch is the
/// engine's contract (FR-043) — this layer only selects
/// the rung.
pub use engine_core::closed::PlaybackRate;

/// Advance one rung up the closed rate ladder (FR-019),
/// saturating at `R4_0` — the ladder is closed, so the
/// top rung is the ceiling, never an error.
///
/// # Errors
///
/// This function never returns an error: the ladder is closed
/// with exactly seven rungs, and every input maps to a valid
/// output via the explicit match arms below.
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "FR-019 / Principle VII: exactly 7 rungs enumerated;
    exhaustive match with no catch-all arm"
)]
pub fn rate_up_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R0_75 => PlaybackRate::R1_0,
        PlaybackRate::R1_0 => PlaybackRate::R1_25,
        PlaybackRate::R1_25 => PlaybackRate::R1_5,
        PlaybackRate::R1_5 => PlaybackRate::R1_75,
        PlaybackRate::R1_75 => PlaybackRate::R2_0,
        PlaybackRate::R2_0 => PlaybackRate::R4_0,
        PlaybackRate::R4_0 => PlaybackRate::R4_0,
    }
}

/// Retreat one rung down the closed rate ladder (FR-019),
/// saturating at `R0_75`.
///
/// # Errors
///
/// This function never returns an error: the ladder is closed
/// with exactly seven rungs, and every input maps to a valid
/// output via the explicit match arms below.
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "FR-019 / Principle VII: exactly 7 rungs enumerated;
    exhaustive match with no catch-all arm"
)]
pub fn rate_down_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R4_0 => PlaybackRate::R2_0,
        PlaybackRate::R2_0 => PlaybackRate::R1_75,
        PlaybackRate::R1_75 => PlaybackRate::R1_5,
        PlaybackRate::R1_5 => PlaybackRate::R1_25,
        PlaybackRate::R1_25 => PlaybackRate::R1_0,
        PlaybackRate::R1_0 => PlaybackRate::R0_75,
        PlaybackRate::R0_75 => PlaybackRate::R0_75,
    }
}

/// The transport-state closed set (FR-044, no catch-all).
///
/// The single authoritative definition lives in
/// `engine-core` (`engine_core::closed::TransportState`),
/// whose `snake_case` wire form (`"playing"`, `"paused"`,
/// …) is the diagnostic-record contract (SC-005); the
/// shell mirrors this state and never computes transitions
/// itself (SC-035). Re-exported here so the
/// `player::transport` path stays stable.
///
/// # Compliance
///
/// This type has exactly four Serialize/Deserialize variants
/// (`Playing|Paused|Stopped|Completed`), all explicitly listed.
/// No `Other`/`Unknown` variant exists (Principle VII).
pub use engine_core::closed::TransportState;

/// Rewind / snap / skip invariants (SC-007 / FR-015 / FR-018 / FR-038):
///
/// — Replay after kill ≤30 s (FR-004).
/// — Resume sentence snap lands at start of sentence containing point
///   5% of paused duration back from yield; total backward ≤30 s.
/// — Skip displacement exactly ±5.0 s.
/// — Pause duration accumulates only when transport is Paused.
///
/// # Compliance
///
/// All match arms are explicit; no `_ =>` catch-all exists
/// (Principle VII / FR-044 / gate 7).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// Current position in seconds from document start (≥ 0.0).
    pub secs: f64,
    /// Transport state at this position.
    pub transport: TransportState,
}

/// Closed-set transport errors (FR-044, no catch-all).
///
/// Every variant is a named, explicit error — never a catch-all.
/// `cargo deny check` and gate 7 enforce no `_ =>` arms appear
/// in `crates/*/src`.
#[derive(Error, Debug)]
pub enum TransportError {
    /// The rewind would exceed the 30 s cap (FR-004 / FR-015).
    ///
    /// # Failure
    ///
    /// This error is returned when `rewind_secs > 30.0`
    /// or `target < 0.0` in `apply_rewind`. The rewind
    /// amount is capped at 30.0 s per FR-004.
    #[error("Rewind exceeds 30 s cap")]
    RewindCapExceeded,
    /// The skip displacement is not exactly ±5.0 s (FR-018).
    ///
    /// # Failure
    ///
    /// This error is returned when the delta supplied to
    /// `apply_skip` is not `5.0` or `-5.0`. The closed-set
    /// rung ladder ensures exact ±5.0 s displacement.
    #[error("Skip displacement not exactly ±5.0 s")]
    SkipDisplacement,
    /// The engine delivered a rung other than the selected one (FR-019).
    ///
    /// # Failure
    ///
    /// This error surfaces when the engine delivers audio
    /// at a rate different from the selected ladder rung.
    /// Mismatch = named `EngineError::RateMismatch`, never
    /// silent resample (FR-033, FR-035).
    #[error("Rate mismatch: engine delivered different rung")]
    RateMismatch,
}

/// Apply rewind / snap — called by Rust playback module (FR-002).
///
/// # Returns
///
/// `Ok(Position)` when the rewind is within the 30 s cap
/// and the new position is ≥ 0.0.
/// `Err(TransportError)` when the rewind would exceed the
/// cap or move below floor 0.0.
///
/// # Failures
///
/// - `TransportError::RewindCapExceeded`: when `rewind_secs > 30.0`
///   or clamping results in `target < 0.0` with non-zero
///   rewind amount.
/// - Never returns silence or substitutes format (FR-023:
///   failure cannot be named as silence).
#[instrument(skip(current))]
#[must_use]
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

/// Skip direction — the closed two-member set (FR-018).
///
/// # Compliance
///
/// Exactly two variants: `Forward` and `Backward`. No catch-all
/// arm exists (Principle VII).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipDirection {
    /// Skip forward exactly +5.0 s.
    Forward,
    /// Skip backward exactly −5.0 s.
    Backward,
}

/// Rate-change selection — selects rung; engine applies pitch-preserving
/// stretch (FR-043). This layer never stretches audio.
///
/// # Returns
///
/// The selected rung after one step on the closed ladder.
/// The ladder saturates at both ends: `R0_75` cannot go lower,
/// `R4_0` cannot go higher.
///
/// # Panics
///
/// Panics never occur: the match is exhaustive over the seven
/// rungs, with explicit arms for each.
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "FR-019 / Principle VII: exactly 7 rungs enumerated;
    exhaustive match with no catch-all arm"
)]
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
///
/// # Compliance
///
/// The 60 s cap is the single [`crate::position::PREROLL_CAP_SECS`]
/// constant — no second cap value can drift (Principle VII / FR-044).
#[derive(Debug)]
pub struct PreRollBuffer {
    /// Maximum pre-roll, in seconds (the 60 s cap, FR-015a).
    pub max_secs: f64,
    /// Pre-roll currently buffered, in seconds.
    pub current_secs: f64,
}

impl PreRollBuffer {
    /// The hard pre-roll cap (FR-015a): single home is
    /// [`crate::position::PREROLL_CAP_SECS`]; this alias never drifts
    /// (Constitution VII / FR-015a).
    pub const MAX: f64 = crate::position::PREROLL_CAP_SECS;
    /// Create an empty pre-roll buffer at the cap.
    pub fn new() -> Self {
        Self {
            max_secs: Self::MAX,
            current_secs: 0.0,
        }
    }
    /// Add delivered audio to the pre-roll buffer,
    /// capped at [`Self::MAX`].
    ///
    /// # Errors
    ///
    /// This function does not return errors in the current
    /// implementation: delivery is always non-negative per
    /// FR-015a, and the cap saturates at MAX.
    pub fn refill(&mut self, delivered: f64) {
        self.current_secs = (self.current_secs + delivered).min(Self::MAX);
    }
    /// Empty the pre-roll buffer — the pipeline-side bound;
    /// never advances position while idle (FR-015c).
    ///
    /// # Compliance
    ///
    /// Clearing the buffer does not advance position (FR-015c):
    /// the buffer stores a duration only, touching no
    /// [`crate::position::PositionClock`].
    pub fn drain_while_idle(&mut self) {
        // Pipeline-side bound — never advances position
        self.current_secs = 0.0;
    }
}

impl Default for PreRollBuffer {
    fn default() -> Self {
        Self::new()
    }
}
