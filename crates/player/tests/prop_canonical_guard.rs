//! T114 — Phase 12 convergence BLOCKING guard: canonical rewind + idle-drain paths.
//!
//! This is a GUARD test: it must be GREEN before T115 and stay GREEN after —
//! T115 MUST NOT land while it is red. T115 deletes dead code from
//! `crates/player/src/transport.rs` (the pre-amendment `apply_rewind` stub
//! with no sentence snap, and the legacy pre-roll shape without an
//! idle/preparing gate). The CANONICAL implementations that must be
//! preserved are [`player::rewind::resolve_resume`] (sentence snap per the
//! FR-015 amendment) and [`player::buffer::PreRollBuffer`] (idle drain per
//! FR-015c). This file asserts strictly those canonical paths.
//!
//! Invariants under test:
//!
//! 1. Sentence snap: [`player::rewind::resolve_resume`] lands on the start
//!    of the nearest completed sentence at or before the 5%-back cursor,
//!    never further back than the 30 s cap point.
//! 2. Idle drain: a new [`player::buffer::PreRollBuffer`] is empty;
//!    [`player::buffer::PreRollBuffer::drain_for_idle`] empties a filled
//!    buffer; nothing grows the buffer while idle (no preparing state
//!    implies no growth — an idle delivery is rejected).
//!
//! What this file does NOT touch: the transport dead code T115 removes.
//! It never references the transport rewind stub or the transport pre-roll
//! shape — only the canonical `player::rewind` / `player::buffer` paths.
//!
//! Spec anchors: FR-015 (`specs/001-android-media-shell/spec.md` — the
//! 5%-capped-at-30 s cursor, the containing-boundary snap with the hard
//! 30 s ceiling taking precedence); FR-015c (while a document sits paused
//! and the reader has taken no action, the system prepares nothing —
//! pre-roll follows the reader's intent to listen, not the existence of a
//! paused session); `REWIND_FRACTION` / `REWIND_CAP_SECS`
//! (`crates/player/src/position.rs` — the single formula home);
//! `cursor` / `resolve_resume` (`crates/player/src/rewind.rs`);
//! `PreRollBuffer` (`crates/player/src/buffer.rs`).
//!
//! Exactness note: yields, boundaries, and deliveries are generated on
//! exact binary-fraction grids (quarter-second / centisecond), and the
//! oracles assert with the same expressions the contract states, so
//! boundary and cap comparisons are bit-exact. A tolerance would wrongly
//! admit an off-boundary snap or a cap violation (the 30-second cap is a
//! hard ceiling, FR-015).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures (`{var}`).
//! proptest 1.11 builds the message through `concat!`, and rustc rejects
//! implicit captures in macro-expanded format strings.

use proptest::prelude::*;

use engine_core::trait_::AlignmentGranularity;
use player::buffer::{BufferError, PreRollBuffer};
use player::position::REWIND_CAP_SECS;
use player::rewind::{ResumeInput, cursor, resolve_resume};

/// Yield positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
fn yield_position_strategy() -> impl Strategy<Value = f64> {
    (0u32..=14_400).prop_map(|quarters| f64::from(quarters) * 0.25)
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u32..=720_000).prop_map(|centis| f64::from(centis) / 100.0)
}

/// Ascending sentence-start boundaries on the quarter-second grid, always
/// including the document start `0.0` (every document has a first
/// boundary).
fn boundaries_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u32..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.push(0);
        quarters.sort_unstable();
        quarters.dedup();
        quarters.into_iter().map(|q| f64::from(q) * 0.25).collect()
    })
}

/// The `Word | Sentence` granularities: both snap to the caller-supplied
/// boundary list on the canonical path (`None` degrades instead, which is
/// not the sentence-snap path this guard pins).
fn snapping_granularity_strategy() -> impl Strategy<Value = AlignmentGranularity> {
    prop_oneof![
        Just(AlignmentGranularity::Word),
        Just(AlignmentGranularity::Sentence),
    ]
}

/// Pre-roll deliveries on the centisecond grid: `k × 0.01` s for
/// `k ∈ [1, 6000]` — the open-lower interval (0, 60] seconds, bounded by
/// the 60 s pre-roll cap.
fn pre_roll_secs_strategy() -> impl Strategy<Value = f64> {
    (1u32..=6_000).prop_map(|centis| f64::from(centis) / 100.0)
}

