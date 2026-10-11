//! T021 — US1 integration: kill-recovery bound (FR-004 / FR-030 / FR-031 /
//! FR-032 / SC-007 / SC-011).
//!
//! Host-runnable replay of the kill/power-loss device fixture
//! (`specs/001-android-media-shell/tests/device/fixtures/service_kill_power_loss.jsonl`):
//! every fixture step is driven through the Rust-side session
//! (`player::cli::PlaybackSession`), the authoritative position clock
//! (`player::position::PositionClock`), the resume-offer machines
//! (`player::resume` / `session::resume_offer`), and the redb bookmark
//! table (`session::BOOKMARKS`), asserting the outcome each fixture
//! `result` token records.
//!
//! Trial legs (tasks.md T021, verbatim): "kill mid-playback → relaunch
//! offers resume at last spoken position replaying ≤30 s per FR-004,
//! SC-007; unexpected end → paused + explicit resume per FR-030, never
//! auto-audio per FR-031, SC-011".
//!
//! - Kill mid-playback: a session driven `Playing` with audible progress
//!   is hard-killed — dropped with no Stop verb, so the record ends
//!   mid-session with no session_end entry. The kill-replay basis
//!   (`stored_basis`, the position at the last audible tick) must trail
//!   the final position by at most [`REWIND_CAP_SECS`]: that is the
//!   ≤30 s replay bound of FR-004/SC-007, the bound that holds exactly
//!   because no final write can be relied upon under a hard kill.
//! - Relaunch: a fresh session opened with the stored basis is offered a
//!   resume at the last spoken position (the basis), and the rewind
//!   applied on resume is exactly `min(0.05 × paused, 30.0)` measured
//!   from the yield, so total backward movement never exceeds 30 s.
//! - Unexpected end: a session ended with `ShellVerb::Interrupt` (the
//!   service-killed end) presents a paused state at the last spoken
//!   position with an explicit resume control — never an auto-start
//!   (FR-030/FR-031/FR-032d/SC-011): no audio path may begin without
//!   an explicit Play/Resume verb. The offer expires after 3 s
//!   (FR-041/FR-032c) converting to explicit-resume, never auto-audio.
//! - Pause-preserves vs Stop-discards: a deliberate Pause keeps the
//!   stored basis available for the resume offer while a Stop clears it
//!   (FR-032/FR-032a/FR-032f) — the redb bookmark entry survives the
//!   pause and is removed by Stop, so a stopped document is never
//!   resumable.
//!
//! Spec anchors: FR-004 ("Playback position MUST survive process death
//! and be recoverable to the last spoken position rather than the
//! document start, replaying at most 30 seconds of already-spoken audio
//! in the process"), FR-030 ("When a previous session ends unexpectedly
//! because the service was killed, the next launch MUST present a paused
//! state positioned at the last spoken position, with an explicit resume
//! control"), FR-031 ("Following an unexpected session end, the system
//! MUST NOT begin producing audio on its own … every other recovery path
//! remains silent"), FR-032 ("A session the user ended with Stop MUST NOT
//! be presented as an interrupted session on next launch, and MUST NOT be
//! resumable"), FR-041 (the 3-second bounded offer, expiring everywhere
//! after 3 s), SC-007 ("After an unexpected process termination and
//! relaunch, playback can be resumed at the last spoken position,
//! replaying at most 30 seconds of already-spoken audio"), and SC-011
//! ("In 10 out of 10 trials where the service is killed under memory
//! pressure, the next launch shows a paused state at the last spoken
//! position and produces no audio until the user activates resume").
//!
//! Exactness note: progress and pause lengths are generated on the
//! centisecond grid (`k × 0.01`) and the rewind is asserted with the
//! same expression the contract states —
//! `min(REWIND_FRACTION × paused, REWIND_CAP_SECS)` — so the equality
//! assertions are bit-exact. A tolerance would wrongly admit a replay
//! beyond the 30 s hard ceiling (FR-004: the bound is a hard ceiling,
//! not a target).

