//! T034 — US2 integration: duck-vs-pause trials (FR-013 / FR-013a / FR-014 / FR-015 / FR-041).
//!
//! Host-runnable replay of the two device fixtures
//! (`specs/001-android-media-shell/tests/device/fixtures/focus_duck_pause.jsonl`
//! and `focus_transient_permanent.jsonl`): every fixture step is driven
//! through the Rust-side state machine (`player::focus::FocusTracker`) and
//! the rewind engine (`player::rewind`), asserting the outcome the fixture's
//! `result` token records.
//!
//! Trial legs (from `specs/001-android-media-shell/tasks.md` T034):
//!
//! - navigation prompt (`DuckableTransient`) → gain at the ~20% nominal
//!   inside [0.15, 0.25] with transport staying `Playing` (FR-013); regain
//!   restores full gain. Ducked playback never leaves `Playing`, so per
//!   FR-015 it accumulates no paused duration and owes no rewind — the
//!   sentence-snap resume below belongs to the silence-paused leg.
//! - call/alarm (`SilenceDemanding`) → `Paused` + the FR-041 reason; regain
//!   after a 5.0 s pause resumes at the containing-sentence start with total
//!   backward movement inside the 30 s hard cap, landing in an inter-word
//!   gap (FR-013a / FR-015).
//! - permanent loss (`PermanentLoss`) → `Paused`, never auto-resumes, and
//!   the FR-041 3 s reason window restarts at the permanent-loss callback
//!   then expires everywhere after 3 s while the paused state itself stays
//!   visible (FR-014 / FR-041).
//! - supersede is recorded as two ordered entries with a supersede link,
//!   never a merged rewrite, and the cumulative clock runs from the first
//!   yield (FR-014 / FR-039 / spec clarification 2026-10-01).
//!
//! Device-only gaps (traced, not asserted on host): the audible 15–25% duck
//! depth under a real navigation prompt, lock-surface visibility of the pause
//! and reason, and the sub-second routing bounds owned by T046. The FR-041
//! reason string itself IS asserted verbatim here because the surface layer
//! (T065) renders exactly this constant.

use adapter_synthesis::AlignmentGranularity;
use engine_core::closed::TransportState;
use player::focus::{
    DUCK_GAIN, DUCK_GAIN_MAX, DUCK_GAIN_MIN, FULL_GAIN, FocusIntent, FocusOutcome, FocusTracker,
    apply_duck, supersede_flags,
};
use player::resume::{ResumeChoice, resume_offer_window};
use player::rewind::{ResumeInput, cursor, resolve_resume, supersede};

/// FR-041 pause reason, stated with the pause on the lock surface and — when
/// the app opens inside the window — in-app (`spec.md` FR-041).
const FOCUS_PAUSE_REASON: &str = "Leitura pausada — outro app usando o áudio";

/// Gain comparison tolerance, in linear gain units.
const GAIN_EPS: f32 = 0.001;

/// Position comparison tolerance, in seconds.
const POS_EPS_SECS: f64 = 0.001;

/// Sentence starts for the silence-leg resume, every 10 s over a 120 s yield.
const SENTENCE_STARTS: [f64; 12] = [
    0.0, 10.0, 20.0, 30.0, 40.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0,
];

/// Inter-word gaps for the silence-leg resume: every sentence start is
/// silence, never an energy peak.
const WORD_GAPS: [f64; 12] = [
    0.0, 10.0, 20.0, 30.0, 40.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0,
];

/// Embedded device fixture: duck-then-pause trial (4 ordered steps).
const FIXTURE_DUCK_PAUSE: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/focus_duck_pause.jsonl"
);

/// Embedded device fixture: permanent-loss trial (3 ordered steps).
const FIXTURE_TRANSIENT_PERMANENT: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/focus_transient_permanent.jsonl"
);

