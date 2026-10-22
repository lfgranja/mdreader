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
//! 1. Sentence snap + FR-007 context cue: [`player::rewind::resolve_resume`]
//!    lands on the start of the nearest completed sentence at or before the
//!    5%-back cursor, steps back the fixed 5 s context cue
//!    (`contracts/resume-algorithm.md` step 2), never further back than the
//!    30 s cap point.
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
//! 30 s ceiling taking precedence); FR-007 (`specs/004-ressoa-dogfood-build/spec.md`
//! — "at containing-sentence-start snapped backward, then further backward
//! by a fixed 5 s context cue, total backward ≤30 s interruption / ≤60 s
//! kill, never mid-word, never doc start") and `contracts/resume-algorithm.md`
//! steps 1-6 (containing-sentence-start snap, fixed 5 s cue, cap
//! precedence, never-mid-word, never-doc-start, degrade to the raw
//! cursor); FR-015c (while a document sits paused and the reader has taken
//! no action, the system prepares nothing — pre-roll follows the reader's
//! intent to listen, not the existence of a paused session);
//! `REWIND_FRACTION` / `REWIND_CAP_SECS`
//! (`crates/player/src/position.rs` — the single formula home);
//! `cursor` / `resolve_resume` (`crates/player/src/rewind.rs`);
//! `PreRollBuffer` (`crates/player/src/buffer.rs`).
//!
//! T163 (Phase 17 Addendum 6) — the oracle repair, and what it is NOT:
//! the oracle this file carried from `9455167` (2026-10-12) predated the
//! FR-007 5 s context cue the production gained at `4050021` (2026-10-14),
//! so it asserted the PRE-cue landing (the bare containing-sentence start)
//! and went red on ~1 run in 40 — exactly the inputs whose containing
//! start sits inside the `[cap, cap+5]` window, where the cue point falls
//! before the cap, the cap absorbs the cue, and the resume lands exactly
//! at the cap point. The PRODUCTION is contract-correct and is untouched
//! by this repair; only this oracle changed. The oracle below now
//! re-derives the landing from the six numbered steps of
//! `contracts/resume-algorithm.md` — cue and never-mid-word forward snap
//! included — and the `[cap, cap+5]` window is generated on EVERY run by
//! `canonical_resume_lands_at_the_cap_for_every_containing_start_inside_the_cue_window`,
//! so that class can never silently return.
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
use player::rewind::{CUE_SECS, ResumeInput, cursor, resolve_resume};

/// The FR-007 context-cue distance, in seconds
/// (`contracts/resume-algorithm.md` step 2: "From the sentence start,
/// move further backward by exactly 5 seconds of content"; FR-007:
/// "then further backward by a fixed 5 s context cue").
///
/// The oracle reads the CONTRACT number rather than the production
/// constant, so a drift in [`player::rewind::CUE_SECS`] fails this guard
/// instead of silently redefining the oracle. The two are tied together
/// by an explicit assertion below.
const CONTRACT_CUE_SECS: f64 = 5.0;

/// Gap-membership tolerance for the oracle, in seconds.
///
/// Mirrors the private `GAP_EPS_SECS` of `crates/player/src/rewind.rs`:
/// boundary lists are engine-declared `f64` positions, so the oracle
/// reads "lands in a gap" with the same decidable epsilon the
/// implementation uses.
const TEST_GAP_EPS_SECS: f64 = 0.001;

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

/// Whether `point` sits on a declared inter-word gap (silence, never
/// an energy peak) within the shared epsilon.
fn is_test_gap(point: f64, gaps: &[f64]) -> bool {
    gaps.iter()
        .any(|gap| (*gap - point).abs() <= TEST_GAP_EPS_SECS)
}

/// The smallest declared gap strictly after `point` — the "next word
/// boundary" of contract step 4 — or `None` when no declared gap lies
/// after it.
fn next_gap_after(point: f64, gaps: &[f64]) -> Option<f64> {
    gaps.iter()
        .copied()
        .filter(|gap| *gap > point)
        .min_by(f64::total_cmp)
}

