//! T019 — US1 transport-math property tests (Phase 3, BLOCKING).
//!
//! Invariants under test (tasks.md T019, verbatim): "position
//! monotonicity + SC-005 record rule + FR-001/FR-002/FR-003/FR-005
//! authority (proptest invariants: `position_secs: f64 ≥ 0` never
//! decreases during `Playing`; screen-off playback continues per
//! FR-001; never report playing while silent per FR-005; no second
//! source of truth; every `transport:"playing"` record entry is
//! followed same-session by strictly greater `position_secs` unless
//! intervened by `completed|stopped|paused|device-loss|failure`;
//! pre-roll buffering NEVER advances position; FAIL before
//! implementation)" — plus the US1 test brief: "rewind/snap/cap
//! exactly ±5.0 s; sentence-snap resume 5%-of-pause ≤30 s total
//! backward, zero mid-word, monotonically advancing position while
//! Playing; rate ladder 7 rungs (FR-043 engine stretch)".
//!
//! Spec anchors: FR-018 ("Skip MUST move the playback position by
//! exactly 5 seconds — forward for skip-forward, backward for
//! skip-backward — measured against the position at the moment the
//! control was pressed, and MUST NOT depend on document structure"),
//! FR-015 ("playback MUST resume from a rewind of 5% of the paused
//! duration, capped at 30 seconds, measured back from the position
//! at which playback yielded … the position the audio actually
//! resumes from is the start of the sentence containing that cursor
//! position, so the resume lands on a sentence boundary rather than
//! part-way through a word. Total backward movement from the yield
//! point MUST NOT exceed 30 seconds under any pause length"), FR-019
//! (the closed ladder `0.75x|1.0x|1.25x|1.5x|1.75x|2.0x|4.0x`;
//! "Each rate control press MUST advance or retreat exactly one step
//! on that ladder, and a press at either end MUST saturate"), FR-043
//! (the engine exposes the rate as a pitch-preserving time-stretch,
//! so a rate change never alters the delivered sample rate), FR-002
//! (the player crate is the single authority for position and
//! transport state), FR-005 ("The system MUST NOT report a playing
//! state while producing no audible output"), SC-005 (zero
//! `transport:"playing"` entries not followed, in the same session,
//! by a later entry carrying a strictly greater `position_secs`,
//! except where a completion, stop, pause, device loss, or named
//! failure intervenes), and data-model.md `PreRollBuffer` ("Ring ≤60
//! s of engine-delivered audio; never advances position").
//!
//! FAILS before `crates/player/src/position.rs` (T025) and
//! `crates/player/src/cli.rs` (T027) exist: the `player::position`
//! and `player::cli` modules are unresolved, so this test target
//! does not build. That build failure is the required
//! pre-implementation red state.
//!
//! Expected API contract for T025/T027:
//! - `player::position::PositionClock` — the authoritative position
//!   clock (FR-002), with `pub const SKIP_STEP_SECS: f64 = 5.0`,
//!   `pub const REWIND_FRACTION: f64 = 0.05`, and
//!   `pub const REWIND_CAP_SECS: f64 = 30.0`; `new(start_secs: f64)`
//!   (clamped at `0.0` — `position_secs: f64 ≥ 0`), `position_secs(
//!   &self) -> f64`, `stored_basis(&self) -> f64` (the kill-replay
//!   basis of FR-004, trailing the position by at most
//!   `REWIND_CAP_SECS`), `on_audible_progress(&mut self,
//!   elapsed_secs: f64)` (the ONLY advance path — the backend calls
//!   it as audio is delivered; it never runs on its own and the
//!   session feeds it only while audio is audible), `skip_forward(
//!   &mut self) -> f64` (exactly `+SKIP_STEP_SECS`), `skip_back(
//!   &mut self) -> f64` (exactly `-SKIP_STEP_SECS`, clamped at
//!   `0.0`), `rewind_for_silence(&mut self, paused_duration_secs:
//!   f64) -> f64` (moves the cursor back from the yield by exactly
//!   `min(REWIND_FRACTION × d, REWIND_CAP_SECS)`, clamped at `0.0`,
//!   and remembers the yield for the cap), and `snap_to_boundary(
//!   &self, boundaries: &[f64]) -> f64` (the greatest boundary at or
//!   before the cursor, never earlier than the `REWIND_CAP_SECS` cap
//!   point — the cap takes precedence over the zero-mid-word rule,
//!   FR-015).
//! - `player::cli::PlaybackSession` — the single dispatch every
//!   surface routes through (FR-042): `new() -> Self` (position
//!   `0.0`, rate `R1_0` per FR-019b, transport `Paused`),
//!   `snapshot(&self) -> SessionSnapshot { transport, position_secs,
//!   rate, engine_id }`, `execute(&mut self, ShellVerb) ->
//!   Result<SessionSnapshot, PlayerError>`, and `on_audible_progress(
//!   &mut self, elapsed_secs: f64)` (gated on `Playing` — position
//!   advances only on audible playback, never while silent).
//!
//! Exactness note: every position, boundary, and yield below is
//! generated on the quarter-second grid (`k × 0.25`), an exact
//! binary-fraction grid, and every formula is asserted with the same
//! expression the contract states, so all comparisons are bit-exact.
//! A tolerance would wrongly admit an off-step skip (FR-018 demands
//! *exactly* ±5.0 s) or a cap violation (FR-015's 30-second cap is
//! a hard ceiling, not a target).

use proptest::prelude::*;

use engine_core::closed::{PlaybackRate, TransportState};
use engine_core::format::{DeclaredFormat, check_handoff};
use engine_core::standin_a::StandinA;
use player::buffer::PreRollBuffer;
use player::cli::{PlaybackSession, ShellVariant, ShellVerb};
use player::position::{PositionClock, REWIND_CAP_SECS, REWIND_FRACTION, SKIP_STEP_SECS};
use player::rate::{LADDER, RATE_EFFECTIVE_BOUND_SECS, step_down, step_up};
use player::skip::{apply_skip_exact, rate_change_effective_time};

/// Positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Sentence boundaries on the same grid: sorted, deduplicated, and
/// always containing the document start `0.0` (so a greatest-boundary-
/// at-or-before lookup always has an answer).
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn boundaries_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.push(0);
        quarters.sort_unstable();
        quarters.dedup();
        quarters.into_iter().map(|q| q as f64 * 0.25).collect()
    })
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=720_000).prop_map(|centis| centis as f64 / 100.0)
}

/// Audible progress in `(0.00, 5.00]` s — strictly positive, so a
/// progress tick while `Playing` is a strictly greater position.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn progress_strategy() -> impl Strategy<Value = f64> {
    (1u64..=500).prop_map(|centis| centis as f64 / 100.0)
}