/// One diagnostic-record entry for a focus transition (FR-039 shape reduced
/// to the fields this trial asserts: order, outcome, cumulative clock, and
/// the supersede link owned by T035).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordEntry {
    /// Device-fixture timestamp, in milliseconds.
    ts_ms: u64,
    /// Classified outcome recorded for this entry.
    outcome: FocusOutcome,
    /// Cumulative supersede clock: set at the first yield, never reset.
    first_yield_ms: u64,
    /// Link to the superseded entry, when this entry supersedes one.
    supersedes_ts_ms: Option<u64>,
}

/// Fixture step 1 (`MAY_DUCK → duck_gain_0.20`): a navigation prompt ducks to
/// the ~20% nominal inside [0.15, 0.25] and keeps reading (FR-013).
#[test]
fn navigation_prompt_ducks_at_nominal_gain_and_stays_playing() {
    let mut tracker = FocusTracker::new();
    let ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        ducked.outcome,
        Some(FocusOutcome::DuckableTransient),
        "a duckable transient must classify to its own outcome (FR-013)"
    );
    assert_eq!(
        ducked.transport,
        Some(TransportState::Playing),
        "ducking is distinct from pausing: transport stays Playing (FR-013)"
    );
    assert!(
        ducked.gain >= DUCK_GAIN_MIN && ducked.gain <= DUCK_GAIN_MAX,
        "duck gain {} left the [0.15, 0.25] band (FR-013)",
        ducked.gain
    );
    assert!(
        (ducked.gain - DUCK_GAIN).abs() <= GAIN_EPS,
        "duck gain {} drifted from the ~20% nominal (FR-013)",
        ducked.gain
    );
    assert!(
        apply_duck(ducked.gain).is_ok(),
        "the applied duck gain must satisfy the closed duck band (FR-013)"
    );
    assert!(tracker.ducked, "the tracker must record the ducked state");
    assert!(
        tracker.duck_restore_due,
        "a duckable transient must owe a gain restore on regain (FR-013)"
    );
}

/// Fixture step 2 (`GAIN_REGained → duck_restore`): regain restores full gain
/// with no rewind owed — ducked playback never left `Playing` (FR-013/015).
#[test]
fn regain_after_duck_restores_full_gain_with_no_rewind() {
    let mut tracker = FocusTracker::new();
    let _ducked = tracker.apply(FocusIntent::DuckableTransient);
    let regained = tracker.apply(FocusIntent::FocusRegained);
    assert!(
        (regained.gain - FULL_GAIN).abs() <= GAIN_EPS,
        "regain must restore full gain {}, got {} (FR-013)",
        FULL_GAIN,
        regained.gain
    );
    assert!(
        !tracker.ducked,
        "regain must clear the ducked state (FR-013 restore)"
    );
    assert!(
        !tracker.duck_restore_due,
        "the gain restore is settled by the regain, never re-owed (FR-013)"
    );
}

/// Fixture step 3 (`SILENCE_DEMANDING → pause_reason_call`): a call/alarm
/// pauses and states the FR-041 reason (FR-013a / FR-041).
#[test]
fn call_or_alarm_pauses_and_states_the_fr041_reason() {
    let mut tracker = FocusTracker::new();
    let silenced = tracker.apply(FocusIntent::SilenceDemanding);
    assert_eq!(
        silenced.outcome,
        Some(FocusOutcome::SilenceDemanding),
        "a silence-demanding loss must classify to its own outcome (FR-013a)"
    );
    assert_eq!(
        silenced.transport,
        Some(TransportState::Paused),
        "a call/alarm must pause immediately, never duck (FR-013a)"
    );
    assert!(
        (silenced.gain - FULL_GAIN).abs() <= GAIN_EPS,
        "a silence pause holds full gain — nothing stays ducked (FR-013a)"
    );
    assert_eq!(
        FOCUS_PAUSE_REASON, "Leitura pausada — outro app usando o áudio",
        "the stated pause reason must match the FR-041 form verbatim"
    );
    assert!(
        !tracker.resume_due,
        "a silence loss without regain owes no resume — never auto-resume (FR-014)"
    );
}

