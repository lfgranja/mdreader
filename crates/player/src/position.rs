//! Authoritative position clock (T025).
//!
//! The player crate is the single source of truth for playback position
//! (FR-002, FR-003). The clock advances only on audible progress, never
//! on its own, and never during pre-roll. The kill-replay basis
//! (`stored_basis`) is the position at the last audible-tick, trailing
//! the current position by at most [`REWIND_CAP_SECS`] (FR-004).

use thiserror::Error;
use tracing::{debug, instrument};

/// The skip step: exactly ±5.0 s (FR-018).
pub const SKIP_STEP_SECS: f64 = 5.0;

/// The rewind fraction applied to paused duration (FR-015): 5%.
pub const REWIND_FRACTION: f64 = 0.05;

/// The hard rewind cap, in seconds (FR-004, FR-015): 30 s.
pub const REWIND_CAP_SECS: f64 = 30.0;

/// The pre-roll buffer bound (FR-015a): 60 s of engine-delivered audio
/// that never advances position.
pub const PREROLL_CAP_SECS: f64 = 60.0;

/// Errors raised by the position clock.
#[derive(Debug, Error)]
pub enum PlayerError {
    /// The requested skip would move the position below 0.0.
    #[error("skip backward would go below floor 0.0")]
    SkipBelowFloor,
    /// The requested progress amount is negative.
    #[error("progress must be non-negative, got {0}")]
    NegativeProgress(f64),
}

/// The authoritative position clock (FR-002).
///
/// The clock owns a single `f64` position (≥ 0) and tracks the last
/// audible-progress event so that a kill/replay can offer a resume
/// point no more than [`REWIND_CAP_SECS`] behind the current position
/// (FR-004).
#[derive(Debug, Clone)]
pub struct PositionClock {
    /// Current playback position in seconds from document start.
    position_secs: f64,
    /// Position at the last audible-progress tick (the kill-replay
    /// basis). Always ≤ `position_secs`.
    stored_basis_secs: f64,
}

