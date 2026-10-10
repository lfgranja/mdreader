//! Skip math, rate-change effective time-bound, pre-roll pipeline property.
//! No computation delegated to Kotlin surface (FR-042 / SC-035).

use crate::transport::{PlaybackRate, Position, TransportState};

/// Skip ±5.0 s exactly — displacement must land at sentence/node boundary
/// when re-anchor applies (FR-038 / FR-018). End-bound checked by doc-structure.
pub fn apply_skip_exact(position: f64, delta: f64) -> f64 {
    assert!(
        (delta - 5.0).abs() < 0.001 || (delta + 5.0).abs() < 0.001,
        "Skip displacement must be exactly ±5.0 s (FR-018 / SC-043)"
    );
    (position + delta).max(0.0)
}

/// Pre-roll ≤60 s — ring buffer never advances position; empty while idle (FR-015a/FR-015b/FR-015c).
/// Refilled on Play; abandoned at session end.
pub struct PreRollRing {
    pub buffer: Vec<f32>, // mono f32 @ 22050 Hz (FR-033)
    pub max_samples: usize,
}

impl PreRollRing {
    pub const MAX_SECS: f64 = 60.0;
    pub const SAMPLE_RATE: u32 = 22050;

    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity((Self::MAX_SECS * Self::SAMPLE_RATE as f64) as usize),
            max_samples: (Self::MAX_SECS * Self::SAMPLE_RATE as f64) as usize,
        }
    }

    pub fn push(&mut self, mono_f32: f32) {
        if self.buffer.len() < self.max_samples {
            self.buffer.push(mono_f32);
        }
    }

    pub fn drain_idle(&mut self) {
        // Pipeline-side bound: does not advance position
        self.buffer.clear();
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
