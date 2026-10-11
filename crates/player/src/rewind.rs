//! Transient-resume rewind/snap engine (T037; FR-015).
//!
//! When transient focus loss ends, audio resumes behind the yield
//! point by exactly [`cursor`] seconds — `min(0.05 × paused, 30.0)`
//! — snapped to the containing-sentence start with total backward
//! movement hard-capped at 30 s. A cap point landing mid-word snaps
//! forward to the next inter-word gap unless that gap leaves the
//! capped rewind window (then it lands exactly at the cap). When the
//! engine declares no sentence granularity, the resume degrades to
//! the raw cursor — never to document start.
//!
//! The `0.05` / `30.0` formula lives here exactly once in derived
//! form: [`cursor`] delegates to
//! [`position::REWIND_FRACTION`](crate::position::REWIND_FRACTION) and
//! [`position::REWIND_CAP_SECS`](crate::position::REWIND_CAP_SECS),
//! and [`resolve_resume`] calls [`cursor`] — there is no second copy
//! of the formula (Principle VII / FR-044: no second list that can
//! drift). The sentence snap here differs deliberately from
//! [`position::PositionClock::snap_to_boundary`](crate::position::PositionClock::snap_to_boundary):
//! that helper has no word-gap input, so the forward mid-word snap
//! and the no-granularity degrade path live only here.
//!
//! Superseded transients share one cumulative clock: `first_yield_ms`
//! is set once at the first yield and never reset at supersede, and
//! every superseding record links back through `supersedes_ts_ms`
//! (two ordered entries, never a merged rewrite — T035).

use adapter_synthesis::AlignmentGranularity;

use crate::position::{REWIND_CAP_SECS, REWIND_FRACTION};

/// Gap-membership tolerance for the mid-word check, in seconds.
///
/// Boundary lists are engine-declared `f64` positions, so exact
/// equality would misclassify a cap point that merely prints near a
/// gap; the epsilon keeps the check decidable without admitting a
/// real energy peak as a gap.
const GAP_EPS_SECS: f64 = 0.001;

/// Raw rewind distance for a pause, in seconds (FR-015).
///
/// Exactly `min(0.05 × paused_duration_secs, 30.0)` — the single
/// formula, read from the [`crate::position`] constants. A
/// non-positive pause rewinds nothing.
#[must_use]
pub fn cursor(paused_duration_secs: f64) -> f64 {
    (REWIND_FRACTION * paused_duration_secs.max(0.0)).min(REWIND_CAP_SECS)
}

/// Input to one resume resolution (FR-015).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResumeInput<'a> {
    /// Position at the yield/pause point, in seconds (≥ 0.0).
    pub yield_secs: f64,
    /// How long playback stayed paused/yielded, in seconds.
    pub paused_duration_secs: f64,
    /// Declared alignment granularity: `None` degrades to the raw
    /// cursor (never document start); `Word`/`Sentence` snap to the
    /// caller-supplied boundary list for that granularity.
    pub granularity: AlignmentGranularity,
    /// Ascending sentence/word-start boundaries for the declared
    /// granularity, in seconds. Ignored when `granularity` is
    /// [`AlignmentGranularity::None`].
    pub sentence_starts: &'a [f64],
    /// Ascending inter-word gaps (silence, never an energy peak), in
    /// seconds. Used only for the cap-point mid-word forward snap.
    pub word_gaps: &'a [f64],
    /// Cumulative supersede clock: set once at the first yield and
    /// never reset at supersede (T035: cumulative from first yield).
    pub first_yield_ms: u64,
    /// Link to the superseded entry's timestamp, when this resume
    /// supersedes a transient one (T035: two ordered entries with a
    /// supersede link, never a merged rewrite).
    pub supersedes_ts_ms: Option<u64>,
}

/// Resolved resume point (FR-015).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResumeResult {
    /// Where audio resumes, in seconds. Always within
    /// `[cap_point, raw_cursor]` where `cap_point = yield − 30 s`,
    /// so total backward movement never exceeds 30.0 s.
    pub resume_secs: f64,
    /// `yield_secs − resume_secs`: the total backward movement, in
    /// seconds, always within `[0.0, 30.0]`.
    pub backward_secs: f64,
    /// Passed through unchanged: the cumulative clock is never
    /// reset at supersede.
    pub first_yield_ms: u64,
    /// Passed through unchanged: the link to the superseded entry.
    pub supersedes_ts_ms: Option<u64>,
}

