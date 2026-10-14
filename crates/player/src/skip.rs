//! Skip math, rate-change effective time-bound, pre-roll pipeline property.
//! No computation delegated to Kotlin surface (FR-042 / SC-035).

use crate::transport::{PlaybackRate, SkipDirection};
use doc_structure::{Document, Heading};

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

/// Skip to the next/previous section boundary using
/// document headings (FR-038 / FR-002): the
/// lock-screen skip control jumps to the NEXT
/// section boundary going forward — the first
/// boundary strictly after the press position — and
/// the PREVIOUS one going backward — the greatest
/// boundary strictly before it, because a skip that
/// does not move is not a skip (T013).
///
/// The ±10 s fallback is a ZERO-SECTIONS rule only
/// (FR-002: "falling back to ±10 seconds when the
/// document has no sections"): an empty heading list
/// displaces the position by exactly ±10.0 s. A
/// document WITH sections but no boundary in the
/// skip direction clamps to the document's own end
/// instead — a forward skip past the last section
/// lands at the duration, a backward skip before the
/// first at the `0.0` floor — never at an arbitrary
/// interior point, so the FR-038 re-anchor anchor is
/// preserved by construction.
///
/// The boundary resolution delegates to
/// [`doc_structure::Document`] — the ordered-headings
/// provider normalizes any input order, so the
/// landing is a pure function of the heading set,
/// never of the list's input order. The result is
/// clamped into `[0.0, duration]` (SC-016: a skip
/// that would leave the document clamps to the
/// bound, never an error). Pass `f64::INFINITY` for
/// `duration` on a session with no known end — the
/// identity for the `.min` clamp, the unknown-
/// duration pattern
/// [`PlaybackSession`](crate::cli::PlaybackSession)
/// uses.
///
/// The section-boundary skip is the FR-002 verb
/// path for a document whose headings are known;
/// [`skip_clamped`] remains the canonical ±5.0 s
/// displacement of FR-018, and the verb wiring over
/// the headings lands with the session's heading
/// input (T041).
#[must_use]
pub fn skip_section(
    position: f64,
    headings: &[Heading],
    direction: SkipDirection,
    duration: f64,
) -> f64 {
    let doc = Document::new(headings.to_vec());
    // A document with no sections has no boundary
    // to land on in either direction: the ±10 s
    // displacement is that document's skip.
    let zero_sections = headings.is_empty();
    match direction {
        SkipDirection::Forward => match doc
            .headings()
            .iter()
            .find(|heading| heading.position_secs > position)
        {
            Some(boundary) => boundary.position_secs.max(0.0).min(duration),
            None if zero_sections => (position + 10.0).max(0.0).min(duration),
            // Past the last section of a sectioned
            // document: the skip lands at the document
            // end, never at an interior point.
            None => duration,
        },
        SkipDirection::Backward => {
            // The greatest boundary strictly before the
            // press position — the reverse scan of the
            // provider's ascending list.
            match doc
                .headings()
                .iter()
                .rev()
                .find(|heading| heading.position_secs < position)
            {
                Some(boundary) => boundary.position_secs.max(0.0),
                None if zero_sections => (position - 10.0).max(0.0).min(duration),
                // Before the first section of a sectioned
                // document: the skip lands at the start.
                None => 0.0,
            }
        }
    }
}

/// Pre-roll ≤60 s — ring buffer never advances position; empty while idle (FR-015a/FR-015b/FR-015c).
/// Refilled on Play; abandoned at session end.
#[derive(Debug)]
pub struct PreRollRing {
    /// Buffered mono-`f32` PCM at the declared 22 050 Hz (FR-019) (FR-033, superseded by FR-019).
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
    /// The declared handoff sample rate (FR-024 / FR-019) (FR-033, superseded by FR-019): 22 050 Hz.
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
