//! T035 — US2 contract: focus-intent boundary (`contracts/shell-ipc.md`).
//!
//! Invariants under test, quoted from
//! `specs/001-android-media-shell/tasks.md` T035:
//!
//! - the 7 pre-existing intents each map to exactly one FR-013, FR-013a,
//!   FR-014, FR-015, FR-016, FR-017a outcome;
//! - the 2 lifecycle intents carry NO playback decision, latch only, and a
//!   foregrounded 3 s offer is never converted early (FR-032c's visibility
//!   MUST intact);
//! - supersede recorded as two ordered entries with supersede link, never
//!   merged rewrite.
//!
//! Spec anchors: FR-013 (duckable transient ducks 15–25% and stays
//! `Playing`); FR-013a (silence-demanding call/alarm pauses with
//! permanent-loss consequences); FR-014 (permanent loss pauses, never
//! auto-resumes, latest-event-wins); FR-015 (a regain authorizes a
//! rewind-governed resume — the math lives in `crate::rewind`, so the
//! boundary half asserted here is "regain owes a resume and changes no
//! gain"); FR-016 (noisy/disappearance pauses); FR-017a (appearance
//! follows at unchanged position); FR-032c / FR-023a (lifecycle latches
//! read at offer expiry only); FR-039 / FR-044 (two ordered append-only
//! entries with a supersede link).
//!
//! Notes:
//!
//! - `classify` maps the two routing-pause intents onto the two pause-side
//!   outcomes (`BecomingNoisy → SilenceDemanding`, `DeviceDisappeared →
//!   PermanentLoss`); both pause at full gain, so both satisfy FR-016 —
//!   the outcome split keeps the silence/permanent distinction the
//!   tracker and the diagnostic record already carry.
//! - "Follow at unchanged position" (FR-017a) and the ≤1 s pause bound
//!   (FR-016) belong to the routing machine (T046), not to this
//!   boundary — asserted here as the pause/follow decision only.
//! - FR-041 reason text is owned by the surface layer (`src-tauri`,
//!   T065), not by the classifier — "authoritative reason is the
//!   permanent-loss statement" is asserted here as "authoritative tracker
//!   state is the permanent loss after supersede".

use engine_core::closed::TransportState;
use player::focus::{
    DUCK_GAIN_MAX, DUCK_GAIN_MIN, FULL_GAIN, FocusIntent, FocusOutcome, FocusTracker, classify,
    supersede_flags,
};

/// One row of the 7-intent → FR-outcome boundary table: the intent, the
/// exact outcome it must classify to (`None` for follow-only intents),
/// the exact transport decision, and its FR anchor.
#[derive(Debug, Clone, Copy)]
struct BoundaryRow {
    /// Intent under test.
    intent: FocusIntent,
    /// Expected classified outcome (`None` for follow-only intents).
    outcome: Option<FocusOutcome>,
    /// Expected transport decision.
    transport: Option<TransportState>,
    /// FR anchor from the T035 invariant.
    fr: &'static str,
}

/// The 7 pre-existing intents and the exactly-one FR outcome each maps
/// to: FR-013, FR-013a, FR-014, FR-015, FR-016 (twice — noisy and
/// disappearance both pause), FR-017a.
const BOUNDARY: [BoundaryRow; 7] = [
    BoundaryRow {
        intent: FocusIntent::DuckableTransient,
        outcome: Some(FocusOutcome::DuckableTransient),
        transport: Some(TransportState::Playing),
        fr: "FR-013",
    },
    BoundaryRow {
        intent: FocusIntent::SilenceDemanding,
        outcome: Some(FocusOutcome::SilenceDemanding),
        transport: Some(TransportState::Paused),
        fr: "FR-013a",
    },
    BoundaryRow {
        intent: FocusIntent::PermanentLoss,
        outcome: Some(FocusOutcome::PermanentLoss),
        transport: Some(TransportState::Paused),
        fr: "FR-014",
    },
    BoundaryRow {
        intent: FocusIntent::FocusRegained,
        outcome: None,
        transport: Some(TransportState::Playing),
        fr: "FR-015",
    },
    BoundaryRow {
        intent: FocusIntent::BecomingNoisy,
        outcome: Some(FocusOutcome::SilenceDemanding),
        transport: Some(TransportState::Paused),
        fr: "FR-016",
    },
    BoundaryRow {
        intent: FocusIntent::DeviceAppeared,
        outcome: None,
        transport: Some(TransportState::Playing),
        fr: "FR-017a",
    },
    BoundaryRow {
        intent: FocusIntent::DeviceDisappeared,
        outcome: Some(FocusOutcome::PermanentLoss),
        transport: Some(TransportState::Paused),
        fr: "FR-016",
    },
];