/// The raw cursor oracle (FR-015): the yield moved back by exactly the
/// canonical 5%-capped-at-30 s cursor, clamped at 0.0.
fn raw_cursor_secs(yield_secs: f64, paused_duration_secs: f64) -> f64 {
    (yield_secs - cursor(paused_duration_secs)).max(0.0)
}

/// The cap point oracle (FR-015): the earliest position the resume may
/// land on — the yield less the hard 30-second cap, clamped at 0.0.
fn cap_point_secs(yield_secs: f64) -> f64 {
    (yield_secs - REWIND_CAP_SECS).max(0.0)
}

/// The greatest boundary at or before the raw cursor: the start of the
/// nearest completed sentence — or `None` when the cursor precedes every
/// declared boundary.
fn containing_start(raw_cursor_secs: f64, boundaries: &[f64]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for boundary in boundaries {
        if *boundary <= raw_cursor_secs && best.is_none_or(|prev| *boundary > prev) {
            best = Some(*boundary);
        }
    }
    best
}

/// Union two position lists into one ascending deduplicated list.
fn union_positions(first: &[f64], second: &[f64]) -> Vec<f64> {
    let mut merged: Vec<f64> = first.to_vec();
    merged.extend_from_slice(second);
    merged.sort_by(f64::total_cmp);
    merged.dedup();
    merged
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[expect(
        clippy::float_cmp,
        reason = "the yield, cursor, cap, and boundary positions are exact binary fractions on the quarter- and centisecond grids, so == is the correct oracle assertion"
    )]
    #[test]
    fn canonical_resume_resume_snaps_to_containing_sentence_within_the_cap(
        (yield_secs, paused_duration_secs, boundaries, granularity) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
            snapping_granularity_strategy(),
        )
    ) {
        // FR-015 amendment: the resume is the containing-sentence start
        // (greatest boundary at or before the 5%-back cursor); where that
        // start lies more than 30 s behind the yield — the sentence is
        // longer than the cap — the resume lands exactly at the cap
        // point, never further back. The cap point itself is added to the
        // gap list so the at-cap case is never mid-word here (the
        // mid-word forward snap is a separate path, not this guard's
        // subject): the oracle reads the pure sentence snap with the
        // hard 30 s ceiling.
        let raw = raw_cursor_secs(yield_secs, paused_duration_secs);
        let cap = cap_point_secs(yield_secs);
        let mut gaps = union_positions(&boundaries, &[]);
        gaps.push(cap);
        gaps.sort_by(f64::total_cmp);
        gaps.dedup();
        let input = ResumeInput {
            yield_secs,
            paused_duration_secs,
            granularity,
            sentence_starts: &boundaries,
            word_gaps: &gaps,
            first_yield_ms: 0,
            supersedes_ts_ms: None,
        };
        let result = resolve_resume(&input);
        let expected = match containing_start(raw, &boundaries) {
            Some(boundary) if boundary >= cap => boundary,
            Some(_) => cap,
            None => raw,
        };
        prop_assert_eq!(
            result.resume_secs,
            expected,
            "canonical resume {} was not the containing-sentence start (raw cursor {}, cap {})",
            result.resume_secs, raw, cap
        );
        prop_assert!(
            result.resume_secs >= cap && result.resume_secs <= raw,
            "canonical resume {} left the [cap {}, raw cursor {}] window for yield {}",
            result.resume_secs, cap, raw, yield_secs
        );
        prop_assert!(
            result.backward_secs <= REWIND_CAP_SECS,
            "canonical backward movement {} from yield {} exceeded the 30.0 s cap",
            result.backward_secs, yield_secs
        );
        prop_assert_eq!(
            result.backward_secs,
            yield_secs - result.resume_secs,
            "canonical backward movement did not equal yield {} minus resume {}",
            yield_secs, result.resume_secs
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the delivery is an exact centisecond-grid fraction below the 60 s cap, so held == delivered is the correct assertion"
    )]
    #[test]
    fn canonical_buffer_stays_empty_while_idle_and_drains_for_idle(
        delivered_secs in pre_roll_secs_strategy()
    ) {
        // FR-015c: a new canonical buffer is empty and idle — nothing
        // held, nothing preparing, no synthesis spent.
        let mut buffer = PreRollBuffer::new();
        prop_assert!(
            buffer.is_empty(),
            "a new canonical buffer was not empty before any Play press (FR-015c)"
        );
        prop_assert!(
            !buffer.is_preparing(),
            "a new canonical buffer was preparing before any Play press (FR-015c)"
        );
        // Nothing grows the buffer while idle: a delivery with no Play
        // press is rejected, and the buffer stays empty.
        let rejected = buffer.push_delivered(delivered_secs);
        prop_assert!(
            matches!(rejected, Err(BufferError::NotPreparing)),
            "an idle canonical buffer accepted a {} s delivery without a Play press (FR-015c)",
            delivered_secs
        );
        prop_assert!(
            buffer.is_empty(),
            "an idle canonical buffer grew after a rejected {} s delivery (FR-015c)",
            delivered_secs
        );
        // A Play press opens preparation and the delivery lands.
        buffer.on_play_pressed();
        let held = buffer.push_delivered(delivered_secs);
        prop_assert!(
            held.is_ok(),
            "a preparing canonical buffer rejected a {} s delivery",
            delivered_secs
        );
        let held_secs = buffer.buffered_secs();
        prop_assert_eq!(
            held_secs,
            delivered_secs,
            "a preparing canonical buffer held {} s after a {} s delivery",
            held_secs, delivered_secs
        );
        // drain_for_idle empties the filled buffer and closes
        // preparation; afterwards nothing grows it again without a new
        // Play press.
        buffer.drain_for_idle();
        prop_assert!(
            buffer.is_empty(),
            "drain_for_idle left {} s held (FR-015c)",
            buffer.buffered_secs()
        );
        prop_assert!(
            !buffer.is_preparing(),
            "drain_for_idle left the canonical buffer preparing (FR-015c)"
        );
        let rejected_after = buffer.push_delivered(delivered_secs);
        prop_assert!(
            matches!(rejected_after, Err(BufferError::NotPreparing)),
            "a post-drain canonical buffer accepted a {} s delivery without a Play press (FR-015c)",
            delivered_secs
        );
        prop_assert!(
            buffer.is_empty(),
            "a post-drain canonical buffer grew after a rejected {} s delivery (FR-015c)",
            delivered_secs
        );
    }
}

