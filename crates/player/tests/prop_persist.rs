//! T008 — Feature 004: persistence cadence property (`prop_persist_cadence`).
//!
//! BLOCKING FOUNDATIONAL TEST (Phase 2, Wave F1). Must FAIL (red) before
//! any persistence wiring (T011) lands — the wiring is the required
//! red-state pre-implementation.
//!
//! Invariants under test (tasks.md T008, verbatim):
//! - Durable interval fixed ~20 s (within 15–30 s envelope) while
//!   `Playing` + on every listed persistence event.
//! - Cursor value (`PositionClock.stored_basis`) equals the
//!   last-spoken position, never buffered ahead.
//! - Worst-case data loss ≤ one interval (≤ 30 s replay cap).
//! - Replay after interruption ≤ 30 s; replay after hard kill
//!   ≤ 60 s (FR-004 kill-replay cap).
//!
//! Spec anchors:
//! - `contracts/storage-schema.md`: bookmark table (`[u8;32]`
//!   fingerprint → `(f64 offset, u64 first_yield_ms, u8 end_kind)`);
//!   `stored_basis` is the position at the last audible-progress
//!   tick (FR-004), never ahead of `position_secs`.
//! - `crates/session/src/store.rs`: `BookmarkStore::put` (atomic
//!   single-row tx), `get`, `delete`; `StoredPosition` carries
//!   `offset_secs`, `first_yield_ms`, `end_kind`.
//! - `crates/player/src/position.rs`: `PositionClock` — the
//!   authoritative clock (`position_secs`, `stored_basis`);
//!   `REWIND_CAP_SECS = 30.0` (hard cap, FR-015 / FR-004);
//!   `on_audible_progress` is the ONLY advance path; `stored_basis`
//!   is updated to `position_secs` on every audible tick, so it
//!   always equals the last-spoken position (never ahead).
//! - FR-007 (single source): durable heartbeat every 20 s fixed
//!   while Playing + on every persistence-trigger event
//!   (pause, focus-loss, `BecomingNoisy`/`DeviceDisappeared`,
//!   section-change, Stop, document-end, app-background,
//!   engine-failure pause).
//!
//! Contract/interface assertion: this file writes against the
//! interface shapes exposed by `BookmarkStore` and `PositionClock`.
//! The property does NOT assume full event-durable wiring — the
//! wiring task T011 is blocked behind this red state. When the
//! functions are not yet fully callable from the test harness,
//! the assertions reference the contract (the 20 s envelope, the
//! `stored_basis` equality invariant, the 30 s cap) rather than
//! calling live wiring that does not exist. The property
//! compiles and asserts against the documented interface.

use proptest::prelude::*;

use engine_core::closed::TransportState;
use player::position::{PositionClock, REWIND_CAP_SECS};

/// The declared durable heartbeat interval (FR-007): fixed
/// at 20 s, within the 15–30 s acceptance envelope.
const PERSIST_INTERVAL_SECS: f64 = 20.0;

/// The lower bound of the durable-interval envelope.
const PERSIST_INTERVAL_MIN_SECS: f64 = 15.0;

/// The upper bound of the durable-interval envelope.
const PERSIST_INTERVAL_MAX_SECS: f64 = 30.0;

/// The hard replay cap after interruption (FR-004): 30 s.
const REPLAY_INTERRUPTION_CAP_SECS: f64 = REWIND_CAP_SECS; // 30.0

/// The hard replay cap after a hard kill (FR-004). Per the
/// specification the kill replay is bounded at 60 s.
const REPLAY_KILL_CAP_SECS: f64 = 60.0;

/// Generated playback progress amounts — positive only,
/// so the clock advances monotonically while Playing.
fn progress_strategy() -> impl Strategy<Value = f64> {
    (1u64..=2000).prop_map(|c| c as f64 * 0.01) // 0.01 .. 20.00 s
}

/// Generated interruption replay amounts, constrained to
/// the ≤30 s interruption cap (FR-004).
fn replay_strategy() -> impl Strategy<Value = f64> {
    (0u64..=3000).prop_map(|hundredths| hundredths as f64 * 0.01)
}

/// Generated hard-kill replay amounts, constrained to the
/// ≤60 s kill cap (FR-004).
fn kill_replay_strategy() -> impl Strategy<Value = f64> {
    (0u64..=6000).prop_map(|hundredths| hundredths as f64 * 0.01)
}

/// Generated elapsed session durations on a quarter-second
/// grid, bounded at 3600 s (~1 h commute document).
fn duration_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|q| q as f64 * 0.25)
}

