//! T033 — US2 property: focus classification + duck band
//! (FR-013 / FR-013a / FR-013b / FR-014).
//!
//! Invariants under test, quoted verbatim from
//! `specs/001-android-media-shell/tasks.md` T033:
//!
//! - `DuckableTransient` → gain factor ∈ [0.15, 0.25] AND still
//!   `Playing`;
//! - `SilenceDemanding | PermanentLoss` → `Paused / Interrupted`, gain
//!   restored, never auto-resume;
//! - supersede transient→permanent clears resume/duck-restore flags, and
//!   a late transient gain authorizes nothing;
//! - deferred grant → zero audio before grant, no rewind, no FR-041
//!   reason;
//! - refusal → named failure, no playing state.
//!
//! Spec anchors: FR-013 (`specs/001-android-media-shell/spec.md` —
//! duck to 15–25% of normal gain, restore on return, ducking distinct
//! from pausing); FR-013a (classify by the interrupter's request:
//! duckable prompts duck, silence-demanding calls/alarms pause with
//! permanent-loss consequences); FR-013b (deferred holds zero audio
//! with no rewind and no reason; refused is a named failure under
//! FR-037, never playing); FR-014 (permanent loss pauses, never
//! auto-resumes; latest-event-wins clears transient flags; a late gain
//! from a superseded transient authorizes nothing); `DUCK_GAIN_MIN` /
//! `DUCK_GAIN_MAX` / `FULL_GAIN` (`crates/player/src/focus.rs`).
//!
//! Note on `Paused / Interrupted`: the focus path yields `Paused` for
//! both silence-demanding and permanent loss (`classify` maps both
//! there); `Interrupted` is the closed-set sibling produced by other
//! transport paths, never by focus classification — so the suite
//! asserts `Paused` exactly.
//!
//! Note on the FR-041 reason: reason strings are owned by the surface
//! layer (`src-tauri/src/surface.rs`, T065), not by the classifier — so
//! "no FR-041 reason" is asserted here as "the deferred path changes no
//! tracker state and states no pause" (gain untouched, no resume owed,
//! still awaiting the grant).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest 1.11 builds the message through
//! `concat!`, and rustc (1.98) rejects implicit captures in
//! macro-expanded format strings ("`format_args!` cannot capture
//! variables when the format string is expanded from a macro").

use proptest::prelude::*;

use engine_core::closed::{EngineError, TransportState};
use player::focus::{
    DUCK_GAIN, DUCK_GAIN_MAX, DUCK_GAIN_MIN, FULL_GAIN, FocusGrant, FocusIntent, FocusOutcome,
    FocusTracker, apply_duck, apply_grant, classify, classify_focus, supersede_flags,
};

/// The silence-side loss intents: calls/alarms demanding silence and
/// permanent losses (FR-013a / FR-014).
fn loss_intent_strategy() -> impl Strategy<Value = FocusIntent> {
    prop_oneof![
        Just(FocusIntent::SilenceDemanding),
        Just(FocusIntent::PermanentLoss),
    ]
}

/// The closed three-member focus outcome set (FR-044: no `Other`).
fn outcome_strategy() -> impl Strategy<Value = FocusOutcome> {
    prop_oneof![
        Just(FocusOutcome::DuckableTransient),
        Just(FocusOutcome::SilenceDemanding),
        Just(FocusOutcome::PermanentLoss),
    ]
}

/// The closed three-member focus-grant answer set (FR-013b).
fn grant_strategy() -> impl Strategy<Value = FocusGrant> {
    prop_oneof![
        Just(FocusGrant::Granted),
        Just(FocusGrant::Deferred),
        Just(FocusGrant::Refused),
    ]
}