/// The seven rungs of the closed rate ladder (FR-019).
fn rate_strategy() -> impl Strategy<Value = PlaybackRate> {
    prop_oneof![
        Just(PlaybackRate::R0_75),
        Just(PlaybackRate::R1_0),
        Just(PlaybackRate::R1_25),
        Just(PlaybackRate::R1_5),
        Just(PlaybackRate::R1_75),
        Just(PlaybackRate::R2_0),
        Just(PlaybackRate::R4_0),
    ]
}

/// One driving action from the closed vocabulary any surface can
/// issue (notification, lock screen, CLI, or IPC — all route through
/// the same dispatch per FR-042). `AudibleProgress` is the backend's
/// clock seam: engine audio was delivered and `secs` of it was
/// audible. There is no unlock action — the closed verb set has none,
/// which is what makes a locked trial mechanically unlock-free.
#[derive(Clone, Copy, Debug, PartialEq)]
enum DrivingAction {
    Play,
    Pause,
    Stop,
    Complete,
    Interrupt,
    SkipForward,
    SkipBack,
    RateUp,
    RateDown,
    AudibleProgress(f64),
}

fn driving_action_strategy() -> impl Strategy<Value = DrivingAction> {
    prop_oneof![
        Just(DrivingAction::Play),
        Just(DrivingAction::Pause),
        Just(DrivingAction::Stop),
        Just(DrivingAction::Complete),
        Just(DrivingAction::Interrupt),
        Just(DrivingAction::SkipForward),
        Just(DrivingAction::SkipBack),
        Just(DrivingAction::RateUp),
        Just(DrivingAction::RateDown),
        progress_strategy().prop_map(DrivingAction::AudibleProgress),
    ]
}

fn driving_action_vec_strategy() -> impl Strategy<Value = Vec<DrivingAction>> {
    proptest::collection::vec(driving_action_strategy(), 1..=64)
}

/// One diagnostic-record-shaped entry: a transport state carrying a
/// position, as every entry in the record is (SC-005, FR-039).
#[derive(Clone, Copy, Debug, PartialEq)]
struct RecordEntry {
    transport: TransportState,
    position_secs: f64,
    /// Whether this entry lands a user-initiated skip-backward
    /// repositioning (FR-018). A skip-back legitimately moves the
    /// position backward while staying `Playing`, so it ends the
    /// prior entry's advance window instead of violating it. This is
    /// a simulation seam, not a diagnostic-record field: it marks
    /// which entries the SC-005 scan must treat as repositionings.
    repositioned: bool,
}

/// The enders of SC-005: a completion, a stop, a pause, a device
/// loss, or a named failure — each legitimately ends the advance. In
/// the closed transport set those are exactly the four non-`Playing`
/// states (`Completed`, `Stopped`, `Paused`, and `Interrupted`, which
/// is where device loss and named failure land per data-model.md).
/// The match is exhaustive over the closed set, so a state added
/// without an ender decision cannot compile silently (FR-044).
const fn is_ender(state: TransportState) -> bool {
    match state {
        TransportState::Playing => false,
        TransportState::Paused
        | TransportState::Stopped
        | TransportState::Completed
        | TransportState::Interrupted => true,
    }
}

/// The session has ended: no transport control can move it except a
/// Stop press, which is idempotent.
const fn is_terminal(state: TransportState) -> bool {
    matches!(state, TransportState::Stopped | TransportState::Completed)
}

/// The SC-005 scan: every `transport:"playing"` entry must be
/// followed, in the same session, by a later entry carrying a
/// strictly greater `position_secs`, except where an ender entry
/// intervenes first (the advance legitimately ended) or where a
/// skip-backward repositioning lands first (FR-018: the user moved
/// the cursor back on purpose, ending the prior window and opening
/// a new one at the earlier point). The final entry's window is open — a hard kill cuts the record mid-session
/// with no `session_end` entry (FR-004, SC-007) — so only entries
/// with a later entry can violate. Returns the indices of the
/// violating playing entries; an empty result is the conforming
/// report.
fn sc005_violations(record: &[RecordEntry]) -> Vec<usize> {
    let mut violations = Vec::new();
    for (i, entry) in record.iter().enumerate() {
        if entry.transport != TransportState::Playing {
            continue;
        }
        let followers = &record[i + 1..];
        if followers.is_empty() {
            continue; // open window: the record was cut mid-session
        }
        let mut conform = false;
        for follower in followers {
            if follower.position_secs > entry.position_secs {
                conform = true; // a later entry shows the advance
                break;
            }
            if follower.repositioned {
                conform = true; // a skip-back repositioned the cursor on purpose (FR-018)
                break;
            }
            if is_ender(follower.transport) {
                conform = true; // an ender legitimately ended the advance
                break;
            }
        }
        if !conform {
            violations.push(i);
        }
    }
    violations
}

/// Drive one action, tolerating a rejected press: a driver may press
/// Play twice, and the second press is an `IllegalTransition` the
/// simulation discards — the record only carries state changes.
fn drive(action: DrivingAction, session: &mut PlaybackSession) {
    match action {
        DrivingAction::Play => {
            let _rejected = session.execute(ShellVerb::Play);
        }
        DrivingAction::Pause => {
            let _rejected = session.execute(ShellVerb::Pause);
        }
        DrivingAction::Stop => {
            let _rejected = session.execute(ShellVerb::Stop);
        }
        DrivingAction::Complete => {
            let _rejected = session.execute(ShellVerb::Complete);
        }
        DrivingAction::Interrupt => {
            let _rejected = session.execute(ShellVerb::Interrupt);
        }
        DrivingAction::SkipForward => {
            let _rejected = session.execute(ShellVariant::skip_forward());
        }
        DrivingAction::SkipBack => {
            let _rejected = session.execute(ShellVariant::skip_back());
        }
        DrivingAction::RateUp => {
            let _rejected = session.execute(ShellVerb::RateUp);
        }
        DrivingAction::RateDown => {
            let _rejected = session.execute(ShellVerb::RateDown);
        }
        DrivingAction::AudibleProgress(secs) => {
            session.on_audible_progress(secs);
        }
    }
}