use engine_core::closed::TransportState;
use player::cli::{PlaybackSession, ShellVerb};
use player::position::{PositionClock, REWIND_CAP_SECS, REWIND_FRACTION};
use player::resume::{ResumeChoice, resume_offer_window};
use session::{
    BOOKMARKS, ResumeChoice as StoredResumeChoice, clear_bookmark, fingerprint, resume_offer,
};

use proptest::prelude::*;

/// The fixture's kill/reopen trial, embedded so the host replay is
/// self-contained (the fixture carries no document text, FR-045).
const FIXTURE_SERVICE_KILL_POWER_LOSS: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/service_kill_power_loss.jsonl"
);

/// The 3-second offer window (FR-032c / FR-041), in seconds.
const OFFER_WINDOW_SECS: f64 = 3.0;

/// Stored `end_kind` code for a session ended by a deliberate pause
/// (data-model.md `StoredPosition`: `paused` ⇒ the FR-032c deliberate
/// offer; the stored basis stays available for it).
const END_KIND_PAUSED: u8 = 0;

/// Stored `end_kind` code for a session ended unexpectedly — service
/// kill, power loss, permanent focus loss (data-model.md
/// `StoredPosition`: `interrupted` ⇒ the FR-032d explicit-resume
/// offer, never an auto-start).
const END_KIND_INTERRUPTED: u8 = 2;

/// A synthetic document for the bookmark round-trip — content bytes
/// only, never document text (FR-045: the diagnostic and storage
/// surfaces carry no document body).
const FIXTURE_DOCUMENT_BYTES: &[u8] = b"us1-kill-recovery-fixture-document";

/// Audible progress in `(0.00, 5.00]` s — strictly positive, so a
/// progress tick while `Playing` is a strictly greater position.
fn progress_strategy() -> impl Strategy<Value = f64> {
    (1u64..=500).prop_map(|centis| centis as f64 / 100.0)
}

/// Silence-pause durations `d` ∈ [0, 7200] s as exact centisecond
/// multiples — the FR-015 paused-duration input.
fn paused_duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=720_000).prop_map(|centis| centis as f64 / 100.0)
}

/// Positions on the exact quarter-second grid `k × 0.25` ∈ [0, 3600].
fn position_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// Sentence starts for the relaunch resume snap: ascending, on the same
/// quarter-second grid, always containing the document start `0.0` so a
/// greatest-boundary-at-or-before lookup always has an answer.
fn sentence_starts_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=16).prop_map(|mut quarters| {
        quarters.push(0);
        quarters.sort();
        quarters.dedup();
        quarters.into_iter().map(|q| q as f64 * 0.25).collect()
    })
}