/// The outcome a silence-side intent must classify to.
fn expected_outcome(intent: FocusIntent) -> FocusOutcome {
    if intent == FocusIntent::SilenceDemanding {
        FocusOutcome::SilenceDemanding
    } else {
        FocusOutcome::PermanentLoss
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn given_a_duck_factor_when_applying_then_the_closed_band_decides(
        factor in -1.0f32..=2.0f32
    ) {
        // FR-013: the app-applied gain factor lives in [0.15, 0.25] —
        // anything outside is a named failure, never a silent clamp.
        let expected = if factor < DUCK_GAIN_MIN || factor > DUCK_GAIN_MAX {
            Err(EngineError::RateMismatch)
        } else {
            Ok(factor)
        };
        prop_assert_eq!(
            apply_duck(factor),
            expected,
            "duck factor {} did not resolve against the closed [0.15, 0.25] band",
            factor
        );
    }

    #[test]
    fn given_a_silence_side_loss_when_classifying_then_playback_pauses_with_gain_restored_and_never_auto_resumes(
        intents in proptest::collection::vec(loss_intent_strategy(), 0..=8)
    ) {
        // FR-013a / FR-014: `SilenceDemanding | PermanentLoss` →
        // `Paused`, gain restored to full, and no sequence of losses
        // without a regain ever authorizes resume on its own.
        let mut tracker = FocusTracker::new();
        for intent in &intents {
            let decision = tracker.apply(*intent);
            prop_assert_eq!(
                decision.outcome,
                Some(expected_outcome(*intent)),
                "loss intent {:?} did not classify to its own outcome",
                intent
            );
            prop_assert_eq!(
                decision.transport,
                Some(TransportState::Paused),
                "loss intent {:?} did not pause",
                intent
            );
            prop_assert_eq!(
                decision.gain,
                FULL_GAIN,
                "loss intent {:?} did not restore full gain",
                intent
            );
        }
        prop_assert!(
            !tracker.resume_due,
            "a loss sequence without regain owed resume — playback would auto-resume"
        );
        prop_assert!(
            !tracker.ducked,
            "a loss sequence without regain left playback ducked instead of paused at full gain"
        );
    }

    #[test]
    fn given_any_outcome_transition_when_superseding_then_exactly_the_escalations_to_permanent_loss_clear_flags(
        (previous, next) in (outcome_strategy(), outcome_strategy())
    ) {
        // FR-014 latest-event-wins over the closed 3×3 product:
        // escalation into `PermanentLoss` clears both the resume flag
        // and the duck-restore flag; every other pair — including a
        // late transient after a permanent loss — clears nothing.
        let flags = supersede_flags(previous, next);
        let escalates = next == FocusOutcome::PermanentLoss && previous != FocusOutcome::PermanentLoss;
        prop_assert_eq!(
            flags.clear_resume_flag,
            escalates,
            "transition {:?} → {:?} did not resolve the resume flag by latest-event-wins",
            previous, next
        );
        prop_assert_eq!(
            flags.clear_duck_restore_flag,
            escalates,
            "transition {:?} → {:?} did not resolve the duck-restore flag by latest-event-wins",
            previous, next
        );
    }

    #[test]
    fn given_a_grant_answer_when_applying_then_each_arm_resolves_explicitly(
        grant in grant_strategy()
    ) {
        // FR-013b: `Granted` permits audio; `Deferred` holds zero audio
        // before the grant with no rewind owed and no tracker change
        // (hence no FR-041 reason — there is no pause to state);
        // `Refused` is the named `StartRefused` failure, never a
        // playing state.
        let mut tracker = FocusTracker::new();
        match grant {
            FocusGrant::Granted => {
                prop_assert_eq!(
                    apply_grant(grant),
                    Ok(true),
                    "a granted focus did not permit audio"
                );
            }
            FocusGrant::Deferred => {
                prop_assert_eq!(
                    apply_grant(grant),
                    Ok(false),
                    "a deferred grant produced audio before the grant arrived"
                );
                prop_assert!(
                    !tracker.resume_due,
                    "a deferred grant owed a rewind — a deferral is a delay, not a loss"
                );
                let _held = tracker.apply(FocusIntent::AppBackgrounded);
                prop_assert!(
                    !tracker.resume_due,
                    "a deferred grant owed a rewind while backgrounded"
                );
            }
            FocusGrant::Refused => {
                prop_assert_eq!(
                    apply_grant(grant),
                    Err(EngineError::StartRefused),
                    "a refused focus did not surface the named StartRefused failure"
                );
            }
        }
    }

    #[test]
    fn given_an_unknown_focus_name_when_classifying_then_it_fails_named_never_silent(
        other in any::<String>().prop_filter("not a focus outcome name", |name| {
            name != "DuckableTransient"
                && name != "SilenceDemanding"
                && name != "PermanentLoss"
        })
    ) {
        // FR-044 closed set: only the three outcome names classify; any
        // other string is the named `DeviceRefused` failure — never a
        // silent absorb, never a duck.
        prop_assert_eq!(
            classify_focus(other.as_str()),
            Err(EngineError::DeviceRefused),
            "unknown focus name {:?} did not fail named",
            other
        );
    }
}

/// `DuckableTransient` → gain ∈ [0.15, 0.25] AND still `Playing`
/// (FR-013): ducking is distinct from pausing — the document keeps
/// reading under a navigation prompt.
#[test]
fn duckable_transient_ducks_in_band_and_stays_playing() {
    let decision = classify(FocusIntent::DuckableTransient);
    assert_eq!(
        decision.outcome,
        Some(FocusOutcome::DuckableTransient),
        "a duckable transient must classify to its own outcome"
    );
    assert_eq!(
        decision.transport,
        Some(TransportState::Playing),
        "a duckable transient must stay Playing, never pause"
    );
    assert!(
        decision.gain >= DUCK_GAIN_MIN && decision.gain <= DUCK_GAIN_MAX,
        "duck gain {} left the [0.15, 0.25] band",
        decision.gain
    );
    let mut tracker = FocusTracker::new();
    let _applied = tracker.apply(FocusIntent::DuckableTransient);
    assert!(tracker.ducked, "the tracker must record the ducked state");
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::DuckableTransient),
        "the tracker must hold the transient outcome"
    );
}