/// Simulate a driving session and return its record-shaped entries.
/// An entry is recorded whenever the transport state or the position
/// changes — the record carries a position on every transport change
/// (SC-005: "every transport change therefore carries `position_secs`")
/// and on every audible-progress event. With `hard_kill` the record
/// ends mid-session exactly as a hard kill leaves it (no `session_end`
/// entry, FR-004); otherwise the session is closed with a Stop press
/// so every open playing window ends in an ender entry, as a real
/// session's record does.
#[expect(
    clippy::float_cmp,
    reason = "the record-entry change detection compares the exact stored f64 positions; != on the represented values is the correct change detector and a tolerance would drop or add a recorded entry"
)]
fn simulate(actions: &[DrivingAction], hard_kill: bool) -> Vec<RecordEntry> {
    let mut session = PlaybackSession::new();
    let start = session.snapshot();
    let mut record = vec![RecordEntry {
        transport: start.transport,
        position_secs: start.position_secs,
        repositioned: false,
    }];
    for action in actions {
        let before = session.snapshot();
        drive(*action, &mut session);
        let after = session.snapshot();
        if after.transport != before.transport || after.position_secs != before.position_secs {
            record.push(RecordEntry {
                transport: after.transport,
                position_secs: after.position_secs,
                // A recorded skip-back entry always moved strictly
                // backward (a clamped no-op records nothing), so the
                // marker is exactly the backward-repositioning set.
                repositioned: matches!(action, DrivingAction::SkipBack),
            });
        }
    }
    if !hard_kill {
        let last = record.last();
        let already_ended = last.is_none_or(|entry| is_terminal(entry.transport));
        if !already_ended {
            drive(DrivingAction::Stop, &mut session);
            let after = session.snapshot();
            record.push(RecordEntry {
                transport: after.transport,
                position_secs: after.position_secs,
                repositioned: false,
            });
        }
    }
    record
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn skip_forward_moves_exactly_five_seconds(position in position_strategy()) {
        // FR-018: "Skip MUST move the playback position by exactly 5
        // seconds — forward for skip-forward … and MUST NOT depend on
        // document structure, so that the same press produces the same
        // displacement regardless of headings, paragraphing, or engine."
        let mut clock = PositionClock::new(position);
        let landed = clock.skip_forward();
        prop_assert_eq!(
            landed,
            position + SKIP_STEP_SECS,
            "skip-forward from {} did not land exactly 5.0 s forward", position,
        );
        prop_assert_eq!(
            clock.position_secs(),
            position + SKIP_STEP_SECS,
            "the clock did not hold the skip-forward landing point"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn skip_back_moves_exactly_five_seconds_clamped_at_zero(position in position_strategy()) {
        // FR-018: exactly 5 seconds backward, measured against the
        // position at the moment the control was pressed; the clock's
        // floor is the declared `position_secs: f64 ≥ 0` (data-model).
        let mut clock = PositionClock::new(position);
        let landed = clock.skip_back();
        let expected = (position - SKIP_STEP_SECS).max(0.0);
        prop_assert_eq!(
            landed,
            expected,
            "skip-back from {} did not land exactly 5.0 s back, clamped at 0.0", position,
        );
        prop_assert_eq!(
            clock.position_secs(),
            expected,
            "the clock did not hold the skip-back landing point"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn the_rewind_is_five_percent_of_the_pause_capped_at_thirty_seconds(
        (yield_position, paused_duration) in (position_strategy(), paused_duration_strategy())
    ) {
        // FR-015: "playback MUST resume from a rewind of 5% of the
        // paused duration, capped at 30 seconds, measured back from
        // the position at which playback yielded." The rewind amount
        // is exactly min(0.05 × d, 30.0) — no tolerance, because the
        // proportional figure is the requirement.
        let mut clock = PositionClock::new(yield_position);
        let rewound = clock.rewind_for_silence(paused_duration);
        let rewind_amount = (REWIND_FRACTION * paused_duration).min(REWIND_CAP_SECS);
        let expected = (yield_position - rewind_amount).max(0.0);
        prop_assert_eq!(
            rewound,
            expected,
            "rewind of a {}s pause from {} did not land at the 5%-of-pause cursor capped at 30 s",
            paused_duration,
            yield_position
        );
        prop_assert_eq!(
            clock.position_secs(),
            expected,
            "the clock did not hold the rewound cursor"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn the_sentence_snap_lands_on_a_boundary_never_mid_word(
        (yield_position, paused_duration, boundaries) in
            (position_strategy(), paused_duration_strategy(), boundaries_strategy())
    ) {
        // FR-015: "the position the audio actually resumes from is the
        // start of the sentence containing that cursor position, so
        // the resume lands on a sentence boundary rather than part-way
        // through a word. Total backward movement from the yield point
        // MUST NOT exceed 30 seconds under any pause length." The snap
        // lands on the greatest boundary at or before the cursor, and
        // never earlier than the cap point — where the containing
        // sentence begins earlier than the cap point, the resume lands
        // at the cap point instead (the cap takes precedence).
        let mut clock = PositionClock::new(yield_position);
        let cursor = clock.rewind_for_silence(paused_duration);
        let resumed = clock.snap_to_boundary(&boundaries);
        let cap_point = (yield_position - REWIND_CAP_SECS).max(0.0);
        prop_assert!(
            resumed <= cursor,
            "the resume {resumed} landed after the rewound cursor {cursor} — a snap must move back to a sentence start"
        );
        prop_assert!(
            resumed >= cap_point,
            "the resume {resumed} moved past the 30-second cap point {cap_point} from the yield {yield_position}"
        );
        prop_assert!(
            resumed >= 0.0,
            "the resume {resumed} went below the position floor 0.0"
        );
        let on_a_boundary = boundaries.contains(&resumed);
        let at_the_cap_point = resumed == cap_point;
        prop_assert!(
            on_a_boundary || at_the_cap_point,
            "the resume {resumed} is neither a sentence boundary nor the cap point — a mid-word landing (FR-015 zero-mid-word rule, with the documented cap precedence)"
        );
        // Total backward movement from the yield never exceeds the cap.
        prop_assert!(
            yield_position - resumed <= REWIND_CAP_SECS,
            "total backward movement of {} s from the yield {yield_position} exceeded the 30-second cap",
            yield_position - resumed
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn audible_progress_advances_the_position_exactly_while_playing(
        progress in progress_strategy()
    ) {
        // FR-002: the player crate is the single authority for
        // position. Audible progress while Playing advances the
        // position by exactly the delivered amount.
        let mut session = PlaybackSession::new();
        let started = session.execute(ShellVerb::Play);
        prop_assert!(
            started.is_ok(),
            "Play must succeed from the initial Paused state"
        );
        session.on_audible_progress(progress);
        let snapshot = session.snapshot();
        prop_assert_eq!(
            snapshot.transport,
            TransportState::Playing,
            "the session must still be Playing after audible progress"
        );
        prop_assert_eq!(
            snapshot.position_secs,
            progress,
            "audible progress of {} s did not advance the position by exactly that amount",
            progress
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn audible_progress_never_advances_the_position_while_not_playing(
        progress in progress_strategy()
    ) {
        // FR-005 / SC-005: never report playing while silent, and
        // position advances only on audible playback. The same
        // progress tick that advances a Playing session must leave a
        // Paused session exactly where it was.
        let mut idle = PlaybackSession::new();
        idle.on_audible_progress(progress);
        prop_assert_eq!(
            idle.snapshot().position_secs,
            0.0,
            "audible progress advanced a session that never played"
        );

        let mut paused = PlaybackSession::new();
        let _played = paused.execute(ShellVerb::Play);
        paused.on_audible_progress(progress);
        let _paused = paused.execute(ShellVerb::Pause);
        paused.on_audible_progress(progress);
        prop_assert_eq!(
            paused.snapshot().position_secs,
            progress,
            "audible progress advanced a Paused session — the SC-005 playing-without-advance defect"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn the_position_is_monotonically_non_decreasing_while_playing(
        progresses in proptest::collection::vec(progress_strategy(), 1..=64)
    ) {
        // tasks.md T019: "`position_secs: f64 ≥ 0` never decreases
        // during `Playing`." Time only moves forward: each audible
        // progress tick adds to the position, so the clock tracks the
        // exact accumulated sum.
        let mut session = PlaybackSession::new();
        let _played = session.execute(ShellVerb::Play);
        let mut expected = 0.0;
        for progress in &progresses {
            expected += *progress;
            session.on_audible_progress(*progress);
            let snapshot = session.snapshot();
            prop_assert_eq!(
                snapshot.position_secs,
                expected,
                "the position decreased or diverged from the accumulated audible progress"
            );
            prop_assert!(
                snapshot.position_secs >= 0.0,
                "the position went below the declared floor 0.0"
            );
        }
    }

    #[test]
    fn every_playing_entry_is_followed_by_strictly_greater_position_or_an_ender(
        actions in driving_action_vec_strategy()
    ) {
        // SC-005, verbatim: "Zero occurrences of a `transport:
        // "playing"` entry in the diagnostic record that is not
        // followed, in the same session, by a later entry carrying a
        // strictly greater `position_secs` … except where the entries
        // that intervene are a completion, a stop, a pause, a device
        // loss, or a named failure, each of which legitimately ends
        // the advance." Checked over both record shapes: a closed
        // session (every open window ends in an ender entry) and a
        // hard-killed session (the record ends mid-session, the final
        // entry's window open — the case FR-004/SC-007 makes
        // possible, and the one a silent-playing defect shows up in).
        for hard_kill in [true, false] {
            let record = simulate(&actions, hard_kill);
            let violations = sc005_violations(&record);
            prop_assert!(
                violations.is_empty(),
                "SC-005 violations at record entries {violations:?} in the {} record {record:?}",
                if hard_kill { "hard-killed" } else { "closed" }
            );
        }
    }

    #[test]
    fn the_rate_ladder_moves_one_step_and_saturates_at_both_ends(rung in rate_strategy()) {
        // FR-019: "Each rate control press MUST advance or retreat
        // exactly one step on that ladder, and a press at either end
        // MUST saturate (stay at that end) rather than wrapping,
        // failing, or yielding a rate outside the ladder."
        // Each direction gets its own session walked to the rung,
        // so both one-step assertions are measured from the rung —
        // a single session would measure the down-press from the
        // post-up rung instead.
        let mut up_session = walked_to(rung);
        let mut down_session = walked_to(rung);
        prop_assert_eq!(
            up_session.snapshot().rate,
            rung,
            "the session could not be walked to the {:?} rung",
            rung
        );
        prop_assert_eq!(
            down_session.snapshot().rate,
            rung,
            "the session could not be walked to the {:?} rung",
            rung
        );
        let up = up_session.execute(ShellVerb::RateUp);
        prop_assert_eq!(
            up.ok().map(|snapshot| snapshot.rate),
            Some(rate_up_from(rung)),
            "RateUp from {:?} did not move exactly one step up the ladder",
            rung
        );
        let down = down_session.execute(ShellVerb::RateDown);
        prop_assert_eq!(
            down.ok().map(|snapshot| snapshot.rate),
            Some(rate_down_from(rung)),
            "RateDown from {:?} did not move exactly one step down the ladder",
            rung
        );
    }

    #[test]
    fn a_rate_change_never_alters_the_delivered_handoff_shape(rung in rate_strategy()) {
        // FR-043: the engine exposes the rate as a pitch-preserving
        // time-stretch operation — the rate change is applied by the
        // engine, never by resampling, so the delivered handoff shape
        // is invariant at every rung of the ladder (the engine emits
        // mono f32 already resampled to the one declared 22050 Hz,
        // FR-033; a deviation is a named EngineError, never a silent
        // resample, FR-035). The stand-in is the US1 engine fixture.
        let mut engine = StandinA;
        engine.set_rate(rung);
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            (DeclaredFormat::SAMPLE_RATE, DeclaredFormat::CHANNELS, DeclaredFormat::SAMPLE_WIDTH_BITS),
            "the engine's delivered shape changed at rate {:?} — a rate change must be a pitch-preserving time-stretch, not a resample",
            rung
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "the delivered shape at rate {rung:?} failed its own handoff check"
        );
    }
}

/// The rung's index in the declared ascending ladder, used only to
/// walk a session from the default rung to a generated one.
fn ladder_index(rate: PlaybackRate) -> usize {
    const LADDER: [PlaybackRate; 7] = [
        PlaybackRate::R0_75,
        PlaybackRate::R1_0,
        PlaybackRate::R1_25,
        PlaybackRate::R1_5,
        PlaybackRate::R1_75,
        PlaybackRate::R2_0,
        PlaybackRate::R4_0,
    ];
    LADDER
        .iter()
        .position(|candidate| *candidate == rate)
        .unwrap_or(0)
}

/// A fresh session walked from the default rung (`R1_0`,
/// FR-019b) to `rung` — one RateUp/RateDown press per step,
/// the walk bounded by the closed seven-rung ladder.
fn walked_to(rung: PlaybackRate) -> PlaybackSession {
    let mut session = PlaybackSession::new();
    let upwards = ladder_index(rung) > ladder_index(PlaybackRate::R1_0);
    for _ in 0..7 {
        if session.snapshot().rate == rung {
            break;
        }
        let _stepped = session.execute(if upwards {
            ShellVerb::RateUp
        } else {
            ShellVerb::RateDown
        });
    }
    session
}

/// The one-step-up neighbor on the closed ladder, saturating at
/// `R4_0` (FR-019). The exhaustive match compiles only against
/// exactly the seven declared rungs.
const fn rate_up_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R0_75 => PlaybackRate::R1_0,
        PlaybackRate::R1_0 => PlaybackRate::R1_25,
        PlaybackRate::R1_25 => PlaybackRate::R1_5,
        PlaybackRate::R1_5 => PlaybackRate::R1_75,
        PlaybackRate::R1_75 => PlaybackRate::R2_0,
        PlaybackRate::R2_0 | PlaybackRate::R4_0 => PlaybackRate::R4_0,
    }
}

/// The one-step-down neighbor on the closed ladder, saturating at
/// `R0_75` (FR-019).
const fn rate_down_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R0_75 | PlaybackRate::R1_0 => PlaybackRate::R0_75,
        PlaybackRate::R1_25 => PlaybackRate::R1_0,
        PlaybackRate::R1_5 => PlaybackRate::R1_25,
        PlaybackRate::R1_75 => PlaybackRate::R1_5,
        PlaybackRate::R2_0 => PlaybackRate::R1_75,
        PlaybackRate::R4_0 => PlaybackRate::R2_0,
    }
}

/// The rewind cap boundary: a zero-length pause owes no rewind, a
/// pause of exactly 600 s rewinds exactly 30.0 s (`0.05 × 600 ==
/// 30.0` is exact in f64), and any longer pause still rewinds
/// exactly 30.0 s — the hard ceiling of FR-015.
#[expect(
    clippy::float_cmp,
    reason = "the boundary values 0.0, 600.0, 1200.0, 3600.0, 30.0 and 3570.0 are exact binary fractions and 0.05 × 600.0 == 30.0 exactly, so == is the correct assertion for a boundary test; a tolerance would admit an off-boundary rewind"
)]
#[test]
fn the_rewind_cap_boundary_is_exactly_thirty_seconds() {
    let yield_position = 3600.0;
    let mut clock = PositionClock::new(yield_position);
    assert_eq!(
        clock.rewind_for_silence(0.0),
        yield_position,
        "a zero-length pause owes no rewind"
    );
    let mut clock = PositionClock::new(yield_position);
    assert_eq!(
        clock.rewind_for_silence(600.0),
        yield_position - 30.0,
        "a 600 s pause rewinds exactly 5% = exactly 30.0 s"
    );
    let mut clock = PositionClock::new(yield_position);
    assert_eq!(
        clock.rewind_for_silence(1200.0),
        yield_position - 30.0,
        "a 1200 s pause still rewinds exactly 30.0 s — the cap is a hard ceiling"
    );
    let mut clock = PositionClock::new(yield_position);
    assert_eq!(
        clock.rewind_for_silence(7200.0),
        yield_position - 30.0,
        "a 7200 s pause still rewinds exactly 30.0 s"
    );
}

/// Pre-roll buffering never advances position (data-model.md
/// `PreRollBuffer`: "never advances position"; tasks.md T019:
/// "pre-roll buffering NEVER advances position"). After the Play
/// press the transport is `Playing` while the engine fills the
/// pre-roll ring — and until audio is actually delivered, the
/// authoritative position has not moved. Position moves only
/// through `on_audible_progress`, the backend's audio-delivered
/// clock seam, and a zero-length tick moves nothing.
#[expect(
    clippy::float_cmp,
    reason = "the pre-roll assertion compares the position against the exact binary fraction 0.0 before any audio was delivered; == is exact and a tolerance would admit a clock that advances on its own during pre-roll"
)]
#[test]
fn pre_roll_buffering_never_advances_position() {
    let mut session = PlaybackSession::new();
    let started = session.execute(ShellVerb::Play);
    assert!(
        started.is_ok(),
        "Play must succeed from the initial Paused state"
    );
    // The pre-roll window: Playing, no audio delivered yet.
    assert_eq!(
        session.snapshot().transport,
        TransportState::Playing,
        "the Play press must put the transport in Playing"
    );
    assert_eq!(
        session.snapshot().position_secs,
        0.0,
        "the position advanced during pre-roll before any audio was delivered"
    );
    // A zero-length progress tick is not audible progress.
    session.on_audible_progress(0.0);
    assert_eq!(
        session.snapshot().position_secs,
        0.0,
        "a zero-length progress tick moved the position"
    );
}

/// The position clock's floor: `position_secs: f64 ≥ 0` (data-model
/// `PlaybackSession`). A start below zero is clamped to the declared
/// floor, never carried negative.
#[expect(
    clippy::float_cmp,
    reason = "the floor assertion compares against the exact binary fraction 0.0; == is exact for a clamped start"
)]
#[test]
fn a_negative_start_is_clamped_to_the_declared_floor() {
    let clock = PositionClock::new(-1.0);
    assert_eq!(
        clock.position_secs(),
        0.0,
        "the position clock carried a negative start below the declared floor"
    );
}

// ============================================================================
// T049 — US4 skip + ladder invariants (Phase-11 test-first append).
//
// Invariants under test (tasks.md T049, verbatim): "skip =
// exactly ±5.0 s vs press-time position, clamp at
// [0.0, duration], forward-to-end → paused end-state;
// ladder closed 7 rungs, one step per press, saturate
// both ends never wrap/fail; rate change effective ≤2 s;
// position unchanged by rate change; pre-roll re-prepared
// at new rate; FAIL before implementation".
//
// Spec anchors: FR-018 (skip moves the position by
// exactly 5 s — forward for skip-forward, backward for
// skip-backward — measured against the position at the
// moment the control was pressed, clamped to
// [0.0, duration]; a forward skip that reaches or passes
// the end lands at the end in a PAUSED end-state —
// completion is document-end only, never a skip), FR-019
// (the closed seven-rung ladder
// 0.75|1.0|1.25|1.5|1.75|2.0|4.0; each press moves
// exactly one rung; a press at either end saturates —
// never wraps, never fails, never off-ladder), FR-019a
// (a rate change is effective within 2 s), FR-019b (the
// default rate is 1.0), and FR-043 (a rate change is a
// pitch-preserving time-stretch inside the engine, never
// a local stretch; the position is unchanged by a rate
// change; the pre-roll is re-prepared at the new rate).
//
// Expected API contract for T057 (the production code
// this test gates — it does not exist yet, which is the
// required red state):
// - `player::rate` — `LADDER: [PlaybackRate; 7]` (the
//   closed ladder, ascending), `step_up(rung) ->
//   PlaybackRate` (one rung up, saturating at `R4_0`),
//   `step_down(rung) -> PlaybackRate` (one rung down,
//   saturating at `R0_75`), and
//   `RATE_EFFECTIVE_BOUND_SECS: f64 = 2.0` (FR-019a).
// - `PlaybackSession::with_duration(duration_secs) ->
//   Self` — a session whose known duration bounds every
//   skip (FR-018) and fixes determinate progress
//   (FR-028c) — and `duration_secs(&self) ->
//   Option<f64>` (`Some` for a `with_duration` session,
//   `None` for `new()`, whose skips clamp at the 0.0
//   floor only and never reach an end-state).
// - Skip verb semantics: `SkipForward` lands at
//   `min(press_time + 5.0, duration)` and, when that
//   reaches the duration, lands the transport on
//   `Paused` (the end-state); `SkipBackward` lands at
//   `max(press_time - 5.0, 0.0)` and never changes the
//   transport. Rate verbs change exactly one rung, leave
//   `position_secs` untouched, and clear the pre-roll
//   buffer (it must be re-prepared at the new rate,
//   FR-043).
//
// Oracle note: the unclamped ±5.0 s core is
// `apply_skip_exact` (`player::skip`); the session-level
// oracle additionally clamps at the session's duration
// ceiling. The one-step ladder oracles are this file's
// own `rate_up_from`/`rate_down_from` — exhaustive
// matches over the same closed seven-rung set, with no
// catch-all arm. Every position and duration below is
// generated on the quarter-second grid (`k × 0.25`), an
// exact binary-fraction grid, so all comparisons are
// bit-exact — a tolerance would wrongly admit an
// off-step skip (FR-018 demands *exactly* ±5.0 s).
//
// Format note: every `prop_assert*!` message uses
// explicit positional arguments (`{}`, `arg`), never
// implicit named captures (`{var}`) — proptest builds
// the message through `concat!`, and rustc rejects
// implicit captures in macro-expanded format strings.
// ============================================================================

/// A session duration and a press-time position, both on
/// the quarter-second grid, the position always within
/// `[0.0, duration]` — the press-time state a skip verb
/// is pressed from (FR-018 measures the skip against the
/// position at the moment the control was pressed).
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn duration_and_press_time_strategy() -> impl Strategy<Value = (f64, f64)> {
    (0u64..=14_400, 0u64..=14_400).prop_map(|(duration_quarters, press_quarters)| {
        let press = press_quarters.min(duration_quarters);
        (duration_quarters as f64 * 0.25, press as f64 * 0.25)
    })
}

/// A session duration and a press-time position whose
/// forward skip reaches or passes the end:
/// `press + 5.0 >= duration` (20 quarters = 5.0 s of
/// headroom below the duration), on the quarter-second
/// grid.
#[expect(
    clippy::cast_precision_loss,
    reason = "the strategy bounds every centisecond/quarter-second count at 720 000 / 14 400 (< 2^20), far below f64's exact-integer range (2^53); the grid value converts exactly"
)]
fn end_reaching_press_strategy() -> impl Strategy<Value = (f64, f64)> {
    (0u64..=14_400)
        .prop_flat_map(|duration_quarters| {
            let floor = duration_quarters.saturating_sub(20);
            (Just(duration_quarters), floor..=duration_quarters)
        })
        .prop_map(|(duration_quarters, press_quarters)| {
            (
                duration_quarters as f64 * 0.25,
                press_quarters as f64 * 0.25,
            )
        })
}