/// The fixture's three recorded trial steps, in fixture order: the
/// service kill preserves the bookmark at the killed position, the
/// reopen offers an explicit resume (never auto-audio), and the
/// power loss replays at the last spoken position within the 30 s
/// bound.
#[test]
fn the_kill_power_loss_fixture_replays_within_the_thirty_second_bound() {
    let lines: Vec<&str> = FIXTURE_SERVICE_KILL_POWER_LOSS.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "service_kill_power_loss.jsonl must carry exactly the 3 recorded trial steps"
    );
    assert!(
        lines[0].contains("SERVICE_KILL") && lines[0].contains("bookmark_preserved\":true"),
        "fixture step 1 must be SERVICE_KILL with the bookmark preserved"
    );
    assert!(
        lines[1].contains("REOPEN")
            && lines[1].contains("unexpected_end\":true")
            && lines[1].contains("explicit_resume_not_auto_audio"),
        "fixture step 2 must be REOPEN → explicit_resume_not_auto_audio"
    );
    assert!(
        lines[2].contains("POWER_LOSS")
            && lines[2].contains("replay_bound_ms\":30000")
            && lines[2].contains("replay_at_last_spoken"),
        "fixture step 3 must be POWER_LOSS → replay_at_last_spoken with the 30 s bound"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_hard_kill_replays_at_most_thirty_seconds_of_already_spoken_audio(
        (progresses, pause_lengths) in (
            proptest::collection::vec(progress_strategy(), 1..=64),
            proptest::collection::vec(paused_duration_strategy(), 0..=8),
        )
    ) {
        // The session is driven `Playing` with audible progress — the
        // only advance path (FR-002): position advances only while
        // audio is audible, never while silent (FR-005). The session
        // dispatch models the transport; the authoritative clock is
        // driven with the same ticks plus the silence pauses, each of
        // which yields at the current position and applies the FR-015
        // rewind (the resume-engine operation on the authority, not a
        // session verb). The generated pause lengths exercise every
        // window in which the stored basis can trail the live
        // position.
        let mut session = PlaybackSession::new();
        let started = session.execute(ShellVerb::Play);
        prop_assert!(
            started.is_ok(),
            "Play must succeed from the initial Paused state"
        );
        let mut clock = PositionClock::new(0.0);
        let mut pauses = pause_lengths.iter().copied();
        let mut expected_position = 0.0;
        for progress in &progresses {
            session.on_audible_progress(*progress);
            let _advanced = clock.on_audible_progress(*progress);
            expected_position += *progress;
            if let Some(paused_duration) = pauses.next() {
                let _rewound = clock.rewind_for_silence(paused_duration);
            }
            let snapshot = session.snapshot();
            prop_assert!(
                snapshot.transport == TransportState::Playing,
                "the session must stay Playing while audible progress is delivered (FR-005)"
            );
            prop_assert_eq!(
                snapshot.position_secs,
                expected_position,
                "the session must reflect the accumulated audible progress exactly (FR-002, FR-003)"
            );
        }

        // Hard kill: the session is dropped with no Stop verb — the
        // record ends mid-session exactly as a hard kill leaves it
        // (no session_end entry, FR-004 / SC-007). No final write
        // can be relied upon, so the recovery basis is the last
        // stored basis, not the final position.
        drop(session);
        let final_position = clock.position_secs();
        let kill_replay_basis = clock.stored_basis();

        // FR-004 / SC-007: recovery replays at most 30 s of
        // already-spoken audio — the kill-replay basis trails the
        // final position by at most REWIND_CAP_SECS. A replay
        // beyond the bound is a defect, and so is discarding the
        // position entirely: the basis must stay at the last
        // spoken position, never fall back to the document start.
        prop_assert!(
            final_position - kill_replay_basis <= REWIND_CAP_SECS,
            "the kill-replay basis {kill_replay_basis} trails the final position {final_position} by more than {REWIND_CAP_SECS} s — the relaunch would replay more than 30 s of already-spoken audio (FR-004, SC-007)"
        );
        prop_assert!(
            kill_replay_basis > 0.0,
            "the kill-replay basis was discarded to the document start — the position must survive process death (FR-004)"
        );
        prop_assert!(
            kill_replay_basis >= 0.0,
            "the kill-replay basis {kill_replay_basis} fell below the declared position floor 0.0"
        );
        prop_assert!(
            final_position >= 0.0,
            "the final position {final_position} fell below the declared position floor 0.0"
        );
    }

    #[test]
    fn a_relaunch_opens_at_the_stored_basis_and_the_resume_rewind_stays_within_the_cap(
        (yield_position, paused_duration, sentence_starts) in
            (position_strategy(), paused_duration_strategy(), sentence_starts_strategy())
    ) {
        // The kill left the authority at the yield position with the
        // stored basis at the last audible tick — the last spoken
        // position the relaunch recovers (FR-004).
        let mut killed = PositionClock::new(0.0);
        let _advanced = killed.on_audible_progress(yield_position);
        let stored_basis = killed.stored_basis();

        // The relaunch: a fresh session opened with the stored basis
        // is offered a resume at the last spoken position (the
        // basis) — not the document start (FR-004 / FR-030).
        let mut relaunched = PositionClock::new(stored_basis);
        prop_assert_eq!(
            relaunched.position_secs(),
            stored_basis,
            "the relaunch must open at the stored basis (the last spoken position), not the document start (FR-004)"
        );

        // The rewind applied on resume: exactly min(0.05 × paused,
        // 30.0), clamped at the 0.0 floor, measured from the yield
        // — the same expression the contract states (FR-015).
        let rewind_amount = (REWIND_FRACTION * paused_duration).min(REWIND_CAP_SECS);
        let rewound = relaunched.rewind_for_silence(paused_duration);
        let expected_cursor = (stored_basis - rewind_amount).max(0.0);
        prop_assert_eq!(
            rewound,
            expected_cursor,
            "the resume rewind of a {} s pause from the yield {} did not land at the 5%-of-pause cursor capped at 30 s (FR-015)",
            paused_duration,
            stored_basis,
        );

        // Total backward movement from the yield never exceeds the
        // 30 s hard cap — the FR-004/SC-007 replay bound.
        prop_assert!(
            stored_basis - rewound <= REWIND_CAP_SECS,
            "total backward movement of {} s from the yield {stored_basis} exceeded the 30-second cap (FR-004, FR-015)",
            stored_basis - rewound
        );
        prop_assert!(
            rewind_amount <= REWIND_CAP_SECS,
            "the rewind amount {rewind_amount} exceeded the 30-second cap (FR-015)"
        );

        // The sentence snap never lands earlier than the cap point
        // — the greatest boundary at or before the cursor, never
        // more than REWIND_CAP_SECS behind the yield — so the
        // combined rewind + snap stays inside the same bound.
        let resumed = relaunched.snap_to_boundary(&sentence_starts);
        let cap_point = (stored_basis - REWIND_CAP_SECS).max(0.0);
        prop_assert!(
            resumed >= cap_point,
            "the sentence snap {resumed} landed before the 30-second cap point {cap_point} from the yield {stored_basis} (FR-004, FR-015)"
        );
        prop_assert!(
            stored_basis - resumed <= REWIND_CAP_SECS,
            "total backward movement of {} s including the sentence snap exceeded the 30-second cap (FR-004, FR-015)",
            stored_basis - resumed
        );
        prop_assert!(
            resumed >= 0.0,
            "the resume {resumed} went below the position floor 0.0"
        );
    }
}

