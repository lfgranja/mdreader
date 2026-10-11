//! T032 — US2 property: rewind/snap/cap math (FR-015).
//!
//! Invariants under test, quoted verbatim from
//! `specs/001-android-media-shell/tasks.md` T032:
//!
//! - cursor = `min(0.05 × paused_duration_secs, 30.0)`;
//! - audio resumes at the containing-sentence start;
//! - TOTAL backward movement ≤ 30.0 s even when the sentence is longer
//!   than the cap (land at exactly the cap point);
//! - a cap-point landing mid-word snaps forward to the next inter-word
//!   gap unless that gap lies beyond the cap (land exactly at the cap);
//! - no-sentence-granularity degrades to the raw cursor, never to
//!   document start;
//! - the resume lands in a gap, never on an energy peak, on synthetic
//!   audio with known gaps;
//! - a superseded transient keeps the cumulative clock from the first
//!   yield.
//!
//! Spec anchors: FR-015 (`specs/001-android-media-shell/spec.md` — the
//! 5%-capped-at-30 s cursor, the containing-boundary snap with the hard
//! 30 s ceiling taking precedence, the forward mid-word snap, the raw
//! cursor degrade path, and the cumulative supersede clock);
//! `REWIND_FRACTION` / `REWIND_CAP_SECS`
//! (`crates/player/src/position.rs` — the single formula home);
//! `cursor` / `resolve_resume` / `supersede`
//! (`crates/player/src/rewind.rs` — the implementation under test).
//!
//! Exactness note: yields, boundaries, and gaps are generated on the
//! quarter-second grid (`k × 0.25`), an exact binary-fraction grid, and
//! the oracles assert with the same expressions the contract states, so
//! boundary and cap comparisons are bit-exact. A tolerance would
//! wrongly admit an off-boundary snap or a cap violation (the 30-second
//! cap is a hard ceiling, FR-015).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest 1.11 builds the message through
//! `concat!`, and rustc (1.98) rejects implicit captures in
//! macro-expanded format strings ("`format_args!` cannot capture
//! variables when the format string is expanded from a macro").

use proptest::prelude::*;

use engine_core::trait_::AlignmentGranularity;
use player::position::{REWIND_CAP_SECS, REWIND_FRACTION};
use player::rewind::{ResumeInput, cursor, resolve_resume, supersede};

/// Gap-membership tolerance for the test oracles, in seconds.
///
/// Mirrors the private `GAP_EPS_SECS` of `crates/player/src/rewind.rs`:
/// boundary lists are engine-declared `f64` positions, so the oracle
/// reads "lands in a gap" with the same decidable epsilon the
/// implementation uses.
const TEST_GAP_EPS_SECS: f64 = 0.001;

/// Yield positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
fn yield_position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=720_000).prop_map(|centis| centis as f64 / 100.0)
}

/// Ascending sentence/word-start boundaries on the quarter-second grid,
/// always including the document start `0.0` (every document has a
/// first boundary; the empty list is covered by the degrade tests).
fn boundaries_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.push(0);
        quarters.sort();
        quarters.dedup();
        quarters.into_iter().map(|q| q as f64 * 0.25).collect()
    })
}

/// Extra inter-word gaps on the quarter-second grid (the oracle unions
/// these with the boundaries, so every boundary is also a gap — a
/// boundary is silence, never an energy peak).
fn extra_gaps_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=8)
        .prop_map(|quarters| quarters.into_iter().map(|q| q as f64 * 0.25).collect())
}

/// The declared alignment granularities (the closed three-member set).
fn granularity_strategy() -> impl Strategy<Value = AlignmentGranularity> {
    prop_oneof![
        Just(AlignmentGranularity::None),
        Just(AlignmentGranularity::Word),
        Just(AlignmentGranularity::Sentence),
    ]
}

/// The `Word | Sentence` granularities: both snap to the
/// caller-supplied boundary list (`None` degrades instead).
fn snapping_granularity_strategy() -> impl Strategy<Value = AlignmentGranularity> {
    prop_oneof![
        Just(AlignmentGranularity::Word),
        Just(AlignmentGranularity::Sentence),
    ]
}

/// The raw cursor oracle (FR-015): the yield moved back by exactly
/// `min(REWIND_FRACTION × paused, REWIND_CAP_SECS)`, clamped at 0.0.
fn raw_cursor(yield_secs: f64, paused_duration_secs: f64) -> f64 {
    let rewind_amount = (REWIND_FRACTION * paused_duration_secs).min(REWIND_CAP_SECS);
    (yield_secs - rewind_amount).max(0.0)
}