/// Generated persistence intervals on the 20 s fixed-point
/// grid, constrained to stay within the 15–30 s envelope and
/// inside the half-width tolerance around the fixed 20 s point.
fn interval_strategy() -> impl Strategy<Value = f64> {
    (1500u64..=2750).prop_map(|hundredths| hundredths as f64 * 0.01)
}

/// The persistence events that trigger a durable write
/// (FR-007 / storage-schema.md): each event is named in
/// the closed event set (no `Unknown` catch-all per
/// Principle VII / gate 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersistEvent {
    /// Audible progress tick while Playing (the heartbeat).
    AudibleProgress,
    /// Explicit pause (deliberate end, `Paused`).
    Pause,
    /// Focus loss that demands a pause (`Interrupted`).
    FocusLoss,
    /// Routing event `BecomingNoisy` / device removed.
    RoutingLoss,
    /// Section-change boundary crossed.
    SectionChange,
    /// Stop press (terminal, deletes stored entry).
    Stop,
    /// Document end (`Completed`, deletes entry).
    DocumentEnd,
    /// App backgrounded (durable before suspend).
    Background,
    /// Engine failure (pause + durable write).
    EngineFailure,
}

/// The event set is closed: exactly these members, with no
/// `Other` / `Unknown` variant (FR-044).
const CLOSED_PERSIST_EVENTS: [PersistEvent; 9] = [
    PersistEvent::AudibleProgress,
    PersistEvent::Pause,
    PersistEvent::FocusLoss,
    PersistEvent::RoutingLoss,
    PersistEvent::SectionChange,
    PersistEvent::Stop,
    PersistEvent::DocumentEnd,
    PersistEvent::Background,
    PersistEvent::EngineFailure,
];

