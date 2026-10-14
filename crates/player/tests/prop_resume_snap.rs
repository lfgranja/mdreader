//! T025 — US2 property: the resume snap and the replay caps
//! (FR-007 / FR-015).
//!
//! `specs/004-ressoa-dogfood-build/tasks.md` T025, verbatim:
//! "Property `prop_resume_snap_cap` in
//! `crates/player/tests/prop_resume.rs` (invariants: resume at
//! containing-sentence-start + 5–10 s cue, never mid-word; total
//! backward ≤30 s interruption / ≤60 s kill; persisted cursor is
//! last-spoken)".
//!
//! This file is the T025 BLOCKING test for the feature-004 dogfood
//! wave. It binds to the rewind engine and the position clock:
//!
//! - `player::rewind` — `ResumeInput`, `ResumeResult`,
//!   `resolve_resume`, `cursor` (the resume/snap/cap math);
//! - `player::position` — `PositionClock`, `REWIND_CAP_SECS`,
//!   `REWIND_FRACTION` (the single formula home and the
//!   last-spoken cursor).
//!
//! Spec anchor — FR-007 (`specs/004-ressoa-dogfood-build/spec.md`):
//! "The system MUST resume after an interruption at or just before
//! the point of interruption — at containing-sentence-start snapped
//! backward, then further backward by a fixed 5 s context cue, total
//! backward ≤30 s interruption / ≤60 s kill, never mid-word, never
//! doc start; the persisted cursor MUST be the last-spoken position
//! (never buffered-ahead) in a single atomic transaction..." The
//! algorithm steps are restated in `contracts/resume-algorithm.md`
//! (containing-sentence-start snap, fixed 5 s cue, cap precedence,
//! never-mid-word, never-doc-start, degrade to the raw cursor).
//!
//! RED HALF (recorded, not implemented by this file): the 5–10 s
//! context cue of FR-007 does not exist in
//! `crates/player/src/rewind.rs` — `resolve_resume` stops at the
//! containing-sentence start. The cue test below therefore FAILS RED
//! and is the gate for T032 (sentence-snap resume path in
//! `rewind.rs` + `resume.rs`). Everything else in this file asserts
//! the already-implemented snap/cap contract and is green.
//!
//! Kill-cap note: `ResumeInput` carries no interruption-type input,
//! so the rewind engine cannot distinguish an interruption replay
//! (≤30 s) from a kill/power-loss replay (≤60 s) — the 30 s
//! `REWIND_CAP_SECS` cap governs both host paths, and the 60 s kill
//! half is the persistence property of FR-007 (20 s fixed durable
//! heartbeat within the 15–30 s envelope, worst-case loss one
//! interval) plus the on-device attestation of T051. This file
//! asserts the ≤30 s half as the binding host invariant and records
//! the ≤60 s half as the looser bound every resume also satisfies.
//!
//! Exactness note: yields, boundaries, and gaps are generated on the
//! quarter-second grid (`k × 0.25`) and pauses in exact centiseconds
//! — all exact binary fractions — and the oracles assert with the
//! same expressions the contract states, so every comparison is
//! bit-exact. A tolerance would wrongly admit an off-boundary snap
//! or a cap violation (the 30-second cap is a hard ceiling, FR-007).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest builds the message through `concat!`, and
//! rustc rejects implicit captures in macro-expanded format strings.
//! Plain `assert!` / `assert_eq!` messages use implicit captures, as
//! elsewhere in the suite.

use proptest::prelude::*;

use engine_core::trait_::AlignmentGranularity;
use player::position::{PlayerError, PositionClock, REWIND_CAP_SECS, REWIND_FRACTION};
use player::rewind::{CUE_SECS, ResumeInput, ResumeResult, cursor, resolve_resume};

/// Gap-membership tolerance for the test oracles, in seconds.
///
/// Mirrors the private `GAP_EPS_SECS` of `crates/player/src/rewind.rs`:
/// boundary lists are engine-declared `f64` positions, so the oracle
/// reads "lands in a gap" with the same decidable epsilon the
/// implementation uses.
const TEST_GAP_EPS_SECS: f64 = 0.001;

/// The FR-007 context-cue window, in seconds: the resume lands this
/// far BEFORE the containing-sentence start, placing the listener
/// ahead of the interrupted sentence (T025 states the 5–10 s figure).
const CUE_MIN_SECS: f64 = 5.0;