/// The cap point oracle (FR-015): the earliest position the resume may
/// land on — the yield less the hard 30-second cap, clamped at 0.0.
fn cap_point(yield_secs: f64) -> f64 {
    (yield_secs - REWIND_CAP_SECS).max(0.0)
}

/// The greatest boundary at or before the raw cursor, or `None` when
/// the cursor precedes every declared boundary.
fn containing_start(raw_cursor_secs: f64, boundaries: &[f64]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for boundary in boundaries {
        if *boundary <= raw_cursor_secs && best.is_none_or(|prev| *boundary > prev) {
            best = Some(*boundary);
        }
    }
    best
}

/// Whether `point` sits on a declared inter-word gap within the shared
/// epsilon (silence, never an energy peak).
fn is_test_gap(point: f64, gaps: &[f64]) -> bool {
    gaps.iter()
        .any(|gap| (*gap - point).abs() <= TEST_GAP_EPS_SECS)
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

    #[test]
    fn given_a_paused_duration_when_computing_the_cursor_then_it_is_five_percent_capped_at_thirty_seconds(
        paused_duration_secs in paused_duration_strategy()
    ) {
        // FR-015: cursor = `min(0.05 × paused_duration_secs, 30.0)` —
        // the single formula, read through the shared constants.
        let expected = (REWIND_FRACTION * paused_duration_secs).min(REWIND_CAP_SECS);
        prop_assert_eq!(
            cursor(paused_duration_secs),
            expected,
            "the cursor for a {}s pause was not min(0.05 × paused, 30.0)",
            paused_duration_secs
        );
        prop_assert!(
            cursor(paused_duration_secs) >= 0.0
                && cursor(paused_duration_secs) <= REWIND_CAP_SECS,
            "the cursor {} for a {}s pause left the [0.0, 30.0] window",
            cursor(paused_duration_secs), paused_duration_secs
        );
    }

    #[test]
    fn given_any_yield_pause_and_boundaries_when_resolving_then_total_backward_never_exceeds_thirty_seconds(
        (yield_secs, paused_duration_secs, boundaries, extra_gaps, granularity) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
            extra_gaps_strategy(),
            granularity_strategy(),
        )
    ) {
        // FR-015 hard ceiling: total backward movement from the yield
        // never exceeds 30.0 s under any pause length, any granularity,
        // and the resume always lands inside the [cap, raw-cursor]
        // window — never ahead of the formula point, never behind the
        // cap.
        let gaps = union_positions(&boundaries, &extra_gaps);
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
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let cap = cap_point(yield_secs);
        prop_assert!(
            result.backward_secs <= REWIND_CAP_SECS,
            "total backward movement {} from yield {} exceeded the 30.0 s cap",
            result.backward_secs, yield_secs
        );
        prop_assert!(
            result.resume_secs >= cap && result.resume_secs <= raw,
            "resume {} left the [cap {}, raw cursor {}] window for yield {}",
            result.resume_secs, cap, raw, yield_secs
        );
        prop_assert_eq!(
            result.backward_secs,
            yield_secs.max(0.0) - result.resume_secs,
            "backward movement did not equal yield {} minus resume {}",
            yield_secs, result.resume_secs
        );
    }

    #[test]
    fn given_a_containing_sentence_when_resolving_then_audio_resumes_at_its_start_or_exactly_at_the_cap(
        (yield_secs, paused_duration_secs, boundaries, granularity) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
            snapping_granularity_strategy(),
        )
    ) {
        // FR-015: the resume is the containing-sentence start (greatest
        // boundary at or before the raw cursor); where that start lies
        // more than 30 s behind the yield — the sentence is longer than
        // the cap — the resume lands exactly at the cap point. The cap
        // point itself is added to the gap list so the at-cap case is
        // never mid-word here (the mid-word forward snap is asserted
        // separately below).
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let cap = cap_point(yield_secs);
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
            "resume {} was not the containing-sentence start (raw cursor {}, cap {})",
            result.resume_secs, raw, cap
        );
    }

    #[test]
    fn given_a_cap_point_mid_word_when_snapping_then_the_resume_moves_forward_to_the_next_gap_unless_that_gap_leaves_the_window(
        (yield_secs, paused_duration_secs, gap_quarters) in (
            (240u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25),
            (2_000u64..=40_000).prop_map(|centis| centis as f64 / 100.0),
            1u64..=40,
        )
    ) {
        // FR-015: the sentence list holds only the document start, so
        // the snap falls to the cap point — which is mid-word by
        // construction (the gap list holds 0.0 and one constructed gap
        // only). Case A: the next gap sits inside the window, so the
        // resume moves forward to it. Case B: the only next gap sits
        // past the raw cursor, so the resume lands exactly at the cap.
        // Paused ∈ [20, 400] s keeps a ≥ 10 s window; yield ≥ 60 s
        // keeps the cap strictly above the document start.
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let cap = cap_point(yield_secs);
        let in_window_gap = cap + gap_quarters as f64 * 0.25;
        let past_window_gap = raw + 1.0;
        let starts = [0.0];
        let input_in_window = ResumeInput {
            yield_secs,
            paused_duration_secs,
            granularity: AlignmentGranularity::Sentence,
            sentence_starts: &starts,
            word_gaps: &[0.0, in_window_gap],
            first_yield_ms: 0,
            supersedes_ts_ms: None,
        };
        let result_in_window = resolve_resume(&input_in_window);
        prop_assert_eq!(
            result_in_window.resume_secs,
            in_window_gap,
            "a mid-word cap {} did not snap forward to the in-window gap {} (raw cursor {})",
            cap, in_window_gap, raw
        );
        let input_past_window = ResumeInput {
            yield_secs,
            paused_duration_secs,
            granularity: AlignmentGranularity::Sentence,
            sentence_starts: &starts,
            word_gaps: &[0.0, past_window_gap],
            first_yield_ms: 0,
            supersedes_ts_ms: None,
        };
        let result_past_window = resolve_resume(&input_past_window);
        prop_assert_eq!(
            result_past_window.resume_secs,
            cap,
            "a mid-word cap {} with the next gap {} past the raw cursor {} did not land exactly at the cap",
            cap, past_window_gap, raw
        );
        prop_assert_eq!(
            result_past_window.backward_secs,
            REWIND_CAP_SECS,
            "total backward movement from yield {} was not exactly the 30.0 s cap",
            yield_secs
        );
    }

    #[test]
    fn given_no_sentence_granularity_when_resolving_then_the_resume_is_the_raw_cursor_never_document_start(
        (yield_secs, paused_duration_secs, boundaries) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
        )
    ) {
        // FR-015 degrade path: `None` ignores the boundary list, and an
        // empty boundary list under `Word`/`Sentence` has no snap to
        // apply — both hold the raw cursor, which is document start
        // only when the cursor itself is there.
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let gaps = union_positions(&boundaries, &[]);
        let input_none = ResumeInput {
            yield_secs,
            paused_duration_secs,
            granularity: AlignmentGranularity::None,
            sentence_starts: &boundaries,
            word_gaps: &gaps,
            first_yield_ms: 0,
            supersedes_ts_ms: None,
        };
        let result_none = resolve_resume(&input_none);
        prop_assert_eq!(
            result_none.resume_secs,
            raw,
            "no-granularity resume {} was not the raw cursor {} for yield {}",
            result_none.resume_secs, raw, yield_secs
        );
        prop_assert!(
            result_none.resume_secs != 0.0 || raw == 0.0,
            "no-granularity resume fell back to document start while the raw cursor was {}",
            raw
        );
        for granularity in [AlignmentGranularity::Word, AlignmentGranularity::Sentence] {
            let empty: &[f64] = &[];
            let input_empty = ResumeInput {
                yield_secs,
                paused_duration_secs,
                granularity,
                sentence_starts: empty,
                word_gaps: empty,
                first_yield_ms: 0,
                supersedes_ts_ms: None,
            };
            let result_empty = resolve_resume(&input_empty);
            prop_assert_eq!(
                result_empty.resume_secs,
                raw,
                "empty-boundary resume {} was not the raw cursor {} for yield {}",
                result_empty.resume_secs, raw, yield_secs
            );
            prop_assert!(
                result_empty.resume_secs != 0.0 || raw == 0.0,
                "empty-boundary resume fell back to document start while the raw cursor was {}",
                raw
            );
        }
    }

    #[test]
    fn given_synthetic_audio_with_known_gaps_when_resolving_then_the_resume_lands_in_a_gap_never_on_an_energy_peak(
        (yield_secs, paused_duration_secs, boundaries, extra_gaps, granularity) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
            extra_gaps_strategy(),
            snapping_granularity_strategy(),
        )
    ) {
        // FR-015 verification rule: on synthetic audio with known
        // inter-word gaps the resume lands in a gap, never on an energy
        // peak. The only exception is the cap-precedence escape: where
        // the cap point is mid-word and the next gap lies past the raw
        // cursor, the resume lands exactly at the cap (the 30-second
        // ceiling no snap may exceed).
        let gaps = union_positions(&boundaries, &extra_gaps);
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
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let cap = cap_point(yield_secs);
        let in_gap = is_test_gap(result.resume_secs, &gaps);
        let beyond_cap_escape = result.resume_secs == cap
            && gaps
                .iter()
                .filter(|gap| **gap > cap)
                .all(|gap| *gap > raw);
        prop_assert!(
            in_gap || beyond_cap_escape,
            "resume {} landed on an energy peak — neither in a known gap nor exactly at the cap {} (raw cursor {})",
            result.resume_secs, cap, raw
        );
    }

    #[test]
    fn given_a_superseded_transient_when_linking_then_the_clock_stays_cumulative_from_the_first_yield(
        (first_yield_ms, chain) in (any::<u64>(), proptest::collection::vec(any::<u64>(), 1..=8))
    ) {
        // FR-015 supersede clock: `first_yield_ms` is set once at the
        // first yield and never reset at supersede; every superseding
        // record links back through `supersedes_ts_ms` (two ordered
        // entries, never a merged rewrite).
        let mut carried_first_yield_ms = first_yield_ms;
        let mut last_now_ms = first_yield_ms;
        for now_ms in &chain {
            let link = supersede(carried_first_yield_ms, *now_ms);
            prop_assert_eq!(
                link.first_yield_ms,
                first_yield_ms,
                "the supersede link reset the cumulative clock from {} (supersede at {})",
                first_yield_ms, now_ms
            );
            prop_assert_eq!(
                link.supersedes_ts_ms,
                Some(*now_ms),
                "the supersede link did not point at the superseded entry {}",
                now_ms
            );
            carried_first_yield_ms = link.first_yield_ms;
            last_now_ms = *now_ms;
        }
        let boundaries = [0.0];
        let input = ResumeInput {
            yield_secs: 120.0,
            paused_duration_secs: 60.0,
            granularity: AlignmentGranularity::Sentence,
            sentence_starts: &boundaries,
            word_gaps: &boundaries,
            first_yield_ms,
            supersedes_ts_ms: Some(last_now_ms),
        };
        let result = resolve_resume(&input);
        prop_assert_eq!(
            result.first_yield_ms,
            first_yield_ms,
            "the resume dropped the cumulative first-yield clock {}",
            first_yield_ms
        );
        prop_assert_eq!(
            result.supersedes_ts_ms,
            Some(last_now_ms),
            "the resume dropped the supersede link to {}",
            last_now_ms
        );
    }
}

