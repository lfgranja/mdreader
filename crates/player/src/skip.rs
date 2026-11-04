//! Skip math, rate-change effective time-bound, pre-roll pipeline property.
//! No computation delegated to Kotlin surface (FR-042 / SC-035).

use crate::transport::{PlaybackRate, SkipDirection};
use doc_structure::{Document, Heading};

/// Skip ±5.0 s exactly — displacement must land at sentence/node boundary
/// when re-anchor applies (FR-038 / FR-018).
///
/// Callers pass ±5.0 s by construction (the closed
/// [`SKIP_STEP_SECS`](crate::position::SKIP_STEP_SECS) step); the canonical
/// clamped path — direction-typed so an off-±5.0 displacement is
/// unrepresentable — is [`skip_clamped`].
#[must_use]
pub fn apply_skip_exact(position: f64, delta: f64) -> f64 {
    (position + delta).max(0.0)
}

/// Skip exactly ±5.0 s from `position`, clamped into `[0.0, duration]`
/// (FR-018 / SC-016): `+5.0` for [`SkipDirection::Forward`], `-5.0` for
/// [`SkipDirection::Backward`].
///
/// The direction-typed signature admits only
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
/// document headings (FR-038 / FR-002).
///
/// The lock-screen skip control jumps to the NEXT
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
        clippy::cast_possible_truncation,
        reason = "the product PREROLL_CAP_SECS × 22 050 = 1 323 000 is an exact integer, far below usize range"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "MAX_SECS and SAMPLE_RATE are positive constants, so their product is positive — no sign bit is lost in the cast"
    )]
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity((Self::MAX_SECS * f64::from(Self::SAMPLE_RATE)) as usize),
            max_samples: (Self::MAX_SECS * f64::from(Self::SAMPLE_RATE)) as usize,
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

/// A skip-back inside this many seconds of a chapter start goes to the previous chapter.
pub const CHAPTER_RESTART_WINDOW_SECS: f64 = 3.0;

/// Remaining seconds at which the next chapter's pre-roll synthesis starts (FR-014).
pub const NEXT_CHAPTER_PREROLL_TRIGGER_SECS: f64 = 10.0;

/// Contextual skip-forward: at the last section, advance to chapter N+1,
/// bypassing chapters already marked `Skipped`.
///
/// Returns `Ok(true)` when the queue advanced; `Ok(false)` when not on the
/// last section or there is no further playable chapter.
///
/// # Errors
///
/// Returns [`crate::queue::ChapterQueueError`] on an out-of-bounds queue state.
pub fn contextual_skip_forward(
    queue: &mut crate::queue::ChapterQueue,
    on_last_section: bool,
) -> Result<bool, crate::queue::ChapterQueueError> {
    use crate::queue::ChapterStatus;
    if !on_last_section {
        return Ok(false);
    }
    let from = queue.current_index();
    let target = (from.saturating_add(1)..queue.len()).find(|&i| {
        queue
            .items()
            .get(i)
            .is_some_and(|c| c.playback_status != ChapterStatus::Skipped)
    });
    let Some(target) = target else {
        return Ok(false);
    };
    queue.set_current_index(target)?;
    Ok(true)
}

/// Contextual skip-back: within the first 3.0 s go to chapter N-1 (skipping
/// refused chapters), otherwise restart the current chapter.
///
/// Returns `Ok(true)` when the queue moved to the previous chapter.
///
/// # Errors
///
/// Returns [`crate::queue::ChapterQueueError`] on an out-of-bounds queue state.
pub fn contextual_skip_backward(
    queue: &mut crate::queue::ChapterQueue,
    position_secs: f64,
) -> Result<bool, crate::queue::ChapterQueueError> {
    use crate::queue::ChapterStatus;
    if position_secs > CHAPTER_RESTART_WINDOW_SECS {
        return Ok(false);
    }
    let from = queue.current_index();
    let target = (0..from).rev().find(|&i| {
        queue
            .items()
            .get(i)
            .is_some_and(|c| c.playback_status != ChapterStatus::Skipped)
    });
    let Some(target) = target else {
        return Ok(false);
    };
    queue.set_current_index(target)?;
    Ok(true)
}

/// Cancellable pre-roll of the next chapter (FR-014).
#[derive(Debug, Default)]
pub struct NextChapterPreRoll {
    /// Pre-synthesized mono-`f32` PCM for the pending chapter.
    pub buffer: Vec<f32>,
    pending: Option<(usize, PreRollToken)>,
}

impl NextChapterPreRoll {
    /// An idle pre-roll.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            pending: None,
        }
    }

    /// Whether `remaining_secs` has reached the trigger.
    #[must_use]
    pub fn should_trigger(&self, remaining_secs: f64) -> bool {
        remaining_secs <= NEXT_CHAPTER_PREROLL_TRIGGER_SECS
    }

    /// Begins pre-rolling `chapter_index`; any prior pre-roll is cancelled
    /// first. Returns the cooperative token for the synthesis worker.
    pub fn begin(&mut self, chapter_index: usize) -> PreRollToken {
        self.cancel_and_purge();
        let token = PreRollToken::default();
        self.pending = Some((chapter_index, token.clone()));
        token
    }

    /// The chapter being pre-rolled, if any.
    #[must_use]
    pub fn pending_chapter(&self) -> Option<usize> {
        self.pending.as_ref().map(|(index, _)| *index)
    }

    /// Cancels the in-flight pre-roll and purges its buffer.
    pub fn cancel_and_purge(&mut self) {
        if let Some((_, token)) = self.pending.take() {
            token.cancel();
        }
        self.buffer.clear();
    }
}

/// Cooperative cancellation token shared with the synthesis worker.
#[derive(Debug, Clone, Default)]
pub struct PreRollToken {
    flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl PreRollToken {
    /// Requests cancellation; the worker observes it at its next poll.
    pub fn cancel(&self) {
        self.flag.store(true, std::sync::atomic::Ordering::Release);
    }

    /// Whether the pre-roll was cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// Rate-change effective ≤2 s (FR-019c). Engine applies pitch-preserving stretch;
/// this layer selects rung only (FR-043 contract).
#[must_use]
pub const fn rate_change_effective_time(rung: PlaybackRate) -> f64 {
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