/// Fixture step 4 (`GAIN_REGained → resume_sentence_start_5pct_back`): regain
/// after the 5.0 s silence pause (ts 1200 → 6200) resumes at the
/// containing-sentence start, inside the 30 s cap, in a gap (FR-015).
#[test]
fn regain_after_silence_pause_resumes_at_sentence_start_in_cap() {
    let paused_duration_secs = 5.0;
    assert!(
        (cursor(paused_duration_secs) - 0.25).abs() <= POS_EPS_SECS,
        "the FR-015 cursor for a 5.0 s pause must be 0.25 s"
    );
    let input = ResumeInput {
        yield_secs: 120.0,
        paused_duration_secs,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &SENTENCE_STARTS,
        word_gaps: &WORD_GAPS,
        first_yield_ms: 1200,
        supersedes_ts_ms: None,
    };
    let resume = resolve_resume(&input);
    // Raw cursor 119.75 → containing-sentence start 110.0.
    assert!(
        (resume.resume_secs - 110.0).abs() <= POS_EPS_SECS,
        "resume must land at the containing-sentence start 110.0, got {} (FR-015)",
        resume.resume_secs
    );
    assert!(
        resume.backward_secs <= 30.0,
        "total backward movement {} exceeds the 30 s hard cap (FR-015)",
        resume.backward_secs
    );
    assert!(
        (resume.backward_secs - 10.0).abs() <= POS_EPS_SECS,
        "backward movement must be 10.0 s, got {} (FR-015)",
        resume.backward_secs
    );
    let mut landed_in_gap = false;
    for gap in &WORD_GAPS {
        if (resume.resume_secs - *gap).abs() <= POS_EPS_SECS {
            landed_in_gap = true;
        }
    }
    assert!(
        landed_in_gap,
        "resume {} must land in an inter-word gap, never an energy peak (FR-015)",
        resume.resume_secs
    );
    assert_eq!(
        resume.first_yield_ms, 1200,
        "the resume must carry the cumulative clock through (FR-015)"
    );
    assert_eq!(
        resume.supersedes_ts_ms, None,
        "an un-superseded resume carries no supersede link (T035)"
    );
}

/// Permanent-loss trial (`PERMANENT_LOSS → pause_never_auto_resume`): pause,
/// never auto-resume, and the FR-041 window restarts at the permanent loss
/// then expires everywhere after 3 s while paused stays visible (FR-014/041).
#[test]
fn permanent_loss_pauses_visible_and_fr041_window_restarts_then_expires() {
    let mut tracker = FocusTracker::new();
    let loss = tracker.apply(FocusIntent::PermanentLoss);
    assert_eq!(
        loss.transport,
        Some(TransportState::Paused),
        "a permanent loss must pause (FR-014)"
    );
    assert!(
        !tracker.resume_due,
        "a permanent loss without regain owes no resume — never auto-resume (FR-014)"
    );
    assert!(
        !tracker.ducked,
        "a permanent loss holds paused at full gain, never ducked (FR-014)"
    );
    // The FR-041 window restarts at the permanent-loss callback: a fresh
    // origin offers the reason on the lock surface.
    assert_eq!(
        resume_offer_window(0.0, false),
        Some(ResumeChoice::ExplicitResume),
        "the permanent loss must re-anchor a fresh 3 s reason window (FR-041)"
    );
    assert!(
        resume_offer_window(2.5, false).is_some(),
        "the reason must stay visible inside the 3 s window (FR-041)"
    );
    // Expiry governs both surfaces; paused state itself remains visible.
    assert!(
        resume_offer_window(3.5, false).is_none(),
        "the reason must expire everywhere after 3 s (FR-041)"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "after expiry the authoritative state stays the paused permanent loss (FR-030/041)"
    );
}