/// The session-level skip oracle (FR-018): the unclamped
/// ±5.0 s core (`apply_skip_exact`, whose own 0.0-floor
/// clamp is part of the contract) further clamped at the
/// session's duration ceiling — the document end bounds
/// every skip on a `with_duration` session.
fn session_skip_landing(press_time_secs: f64, delta_secs: f64, duration_secs: f64) -> f64 {
    apply_skip_exact(press_time_secs, delta_secs).min(duration_secs)
}

/// Drive a `with_duration` session to a press-time
/// position: the Play press opens playback and the
/// audible-progress tick moves the authoritative clock to
/// exactly `press_time_secs`, leaving the session
/// `Playing` — the state a skip or rate press is issued
/// in.
fn playing_at_position(session: &mut PlaybackSession, press_time_secs: f64) {
    let _played = session.execute(ShellVerb::Play);
    session.on_audible_progress(press_time_secs);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[expect(
        clippy::float_cmp,
        reason = "press, SKIP_STEP_SECS and duration are exact binary fractions on the quarter-second grid; the >= test decides whether the skip reaches the end (FR-018), and a tolerance would misclassify a boundary landing"
    )]
    #[test]
    fn skip_verbs_move_exactly_five_seconds_against_the_press_time(
        (duration, press) in duration_and_press_time_strategy(),
        forward in prop_oneof![Just(true), Just(false)],
    ) {
        // P1 (FR-018): exactly ±5.0 s measured against the
        // press-time position, clamped to [0.0, duration] —
        // never dependent on document structure.
        let mut session = PlaybackSession::with_duration(duration);
        prop_assert_eq!(
            session.duration_secs(),
            Some(duration),
            "a with_duration session did not report its known duration"
        );
        playing_at_position(&mut session, press);
        let verb = if forward {
            ShellVariant::skip_forward()
        } else {
            ShellVariant::skip_back()
        };
        let delta = if forward {
            SKIP_STEP_SECS
        } else {
            -SKIP_STEP_SECS
        };
        let expected = session_skip_landing(press, delta, duration);
        let landed = session.execute(verb);
        prop_assert!(
            landed.is_ok(),
            "a {} skip from press-time {} was rejected",
            if forward { "forward" } else { "backward" },
            press
        );
        let snapshot = session.snapshot();
        prop_assert_eq!(
            snapshot.position_secs,
            expected,
            "a {} skip from press-time {} did not land at exactly the clamped ±5.0 s point {}",
            if forward { "forward" } else { "backward" },
            press,
            expected
        );
        // The clamp at both ends: the landing never leaves
        // [0.0, duration].
        prop_assert!(
            snapshot.position_secs >= 0.0,
            "the skip landed below the 0.0 floor (FR-018)"
        );
        prop_assert!(
            snapshot.position_secs <= duration,
            "the skip landed past the duration ceiling {} (FR-018)",
            duration
        );
        // A skip is not a transport control: only a forward
        // skip that reaches the end lands the paused
        // end-state.
        let reaches_the_end = forward && press + SKIP_STEP_SECS >= duration;
        if reaches_the_end {
            prop_assert_eq!(
                snapshot.transport,
                TransportState::Paused,
                "a forward skip reaching the end did not land the paused end-state (FR-018)"
            );
        } else {
            prop_assert_eq!(
                snapshot.transport,
                TransportState::Playing,
                "a skip that does not reach the end changed the transport (FR-018)"
            );
        }
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn a_forward_skip_that_reaches_the_end_lands_paused_at_the_duration(
        (duration, press) in end_reaching_press_strategy(),
    ) {
        // P2 (FR-018): a forward skip that reaches or passes
        // the end lands at exactly the duration in the
        // PAUSED end-state — never Completed (completion is
        // document-end only) and never still Playing.
        let mut session = PlaybackSession::with_duration(duration);
        playing_at_position(&mut session, press);
        let landed = session.execute(ShellVariant::skip_forward());
        prop_assert!(
            landed.is_ok(),
            "a forward skip reaching the end from {} was rejected",
            press
        );
        let snapshot = session.snapshot();
        prop_assert_eq!(
            snapshot.position_secs,
            duration,
            "a forward skip reaching the end from {} did not land at exactly the duration {}",
            press,
            duration
        );
        prop_assert_eq!(
            snapshot.transport,
            TransportState::Paused,
            "the end-state of a forward skip that reaches the end is Paused, not {:?}",
            snapshot.transport
        );
        prop_assert!(
            snapshot.transport != TransportState::Completed,
            "a skip reached the end but reported Completed — completion is document-end only (FR-018)"
        );
        prop_assert!(
            snapshot.transport != TransportState::Playing,
            "a skip that reached the end left the transport Playing (FR-018)"
        );
    }

    #[test]
    fn the_rate_ladder_is_closed_and_moves_exactly_one_rung(rung in rate_strategy()) {
        // P3 (FR-019): the ladder is closed at exactly seven
        // rungs; each press moves exactly one rung; a press
        // at either end saturates — never wraps, never
        // fails, never off-ladder. The one-step oracles
        // enumerate all seven rungs explicitly (no
        // catch-all), so the saturation cells
        // (`R4_0.step_up() == R4_0`, `R0_75.step_down() ==
        // R0_75`) are asserted as the ordinary end cells.
        let up = step_up(rung);
        let down = step_down(rung);
        prop_assert!(
            LADDER.contains(&up),
            "step_up({:?}) left the closed seven-rung ladder (FR-019)",
            rung
        );
        prop_assert!(
            LADDER.contains(&down),
            "step_down({:?}) left the closed seven-rung ladder (FR-019)",
            rung
        );
        prop_assert_eq!(
            up,
            rate_up_from(rung),
            "step_up({:?}) did not move exactly one rung up the ladder (FR-019)",
            rung
        );
        prop_assert_eq!(
            down,
            rate_down_from(rung),
            "step_down({:?}) did not move exactly one rung down the ladder (FR-019)",
            rung
        );
    }

    #[test]
    fn consecutive_rate_presses_move_one_rung_per_press_and_saturate(presses in 0usize..=30) {
        // P4 (FR-019): N consecutive presses from the
        // default rung (`R1_0`, FR-019b) land exactly
        // `min(R1_0 + N, R4_0)` rungs up — never skipping a
        // rung, never wrapping. The oracle folds the pure
        // one-step move, so a session that jumps a rung,
        // wraps, or fails a press diverges from it.
        let mut session = PlaybackSession::with_duration(3600.0);
        let mut expected = PlaybackRate::R1_0;
        for press in 1..=presses {
            let stepped = session.execute(ShellVerb::RateUp);
            prop_assert!(
                stepped.is_ok(),
                "RateUp press {} was rejected — a ladder press never fails (FR-019)",
                press
            );
            expected = step_up(expected);
        }
        prop_assert_eq!(
            session.snapshot().rate,
            expected,
            "{} consecutive RateUp presses did not land on min(R1_0 + N, R4_0) (FR-019)",
            presses
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn a_rate_change_is_effective_within_the_bound_and_never_moves_the_position(
        (rung, press) in (rate_strategy(), position_strategy()),
    ) {
        // P5 (FR-019a / FR-043): the rate change is
        // effective within the 2.0 s bound at every rung;
        // the position is bit-identical before and after a
        // rate verb (the stretch is the engine's
        // pitch-preserving contract op, never a local
        // stretch); the session's rate equals the expected
        // rung.
        let effective = rate_change_effective_time(rung);
        prop_assert!(
            effective <= RATE_EFFECTIVE_BOUND_SECS,
            "the rate change at {:?} took {} s, past the {} s effective bound (FR-019a)",
            rung,
            effective,
            RATE_EFFECTIVE_BOUND_SECS
        );
        let mut session = walked_to(rung);
        let _played = session.execute(ShellVerb::Play);
        session.on_audible_progress(press);
        let _paused = session.execute(ShellVerb::Pause);
        let before = session.snapshot();
        let changed = session.execute(ShellVerb::RateUp);
        prop_assert!(
            changed.is_ok(),
            "the RateUp press from {:?} was rejected (FR-019)",
            rung
        );
        let after = session.snapshot();
        prop_assert_eq!(
            after.position_secs,
            before.position_secs,
            "the position moved from {} to {} across a rate change — a rate change never moves the position (FR-043)",
            before.position_secs,
            after.position_secs
        );
        prop_assert_eq!(
            after.rate,
            step_up(rung),
            "the rate change from {:?} did not land on exactly one rung up (FR-019)",
            rung
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the grid positions, the 5 %-of-pause rewind and the recorded f64 values are exact quarter/centisecond arithmetic; == pins the exact-value contract and a tolerance would admit drift"
    )]
    #[test]
    fn a_rate_change_discards_the_pre_roll_so_it_is_re_prepared_at_the_new_rate(
        (rung, delivered) in (rate_strategy(), progress_strategy()),
    ) {
        // P6 (FR-043): the pre-roll is re-prepared at the
        // new rate — audio held at the old rung is
        // discarded, never played at a mismatched rate. The
        // session does not expose its pre-roll buffer, so
        // the property is asserted through the buffer type
        // directly (`player::buffer::PreRollBuffer`): the
        // rate verb's buffer-side effect is the
        // re-preparation at the new rung.
        let mut buffer = PreRollBuffer::new();
        buffer.on_play_pressed();
        let filled = buffer.push_delivered(delivered);
        prop_assert!(
            filled.is_ok(),
            "a {} s pre-roll delivery while preparing was rejected",
            delivered
        );
        prop_assert!(
            !buffer.is_empty(),
            "the pre-roll held no audio after a {} s delivery",
            delivered
        );
        let new_rung = step_up(rung);
        buffer.reprepare_for_rate(new_rung);
        prop_assert!(
            buffer.is_empty(),
            "the pre-roll was not discarded by the rate change — stale audio would play at a mismatched rate (FR-043)"
        );
        prop_assert_eq!(
            buffer.buffered_secs(),
            0.0,
            "the pre-roll still held {} s after the rate change (FR-043)",
            buffer.buffered_secs()
        );
        prop_assert_eq!(
            buffer.prepared_rate(),
            new_rung,
            "the pre-roll was not re-prepared at the new rung {:?} (FR-043)",
            new_rung
        );
    }
}

/// The closed ladder is exactly the seven declared rungs,
/// ascending (FR-019) — the closed set every ladder
/// operation saturates within.
#[test]
fn the_ladder_is_exactly_the_seven_declared_rungs_ascending() {
    assert_eq!(
        LADDER.len(),
        7,
        "the ladder did not hold exactly seven rungs (FR-019)"
    );
    assert_eq!(
        LADDER,
        [
            PlaybackRate::R0_75,
            PlaybackRate::R1_0,
            PlaybackRate::R1_25,
            PlaybackRate::R1_5,
            PlaybackRate::R1_75,
            PlaybackRate::R2_0,
            PlaybackRate::R4_0,
        ],
        "the ladder did not match the declared ascending rungs (FR-019)"
    );
}

/// The rate-change effective bound is exactly 2.0 s
/// (FR-019a), and every rung's effective time sits within
/// it.
#[expect(
    clippy::float_cmp,
    reason = "RATE_EFFECTIVE_BOUND_SECS (2.0) and every rung's effective time are exact binary fractions; == and <= are the correct assertions and a tolerance would admit a drifted bound"
)]
#[test]
fn the_rate_change_effective_bound_is_exactly_two_seconds() {
    assert_eq!(
        RATE_EFFECTIVE_BOUND_SECS, 2.0,
        "the rate-change effective bound drifted from 2.0 s (FR-019a)"
    );
    for rung in LADDER {
        assert!(
            rate_change_effective_time(rung) <= RATE_EFFECTIVE_BOUND_SECS,
            "the rate change at {rung:?} exceeded the effective bound (FR-019a)"
        );
    }
}

/// A `with_duration` session reports its known duration,
/// and a default session reports none — the FR-028c
/// determinacy input and the FR-018 clamp ceiling.
#[test]
fn a_with_duration_session_reports_its_known_duration() {
    let known = PlaybackSession::with_duration(3600.0);
    assert_eq!(
        known.duration_secs(),
        Some(3600.0),
        "a with_duration session did not report its known duration"
    );
    let unknown = PlaybackSession::new();
    assert_eq!(
        unknown.duration_secs(),
        None,
        "a default session reported a duration — the total is unknown until declared (FR-028c)"
    );
}

/// Skip forward lands exactly 5.0 s past the press time
/// (FR-018), leaving the transport `Playing`.
#[expect(
    clippy::float_cmp,
    reason = "the press time 100.0 and SKIP_STEP_SECS 5.0 are exact binary fractions; == is exact and a tolerance would admit an off-step skip"
)]
#[test]
fn skip_forward_lands_exactly_five_seconds_past_the_press_time() {
    let mut session = PlaybackSession::with_duration(3600.0);
    playing_at_position(&mut session, 100.0);
    let landed = session.execute(ShellVariant::skip_forward());
    assert!(landed.is_ok(), "a forward skip from 100.0 s was rejected");
    assert_eq!(
        session.snapshot().position_secs,
        105.0,
        "a forward skip from 100.0 s did not land at exactly 105.0 s (FR-018)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Playing,
        "a forward skip short of the end must leave the transport Playing (FR-018)"
    );
}