/// A new canonical buffer is empty and idle: nothing held, nothing
/// preparing, no synthesis spent (FR-015c).
#[test]
fn new_canonical_buffer_is_empty_and_idle() {
    let buffer = PreRollBuffer::new();
    assert!(
        buffer.is_empty(),
        "a new canonical buffer must hold no audio before any Play press"
    );
    assert!(
        !buffer.is_preparing(),
        "a new canonical buffer must prepare nothing before any Play press"
    );
}

/// `drain_for_idle` empties a filled canonical buffer and closes
/// preparation (FR-015c).
// Exact-value contract: 12.5 is an exact binary fraction and the single delivery is held verbatim
// below the 60 s cap, so == is the correct assertion
#[test]
fn drain_for_idle_empties_a_filled_canonical_buffer() {
    let mut buffer = PreRollBuffer::new();
    buffer.on_play_pressed();
    let held = buffer.push_delivered(12.5);
    assert!(
        held.is_ok(),
        "a preparing canonical buffer must accept the delivery"
    );
    assert_eq!(
        buffer.buffered_secs(),
        12.5,
        "the canonical buffer must hold exactly what the engine delivered"
    );
    buffer.drain_for_idle();
    assert!(
        buffer.is_empty(),
        "drain_for_idle must empty the filled canonical buffer"
    );
    assert!(
        !buffer.is_preparing(),
        "drain_for_idle must leave the canonical buffer non-preparing"
    );
}

/// An idle delivery is rejected and grows nothing: no preparing state
/// implies no growth (FR-015c).
#[test]
fn idle_delivery_is_rejected_and_grows_nothing() {
    let mut buffer = PreRollBuffer::new();
    let rejected = buffer.push_delivered(1.0);
    assert!(
        matches!(rejected, Err(BufferError::NotPreparing)),
        "an idle canonical buffer must reject a delivery without a Play press"
    );
    assert!(
        buffer.is_empty(),
        "a rejected idle delivery must grow nothing"
    );
}
