//! Skip math, rate-change effective time-bound, pre-roll pipeline property.
//! No computation delegated to Kotlin surface (FR-042 / SC-035).

use crate::transport::{PlaybackRate, SkipDirection};

/// Skip ±5.0 s exactly — displacement must land at sentence/node boundary
/// when re-anchor applies (FR-038 / FR-018). Callers pass ±5.0 s by
/// construction (the closed [`SKIP_STEP_SECS`](crate::position::SKIP_STEP_SECS)
/// step); the canonical clamped path — direction-typed so an off-±5.0
/// displacement is unrepresentable — is [`skip_clamped`].
pub fn apply_skip_exact(position: f64, delta: f64) -> f64 {
    (position + delta).max(0.0)
}

/// Skip exactly ±5.0 s from `position`, clamped into `[0.0, duration]`
/// (FR-018 / SC-016): `+5.0` for [`SkipDirection::Forward`], `-5.0` for
/// [`SkipDirection::Backward`]. The direction-typed signature admits only
/// the two closed displacements, so no assertion or error arm is needed —
/// the clamp-without-error rule holds by construction and this function
/// never panics and never errors.
#[must_use]
pub fn skip_clamped(position: f64, direction: SkipDirection, duration: f64) -> f64 {
    let delta = match direction {
        SkipDirection::Forward => 5.0,
        SkipDirection::Backward => -5.0,
    };
    (position + delta).max(0.0).min(duration)
}

/// Pre-roll ≤60 s — ring buffer never advances position; empty while idle (FR-015a/FR-015b/FR-015c).
/// Refilled on Play; abandoned at session end.
#[derive(Debug)]
pub struct PreRollRing {
    /// Buffered mono-`f32` PCM at the declared 22 050 Hz (FR-033).
    pub buffer: Vec<f32>,
    /// Ring capacity, in samples (`MAX_SECS × SAMPLE_RATE`).
    pub max_samples: usize,
}

impl PreRollRing {
    /// The hard pre-roll cap (FR-015a): single home is
    /// [`crate::position::PREROLL_CAP_SECS`]; this alias never drifts
    /// (Constitution VII / FR-015a). Delegation to
    /// [`crate::buffer::PreRollBuffer`] was considered but rejected:
    /// that type tracks buffered seconds with Play-gating, while this
    /// ring tracks per-sample PCM capacity — different API, so an
    /// alias preserves behavior exactly.
    pub const MAX_SECS: f64 = crate::position::PREROLL_CAP_SECS;
    /// The declared handoff sample rate (FR-024 / FR-033): 22 050 Hz.
    pub const SAMPLE_RATE: u32 = 22050;

    /// Create an empty ring with capacity for the full 60 s cap.
    #[expect(
        clippy::cast_precision_loss,
        reason = "SAMPLE_RATE (22 050) and MAX_SECS (PREROLL_CAP_SECS) are exact values; their product is an exact f64 integer"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the product PREROLL_CAP_SECS × 22 050 = 1 323 000 is an exact integer, far below usize range"
    )]
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity((Self::MAX_SECS * Self::SAMPLE_RATE as f64) as usize),
            max_samples: (Self::MAX_SECS * Self::SAMPLE_RATE as f64) as usize,
        }
    }

    /// Push one mono-`f32` sample into the ring, up to capacity.
    pub fn push(&mut self, mono_f32: f32) {
        if self.buffer.len() < self.max_samples {
            self.buffer.push(mono_f32);
        }
    }

    /// Empty the ring — the pipeline-side bound; never advances
    /// position while idle (FR-015c).
    pub fn drain_idle(&mut self) {
        // Pipeline-side bound: does not advance position
        self.buffer.clear();
    }
}

impl Default for PreRollRing {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate-change effective ≤2 s (FR-019c). Engine applies pitch-preserving stretch;
/// this layer selects rung only (FR-043 contract).
pub fn rate_change_effective_time(rung: PlaybackRate) -> f64 {
    match rung {
        PlaybackRate::R0_75
        | PlaybackRate::R1_0
        | PlaybackRate::R1_25
        | PlaybackRate::R1_5
        | PlaybackRate::R1_75
        | PlaybackRate::R2_0
        | PlaybackRate::R4_0 => 2.0,
    }
}