/// The upper end of the FR-007 context-cue window, in seconds.
const CUE_MAX_SECS: f64 = 10.0;

/// The FR-007 interruption-replay cap, in seconds: the rewind
/// engine's hard cap is the interruption cap (FR-015 / FR-007).
const INTERRUPTION_REPLAY_CAP_SECS: f64 = REWIND_CAP_SECS;

/// The FR-007 kill/power-loss replay cap, in seconds: the looser
/// persistence bound (20 s fixed durable heartbeat within the 15–30 s
/// envelope, worst-case loss one interval). Every resume the rewind
/// engine computes also satisfies it.
const KILL_REPLAY_CAP_SECS: f64 = 60.0;

/// Yield positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
fn yield_position_strategy() -> impl Strategy<Value = f64> {
    (0u32..=14_400).prop_map(|quarters| f64::from(quarters) * 0.25)
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u32..=720_000).prop_map(|centis| f64::from(centis) / 100.0)
}

/// Ascending sentence/word-start boundaries on the quarter-second
/// grid — sometimes empty (the no-granularity degrade path) and
/// sometimes without the document start (the cursor-precedes-every-
/// boundary path), so both degrade branches are generated rather than
/// assumed away.
fn boundaries_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u32..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.sort_unstable();
        quarters.dedup();
        quarters
            .into_iter()
            .map(|quarter| f64::from(quarter) * 0.25)
            .collect()
    })
}

/// Extra inter-word gaps on the quarter-second grid (silence, never an
/// energy peak).
fn extra_gaps_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u32..=14_400, 0..=8).prop_map(|quarters| {
        quarters
            .into_iter()
            .map(|quarter| f64::from(quarter) * 0.25)
            .collect()
    })
}

