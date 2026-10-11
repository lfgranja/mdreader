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
use player::cli::{PlaybackSession, ShellVariant, ShellVerb};
use player::position::{PositionClock, REWIND_CAP_SECS, REWIND_FRACTION, SKIP_STEP_SECS};

/// Positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Sentence boundaries on the same grid: sorted, deduplicated, and
/// always containing the document start `0.0` (so a greatest-boundary-
/// at-or-before lookup always has an answer).
fn boundaries_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.push(0);
        quarters.sort();
        quarters.dedup();
        quarters.into_iter().map(|q| q as f64 * 0.25).collect()
    })
}

/// Paused durations `d` ∈ [0, 7200] s as exact centisecond multiples.
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=720_000).prop_map(|centis| centis as f64 / 100.0)
}

/// Audible progress in `(0.00, 5.00]` s — strictly positive, so a
/// progress tick while `Playing` is a strictly greater position.
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
fn is_ender(state: TransportState) -> bool {
    match state {
        TransportState::Playing => false,
        TransportState::Paused | TransportState::Stopped => true,
        TransportState::Completed | TransportState::Interrupted => true,
    }
}

/// The session has ended: no transport control can move it except a
/// Stop press, which is idempotent.
fn is_terminal(state: TransportState) -> bool {
    matches!(state, TransportState::Stopped | TransportState::Completed)
}

/// The SC-005 scan: every `transport:"playing"` entry must be
/// followed, in the same session, by a later entry carrying a
/// strictly greater `position_secs`, except where an ender entry
/// intervenes first (the advance legitimately ended) or where a
/// skip-backward repositioning lands first (FR-018: the user moved
/// the cursor back on purpose, ending the prior window and opening
/// a new one at the earlier point). The final entry's window is open — a hard kill cuts the record mid-session
/// with no session_end entry (FR-004, SC-007) — so only entries
/// with a later entry can violate. Returns the indices of the
/// violating playing entries; an empty result is the conforming
/// report.
#[expect(
    clippy::float_cmp,
    reason = "the scan compares recorded position_secs values for strict order (>, the SC-005 advance check); == and ordering on f64 record values are exact comparisons of the represented values, and a tolerance would admit a no-advance playing window as conforming, defeating the rule (FR-005)"
)]
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
/// (SC-005: "every transport change therefore carries position_secs")
/// and on every audible-progress event. With `hard_kill` the record
/// ends mid-session exactly as a hard kill leaves it (no session_end
/// entry, FR-004); otherwise the session is closed with a Stop press
/// so every open playing window ends in an ender entry, as a real
/// session's record does.
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
        let already_ended = match last {
            None => true,
            Some(entry) => is_terminal(entry.transport),
        };
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
fn rate_up_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R0_75 => PlaybackRate::R1_0,
        PlaybackRate::R1_0 => PlaybackRate::R1_25,
        PlaybackRate::R1_25 => PlaybackRate::R1_5,
        PlaybackRate::R1_5 => PlaybackRate::R1_75,
        PlaybackRate::R1_75 => PlaybackRate::R2_0,
        PlaybackRate::R2_0 => PlaybackRate::R4_0,
        PlaybackRate::R4_0 => PlaybackRate::R4_0,
    }
}

/// The one-step-down neighbor on the closed ladder, saturating at
/// `R0_75` (FR-019).
fn rate_down_from(rate: PlaybackRate) -> PlaybackRate {
    match rate {
        PlaybackRate::R0_75 => PlaybackRate::R0_75,
        PlaybackRate::R1_0 => PlaybackRate::R0_75,
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