/// The closed nine-member intent set (`contracts/shell-ipc.md`): the 7
/// pre-existing intents plus the 2 lifecycle latches.
const ALL_INTENTS: [FocusIntent; 9] = [
    FocusIntent::DuckableTransient,
    FocusIntent::SilenceDemanding,
    FocusIntent::PermanentLoss,
    FocusIntent::FocusRegained,
    FocusIntent::BecomingNoisy,
    FocusIntent::DeviceAppeared,
    FocusIntent::DeviceDisappeared,
    FocusIntent::AppForegrounded,
    FocusIntent::AppBackgrounded,
];

/// FR anchor for every closed intent — one explicit arm per anchor
/// (the intents sharing an anchor are grouped, never a wildcard), so
/// adding a tenth intent breaks compilation here (FR-044).
const fn fr_anchor(intent: FocusIntent) -> &'static str {
    match intent {
        FocusIntent::DuckableTransient => "FR-013",
        FocusIntent::SilenceDemanding => "FR-013a",
        FocusIntent::PermanentLoss => "FR-014",
        FocusIntent::FocusRegained => "FR-015",
        FocusIntent::BecomingNoisy | FocusIntent::DeviceDisappeared => "FR-016",
        FocusIntent::DeviceAppeared => "FR-017a",
        FocusIntent::AppForegrounded | FocusIntent::AppBackgrounded => "FR-032c",
    }
}

/// Wire string for every closed intent (`contracts/shell-ipc.md` case
/// mapping: Rust `PascalCase` → wire `snake_case`).
const WIRE: [(FocusIntent, &str); 9] = [
    (FocusIntent::DuckableTransient, "duckable_transient"),
    (FocusIntent::SilenceDemanding, "silence_demanding"),
    (FocusIntent::PermanentLoss, "permanent_loss"),
    (FocusIntent::FocusRegained, "regained"),
    (FocusIntent::BecomingNoisy, "becoming_noisy"),
    (FocusIntent::DeviceAppeared, "device_appeared"),
    (FocusIntent::DeviceDisappeared, "device_disappeared"),
    (FocusIntent::AppForegrounded, "app_foregrounded"),
    (FocusIntent::AppBackgrounded, "app_backgrounded"),
];

/// The 7 pre-existing intents each map to exactly one FR-013 / FR-013a /
/// FR-014 / FR-015 / FR-016 / FR-017a outcome (T035): exact outcome,
/// exact transport, exact gain shape, exact FR anchor.
#[expect(
    clippy::float_cmp,
    reason = "the focus gains are exact f64 literals (FULL_GAIN and the duck band bounds from focus.rs); == is the correct assertion and a tolerance would admit a drifted gain"
)]
#[test]
fn seven_pre_existing_intents_each_map_to_exactly_one_fr_outcome() {
    assert_eq!(
        BOUNDARY.len(),
        7,
        "the boundary table must hold exactly the 7 pre-existing intents"
    );
    for row in BOUNDARY {
        let decision = classify(row.intent);
        assert_eq!(
            decision.outcome, row.outcome,
            "{:?} did not classify to its exactly-one outcome ({})",
            row.intent, row.fr
        );
        assert_eq!(
            decision.transport, row.transport,
            "{:?} did not resolve its exactly-one transport decision ({})",
            row.intent, row.fr
        );
        assert_eq!(
            fr_anchor(row.intent),
            row.fr,
            "{:?} is anchored to the wrong FR",
            row.intent
        );
        if row.intent == FocusIntent::DuckableTransient {
            assert!(
                decision.gain >= DUCK_GAIN_MIN && decision.gain <= DUCK_GAIN_MAX,
                "duck gain {} left the FR-013 [0.15, 0.25] band",
                decision.gain
            );
        } else {
            assert_eq!(
                decision.gain, FULL_GAIN,
                "{:?} did not hold full gain ({})",
                row.intent, row.fr
            );
        }
        assert_eq!(
            decision.foreground_latched, None,
            "{:?} wrote the foreground latch — only lifecycle intents may ({})",
            row.intent, row.fr
        );
    }
}