/// The three outcome names classify to their own outcomes; the duck
/// nominal sits inside the band (FR-013 / FR-044).
#[test]
fn focus_names_classify_to_their_own_outcomes() {
    let _duckable = classify_focus("DuckableTransient");
    assert_eq!(
        classify_focus("DuckableTransient"),
        Ok(FocusOutcome::DuckableTransient),
        "the duckable name must classify to its outcome"
    );
    assert_eq!(
        classify_focus("SilenceDemanding"),
        Ok(FocusOutcome::SilenceDemanding),
        "the silence-demanding name must classify to its outcome"
    );
    assert_eq!(
        classify_focus("PermanentLoss"),
        Ok(FocusOutcome::PermanentLoss),
        "the permanent-loss name must classify to its outcome"
    );
    assert!(
        DUCK_GAIN >= DUCK_GAIN_MIN && DUCK_GAIN <= DUCK_GAIN_MAX,
        "the nominal duck gain {} left the [0.15, 0.25] band",
        DUCK_GAIN
    );
}

/// The gain constants match the FR-013 figure verbatim: the 15–25%
/// band and full-gain restore.
#[expect(
    clippy::float_cmp,
    reason = "DUCK_GAIN_MIN/MAX, DUCK_GAIN and FULL_GAIN are exact decimal literals from focus.rs; == is the correct assertion and a tolerance would admit a drifted band"
)]
#[test]
fn duck_band_constants_match_fr013_figure_verbatim() {
    assert_eq!(DUCK_GAIN_MIN, 0.15, "the duck band floor drifted from 0.15");
    assert_eq!(
        DUCK_GAIN_MAX, 0.25,
        "the duck band ceiling drifted from 0.25"
    );
    assert_eq!(DUCK_GAIN, 0.20, "the nominal duck gain drifted from ~20%");
    assert_eq!(FULL_GAIN, 1.0, "full gain drifted from 1.0");
}

/// Supersede transient→permanent clears the resume and duck-restore
/// flags and requires an explicit Play press (FR-014): a transient
/// resume owed while the transient stood dies with the supersede.
#[test]
fn transient_then_permanent_clears_resume_and_duck_restore() {
    let mut tracker = FocusTracker::new();
    let _regained = tracker.apply(FocusIntent::FocusRegained);
    let _ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert!(tracker.resume_due, "a regain must owe a transient resume");
    assert!(
        tracker.duck_restore_due,
        "a duckable transient must owe a gain restore"
    );
    let decision = tracker.apply(FocusIntent::PermanentLoss);
    assert_eq!(
        decision.transport,
        Some(TransportState::Paused),
        "a superseding permanent loss must pause"
    );
    assert!(
        !tracker.resume_due,
        "supersede must clear the transient resume flag — no auto-resume"
    );
    assert!(
        !tracker.duck_restore_due,
        "supersede must clear the duck-restore flag — gain is restored, not re-ducked"
    );
    assert!(
        !tracker.ducked,
        "supersede must un-duck: the paused state holds at full gain"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "the authoritative state after supersede is the permanent loss"
    );
}