/// Skip backward lands exactly 5.0 s before the press
/// time (FR-018), never changing the transport.
#[expect(
    clippy::float_cmp,
    reason = "the press time 100.0 and SKIP_STEP_SECS 5.0 are exact binary fractions; == is exact and a tolerance would admit an off-step skip"
)]
#[test]
fn skip_backward_lands_exactly_five_seconds_before_the_press_time() {
    let mut session = PlaybackSession::with_duration(3600.0);
    playing_at_position(&mut session, 100.0);
    let landed = session.execute(ShellVariant::skip_back());
    assert!(landed.is_ok(), "a backward skip from 100.0 s was rejected");
    assert_eq!(
        session.snapshot().position_secs,
        95.0,
        "a backward skip from 100.0 s did not land at exactly 95.0 s (FR-018)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Playing,
        "a backward skip never changes the transport (FR-018)"
    );
}

/// The backward skip clamps at the 0.0 floor (FR-018).
#[expect(
    clippy::float_cmp,
    reason = "the press time 3.0, the 5.0 s step and the 0.0 floor are exact binary fractions; == is exact"
)]
#[test]
fn skip_backward_clamps_at_the_zero_floor() {
    let mut session = PlaybackSession::with_duration(3600.0);
    playing_at_position(&mut session, 3.0);
    let _landed = session.execute(ShellVariant::skip_back());
    assert_eq!(
        session.snapshot().position_secs,
        0.0,
        "a backward skip from 3.0 s did not clamp at the 0.0 floor (FR-018)"
    );
}