/// The unexpected-end leg (fixture step 2, `REOPEN →
/// explicit_resume_not_auto_audio`): a session ended with the
/// interrupt verb — the service-killed end — presents a paused state
/// at the last spoken position with an explicit resume control, and
/// never an auto-start (FR-030 / FR-031 / FR-032d / SC-011).
#[expect(
    clippy::float_cmp,
    reason = "the last spoken position 120.0 is an exact binary fraction and the assertion pins the resume-at-stored-position contract (FR-030); == is exact and a tolerance would admit a position drift"
)]
#[test]
fn an_interrupted_end_presents_an_explicit_resume_never_an_auto_start() {
    // Drive a session Playing with audible progress, then end it
    // unexpectedly: the service is killed under memory pressure
    // (the fixture's SERVICE_KILL / unexpected_end shape).
    let mut session = PlaybackSession::new();
    let started = session.execute(ShellVerb::Play);
    assert!(
        started.is_ok(),
        "Play must succeed from the initial Paused state"
    );
    session.on_audible_progress(120.0);
    let at_yield = session.snapshot();
    assert_eq!(
        at_yield.transport,
        TransportState::Playing,
        "the session must be Playing while audible progress is delivered (FR-005)"
    );

    // The unexpected end itself: the interrupt verb is the closed
    // representation of a service kill / permanent loss end.
    let interrupted = session.execute(ShellVerb::Interrupt);
    assert!(
        interrupted.is_ok(),
        "Interrupt must succeed from the Playing state"
    );
    let ended = session.snapshot();
    assert_eq!(
        ended.transport,
        TransportState::Interrupted,
        "an unexpected end must land the Interrupted session-end state (FR-044)"
    );
    assert_eq!(
        ended.position_secs, 120.0,
        "the interrupted end must keep the last spoken position (FR-030)"
    );

    // The relaunch offer for an unexpected end is the explicit-resume
    // form — never the deliberate-pause auto-start default and never
    // a start-over discard (FR-030 / FR-031 / FR-032d / SC-011).
    let stored_basis = 120.0;
    let choice = resume_offer(Some(stored_basis), Some(END_KIND_INTERRUPTED));
    assert_eq!(
        choice,
        StoredResumeChoice::ExplicitResume,
        "an interrupted end must offer an explicit resume control, never an auto-start (FR-030, FR-031, FR-032d, SC-011)"
    );

    // The same closed-set guarantee through the player-side offer
    // machine: while the offer window is open, an unexpected end
    // presents the explicit-resume choice — never ContinueDefault,
    // the auto-starting form (FR-031: no audio on its own).
    for secs_since_end in [0.0, 1.5, OFFER_WINDOW_SECS] {
        let offered = resume_offer_window(secs_since_end, false);
        assert_eq!(
            offered,
            Some(ResumeChoice::ExplicitResume),
            "the offer at {secs_since_end} s must be the explicit-resume form, never an auto-start (FR-030, FR-031, SC-011)"
        );
    }

    // The offer expires after 3 s (FR-041 / FR-032c): expiry
    // converts to explicit-resume — the paused state stays, awaiting
    // the user's resume — and never auto-audio. Nothing transitions
    // to Playing without an explicit Play/Resume verb, so the session
    // still reports the interrupted end after the window closes.
    assert_eq!(
        resume_offer_window(OFFER_WINDOW_SECS + 0.5, false),
        None,
        "the offer must expire everywhere after 3 s (FR-041, FR-032c)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Interrupted,
        "no audio path may begin without an explicit Play/Resume verb — the session must still report the interrupted end after the offer expires (FR-031, SC-011)"
    );

    // Resuming is an explicit act: only the resume verb moves the
    // interrupted session to Playing, and it resumes at the last
    // spoken position (FR-030 / data-model `Interrupted → Playing`).
    let resumed = session.execute(ShellVerb::ResumeAtStored);
    assert!(
        resumed.is_ok(),
        "ResumeAtStored must succeed from the Interrupted state"
    );
    let after_resume = session.snapshot();
    assert_eq!(
        after_resume.transport,
        TransportState::Playing,
        "the explicit resume must move the interrupted session to Playing (FR-030)"
    );
    assert_eq!(
        after_resume.position_secs, 120.0,
        "the explicit resume must resume at the last spoken position (FR-004, FR-030)"
    );
}