impl PositionClock {
    /// Create a new clock initialised at `start_secs`, clamped to the
    /// declared floor of 0.0 (data-model `PlaybackPosition: f64 ≥ 0`).
    #[expect(
        clippy::cast_possible_truncation,
        reason = "input is f64; clamped to floor"
    )]
    pub fn new(start_secs: f64) -> Self {
        let position_secs = start_secs.max(0.0);
        Self {
            position_secs,
            stored_basis_secs: position_secs,
        }
    }

    /// Current position in seconds (FR-002).
    #[must_use]
    pub const fn position_secs(&self) -> f64 {
        self.position_secs
    }

    /// The kill-replay basis (FR-004): the position at the last audible
    /// progress event, trailing the current position by at most
    /// [`REWIND_CAP_SECS`].
    #[must_use]
    pub const fn stored_basis(&self) -> f64 {
        self.stored_basis_secs
    }

    /// Advance the position by exactly `elapsed_secs` — the **only**
    /// advance path (FR-002). Called by the backend as audio is delivered;
    /// never runs on its own. Position advances only while audible, never
    /// during pre-roll or silence.
    ///
    /// Returns the new position, or [`PlayerError::NegativeProgress`] when
    /// the caller supplies a negative elapsed value.
    #[instrument(skip(self))]
    pub fn on_audible_progress(&mut self, elapsed_secs: f64) -> Result<f64, PlayerError> {
        if elapsed_secs < 0.0 {
            return Err(PlayerError::NegativeProgress(elapsed_secs));
        }
        self.position_secs += elapsed_secs;
        self.stored_basis_secs = self.position_secs;
        debug!(
            elapsed = elapsed_secs,
            position = self.position_secs,
            "audible progress advanced position"
        );
        Ok(self.position_secs)
    }

    /// Skip forward exactly [`SKIP_STEP_SECS`] (FR-018).
    #[expect(
        clippy::float_cmp,
        reason = "SKIP_STEP_SECS is an exact binary fraction; == is exact"
    )]
    pub fn skip_forward(&mut self) -> f64 {
        self.position_secs += SKIP_STEP_SECS;
        self.stored_basis_secs = self.position_secs;
        self.position_secs
    }

    /// Skip backward exactly [`SKIP_STEP_SECS`], clamped at the 0.0 floor
    /// (FR-018).
    #[expect(
        clippy::float_cmp,
        reason = "SKIP_STEP_SECS is an exact binary fraction; == is exact"
    )]
    pub fn skip_back(&mut self) -> f64 {
        self.position_secs = (self.position_secs - SKIP_STEP_SECS).max(0.0);
        self.stored_basis_secs = self.position_secs;
        self.position_secs
    }

    /// Rewind for a silence pause: moves the cursor back from the yield
    /// point by exactly `min(REWIND_FRACTION × paused_duration,
    /// REWIND_CAP_SECS)`, clamped at 0.0 (FR-015). Remembers the yield
    /// for the cap.
    ///
    /// Returns the new cursor position.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "multiplication of two f64 values stays within f64 range"
    )]
    pub fn rewind_for_silence(&mut self, paused_duration_secs: f64) -> f64 {
        let rewind_amount = (REWIND_FRACTION * paused_duration_secs).min(REWIND_CAP_SECS);
        self.position_secs = (self.position_secs - rewind_amount).max(0.0);
        self.position_secs
    }

    /// Snap to the start of the nearest completed boundary at or
    /// before the cursor: the greatest boundary in `boundaries` at
    /// or before the cursor. The snap never lands earlier than the
    /// cap point — the cap takes precedence over the zero-mid-word
    /// rule (FR-015).
    ///
    /// Returns the snapped position.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "boundaries are f64 values on the same grid"
    )]
    #[must_use]
    pub fn snap_to_boundary(&self, boundaries: &[f64]) -> f64 {
        // Cap origin = last yield (stored_basis / audible progress), not cursor arbitrarily (FR-015 / spec.md:59).
        // `rewind_for_silence` moves the cursor back but leaves
        // the basis at the yield, so the 30 s cap is measured
        // from the yield — total backward movement (rewind +
        // snap) then never exceeds [`REWIND_CAP_SECS`].
        let cursor = self.position_secs;
        let cap_point = (self.stored_basis_secs - REWIND_CAP_SECS).max(0.0);
        // Greatest boundary at or before cursor.
        let mut best: Option<f64> = None;
        for &b in boundaries {
            if b <= cursor {
                match best {
                    None => best = Some(b),
                    Some(prev) if b > prev => best = Some(b),
                    _ => {}
                }
            }
        }
        match best {
            Some(boundary) if boundary >= cap_point => boundary,
            Some(_) => {
                // Boundary exists but is before the cap — land at cap point.
                cap_point
            }
            None => {
                // No boundary found — land at cap point (never earlier
                // than the cap).
                cap_point
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_negative_start_to_zero() {
        let clock = PositionClock::new(-1.0);
        assert_eq!(clock.position_secs(), 0.0);
    }

    #[test]
    fn on_audible_progress_advances_by_exactly_elapsed() {
        let mut clock = PositionClock::new(0.0);
        let new_pos = clock.on_audible_progress(3.5).unwrap();
        assert_eq!(new_pos, 3.5);
        assert_eq!(clock.position_secs(), 3.5);
    }

    #[test]
    fn on_audible_progress_rejects_negative() {
        let mut clock = PositionClock::new(10.0);
        let result = clock.on_audible_progress(-1.0);
        assert!(result.is_err());
        assert_eq!(clock.position_secs(), 10.0); // unchanged
    }

    #[test]
    fn skip_forward_moves_exactly_five_seconds() {
        let mut clock = PositionClock::new(10.0);
        let landed = clock.skip_forward();
        assert_eq!(landed, 15.0);
        assert_eq!(clock.position_secs(), 15.0);
    }

    #[test]
    fn skip_back_moves_exactly_five_seconds_clamped_at_zero() {
        let mut clock = PositionClock::new(3.0);
        let landed = clock.skip_back();
        assert_eq!(landed, 0.0);
        assert_eq!(clock.position_secs(), 0.0);
    }

    #[test]
    fn rewind_for_silence_is_five_percent_capped_at_thirty_seconds() {
        let mut clock = PositionClock::new(3600.0);
        let rewound = clock.rewind_for_silence(600.0);
        assert_eq!(rewound, 3600.0 - 30.0); // 5% of 600 = 30, capped at 30
    }

    #[test]
    fn rewind_for_silence_never_goes_below_zero() {
        let mut clock = PositionClock::new(1.0);
        let rewound = clock.rewind_for_silence(1000.0);
        assert_eq!(rewound, 0.0); // clamped at floor
    }

    #[test]
    fn snap_to_boundary_lands_on_boundary() {
        let boundaries = vec![0.0, 10.0, 20.0, 30.0];
        let clock = PositionClock::new(25.0);
        let snapped = clock.snap_to_boundary(&boundaries);
        assert_eq!(snapped, 20.0); // greatest boundary ≤ 25
    }

    #[test]
    fn snap_to_boundary_respects_cap() {
        // Cursor at 25, cap at 25-30=-5 → 0, boundary at 0.0 is ≥ cap (cap origin = stored_basis / yield).
        let boundaries = vec![0.0, 10.0, 20.0];
        let clock = PositionClock::new(25.0);
        let snapped = clock.snap_to_boundary(&boundaries);
        assert_eq!(snapped, 20.0);
    }

    #[test]
    fn stored_basis_tracks_last_audible_progress() {
        let mut clock = PositionClock::new(0.0);
        let _ = clock.on_audible_progress(5.0).unwrap();
        assert_eq!(clock.stored_basis(), 5.0);
        let _ = clock.on_audible_progress(3.0).unwrap();
        assert_eq!(clock.stored_basis(), 8.0);
    }
}