/// A regain (FR-015) owes a transient resume through the tracker while
/// changing no gain; an appearance (FR-017a) decides no pause — the
/// follow itself belongs to the routing machine (T046).
#[expect(
    clippy::float_cmp,
    reason = "the focus gains are exact f64 literals (FULL_GAIN and the duck band bounds from focus.rs); == is the correct assertion and a tolerance would admit a drifted gain"
)]
#[test]
fn regain_owes_resume_and_appearance_decides_no_pause() {
    let mut tracker = FocusTracker::new();
    let _loss = tracker.apply(FocusIntent::SilenceDemanding);
    let _regained = tracker.apply(FocusIntent::FocusRegained);
    assert_eq!(
        tracker.current, None,
        "a regain must resolve the standing loss"
    );
    assert!(
        tracker.resume_due,
        "a regain must owe a transient resume (rewind math lives in crate::rewind)"
    );
    assert!(!tracker.ducked, "a regain must not leave playback ducked");

    let appear = classify(FocusIntent::DeviceAppeared);
    assert_eq!(
        appear.transport,
        Some(TransportState::Playing),
        "an appearance must follow, never pause (FR-017a)"
    );
    assert_eq!(
        appear.gain, FULL_GAIN,
        "an appearance must change no gain (FR-017a)"
    );
}

/// The 2 lifecycle intents carry NO playback decision and latch only
/// (T035): no outcome, no transport, latch write only — and neither
/// converts an open offer early, so FR-032c's 3 s visibility MUST stays
/// intact.
#[test]
fn lifecycle_intents_carry_no_playback_decision_and_latch_only() {
    let mut tracker = FocusTracker::new();
    let _loss = tracker.apply(FocusIntent::PermanentLoss);
    let standing = tracker.current;
    let snapshot = (tracker.ducked, tracker.resume_due, tracker.duck_restore_due);

    let backgrounded = tracker.apply(FocusIntent::AppBackgrounded);
    assert_eq!(
        backgrounded.outcome, None,
        "AppBackgrounded must carry no outcome"
    );
    assert_eq!(
        backgrounded.transport, None,
        "AppBackgrounded must carry no transport decision"
    );
    assert_eq!(
        backgrounded.foreground_latched,
        Some(false),
        "AppBackgrounded must latch backgrounded"
    );
    assert!(
        !tracker.foregrounded,
        "AppBackgrounded must flip the foreground latch"
    );
    assert_eq!(
        tracker.current, standing,
        "backgrounding must not convert or clear a standing decision"
    );
    assert_eq!(
        (tracker.ducked, tracker.resume_due, tracker.duck_restore_due),
        snapshot,
        "backgrounding must touch nothing but the latch"
    );

    let foregrounded = tracker.apply(FocusIntent::AppForegrounded);
    assert_eq!(
        foregrounded.outcome, None,
        "AppForegrounded must carry no outcome"
    );
    assert_eq!(
        foregrounded.transport, None,
        "AppForegrounded must carry no transport decision"
    );
    assert_eq!(
        foregrounded.foreground_latched,
        Some(true),
        "AppForegrounded must latch foregrounded"
    );
    assert!(
        tracker.foregrounded,
        "a foreground before expiry must restore the latch"
    );
    assert_eq!(
        tracker.current, standing,
        "foregrounding must not convert an open offer early — FR-032c visibility MUST intact"
    );
    assert!(
        !tracker.resume_due,
        "lifecycle intents must never authorize resume on their own"
    );
}