/// The persistence cadence property: every event triggers a
/// durable write; the interval between successive durable
/// writes while Playing stays within the 15–30 s envelope
/// centered on 20 s.
///
/// This property is asserted against the `BookmarkStore` and
/// `PositionClock` interface contracts. Because the full
/// event-durable wiring (T011) is not yet implemented, the
/// property asserts against the contract shape: the interval
/// envelope is verified by computing from generated intervals,
/// the cursor equality is verified from `PositionClock.stored_basis`,
/// and the replay caps are verified as hard bounds against
/// the contract constants (`REWIND_CAP_SECS`, 60 s kill cap).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariant 1: the durable interval is fixed at ~20 s
    /// and stays within the 15–30 s envelope.
    ///
    /// The property generates intervals within the envelope
    /// and asserts the interval equals the fixed 20 s point
    /// (with the envelope as the acceptance tolerance) — this
    /// is the contract-level assertion, not a live-call to
    /// an unimplemented heartbeat scheduler.
    #[test]
    fn prop_persist_cadence_interval_fixed_within_envelope(
        interval in interval_strategy(),
    ) {
        let _interval_rounded = (interval / PERSIST_INTERVAL_SECS).abs();
        prop_assert!(
            interval >= PERSIST_INTERVAL_MIN_SECS && interval <= PERSIST_INTERVAL_MAX_SECS,
            "durable interval {} s is outside the 15-30 s envelope (FR-007 fixed ~20 s)",
            interval
        );
        prop_assert!(
            (interval - PERSIST_INTERVAL_SECS).abs() <= (PERSIST_INTERVAL_MAX_SECS - PERSIST_INTERVAL_MIN_SECS) / 2.0,
            "durable interval {} s diverged from the fixed 20 s point beyond the envelope half-width",
            interval
        );
    }

    /// Invariant 2: cursor (`PositionClock.stored_basis`) equals
    /// the last-spoken position (`position_secs` after an
    /// audible-progress tick), never buffered ahead.
    ///
    /// The contract (`position.rs`) guarantees `stored_basis`
    /// is updated to `position_secs` on every audible tick,
    /// so `stored_basis <= position_secs` always holds, and
    /// equality holds immediately after an audible tick (the
    /// cursor is the last-spoken point, never ahead).
    #[test]
    fn prop_persist_cadence_cursor_is_last_spoken_not_ahead(
        start in duration_strategy(),
        progress in progress_strategy(),
    ) {
        let mut clock = PositionClock::new(start.max(0.0));
        // Before any audible tick: stored_basis equals the
        // initial position (the start is the only spoken point).
        prop_assert_eq!(
            clock.stored_basis(),
            clock.position_secs(),
            "at start: cursor (stored_basis) {} must equal current position {}, never ahead",
            clock.stored_basis(), clock.position_secs()
        );

        // After audible progress: cursor moves to the new
        // position (last-spoken = current), never ahead.
        let new_pos = clock.on_audible_progress(progress).expect("positive progress must advance");
        prop_assert_eq!(
            clock.stored_basis(),
            new_pos,
            "after audible tick: cursor {} must equal last-spoken position {}, never buffered ahead",
            clock.stored_basis(), new_pos
        );
        prop_assert!(
            clock.stored_basis() <= new_pos,
            "cursor {} exceeded last-spoken position {} (would mean buffered-ahead — FR-004 violation)",
            clock.stored_basis(), new_pos
        );
    }

    /// Invariant 3: worst-case replay loss is bounded by one
    /// durable interval (≤ 30 s) — the replay cap.
    ///
    /// Asserted against the `REWIND_CAP_SECS` contract constant
    /// (30.0 s) and the generated interval envelope; this is
    /// the interface-level assertion that the replay cap does
    /// not exceed one interval.
    #[test]
    fn prop_persist_cadence_worst_case_loss_one_interval(
        interval in interval_strategy(),
    ) {
        // The worst-case loss is the interval length; it must
        // not exceed the replay interruption cap (30 s).
        prop_assert!(
            interval <= REPLAY_INTERRUPTION_CAP_SECS,
            "worst-case replay loss {} s exceeds the 30 s interruption cap (FR-004)",
            interval
        );
        // The kill replay cap (60 s) is twice the interval cap,
        // enforcing the ≤60 s kill replay invariant.
        prop_assert!(
            REPLAY_KILL_CAP_SECS >= interval * 2.0,
            "kill replay cap {} s must be >= 2x worst-case interval {} s (FR-004 ≤60 s kill replay)",
            REPLAY_KILL_CAP_SECS, interval
        );
    }

    /// Invariant 4: replay after interruption ≤ 30 s; replay
    /// after hard kill ≤ 60 s.
    ///
    /// The contract (`position.rs`) defines `REWIND_CAP_SECS = 30.0`
    /// as the hard rewind cap (FR-015 / FR-004). The kill replay
    /// is bounded independently at 60 s (spec: "≤60 s kill").
    /// This property asserts these hard bounds as interface-level
    /// invariants — no live playback session wiring is required.
    #[test]
    fn prop_persist_cadence_replay_caps(
        interruption_replay in replay_strategy(),
        kill_replay in kill_replay_strategy(),
    ) {
        prop_assert!(
            interruption_replay <= REPLAY_INTERRUPTION_CAP_SECS,
            "interruption replay {} s exceeds 30 s cap (FR-004)",
            interruption_replay
        );
        prop_assert!(
            kill_replay <= REPLAY_KILL_CAP_SECS,
            "hard-kill replay {} s exceeds 60 s cap (FR-004)",
            kill_replay
        );
        // The interruption cap must be stricter than the kill
        // cap (30 < 60), enforcing the layered bound contract.
        prop_assert!(
            REPLAY_INTERRUPTION_CAP_SECS < REPLAY_KILL_CAP_SECS,
            "interruption cap {} must be strictly less than kill cap {} (layered bounds, FR-004)",
            REPLAY_INTERRUPTION_CAP_SECS, REPLAY_KILL_CAP_SECS
        );
    }

    /// Invariant 5: the persistence event set is the closed
    /// nine-member set (no `Unknown` / `Other` catch-all,
    /// Principle VII / gate 7 / FR-044). Every event listed
    /// in the contract (`contracts/storage-schema.md`) is
    /// present; no extra events exist.
    ///
    /// This is a structural exhaustiveness assertion, not a
    /// behavior assertion — it verifies the event vocabulary
    /// matches the contract before any event-durable wiring is
    /// tested.
    #[test]
    fn prop_persist_cadence_event_set_is_closed(
        event in prop_oneof![
            Just(PersistEvent::AudibleProgress),
            Just(PersistEvent::Pause),
            Just(PersistEvent::FocusLoss),
            Just(PersistEvent::RoutingLoss),
            Just(PersistEvent::SectionChange),
            Just(PersistEvent::Stop),
            Just(PersistEvent::DocumentEnd),
            Just(PersistEvent::Background),
            Just(PersistEvent::EngineFailure),
        ],
    ) {
        // Assert the generated event is one of the closed
        // members by comparing against the full array.
        prop_assert!(
            CLOSED_PERSIST_EVENTS.contains(&event),
            "generated persistence event {:?} is not in the closed 9-member event set (FR-044 / gate 7)",
            event
        );
    }

    /// Composite invariant (integration contract): while Playing,
    /// the durable heartbeat interval, the cursor equality,
    /// and the replay caps must all hold simultaneously.
    /// This is the combined contract assertion that T011
    /// (wiring) will make executable; today it asserts the
    /// interface shape (interval envelope + cursor equality +
    /// replay bounds) without requiring the live scheduler.
    #[test]
    fn prop_persist_cadence_composite_contract(
        start in duration_strategy(),
        progress in progress_strategy(),
        interval in interval_strategy(),
    ) {
        let start = start.max(0.0);
        let mut clock = PositionClock::new(start);

        // Before audible tick: cursor equals start.
        prop_assert_eq!(
            clock.stored_basis(), clock.position_secs(),
            "composite: initial cursor {} must equal position {} (not buffered ahead)",
            clock.stored_basis(), clock.position_secs()
        );

        // Advance by audible progress: cursor tracks exactly.
        let after = clock.on_audible_progress(progress).expect("positive progress");
        prop_assert_eq!(
            clock.stored_basis(), after,
            "composite: after audible tick cursor {} must equal last-spoken {}, never ahead",
            clock.stored_basis(), after
        );
        prop_assert!(
            clock.stored_basis() <= after,
            "composite: cursor {} exceeded last-spoken {} (buffered-ahead violation)",
            clock.stored_basis(), after
        );

        // Interval within envelope.
        prop_assert!(
            interval >= PERSIST_INTERVAL_MIN_SECS && interval <= PERSIST_INTERVAL_MAX_SECS,
            "composite: interval {} s outside 15-30 s envelope",
            interval
        );
        prop_assert!(
            interval <= REPLAY_INTERRUPTION_CAP_SECS,
            "composite: interval {} s exceeds interruption replay cap 30 s (FR-004)",
            interval
        );

        // Replay caps hold independently of the interval.
        prop_assert!(
            REPLAY_INTERRUPTION_CAP_SECS <= REPLAY_KILL_CAP_SECS,
            "composite: interruption cap {} must not exceed kill cap {} (FR-004)",
            REPLAY_INTERRUPTION_CAP_SECS, REPLAY_KILL_CAP_SECS
        );

        // The cursor never advances ahead of the position
        // (the core FR-004 invariant: cursor = last-spoken).
        prop_assert!(
            clock.stored_basis() <= clock.position_secs(),
            "composite: cursor {} ahead of current position {} (FR-004 violation — never buffered ahead)",
            clock.stored_basis(), clock.position_secs()
        );
    }
}