/// The expected landing, re-derived from the six numbered steps of
/// `contracts/resume-algorithm.md`:
///
/// 1. the containing-sentence start — the greatest declared boundary at
///    or before the raw cursor;
/// 2. the fixed 5 s context cue, stepped back from that start;
/// 3. the total-backward cap — the snap lands exactly at the cap point
///    when the sentence is longer than the cap, and the cap absorbs a
///    cue that would fall before it (this is the case the pre-T163
///    oracle missed: a containing start inside `[cap, cap+5]` puts the
///    cue point before the cap, so the resume lands AT the cap);
/// 4. never mid-word — a landing that is not a declared gap snaps
///    forward to the next gap inside the rewind window, and holds the
///    cue point when no gap lies inside the window (a snap past the raw
///    cursor would skip content the rewind owed);
/// 5. never before the document start;
/// 6. the degrade path — no declared granularity, an empty boundary
///    list, or no boundary at or before the cursor — holds the raw
///    cursor and cuts no cue.
fn expected_resume_secs(
    yield_secs: f64,
    paused_duration_secs: f64,
    granularity: AlignmentGranularity,
    boundaries: &[f64],
    gaps: &[f64],
) -> f64 {
    let raw = raw_cursor_secs(yield_secs, paused_duration_secs);
    let cap = cap_point_secs(yield_secs);
    // Step 6 — the degrade paths hold the raw cursor, never the
    // document start, and cut no cue.
    if granularity == AlignmentGranularity::None || boundaries.is_empty() {
        return raw;
    }
    // Steps 1 + 6 — no boundary at or before the cursor: hold the raw
    // cursor rather than jumping to the document start.
    let Some(containing) = containing_start(raw, boundaries) else {
        return raw;
    };
    // Step 3 (cap precedence) — a sentence longer than the cap lands
    // the snap exactly at the cap point.
    let snapped = if containing >= cap { containing } else { cap };
    // Steps 2 + 3 + 5 — the fixed cue, never before the cap point and
    // never before the document start.
    let cue_point = (snapped - CONTRACT_CUE_SECS).max(cap).max(0.0);
    // Step 4 — a cue point that is not a declared gap snaps forward to
    // the next gap inside the rewind window.
    if is_test_gap(cue_point, gaps) || cue_point >= raw {
        return cue_point;
    }
    match next_gap_after(cue_point, gaps) {
        Some(gap) if gap <= raw => gap,
        _ => cue_point,
    }
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
        // FR-015 amendment + FR-007 cue: the resume is the
        // containing-sentence start (greatest boundary at or before the
        // 5%-back cursor); where that start lies more than 30 s behind
        // the yield — the sentence is longer than the cap — the resume
        // lands exactly at the cap point, never further back. The
        // FR-007 context cue then steps a fixed 5 s further back from
        // that landing, clamped by the cap (the cap absorbs a cue that
        // would fall before it) and by the document start, and a landing
        // that is not a declared gap snaps forward to the next gap
        // inside the rewind window (never mid-word). The cap point
        // itself is added to the gap list so the at-cap case is never
        // mid-word here (the mid-word forward snap itself is a separate
        // path, not this guard's subject): the oracle reads the pure
        // sentence snap with the hard 30 s ceiling, the cue, and the
        // never-mid-word forward snap.
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
        let expected = expected_resume_secs(
            yield_secs,
            paused_duration_secs,
            granularity,
            &boundaries,
            &gaps,
        );
        prop_assert_eq!(
            result.resume_secs,
            expected,
            "canonical resume {} was not the contract landing {} (containing start {:?}, raw cursor {}, cap {})",
            result.resume_secs,
            expected,
            containing_start(raw, &boundaries),
            raw,
            cap
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

    /// The FR-007 cue absorbed by the cap: for EVERY containing start
    /// inside the `[cap, cap+5]` window, the 5 s cue point falls before
    /// the cap, the cap absorbs the cue (contract step 3 — "If the
    /// computed position exceeds the cap, clamp to the cap"), and the
    /// resume lands exactly at the cap point — never at the containing
    /// start.
    ///
    /// This is the class the pre-T163 oracle failed on (~1 run in 40):
    /// the whole window is generated here on EVERY run, so the class
    /// can never silently return.
    #[expect(
        clippy::float_cmp,
        reason = "the yield, cap, and cue-window positions are exact binary fractions on the quarter-second grid, so == is the correct oracle assertion"
    )]
    #[test]
    fn canonical_resume_lands_at_the_cap_for_every_containing_start_inside_the_cue_window(
        (yield_secs, paused_duration_secs) in (
            (120u32..=14_400).prop_map(|quarters| f64::from(quarters) * 0.25),
            (0u32..=50_000).prop_map(|centis| f64::from(centis) / 100.0),
        )
    ) {
        // The pause is bounded so the window stays inside the rewind:
        // cursor = min(0.05 × paused, 30) ≤ 25 s keeps cap + 5 s at or
        // before the raw cursor for every yield ≥ 30 s, so the declared
        // containing start IS the containing start of the cursor.
        let raw = raw_cursor_secs(yield_secs, paused_duration_secs);
        let cap = cap_point_secs(yield_secs);
        for cue_window_quarters in 1u32..=20 {
            // The containing start sits 0.25 s .. 5.00 s inside the cap
            // window, on the quarter-second grid.
            let offset = f64::from(cue_window_quarters) * 0.25;
            let containing = cap + offset;
            let boundaries = [0.0, containing];
            let mut gaps = union_positions(&boundaries, &[]);
            gaps.push(cap);
            gaps.sort_by(f64::total_cmp);
            gaps.dedup();
            let input = ResumeInput {
                yield_secs,
                paused_duration_secs,
                granularity: AlignmentGranularity::Sentence,
                sentence_starts: &boundaries,
                word_gaps: &gaps,
                first_yield_ms: 0,
                supersedes_ts_ms: None,
            };
            let result = resolve_resume(&input);
            prop_assert_eq!(
                result.resume_secs,
                cap,
                "the canonical resume {} was not the cap point {} for the containing start {} ({} s inside the cue window)",
                result.resume_secs, cap, containing, offset
            );
            prop_assert_eq!(
                result.backward_secs,
                REWIND_CAP_SECS,
                "the canonical backward movement {} from the yield {} was not the 30.0 s cap",
                result.backward_secs, yield_secs
            );
            prop_assert!(
                result.resume_secs >= cap && result.resume_secs <= raw,
                "the canonical resume {} left the [cap {}, raw cursor {}] window for yield {}",
                result.resume_secs, cap, raw, yield_secs
            );
        }
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

/// The oracle's cue distance is the contract's: FR-007 mandates a
/// fixed 5 s context cue, so the production constant and the oracle
/// constant must agree — a drift in either fails this guard instead of
/// silently redefining the other.
#[test]
fn the_contract_cue_distance_is_the_production_cue_distance() {
    assert_eq!(
        CUE_SECS, CONTRACT_CUE_SECS,
        "the production cue distance drifted from the FR-007 contract cue distance"
    );
}

/// The T163 counterexample, pinned in code: the containing start
/// `1010.5` sits 0.25 s inside the cap window (`cap = 1010.25` for the
/// yield `1040.25`), so the 5 s cue point falls before the cap, the cap
/// absorbs the cue, and the resume lands exactly at the cap point.
///
/// The pre-T163 oracle read this input as "the resume must be the
/// containing-sentence start" and made the guard red ~1 run in 40.
// Exact-value contract: the yield 1040.25, the cap point 1010.25, and the 30.0 s backward movement are
// exact binary fractions on the quarter-second grid; == is the correct assertion and a tolerance would
// admit a landing outside the cap window
#[test]
fn the_cue_window_counterexample_lands_at_the_cap_point() {
    let yield_secs = 1040.25;
    let boundaries = [0.0, 1010.5];
    let cap = cap_point_secs(yield_secs);
    let mut gaps = union_positions(&boundaries, &[]);
    gaps.push(cap);
    gaps.sort_by(f64::total_cmp);
    gaps.dedup();
    let input = ResumeInput {
        yield_secs,
        paused_duration_secs: 0.0,
        granularity: AlignmentGranularity::Word,
        sentence_starts: &boundaries,
        word_gaps: &gaps,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    assert_eq!(
        result.resume_secs, cap,
        "the cue-window counterexample must land at the cap point {cap} — the 5 s cue falls before the cap and the cap absorbs it"
    );
    assert_eq!(
        result.backward_secs, REWIND_CAP_SECS,
        "the cue-window counterexample must rewind exactly the 30.0 s cap"
    );
}

/// The second recorded counterexample, pinned in code: the cue point
/// (`513.5`) is itself a declared gap, so the never-mid-word forward
/// snap does not move it and the resume lands exactly on the cue point —
/// a fixed 5 s before the containing start `518.5`.
// Exact-value contract: the yield 527.25, the cue point 513.5, and the 13.75 s backward movement are
// exact binary fractions on the quarter-second grid; == is the correct assertion and a tolerance would
// admit an off-cue landing
#[test]
fn the_cue_point_lands_when_the_cue_point_is_itself_a_gap() {
    let yield_secs = 527.25;
    let containing_start = 518.5;
    let cue_point = containing_start - CONTRACT_CUE_SECS;
    let boundaries = [0.0, 513.5, containing_start];
    let mut gaps = union_positions(&boundaries, &[]);
    gaps.push(cap_point_secs(yield_secs));
    gaps.sort_by(f64::total_cmp);
    gaps.dedup();
    let input = ResumeInput {
        yield_secs,
        paused_duration_secs: 0.0,
        granularity: AlignmentGranularity::Word,
        sentence_starts: &boundaries,
        word_gaps: &gaps,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    assert_eq!(
        result.resume_secs, cue_point,
        "the resume must land on the cue point {cue_point} — a fixed 5 s before the containing start {containing_start}"
    );
    assert_eq!(
        result.backward_secs,
        yield_secs - cue_point,
        "the total backward movement must equal the yield {yield_secs} minus the cue point {cue_point}"
    );
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