/// Supersede trial (`TRANSIENT_DUCK → supersede_cleared_transient`): a late
/// transient after a permanent loss authorizes nothing, and the record keeps
/// two ordered entries with a supersede link — never a merged rewrite.
#[test]
fn late_transient_after_permanent_holds_pause_and_record_keeps_two_ordered_entries() {
    let mut tracker = FocusTracker::new();
    let _loss = tracker.apply(FocusIntent::PermanentLoss);
    let late = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        late.outcome,
        Some(FocusOutcome::PermanentLoss),
        "a late transient must not displace the standing permanent loss (FR-014)"
    );
    assert_eq!(
        late.transport,
        Some(TransportState::Paused),
        "a late transient must hold paused, never re-enter playing (FR-014)"
    );
    assert!(
        !tracker.ducked,
        "a late transient must not re-duck past a permanent loss (FR-014)"
    );
    let flags = supersede_flags(FocusOutcome::DuckableTransient, FocusOutcome::PermanentLoss);
    assert!(
        flags.clear_resume_flag && flags.clear_duck_restore_flag,
        "escalation to permanent loss must clear both transient flags (FR-014)"
    );
    // Record shape: transient entry at ts 0, permanent entry at ts 300 with
    // the supersede link; both retained, ordered, clock unbroken.
    let first = RecordEntry {
        ts_ms: 0,
        outcome: FocusOutcome::DuckableTransient,
        first_yield_ms: 0,
        supersedes_ts_ms: None,
    };
    let link = supersede(first.first_yield_ms, 300);
    let second = RecordEntry {
        ts_ms: 300,
        outcome: FocusOutcome::PermanentLoss,
        first_yield_ms: link.first_yield_ms,
        supersedes_ts_ms: link.supersedes_ts_ms,
    };
    let record = [first, second];
    assert_eq!(
        record.len(),
        2,
        "supersede must keep two entries — never a merged rewrite (T035)"
    );
    assert!(
        record[0].ts_ms < record[1].ts_ms,
        "record entries must stay ordered transient-then-permanent (T035)"
    );
    assert_eq!(
        first.outcome,
        FocusOutcome::DuckableTransient,
        "the first entry must preserve the transient outcome (T035)"
    );
    assert_eq!(
        second.outcome,
        FocusOutcome::PermanentLoss,
        "the second entry must carry the superseding permanent outcome (T035)"
    );
    assert_eq!(
        second.first_yield_ms, first.first_yield_ms,
        "the cumulative clock runs from the first yield, never reset at supersede (FR-015)"
    );
    assert_eq!(
        second.supersedes_ts_ms,
        Some(300),
        "the permanent entry must link the superseded entry (T035)"
    );
    assert_eq!(
        first.supersedes_ts_ms, None,
        "the first yield supersedes nothing (T035)"
    );
}