/// The shared constants match the FR-015 figure verbatim: a 5% fraction
/// and a 30-second hard cap.
#[expect(
    clippy::float_cmp,
    reason = "REWIND_FRACTION and REWIND_CAP_SECS are exact decimal literals from position.rs; == is the correct assertion and a tolerance would admit a drifted formula"
)]
#[test]
fn rewind_constants_match_fr015_figure_verbatim() {
    assert_eq!(
        REWIND_FRACTION, 0.05,
        "the rewind fraction drifted from 0.05"
    );
    assert_eq!(REWIND_CAP_SECS, 30.0, "the rewind cap drifted from 30.0 s");
}

/// A non-positive pause rewinds nothing: the cursor for `0.0` and for a
/// negative duration is exactly `0.0`.
#[expect(
    clippy::float_cmp,
    reason = "cursor(0.0) and cursor(-100.0) are exactly 0.0 by the max(0.0) clamp; == is the correct assertion"
)]
#[test]
fn non_positive_pause_rewinds_nothing() {
    assert_eq!(cursor(0.0), 0.0, "a zero pause must rewind nothing");
    assert_eq!(
        cursor(-100.0),
        0.0,
        "a negative pause must rewind nothing, not move forward"
    );
}

/// Boundary case: the cursor precedes every declared boundary, so no
/// backward snap exists — the resume holds the raw cursor rather than
/// jumping to document start or forward into unplayed audio.
#[expect(
    clippy::float_cmp,
    reason = "the yield 10.0, pause 100.0, and boundary 50.0 are exact binary fractions and 0.05 × 100.0 is exactly 5.0, so == is the correct assertion"
)]
#[test]
fn cursor_preceding_all_boundaries_holds_the_raw_cursor() {
    let boundaries = [50.0];
    let input = ResumeInput {
        yield_secs: 10.0,
        paused_duration_secs: 100.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &boundaries,
        word_gaps: &boundaries,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    assert_eq!(
        result.resume_secs, 5.0,
        "with no boundary at or before the raw cursor 5.0, the resume must hold the cursor"
    );
    assert_eq!(
        result.backward_secs, 5.0,
        "backward movement must equal yield 10.0 minus resume 5.0"
    );
}