/// The forward skip clamps at the duration ceiling
/// (FR-018).
#[expect(
    clippy::float_cmp,
    reason = "the duration 100.0, the press time 98.0 and the 5.0 s step are exact binary fractions; == is exact"
)]
#[test]
fn skip_forward_clamps_at_the_duration_ceiling() {
    let mut session = PlaybackSession::with_duration(100.0);
    playing_at_position(&mut session, 98.0);
    let _landed = session.execute(ShellVariant::skip_forward());
    assert_eq!(
        session.snapshot().position_secs,
        100.0,
        "a forward skip from 98.0 s in a 100.0 s document did not clamp at the duration ceiling (FR-018)"
    );
}

/// A forward skip that reaches the end lands the paused
/// end-state at exactly the duration — never Completed,
/// never Playing (FR-018: completion is document-end
/// only).
#[expect(
    clippy::float_cmp,
    reason = "the duration 100.0 and the press time 97.0 are exact binary fractions; == is exact"
)]
#[test]
fn a_forward_skip_that_reaches_the_end_lands_the_paused_end_state() {
    let mut session = PlaybackSession::with_duration(100.0);
    playing_at_position(&mut session, 97.0);
    let landed = session.execute(ShellVariant::skip_forward());
    assert!(
        landed.is_ok(),
        "a forward skip reaching the end was rejected"
    );
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.position_secs, 100.0,
        "a forward skip reaching the end did not land at exactly the duration (FR-018)"
    );
    assert_eq!(
        snapshot.transport,
        TransportState::Paused,
        "the end-state of a forward skip that reaches the end is Paused (FR-018)"
    );
    assert!(
        snapshot.transport != TransportState::Completed,
        "a skip reported Completed — completion is document-end only (FR-018)"
    );
}