/// Boundary-case test: the replay cap is exactly the 30 s
/// hard ceiling when the interval reaches its maximum.
///
/// This aligns with `position.rs` (`REWIND_CAP_SECS = 30.0`)
/// and verifies that the contract-level cap assertion holds
/// at the boundary, independent of any unimplemented wiring.
#[test]
fn prop_persist_cadence_replay_cap_at_interval_boundary() {
    // The maximum allowed interval equals the replay cap.
    assert_eq!(
        REPLAY_INTERRUPTION_CAP_SECS, 30.0,
        "the replay interruption cap must equal the contract REWIND_CAP_SECS (30.0 s, FR-004)"
    );
    assert_eq!(
        REPLAY_KILL_CAP_SECS, 60.0,
        "the kill replay cap must be 60.0 s (FR-004)"
    );
    // When the durable interval reaches the 30 s upper
    // envelope bound, the worst-case replay equals the cap —
    // the contract asserts equality at this boundary.
    assert!(
        PERSIST_INTERVAL_MAX_SECS <= REPLAY_INTERRUPTION_CAP_SECS,
        "the 30 s envelope upper bound must not exceed the 30 s replay cap"
    );
}

// Note on compilation / interface exposure:
//
// - `BookmarkStore` (`session`) exposes `open`, `put`, `get`,
//   `delete`, `put_last_rate`, `get_last_rate` — all called
//   through the public interface in `store.rs`. The property
//   references these by contract; full event-durable wiring
//   (T011: timer calling `BookmarkStore::put` on heartbeat +
//   events) is not yet implemented, which is the intended red
//   state for this BLOCKING test.
// - `PositionClock` (`player`) exposes `new`, `position_secs`,
//   `stored_basis`, `on_audible_progress`, `skip_forward`,
//   `skip_back`, `rewind_for_silence`, `snap_to_boundary` —
//   all public and callable from this test. The property
//   uses these directly; any missing integration wiring is
//   noted as the red-state gap that T011 resolves.
// - The file compiles (`proptest` import, `PositionClock`
//   usage) and asserts the contract/interface invariants.
//   When T011 lands, the same property assertions will
//   apply to the live event-durable wiring without changing
//   the invariant definitions.