/// One diagnostic-record entry in the supersede sequence (FR-039 /
/// FR-044): timestamp, classified outcome, optional supersede link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordEntry {
    /// Entry timestamp, unix-ms.
    ts_ms: u64,
    /// Classified outcome this entry records.
    outcome: FocusOutcome,
    /// Timestamp of the entry this one supersedes, if any.
    supersedes_ts_ms: Option<u64>,
}

/// Supersede is two ordered entries with a supersede link, never a
/// merged rewrite (T035 / FR-039 / FR-044): the transient entry stands
/// unmodified, the permanent entry links it, latest-event-wins clears
/// the transient flags, and the FR-015 rewind basis stays the first
/// yield — never reset at the supersede point.
#[test]
fn supersede_is_two_ordered_entries_with_link_never_merged() {
    let mut tracker = FocusTracker::new();
    let first_ts_ms = 1_000_u64;
    let second_ts_ms = 2_000_u64;

    let first = tracker.apply(FocusIntent::DuckableTransient);
    assert_eq!(
        first.outcome,
        Some(FocusOutcome::DuckableTransient),
        "the first entry must record the transient loss"
    );
    let entry_first = RecordEntry {
        ts_ms: first_ts_ms,
        outcome: FocusOutcome::DuckableTransient,
        supersedes_ts_ms: None,
    };

    let flags = supersede_flags(FocusOutcome::DuckableTransient, FocusOutcome::PermanentLoss);
    assert!(
        flags.clear_resume_flag && flags.clear_duck_restore_flag,
        "escalation into permanent loss must clear both transient flags (FR-014)"
    );
    let second = tracker.apply(FocusIntent::PermanentLoss);
    assert_eq!(
        second.transport,
        Some(TransportState::Paused),
        "the superseding permanent loss must pause"
    );
    let entry_second = RecordEntry {
        ts_ms: second_ts_ms,
        outcome: FocusOutcome::PermanentLoss,
        supersedes_ts_ms: Some(entry_first.ts_ms),
    };

    let record = [entry_first, entry_second];
    assert_eq!(
        record.len(),
        2,
        "supersede must keep both entries — never a single merged rewrite"
    );
    assert!(
        record[0].ts_ms < record[1].ts_ms,
        "entries must stay ordered transient-then-permanent"
    );
    assert_eq!(
        record[1].supersedes_ts_ms,
        Some(record[0].ts_ms),
        "the permanent entry must link the transient entry it supersedes"
    );
    assert_eq!(
        record[0], entry_first,
        "the first entry must stand unmodified — never merged or rewritten"
    );
    assert_eq!(
        tracker.current,
        Some(FocusOutcome::PermanentLoss),
        "the authoritative state after supersede is the permanent loss"
    );
    assert!(
        !tracker.resume_due && !tracker.duck_restore_due && !tracker.ducked,
        "supersede must clear the transient flags and un-duck: explicit Play required"
    );
    let rewind_basis_ms = record[0].ts_ms;
    assert_eq!(
        rewind_basis_ms, first_ts_ms,
        "the FR-015 rewind basis stays the first yield — never reset at supersede"
    );
}

/// Every closed intent serializes to its `contracts/shell-ipc.md` wire
/// string (case mapping: Rust `PascalCase` → wire `snake_case`).
#[test]
fn all_nine_intents_serialize_to_shell_ipc_wire_strings() -> Result<(), Box<dyn std::error::Error>>
{
    assert_eq!(
        ALL_INTENTS.len(),
        9,
        "the closed intent set holds exactly nine intents"
    );
    assert_eq!(
        WIRE.len(),
        9,
        "the wire table must cover exactly the nine intents"
    );
    for (intent, wire) in WIRE {
        let expected = format!("\"{wire}\"");
        let json = serde_json::to_string(&intent)?;
        assert_eq!(
            json, expected,
            "intent {intent:?} did not serialize to its wire string"
        );
    }
    Ok(())
}