/// Both ladder ends saturate — a press at either end
/// stays at that end, never wrapping, never failing
/// (FR-019).
#[test]
fn the_ladder_saturates_at_both_ends() {
    assert_eq!(
        step_up(PlaybackRate::R4_0),
        PlaybackRate::R4_0,
        "step_up at the fastest rung must saturate at R4_0 (FR-019)"
    );
    assert_eq!(
        step_down(PlaybackRate::R0_75),
        PlaybackRate::R0_75,
        "step_down at the slowest rung must saturate at R0_75 (FR-019)"
    );
    let mut up = PlaybackSession::with_duration(3600.0);
    for _ in 0..10 {
        let _stepped = up.execute(ShellVerb::RateUp);
    }
    assert_eq!(
        up.snapshot().rate,
        PlaybackRate::R4_0,
        "ten RateUp presses must saturate at R4_0 (FR-019)"
    );
    let mut down = PlaybackSession::with_duration(3600.0);
    for _ in 0..10 {
        let _stepped = down.execute(ShellVerb::RateDown);
    }
    assert_eq!(
        down.snapshot().rate,
        PlaybackRate::R0_75,
        "ten RateDown presses must saturate at R0_75 (FR-019)"
    );
}

/// The 2.0x → 4.0x step: 4.0x is exactly one rung past
/// 2.0x, and the two-step walk 1.75x → 2.0x → 4.0x ends
/// at the fastest rung (FR-019).
#[test]
fn the_two_step_to_quadruple_ends_at_the_fastest_rung() {
    assert_eq!(
        step_up(PlaybackRate::R2_0),
        PlaybackRate::R4_0,
        "one step past 2.0x is 4.0x (FR-019)"
    );
    let mut session = PlaybackSession::with_duration(3600.0);
    for _ in 0..3 {
        let _stepped = session.execute(ShellVerb::RateUp);
    }
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R1_75,
        "three RateUp presses from the default must land on 1.75x (FR-019)"
    );
    for _ in 0..2 {
        let _stepped = session.execute(ShellVerb::RateUp);
    }
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R4_0,
        "the two-step walk 1.75x → 2.0x → 4.0x must end at 4.0x (FR-019)"
    );
}