/// A late transient arriving after a permanent loss authorizes nothing
/// (FR-014): the paused state holds at full gain, still owed no resume
/// by the superseded transient.
#[test]
fn late_transient_after_permanent_loss_authorizes_nothing() {
    let mut tracker = FocusTracker::new();
    let _loss = tracker.apply(FocusIntent::PermanentLoss);
    let late = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        late.outcome,
        Some(FocusOutcome::PermanentLoss),
        "a late transient must not displace the standing permanent loss"
    );
    assert_eq!(
        late.transport,
        Some(TransportState::Paused),
        "a late transient must hold paused, never re-enter playing"
    );
    assert!(
        !tracker.ducked,
        "a late transient must not re-duck past a permanent loss"
    );
    assert!(
        !tracker.resume_due,
        "a late transient must not authorize resume"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "the authoritative state stays the permanent loss"
    );
}

/// Escalation duck→silence clears the duck-restore flag (FR-014
/// latest-event-wins, T094): the silence pause holds at full gain,
/// never re-ducks, never compounds a transient resume.
#[test]
fn duck_then_silence_clears_duck_restore_and_holds_paused_at_full_gain() {
    let flags = supersede_flags(
        FocusOutcome::DuckableTransient,
        FocusOutcome::SilenceDemanding,
    );
    // Declared invariant (closed-set property test above, T033):
    // `supersede_flags` clears the resume and duck-restore flags
    // exactly on escalations into `PermanentLoss`; a
    // duck→silence transition is not such an escalation, so
    // the pure supersession function clears nothing here. The
    // stale duck-restore flag IS cleared at tracker level —
    // `FocusTracker::apply` clears it when `SilenceDemanding`
    // supersedes a transient (FR-014 latest-event-wins,
    // T094/T101) — asserted by the tracker-level checks below.
    assert!(
        !flags.clear_duck_restore_flag,
        "duck→silence is not an escalation into PermanentLoss: supersede_flags clears nothing (T033 closed-set invariant)"
    );
    assert!(
        !flags.clear_resume_flag,
        "duck→silence is not an escalation into PermanentLoss: supersede_flags clears nothing (T033 closed-set invariant)"
    );
    let mut tracker = FocusTracker::new();
    let _regained = tracker.apply(FocusIntent::FocusRegained);
    let _ducked = tracker.apply(FocusIntent::DuckableTransient);
    assert!(
        tracker.duck_restore_due,
        "a duckable transient must owe a gain restore before escalation"
    );
    let decision = tracker.apply(FocusIntent::SilenceDemanding);
    assert_eq!(
        decision.transport,
        Some(TransportState::Paused),
        "escalation duck→silence must pause"
    );
    assert_eq!(
        decision.gain, FULL_GAIN,
        "escalation duck→silence must hold full gain"
    );
    assert!(
        !tracker.duck_restore_due,
        "escalation duck→silence must clear the duck-restore flag — gain is restored, not re-ducked"
    );
    assert!(
        !tracker.resume_due,
        "a regain-owed resume must NOT survive a pause-demanding outcome: FR-005 (004 data-model.md interruption table: every pause outcome — silence-demanding OR permanent — is 'pause immediately, never auto-resume') requires explicit Play. The 001-era reading ('only an escalation into PermanentLoss clears it', T033) is SUPERSEDED by the 004 FR-005 contract, whose prop_focus8 property (test written first, RED against this behavior) pins the clear on any non-duck outcome — see crates/player/src/focus.rs FocusTracker::apply and the commit that closed it"
    );
    assert!(
        !tracker.ducked,
        "escalation duck→silence must un-duck: the paused state holds at full gain"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::SilenceDemanding),
        "the authoritative state after escalation is the silence-demanding loss"
    );
}