/// The declared alignment granularities — the closed three-member set.
fn granularity_strategy() -> impl Strategy<Value = AlignmentGranularity> {
    prop_oneof![
        Just(AlignmentGranularity::None),
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

/// The cap-point oracle (FR-015 / FR-007): the earliest position the
/// resume may land on — the yield less the hard cap, clamped at 0.0.
fn cap_point(yield_secs: f64) -> f64 {
    (yield_secs - REWIND_CAP_SECS).max(0.0)
}

/// The greatest boundary at or before the raw cursor — the
/// containing-sentence start — or `None` when the cursor precedes
/// every declared boundary.
fn containing_start(raw_cursor_secs: f64, boundaries: &[f64]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for boundary in boundaries {
        if *boundary <= raw_cursor_secs && best.is_none_or(|previous| *boundary > previous) {
            best = Some(*boundary);
        }
    }
    best
}

/// Whether `point` sits on a declared gap within the shared epsilon
/// (silence, never an energy peak). An empty gap list reports every
/// point as mid-word.
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

/// The gap list for one case: the declared boundaries and inter-word
/// gaps, plus the cap point itself.
///
/// The cap point is added so the at-cap landing is never mid-word in
/// the oracle (the mid-word forward snap of the cap point is
/// asserted separately below, where the cap point is deliberately
/// NOT a gap).
fn gaps_for(boundaries: &[f64], extra_gaps: &[f64], cap: f64) -> Vec<f64> {
    let mut gaps = union_positions(boundaries, extra_gaps);
    gaps.push(cap);
    gaps.sort_by(f64::total_cmp);
    gaps.dedup();
    gaps
}

/// Build one resume input on the generated grid.
fn resume_input<'a>(
    yield_secs: f64,
    paused_duration_secs: f64,
    granularity: AlignmentGranularity,
    sentence_starts: &'a [f64],
    word_gaps: &'a [f64],
) -> ResumeInput<'a> {
    ResumeInput {
        yield_secs,
        paused_duration_secs,
        granularity,
        sentence_starts,
        word_gaps,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// T025 `prop_resume_snap_cap`: over arbitrary yields, pauses,
    /// boundary lists, gap lists, and granularities the resume lands
    /// at the containing-sentence start — or exactly at the cap when
    /// the containing sentence is longer than the cap, or at the raw
    /// cursor on the degrade paths — never mid-word, never ahead of
    /// the formula point, and never further back than 30 s.
    #[test]
    fn prop_resume_snap_cap(
        (yield_secs, paused_duration_secs, boundaries, extra_gaps, granularity) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            boundaries_strategy(),
            extra_gaps_strategy(),
            granularity_strategy(),
        )
    ) {
        let raw = raw_cursor(yield_secs, paused_duration_secs);
        let cap = cap_point(yield_secs);
        let gaps = gaps_for(&boundaries, &extra_gaps, cap);
        let input = resume_input(
            yield_secs,
            paused_duration_secs,
            granularity,
            &boundaries,
            &gaps,
        );
        let result = resolve_resume(&input);

        // The cursor is the single FR-015 formula, read through the
        // shared constants: `min(0.05 × paused, 30)`.
        prop_assert_eq!(
            cursor(paused_duration_secs),
            (REWIND_FRACTION * paused_duration_secs).min(REWIND_CAP_SECS),
            "the cursor for a {}s pause was not min(0.05 × paused, 30.0)",
            paused_duration_secs
        );

        // The snap oracle: the greatest boundary at or before the raw
        // cursor when the engine declares a granularity and the
        // boundary list is non-empty; the raw cursor on both degrade
        // paths (no granularity, or no boundary at or before the
        // cursor) — never document start. The cue contract (FR-007 /
        // T032): the resume steps the fixed cue distance back from
        // the snap (or from the cap when the sentence is longer than
        // the cap), so the window floor is the cue point and the
        // landing snaps forward into a gap when the cue point is
        // mid-word.
        let degrade = granularity == AlignmentGranularity::None || boundaries.is_empty();
        let containing = if degrade {
            None
        } else {
            containing_start(raw, &boundaries)
        };
        if degrade || containing.is_none() {
            // The degrade paths hold the raw cursor exactly and cut
            // no cue (rewind.rs: the algorithm never moved there).
            prop_assert_eq!(
                result.resume_secs,
                raw,
                "the resume {} was not the raw cursor {} on the degrade path (never document start)",
                result.resume_secs,
                raw
            );
        } else {
            let boundary = containing.expect("checked above");
            let cue_floor = if boundary >= cap {
                (boundary - CUE_SECS).max(cap)
            } else {
                // Sentence longer than the cap: the cap absorbs the cue.
                cap
            };
            prop_assert!(
                result.resume_secs >= cue_floor && result.resume_secs <= raw,
                "the resume {} left the [cue floor {}, raw cursor {}] window (raw cursor {}, cap {}, yield {})",
                result.resume_secs,
                cue_floor,
                raw,
                raw,
                cap,
                yield_secs
            );
            // Never mid-word: the resume sits on a declared gap, or
            // holds the cue floor when no gap lies inside the window
            // (the forward snap's documented fallback).
            let in_gap = is_test_gap(result.resume_secs, &gaps);
            let at_cue_floor = (result.resume_secs - cue_floor).abs() < 1e-9;
            prop_assert!(
                in_gap || at_cue_floor,
                "the resume {} landed on an energy peak — neither in a declared gap nor the cue floor {}",
                result.resume_secs,
                cue_floor
            );
        }

        // Never mid-word: the resume sits on a declared gap, unless it
        // is the raw last-spoken cursor held by a degrade path (the
        // algorithm cuts no word there — it simply never moved).
        let in_gap = is_test_gap(result.resume_secs, &gaps);
        let holds_raw_cursor = result.resume_secs == raw;
        prop_assert!(
            in_gap || holds_raw_cursor,
            "the resume {} landed on an energy peak — neither in a declared gap nor the raw cursor {}",
            result.resume_secs,
            raw
        );

        // Total backward ≤30 s interruption / ≤60 s kill, and never a
        // forward jump: the resume stays inside [cap, raw].
        prop_assert!(
            result.backward_secs <= INTERRUPTION_REPLAY_CAP_SECS,
            "total backward movement {} from the yield {} exceeded the 30 s interruption cap",
            result.backward_secs,
            yield_secs
        );
        prop_assert!(
            result.backward_secs <= KILL_REPLAY_CAP_SECS,
            "total backward movement {} from the yield {} exceeded the 60 s kill cap",
            result.backward_secs,
            yield_secs
        );
        prop_assert!(
            result.resume_secs >= cap && result.resume_secs <= raw,
            "the resume {} left the [cap {}, raw cursor {}] window for the yield {}",
            result.resume_secs,
            cap,
            raw,
            yield_secs
        );
        prop_assert!(
            result.resume_secs >= 0.0,
            "the resume {} went before document start (FR-007: never doc start)",
            result.resume_secs
        );
        prop_assert_eq!(
            result.backward_secs,
            yield_secs.max(0.0) - result.resume_secs,
            "total backward movement did not equal the yield {} minus the resume {}",
            yield_secs,
            result.resume_secs
        );
        prop_assert_eq!(
            result.first_yield_ms, 0,
            "the resume dropped the cumulative first-yield clock"
        );
        prop_assert_eq!(
            result.supersedes_ts_ms, None,
            "the resume invented a supersede link where none was given"
        );
    }

    /// The persisted cursor is the last-spoken position (FR-007): the
    /// resume computed from a clock's stored basis never rewinds
    /// further than the cap behind it, whatever the pause length.
    #[test]
    fn prop_resume_from_the_last_spoken_cursor_stays_inside_the_cap(
        (elapsed_secs, paused_duration_secs, boundary_quarters) in (
            yield_position_strategy(),
            paused_duration_strategy(),
            proptest::collection::vec(0u32..=14_400, 0..=8),
        )
    ) {
        let mut clock = PositionClock::new(0.0);
        prop_assert!(
            clock.on_audible_progress(elapsed_secs).is_ok(),
            "a non-negative audible tick on the generated grid must advance the clock"
        );
        let position = clock.position_secs();
        let mut boundaries: Vec<f64> = boundary_quarters
            .into_iter()
            .map(|quarter| f64::from(quarter) * 0.25)
            .collect();
        boundaries.sort_by(f64::total_cmp);
        boundaries.dedup();
        let cap = cap_point(clock.stored_basis());
        let gaps = gaps_for(&boundaries, &[], cap);
        let input = resume_input(
            position,
            paused_duration_secs,
            AlignmentGranularity::Sentence,
            &boundaries,
            &gaps,
        );
        let result: ResumeResult = resolve_resume(&input);
        prop_assert!(
            clock.position_secs() - clock.stored_basis() <= REWIND_CAP_SECS,
            "the kill-replay basis trails the position by more than the 30 s cap"
        );
        prop_assert!(
            result.backward_secs <= INTERRUPTION_REPLAY_CAP_SECS,
            "a resume from the last-spoken cursor {} rewound {} s, over the 30 s interruption cap",
            clock.stored_basis(),
            result.backward_secs
        );
        prop_assert!(
            result.resume_secs >= cap,
            "the resume {} went behind the cap point {} measured from the last-spoken cursor",
            result.resume_secs,
            cap
        );
    }
}