/// Supersede link for a transient superseded by a later event
/// (FR-014 latest-event-wins; T035 two-entry record).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupersedeLink {
    /// The preserved cumulative clock — the previous first yield,
    /// never reset.
    pub first_yield_ms: u64,
    /// Timestamp of the superseded entry this record supersedes.
    pub supersedes_ts_ms: Option<u64>,
}

/// Build the supersede link for a new event at `now_ms` that
/// supersedes the entry chain rooted at `previous_first_yield_ms`.
///
/// The first yield passes through untouched; the link points at the
/// superseded entry. No clock is ever reset here.
#[must_use]
pub const fn supersede(previous_first_yield_ms: u64, now_ms: u64) -> SupersedeLink {
    SupersedeLink {
        first_yield_ms: previous_first_yield_ms,
        supersedes_ts_ms: Some(now_ms),
    }
}

/// Resolve where audio resumes after a transient yield (FR-015).
///
/// * Raw cursor = `yield − cursor(paused)`, clamped at `0.0`.
/// * No granularity (`None`) or an empty boundary list degrades to
///   the raw cursor — never to document start.
/// * Otherwise the resume is the containing-sentence start (greatest
///   boundary at or before the raw cursor); when that start lies
///   more than 30 s behind the yield — the sentence is longer than
///   the cap — the resume lands exactly at the cap point.
/// * A cap-point landing that is mid-word snaps forward to the next
///   inter-word gap, unless that gap lies past the raw cursor (past
///   the capped rewind window) — then it lands exactly at the cap.
///   "Beyond the cap" is read here as beyond the window's
///   cursor-side bound: snapping past the raw cursor would resume
///   ahead of the formula point, skipping content the rewind owed.
/// * The resume therefore always lands in a gap, never on an energy
///   peak, and total backward movement never exceeds 30.0 s.
#[must_use]
pub fn resolve_resume(input: &ResumeInput<'_>) -> ResumeResult {
    let yield_secs = input.yield_secs.max(0.0);
    let raw_cursor = (yield_secs - cursor(input.paused_duration_secs)).max(0.0);
    let cap_point = (yield_secs - REWIND_CAP_SECS).max(0.0);

    let granularity_none = input.granularity == AlignmentGranularity::None;
    let snapped = if granularity_none || input.sentence_starts.is_empty() {
        // No-granularity degrade path: raw cursor, never document start.
        raw_cursor
    } else {
        let mut best: Option<f64> = None;
        for boundary in input.sentence_starts.iter() {
            let b = *boundary;
            if b <= raw_cursor && best.is_none_or(|prev| b > prev) {
                best = Some(b);
            }
        }
        if let Some(boundary) = best {
            if boundary >= cap_point {
                boundary
            } else {
                // Sentence longer than the cap: land exactly at cap.
                cap_point
            }
        } else {
            // Cursor precedes every declared boundary: no backward
            // snap exists, so hold the raw cursor rather than jumping
            // to document start or forward into unplayed audio.
            raw_cursor
        }
    };

    let at_cap = (snapped - cap_point).abs() <= GAP_EPS_SECS;
    let mid_word = !is_gap(snapped, input.word_gaps);
    let resume_secs = if at_cap && mid_word && cap_point < raw_cursor {
        let mut next_gap: Option<f64> = None;
        for gap in input.word_gaps.iter() {
            let g = *gap;
            if g > cap_point && next_gap.is_none_or(|prev| g < prev) {
                next_gap = Some(g);
            }
        }
        if let Some(gap) = next_gap {
            if gap <= raw_cursor { gap } else { cap_point }
        } else {
            cap_point
        }
    } else {
        snapped
    };

    ResumeResult {
        resume_secs,
        backward_secs: yield_secs - resume_secs,
        first_yield_ms: input.first_yield_ms,
        supersedes_ts_ms: input.supersedes_ts_ms,
    }
}

/// Whether `point` sits on a declared inter-word gap (silence,
///
/// never an energy peak), within [`GAP_EPS_SECS`]. An empty gap
/// list reports every point as mid-word.
#[must_use]
fn is_gap(point: f64, word_gaps: &[f64]) -> bool {
    let mut found = false;
    for gap in word_gaps.iter() {
        if (*gap - point).abs() <= GAP_EPS_SECS {
            found = true;
        }
    }
    found
}
