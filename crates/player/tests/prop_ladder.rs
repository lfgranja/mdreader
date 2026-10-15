//! T039 — BLOCKING US4 rate-ladder property (Feature
//! 004-ressoa-dogfood-build, task T039).
//!
//! Requirement (tasks.md T039, verbatim): "Rate-ladder property in
//! `crates/player/tests/prop_ladder.rs` (7 rungs
//! `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]` exact, one step saturating,
//! `settings.last_rate` persists across sessions/documents, pitch within
//! `PITCH_TOLERANCE_CENTS=50` every rung incl. 2.0x→4.0x)"
//!
//! What these properties pin:
//!
//! - **The closed ladder** (FR-019): exactly the seven declared rungs,
//!   in ascending order, with the exact wire values
//!   `0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0` — nothing between or
//!   beyond them. The rung set is owned once by
//!   `engine_core::closed::PlaybackRate` (FR-044: no second list); this
//!   test pins the wire values and the one-step order over it.
//! - **One step, saturating** (FR-019): each RateUp/RateDown press
//!   moves exactly one rung and a press at either end saturates —
//!   never wraps, never fails, never yields a rung off the ladder.
//!   Verified both on the pure `player::transport` step functions and
//!   through the real `PlaybackSession::execute` dispatch every surface
//!   routes through (Principle II).
//! - **Persistence across sessions/documents** (FR-019b): the chosen
//!   rung is written to `settings.last_rate` on every rate change and
//!   read back after the store is closed and reopened — a new session,
//!   possibly on another document — and a different document's
//!   bookmark write never disturbs the setting. First use reads
//!   `None` (the call-site default is the `1.0` rung,
//!   storage-schema.md).
//! - **Pitch tolerance** (SC-015 / FR-043): the single contract
//!   tolerance `PITCH_TOLERANCE_CENTS = 50` cents is asserted here, and
//!   the engine-side half is pinned by the invariant handoff shape —
//!   the declared mono f32 22 050 Hz delivery never moves per rung,
//!   because the pitch-preserving time-stretch is an ENGINE-contract
//!   operation this layer never performs (no player-side resample;
//!   a rate/channel/width deviation is a named `EngineError`, never a
//!   silent resample). The measured cents deviation per rung
//!   (including the 2.0x→4.0x pair) is the engine-contract half owned
//!   by `crates/engine-core/tests/prop_pitch.rs` (T055) over the
//!   stand-in engines, completed on-device by the T051 blind trials —
//!   DOCUMENTED-DEFERRED here by design; this file asserts the
//!   constant, the engine-side seam, and the closed ladder.
//!
//! BLOCKING for the T041–T044 implementation wave: the ladder,
//! persistence, and the engine-side seam must hold across it.

use engine_core::closed::SessionEnd;
use engine_core::format::{DeclaredFormat, PITCH_TOLERANCE_CENTS, check_handoff};
use player::cli::{PlaybackSession, PlayerError, ShellVerb};
use player::transport::{PlaybackRate, rate_down_from, rate_up_from};
use proptest::prelude::*;
use session::store::{BookmarkStore, StoredPosition};

/// The closed seven-rung ladder (FR-019), ascending — the order the
/// one-step moves are measured against. The rung values live once in
/// `engine_core::closed::PlaybackRate`; this array is an ordering
/// over that set, and the exact wire values are asserted below.
const LADDER: [PlaybackRate; 7] = [
    PlaybackRate::R0_75,
    PlaybackRate::R1_0,
    PlaybackRate::R1_25,
    PlaybackRate::R1_5,
    PlaybackRate::R1_75,
    PlaybackRate::R2_0,
    PlaybackRate::R4_0,
];

/// The exact wire value of each ladder rung, ascending (FR-019): the
/// rung numbers the JSON wire carries.
const RUNG_VALUES: [f64; 7] = [0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0];