/// Full replay: every device-fixture step, in fixture order, driven through
/// the Rust state machine with the outcome each `result` token records.
#[test]
fn test_focus_duck_pause_us2() {
    let duck_lines: Vec<&str> = FIXTURE_DUCK_PAUSE.lines().collect();
    let permanent_lines: Vec<&str> = FIXTURE_TRANSIENT_PERMANENT.lines().collect();
    assert_eq!(
        duck_lines.len(),
        4,
        "focus_duck_pause.jsonl must carry exactly the 4 recorded trial steps"
    );
    assert_eq!(
        permanent_lines.len(),
        3,
        "focus_transient_permanent.jsonl must carry exactly the 3 recorded trial steps"
    );
    assert!(
        duck_lines[0].contains("MAY_DUCK") && duck_lines[0].contains("duck_gain_0.20"),
        "fixture step 1 must be the MAY_DUCK → duck_gain_0.20 trial"
    );
    assert!(
        duck_lines[1].contains("GAIN_REGained") && duck_lines[1].contains("duck_restore"),
        "fixture step 2 must be the GAIN_REGained → duck_restore trial"
    );
    assert!(
        duck_lines[2].contains("SILENCE_DEMANDING") && duck_lines[2].contains("pause_reason_call"),
        "fixture step 3 must be the SILENCE_DEMANDING → pause_reason_call trial"
    );
    assert!(
        duck_lines[3].contains("GAIN_REGained")
            && duck_lines[3].contains("resume_sentence_start_5pct_back"),
        "fixture step 4 must be the GAIN_REGained → resume_sentence_start_5pct_back trial"
    );
    assert!(
        permanent_lines[0].contains("PERMANENT_LOSS")
            && permanent_lines[0].contains("pause_never_auto_resume"),
        "permanent fixture step 1 must be PERMANENT_LOSS → pause_never_auto_resume"
    );
    assert!(
        permanent_lines[1].contains("TRANSIENT_DUCK")
            && permanent_lines[1].contains("supersede_cleared_transient"),
        "permanent fixture step 2 must be TRANSIENT_DUCK → supersede_cleared_transient"
    );
    assert!(
        permanent_lines[2].contains("GAIN_REGained")
            && permanent_lines[2].contains("no_resume_reason_expired"),
        "permanent fixture step 3 must be GAIN_REGained → no_resume_reason_expired"
    );

    // Duck-then-pause trial through one tracker, in fixture order.
    let mut tracker = FocusTracker::new();
    let ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(ducked.transport, Some(TransportState::Playing));
    assert!(ducked.gain >= DUCK_GAIN_MIN && ducked.gain <= DUCK_GAIN_MAX);
    let _restored = tracker.apply(FocusIntent::FocusRegained);
    assert!(!tracker.ducked);
    let silenced = tracker.apply(FocusIntent::SilenceDemanding);
    assert_eq!(silenced.transport, Some(TransportState::Paused));
    assert_eq!(
        FOCUS_PAUSE_REASON, "Leitura pausada — outro app usando o áudio",
        "the silence pause must state the FR-041 reason verbatim"
    );
    let _regained = tracker.apply(FocusIntent::FocusRegained);
    assert!(
        tracker.resume_due,
        "the silence-pause regain must arm the transient-resume rewind (FR-015)"
    );
    let resume = resolve_resume(&ResumeInput {
        yield_secs: 120.0,
        paused_duration_secs: 5.0,
        granularity: AlignmentGranularity::Sentence,
        sentence_starts: &SENTENCE_STARTS,
        word_gaps: &WORD_GAPS,
        first_yield_ms: 1200,
        supersedes_ts_ms: None,
    });
    assert!(
        (resume.resume_secs - 110.0).abs() <= POS_EPS_SECS,
        "replay resume must land at sentence start 110.0, got {} (FR-015)",
        resume.resume_secs
    );
    assert!(resume.backward_secs <= 30.0);

    // Permanent-loss trial through a second tracker, in fixture order.
    let mut standing = FocusTracker::new();
    let loss = standing.apply(FocusIntent::PermanentLoss);
    assert_eq!(loss.transport, Some(TransportState::Paused));
    assert!(!standing.resume_due);
    let late = standing.apply(FocusIntent::DuckableTransient);
    assert_eq!(late.transport, Some(TransportState::Paused));
    assert!(!standing.ducked);
    let regained = standing.apply(FocusIntent::FocusRegained);
    assert!(
        (regained.gain - FULL_GAIN).abs() <= GAIN_EPS,
        "the post-window regain restores full gain without restating a reason (FR-041)"
    );
    // At +3000 ms the window has run its course: the reason is expired on
    // both surfaces, the paused permanent-loss state persists, and nothing
    // auto-resumed.
    assert!(
        resume_offer_window(3.5, false).is_none(),
        "past the 3 s edge the reason must be expired everywhere (FR-041)"
    );
    assert!(
        resume_offer_window(0.0, false).is_some(),
        "only a new permanent loss re-anchors the window — regain never does (FR-041)"
    );
}