/// The deliberate-pause leg of the same offer machine: a deliberate
/// pause offers continue-by-default inside the 3 s window — the one
/// path FR-031 permits to auto-start — which is exactly what
/// distinguishes it from the unexpected-end leg above.
#[test]
fn a_deliberate_pause_offers_continue_default_inside_the_three_second_window() {
    for secs_since_pause in [0.0, 1.5, OFFER_WINDOW_SECS] {
        let offered = resume_offer_window(secs_since_pause, true);
        assert_eq!(
            offered,
            Some(ResumeChoice::ContinueDefault),
            "a deliberate pause must offer continue-by-default inside the 3 s window (FR-032c)"
        );
    }
    assert_eq!(
        resume_offer_window(OFFER_WINDOW_SECS + 0.5, true),
        None,
        "the deliberate-pause offer must also expire after 3 s (FR-041)"
    );
    // The stored-offer machine maps a paused end to the deliberate
    // offer, whose default is continue from the stored offset.
    assert_eq!(
        resume_offer(Some(120.0), Some(END_KIND_PAUSED)),
        StoredResumeChoice::DeliberatePause,
        "a paused end must offer the deliberate-pause continue form (FR-032c)"
    );
}

/// Pause-preserves vs Stop-discards (FR-032 / FR-032a / FR-032f): a
/// deliberate Pause keeps the stored basis available for the resume
/// offer while a Stop clears it. The bookmark round-trips through
/// the session crate's redb `bookmarks` table on a tempfile-backed
/// database, exactly as the storage schema records it.
#[test]
fn a_pause_preserves_and_a_stop_discards_the_stored_basis() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("bookmarks.redb");
    let db = redb::Database::create(&db_path)?;

    let document_fp = fingerprint(FIXTURE_DOCUMENT_BYTES);
    let stored_basis = 120.0;
    let first_yield_ms = 6_000_u64;

    // Pause path: the deliberate pause persists the stored basis as a
    // bookmark (FR-032a: the position is preserved across every
    // pause-path end), keyed by the content fingerprint.
    let write_tx = db.begin_write()?;
    {
        let mut table = write_tx.open_table(BOOKMARKS)?;
        let _previous = table.insert(
            &document_fp,
            &(stored_basis, first_yield_ms, END_KIND_PAUSED),
        )?;
    }
    write_tx.commit()?;

    // The paused basis is available for the resume offer: the
    // bookmark reads back with the last spoken position and the
    // deliberate-pause end kind, and the offer machine maps it to the
    // FR-032c deliberate offer.
    let read_tx = db.begin_read()?;
    let stored = {
        let table = read_tx.open_table(BOOKMARKS)?;
        let guard = table.get(&document_fp)?;
        guard.map(|entry| entry.value())
    };
    drop(read_tx);
    assert_eq!(
        stored,
        Some((stored_basis, first_yield_ms, END_KIND_PAUSED)),
        "the deliberate pause must preserve the stored basis in the bookmark table (FR-032a)"
    );
    assert_eq!(
        resume_offer(stored.map(|entry| entry.0), stored.map(|entry| entry.2)),
        StoredResumeChoice::DeliberatePause,
        "the preserved basis must be available for the deliberate-pause resume offer (FR-032c)"
    );

    // Stop path: Stop discards the stored position (FR-032: "Stop
    // discards the stored position for that document, so a
    // subsequent open of the same document starts from its first
    // spoken position"). The bookmark entry is removed, so no resume
    // is offered for the stopped document.
    clear_bookmark(&db, &document_fp)?;

    let read_tx = db.begin_read()?;
    let discarded = {
        let table = read_tx.open_table(BOOKMARKS)?;
        let guard = table.get(&document_fp)?;
        guard.map(|entry| entry.value())
    };
    drop(read_tx);
    assert_eq!(
        discarded, None,
        "Stop must clear the stored basis from the bookmark table (FR-032)"
    );
    assert_eq!(
        resume_offer(
            discarded.map(|entry| entry.0),
            discarded.map(|entry| entry.2)
        ),
        StoredResumeChoice::StartOver,
        "a cleared basis must not be resumable — the offer falls back to start-over (FR-032)"
    );

    // An unreachable fingerprint (edited content — a different
    // document, FR-036a) finds no entry either: no offer, because
    // the position is keyed by content, not path.
    let edited_fp = fingerprint(b"us1-kill-recovery-fixture-documenT");
    assert_ne!(
        document_fp, edited_fp,
        "a 1-bit content change must produce a different fingerprint (FR-036a)"
    );
    let read_tx = db.begin_read()?;
    let unreachable = {
        let table = read_tx.open_table(BOOKMARKS)?;
        let guard = table.get(&edited_fp)?;
        guard.map(|entry| entry.value())
    };
    drop(read_tx);
    assert_eq!(
        unreachable, None,
        "an edited document's position must be unreachable — no resume is offered (FR-036a)"
    );
    Ok(())
}