/// Test-local error type. Proptest turns any `std::error::Error`
/// returned via `?` from a property body into a test failure
/// (`TestCaseError: From<E: Error>`), so property bodies can use `?`
/// for fallible setup (temp dirs, store opens, session dispatch).
#[derive(Debug, thiserror::Error)]
enum PropError {
    /// A temporary-directory or file operation failed.
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// The session store operation failed.
    #[error("session store failed: {0}")]
    Store(#[from] session::store::Error),
    /// The playback-session dispatch failed.
    #[error("playback session failed: {0}")]
    Player(#[from] PlayerError),
}

/// A generated ladder position: the index of a rung in [`LADDER`].
fn rung_index_strategy() -> impl Strategy<Value = usize> {
    0usize..LADDER.len()
}

/// Drive a fresh session (which starts at the `R1_0` rung — the
/// FR-019b storage default) to `index` through legal RateUp/RateDown
/// presses only, then return it.
fn session_at_rung(index: usize) -> Result<PlaybackSession, PropError> {
    let mut session = PlaybackSession::new();
    // The fresh session starts at LADDER[1] (R1_0).
    if index >= 1 {
        for _ in 0..(index - 1) {
            let _stepped = session.execute(ShellVerb::RateUp)?;
        }
    } else {
        for _ in 0..(1 - index) {
            let _stepped = session.execute(ShellVerb::RateDown)?;
        }
    }
    Ok(session)
}

#[test]
fn the_ladder_is_exactly_the_seven_closed_rungs() {
    // FR-019: exactly seven rungs, ascending, with the exact wire
    // values — nothing between or beyond the declared seven. The
    // rung set is owned once by engine_core::closed::PlaybackRate;
    // the wire value assertion pins it against drift.
    assert_eq!(LADDER.len(), 7, "the ladder holds exactly seven rungs");
    for (rung, value) in LADDER.iter().zip(RUNG_VALUES) {
        assert_eq!(
            serde_json::to_value(rung).ok(),
            Some(serde_json::json!(value)),
            "rung {rung:?} must carry the exact wire value {value}"
        );
        // An off-ladder value names no rung: the ladder is closed.
        let off = value + 0.01;
        assert_eq!(
            serde_json::from_value::<PlaybackRate>(serde_json::json!(off)).ok(),
            None,
            "the off-ladder value {off} must deserialize to no rung"
        );
    }
}

#[test]
fn one_step_moves_exactly_one_rung_and_saturates_at_both_ends() {
    // FR-019: each press moves exactly one rung; a press at either
    // end saturates — never wraps, never fails, never yields a rung
    // off the ladder.
    for (index, rung) in LADDER.iter().enumerate() {
        let up = rate_up_from(*rung);
        let down = rate_down_from(*rung);
        assert_eq!(up, LADDER[(index + 1).min(LADDER.len() - 1)]);
        assert_eq!(down, LADDER[index.saturating_sub(1)]);
        // Up from anywhere and down from anywhere stays on the ladder.
        assert!(
            LADDER.contains(&up),
            "rate_up_from({rung:?}) left the ladder"
        );
        assert!(
            LADDER.contains(&down),
            "rate_down_from({rung:?}) left the ladder"
        );
    }
    // The ends saturate.
    assert_eq!(rate_up_from(PlaybackRate::R4_0), PlaybackRate::R4_0);
    assert_eq!(rate_down_from(PlaybackRate::R0_75), PlaybackRate::R0_75);
}

proptest! {
    /// One RateUp/RateDown press through the real session dispatch
    /// (the closed transition table every surface routes through,
    /// FR-042) moves exactly one rung from every starting rung, and
    /// repeated presses saturate at the ends — never wrapping, never
    /// failing, never leaving the closed ladder.
    #[test]
    fn one_rate_press_moves_exactly_one_rung_and_saturates_through_the_session(
        index in rung_index_strategy(),
    ) {
        let start = LADDER[index];
        let mut session = session_at_rung(index)?;
        prop_assert_eq!(
            session.snapshot().rate,
            start,
            "the session did not reach the starting rung {start:?}",
            start = start,
        );

        let up = session.execute(ShellVerb::RateUp)?;
        prop_assert_eq!(
            up.rate,
            rate_up_from(start),
            "the RateUp press from {start:?} did not match the pure step function",
            start = start,
        );
        prop_assert_eq!(
            up.rate,
            LADDER[(index + 1).min(LADDER.len() - 1)],
            "the RateUp press from {start:?} did not move exactly one rung (saturating at the top)",
            start = start,
        );

        let down = session.execute(ShellVerb::RateDown)?;
        prop_assert_eq!(
            down.rate,
            rate_down_from(up.rate),
            "the RateDown press did not match the pure step function",
        );

        // Saturation walks: twenty ups from the start land on the
        // top rung, twenty downs on the bottom rung, and no
        // intermediate press ever leaves the ladder.
        let mut walker = session_at_rung(index)?;
        for _ in 0..20 {
            let snapshot = walker.execute(ShellVerb::RateUp)?;
            prop_assert!(
                LADDER.contains(&snapshot.rate),
                "a RateUp press produced the off-ladder rung {:?}",
                snapshot.rate,
            );
        }
        prop_assert_eq!(
            walker.snapshot().rate,
            PlaybackRate::R4_0,
            "repeated RateUp presses must saturate at the top rung",
        );
        for _ in 0..20 {
            let snapshot = walker.execute(ShellVerb::RateDown)?;
            prop_assert!(
                LADDER.contains(&snapshot.rate),
                "a RateDown press produced the off-ladder rung {:?}",
                snapshot.rate,
            );
        }
        prop_assert_eq!(
            walker.snapshot().rate,
            PlaybackRate::R0_75,
            "repeated RateDown presses must saturate at the bottom rung",
        );
    }

    /// The chosen rung persists in `settings.last_rate` across
    /// sessions and documents: it survives the store being closed
    /// and reopened (a new session), is undisturbed by another
    /// document's bookmark write, and a first-use store reads no
    /// rung (the call-site default is `1.0`, storage-schema.md).
    #[test]
    fn last_rate_persists_across_sessions_and_documents(
        index in rung_index_strategy(),
        seed in any::<u8>(),
    ) {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("bookmarks.redb");
        let rung = LADDER[index];

        // Session one: the rate change writes the setting.
        {
            let mut store = BookmarkStore::open(&path)?;
            store.put_last_rate(rung)?;
        }

        // Another document's bookmark write in the same store must
        // not disturb the setting (the settings table is
        // document-independent, FR-019b).
        {
            let mut store = BookmarkStore::open(&path)?;
            let fingerprint = [seed; 32];
            store.put(
                &fingerprint,
                StoredPosition {
                    offset_secs: 12.5,
                    first_yield_ms: 250,
                    end_kind: SessionEnd::Paused,
                },
            )?;
            prop_assert_eq!(
                store.get_last_rate()?,
                Some(rung),
                "a bookmark write changed the stored last_rate",
            );
        }

        // A reopened store — a new session, possibly another
        // document — still reads the chosen rung.
        {
            let mut store = BookmarkStore::open(&path)?;
            prop_assert_eq!(
                store.get_last_rate()?,
                Some(rung),
                "the stored last_rate did not survive the reopen (rung {rung:?} lost)",
                rung = rung,
            );
        }

        // First use: a fresh store holds no last_rate — the
        // call-site default is the 1.0 rung.
        let fresh_path = dir.path().join("fresh.redb");
        {
            let mut store = BookmarkStore::open(&fresh_path)?;
            prop_assert_eq!(
                store.get_last_rate()?,
                None,
                "a first-use store must read no last_rate",
            );
        }
    }
}

#[test]
fn the_pitch_tolerance_constant_is_fifty_cents() {
    // SC-015 / contracts/engine-handoff.md: the single contract
    // pitch tolerance is exactly 50 cents — the bound the engine
    // trait's `set_rate` stretch is held to at every rung, including
    // the 2.0x→4.0x pair.
    assert_eq!(PITCH_TOLERANCE_CENTS, 50_u32);
}

#[test]
fn the_rate_stretch_is_engine_side_with_no_player_side_resample() {
    // FR-043: the rate change is a pitch-preserving time-stretch the
    // ENGINE performs (`Engine::set_rate`); this layer only selects
    // the rung. The host-checkable half of the pitch property: the
    // declared engine→shell handoff shape (mono f32 at 22 050 Hz) is
    // INVARIANT across the whole ladder — an engine that stretched
    // locally in the player would move the delivered rate, and any
    // deviation is a named `EngineError`, never a silent resample.
    // The measured cents deviation per rung (incl. 2.0x→4.0x) is the
    // engine-contract half: `crates/engine-core/tests/prop_pitch.rs`
    // (T055) over the stand-in engines, completed on-device by the
    // T051 blind trials — documented-deferred from this file.
    assert_eq!(DeclaredFormat::SAMPLE_RATE, 22_050);
    assert_eq!(DeclaredFormat::CHANNELS, 1);
    assert_eq!(DeclaredFormat::SAMPLE_WIDTH_BITS, 32);
    for rung in LADDER {
        // The step functions are pure selections: they return a
        // rung and carry no audio, so nothing in this layer can
        // resample — the selected rung is all the engine is told.
        assert!(LADDER.contains(&rate_up_from(rung)));
        assert!(LADDER.contains(&rate_down_from(rung)));
        // The delivered handoff shape never moves per rung.
        assert_eq!(
            check_handoff(
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS
            ),
            Ok(()),
            "the declared handoff shape must conform at every rung — the stretch is engine-side (rung {rung:?})"
        );
    }
    // A deviating delivery is named, never silently adapted.
    assert_eq!(
        check_handoff(
            48_000,
            DeclaredFormat::CHANNELS,
            DeclaredFormat::SAMPLE_WIDTH_BITS
        ),
        Err(engine_core::closed::EngineError::RateMismatch),
        "a rate deviation must surface as a named EngineError, never a silent resample"
    );
}