/// The FR-007 caps, their relationship, and a resume that exercises
/// the interruption cap exactly: a two-hour pause rewinds by exactly
/// 30 s — the hard ceiling — and never before document start.
#[test]
fn the_caps_hold_for_a_two_hour_pause() {
    assert!(
        INTERRUPTION_REPLAY_CAP_SECS <= KILL_REPLAY_CAP_SECS,
        "the interruption replay cap {} must be at most the kill replay cap {}",
        INTERRUPTION_REPLAY_CAP_SECS,
        KILL_REPLAY_CAP_SECS
    );
    let boundaries = [0.0];
    let input = ResumeInput {
        yield_secs: 3_600.0,
        paused_duration_secs: 7_200.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &boundaries,
        word_gaps: &boundaries,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    assert!(
        result.backward_secs <= INTERRUPTION_REPLAY_CAP_SECS,
        "a two-hour pause rewound {} s, over the 30 s interruption cap",
        result.backward_secs
    );
    assert!(
        result.resume_secs >= 0.0,
        "the resume {} went before document start",
        result.resume_secs
    );
    assert!(
        cursor(7_200.0) <= REWIND_CAP_SECS,
        "the two-hour cursor {} left the 30 s cap",
        cursor(7_200.0)
    );
}

/// A cap point landing mid-word snaps forward to the next inter-word
/// gap (FR-007 rule 4: never mid-word), unless that gap leaves the
/// capped rewind window — then the resume lands exactly at the cap
/// (the 30-second ceiling no snap may exceed).
#[expect(
    clippy::float_cmp,
    reason = "the yield 120.0, the in-window gap 92.0, the cap point 90.0, and the 30.0 s backward movement are exact binary fractions; == is the correct assertion and a tolerance would admit a mid-word landing"
)]
#[test]
fn a_mid_word_cap_point_snaps_forward_to_the_next_gap_unless_the_gap_leaves_the_window() {
    // yield 120 s, pause 100 s → raw cursor 115 s, cap point 90 s.
    // The sentence list holds only the document start, so the snap
    // falls to the cap point; the gaps hold 0.0 and one constructed
    // gap only, so the cap point is mid-word by construction.
    let starts = [0.0];
    let in_window_gap = 92.0;
    let past_window_gap = 116.0;
    let input_in_window = ResumeInput {
        yield_secs: 120.0,
        paused_duration_secs: 100.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &starts,
        word_gaps: &[0.0, in_window_gap],
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result_in_window = resolve_resume(&input_in_window);
    assert_eq!(
        result_in_window.resume_secs, in_window_gap,
        "the mid-word cap 90.0 did not snap forward to the in-window gap 92.0 (never mid-word)"
    );
    let input_past_window = ResumeInput {
        yield_secs: 120.0,
        paused_duration_secs: 100.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &starts,
        word_gaps: &[0.0, past_window_gap],
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result_past_window = resolve_resume(&input_past_window);
    assert_eq!(
        result_past_window.resume_secs, 90.0,
        "the mid-word cap 90.0 with the next gap 116.0 past the raw cursor 115.0 did not land exactly at the cap"
    );
    assert_eq!(
        result_past_window.backward_secs, REWIND_CAP_SECS,
        "total backward movement from the yield 120.0 was not exactly the 30.0 s cap"
    );
}

/// The two degrade paths hold the raw cursor — never document start
/// (FR-007 rule 6): no declared granularity, and an empty boundary
/// list, both leave the resume exactly at the raw cursor.
#[expect(
    clippy::float_cmp,
    reason = "raw_cursor(100.0, 40.0) is exactly 98.0 and raw_cursor(10.0, 100.0) is exactly 5.0 on the exact grids used; == is the correct assertion and a tolerance would admit a drift to document start"
)]
#[test]
fn the_degrade_paths_hold_the_raw_cursor_never_document_start() {
    let boundaries = [10.0, 20.0, 30.0];
    let input_none = ResumeInput {
        yield_secs: 100.0,
        paused_duration_secs: 40.0,
        granularity: AlignmentGranularity::None,
        sentence_starts: &boundaries,
        word_gaps: &boundaries,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result_none = resolve_resume(&input_none);
    assert_eq!(
        result_none.resume_secs,
        raw_cursor(100.0, 40.0),
        "the no-granularity degrade path did not hold the raw cursor"
    );
    let empty: &[f64] = &[];
    for granularity in [AlignmentGranularity::Word, AlignmentGranularity::Sentence] {
        let input_empty = ResumeInput {
            yield_secs: 100.0,
            paused_duration_secs: 40.0,
            granularity,
            sentence_starts: empty,
            word_gaps: empty,
            first_yield_ms: 0,
            supersedes_ts_ms: None,
        };
        let result_empty = resolve_resume(&input_empty);
        assert_eq!(
            result_empty.resume_secs,
            raw_cursor(100.0, 40.0),
            "the empty-boundary degrade path did not hold the raw cursor"
        );
    }
    let late_boundaries = [50.0];
    let input_late = ResumeInput {
        yield_secs: 10.0,
        paused_duration_secs: 100.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &late_boundaries,
        word_gaps: &late_boundaries,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result_late = resolve_resume(&input_late);
    assert_eq!(
        result_late.resume_secs, 5.0,
        "with no boundary at or before the raw cursor 5.0, the resume must hold the cursor"
    );
    assert_eq!(
        result_late.backward_secs, 5.0,
        "total backward movement must equal the yield 10.0 minus the resume 5.0"
    );
}

/// The persisted cursor is the last-spoken position (FR-007): the
/// kill-replay basis follows the last audible tick exactly, never
/// buffered ahead, and trails the current position by at most
/// [`REWIND_CAP_SECS`]. The atomic single-transaction write of that
/// cursor is the session store's contract
/// (`contracts/resume-algorithm.md` "Atomic write"); this file owns
/// the position half.
#[expect(
    clippy::float_cmp,
    reason = "stored_basis() must equal the exact sum of the delivered ticks (100.0 + 20.5 + 3.0 + 2.0 = 125.5, exact binary fractions); == is the correct assertion and a tolerance would admit a buffered-ahead cursor"
)]
#[test]
fn the_persisted_cursor_is_the_last_spoken_position() -> Result<(), PlayerError> {
    let mut clock = PositionClock::new(0.0);
    let mut last_spoken = 0.0;
    for elapsed in [100.0, 20.5, 3.0, 2.0] {
        last_spoken = clock.on_audible_progress(elapsed)?;
        assert_eq!(
            clock.stored_basis(),
            last_spoken,
            "the stored basis drifted from the last audible tick — the persisted cursor would not be the last-spoken position"
        );
        assert_eq!(
            clock.position_secs(),
            last_spoken,
            "the position drifted from the last audible tick"
        );
    }
    assert!(
        clock.position_secs() - clock.stored_basis() <= REWIND_CAP_SECS,
        "the kill-replay basis trails the position by more than the 30 s cap"
    );
    // The resume is computed at resume time from the persisted cursor
    // (never pre-computed and stored): a resume from the last-spoken
    // position rewinds inside the interruption cap under any pause.
    let gaps = [last_spoken - REWIND_CAP_SECS, last_spoken];
    let input = ResumeInput {
        yield_secs: last_spoken,
        paused_duration_secs: 600.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &gaps,
        word_gaps: &gaps,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    assert!(
        result.backward_secs <= INTERRUPTION_REPLAY_CAP_SECS,
        "a resume from the persisted cursor {} rewound {} s, over the 30 s interruption cap",
        last_spoken,
        result.backward_secs
    );
    Ok(())
}

/// FR-007 RED GATE — the context cue.
///
/// The contract requires the resume to sit a fixed 5 s further back
/// than the containing-sentence start (`contracts/resume-algorithm.md`
/// step 2: "From the sentence start, move further backward by exactly
/// 5 seconds of content"; T025 states the 5–10 s window).
///
/// STATUS: RED. `crates/player/src/rewind.rs::resolve_resume` stops at
/// the containing-sentence start — no cue exists — so the assertion
/// below fails with a cue distance of 0 s. This test is the blocking
/// gate for T032 (sentence-snap resume path in `rewind.rs` +
/// `resume.rs`, "cue 5–10 s") and MUST NOT be implemented here.
///
/// TODO(T032): land the 5–10 s context cue in `resolve_resume`
/// (sentence start − cue, still inside the 30 s cap, still never
/// mid-word) and this test turns green with a cue distance of 5–10 s.
#[test]
fn resume_lands_a_five_to_ten_second_cue_before_the_containing_sentence_start() {
    // yield 120 s, pause 100 s → raw cursor 115 s, cap point 90 s.
    // The containing sentence starts at 100 s, so the cue target is
    // 95 s — inside the cap and itself a declared gap.
    let sentence_start = 100.0;
    let cue_target = sentence_start - CUE_MIN_SECS;
    let starts = [sentence_start];
    let gaps = [sentence_start, cue_target];
    let input = ResumeInput {
        yield_secs: 120.0,
        paused_duration_secs: 100.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &starts,
        word_gaps: &gaps,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let result = resolve_resume(&input);
    let cue_distance = sentence_start - result.resume_secs;
    assert!(
        (CUE_MIN_SECS..=CUE_MAX_SECS).contains(&cue_distance),
        "the resume {} sat {} s before the containing-sentence start {} — FR-007 requires a 5–10 s context cue (TODO T032: no cue exists in rewind.rs yet)",
        result.resume_secs,
        cue_distance,
        sentence_start
    );
    assert!(
        is_test_gap(result.resume_secs, &gaps),
        "the cue target {} must sit on a declared gap, never mid-word",
        result.resume_secs
    );
    assert!(
        result.backward_secs <= INTERRUPTION_REPLAY_CAP_SECS,
        "the cued resume rewound {} s, over the 30 s interruption cap",
        result.backward_secs
    );
}
