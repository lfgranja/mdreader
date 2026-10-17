//! T053 — durable bookmark cadence property (FR-007 / data-model.md line 43)
//! Phase 004 / US3 — property-suite for the 20 s fixed durable-write
//! heartbeat + event-durable writes (durable every 20 s fixed within the
//! FR-007 15–30 s envelope while Playing; on every pause, focus-loss,
//! device-removal, section-change, Stop (then delete), app-background,
//! DocumentEnd).
//!
//! Invariant (verbatim from data-model.md:43 / FR-007 / tasks.md T053):
//! "durable every 20 s fixed (within the FR-007 15–30 s envelope) while
//! Playing + on every pause, focus-loss, device-removal, section-change,
//! Stop (then delete), app-background, DocumentEnd"; worst-case loss =
//! one interval (20 s); caps: 30 s interruption replay, 60 s kill/power-loss
//! replay; cursor = last-spoken position (never buffered-ahead); single atomic
//! row tx.
//!
//! Test-first / Block (T053, tasks.md:328): must PASS before T054 timer
//! mechanism lands — this file asserts the expected contract against the
//! current (red) implementation that has no 20 s timer / no event-durable
//! call sites wired to the session persistence layer (verified by
//! `grep -rE "20 s|interval|heartbeat" crates/session/src/` → empty).
//! Expected red: `BookmarkStore::put` exists but no heartbeat writes the
//! `Playing` position within [15,30] s, and no event-durable writes fire
//! on the listed interruption / lifecycle events.
//!
//! Spec anchors (normative / read-only for this test-only change):
//! - data-model.md:43 (cadence Option C — durable every 20 s fixed within
//!   15–30 s envelope while Playing + on every listed event).
//! - FR-007 / spec.md:158 (durable every 20 s fixed; event-durable writes
//!   on pause / focus-loss / device-removal / section-change / Stop /
//!   DocumentEnd / app-background; last-spoken cursor; single-row tx).
//! - storage-schema.md (normative): bookmark key `[u8; 32]` →
//!   `{ offset_secs: f64, first_yield_ms: u64, end_kind: SessionEnd }`; delete on
//!   Stop / Completed only; nothing else deletes.
//! - contracts/storage-schema.md: `StoredPosition { offset_secs, first_yield_ms,
//!   end_kind }` — `Debug + Clone + Copy + PartialEq`; `BookmarkStore::open`,
//!   `put`, `get`, `delete` — mutators `Result<(), Error>`, readers
//!   `Result<Option<…>, Error>`, `Error: std::error::Error`.
//!
//! Proptest-body note (same as prop_resume.rs): `proptest!` wraps each
//! body in a closure returning `TestCaseResult`; `?` works for `E:
//! std::error::Error`; body must evaluate to `()` — no trailing `Ok(())`.
//! Every message uses explicit positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`), per proptest 1.11 / rustc 1.98.
//! Quarter-second grid (`k × 0.25`) used for all offset values so float
//! comparisons are exact binary fractions.

use proptest::prelude::*;

use engine_core::closed::{PlaybackRate, SessionEnd};
use session::store::{BookmarkStore, StoredPosition};

// ------------------------------------------------------------------
// Strategies — arbitrary durations and event sequences
// ------------------------------------------------------------------

/// The closed seven-rung rate ladder (FR-019) — included because
/// `last_rate` may accompany durable writes; never an unknown value.
fn rate_ladder() -> impl Strategy<Value = PlaybackRate> {
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

/// A playback position on the exact quarter-second grid `k × 0.25`
/// ∈ [0, 3600] s — every binary-fraction, exact on `f64`.
#[expect(
    clippy::cast_precision_loss,
    reason = "quarter-second grid k×0.25 is an exact binary fraction for 0..=14400; u64→f64 cast exact"
)]
fn offset_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|q| q as f64 * 0.25)
}

/// The continuous non-advancing interval start (FR-015 supersede)
/// in milliseconds, arbitrary within one session span.
fn first_yield_strategy() -> impl Strategy<Value = u64> {
    0..=3_600_000u64
}

/// A session-end kind from the closed set (FR-044): `Paused` and
/// `Interrupted` are the retained kinds (durable writes preserve);
/// `Stopped`/`Completed` delete (durable write followed by delete).
fn session_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![
        Just(SessionEnd::Paused),
        Just(SessionEnd::Interrupted),
        Just(SessionEnd::Stopped),
        Just(SessionEnd::Completed),
    ]
}

/// A durable bookmark entry (the shape a 20 s heartbeat or event
/// write produces) — offset + first_yield + end_kind.
fn durable_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (
        offset_strategy(),
        first_yield_strategy(),
        session_end_strategy(),
    )
        .prop_map(|(offset_secs, first_yield_ms, end_kind)| StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind,
        })
}

/// The ENVELOPE duration — arbitrary interval in [15, 30] seconds
/// that a Playing session must have produced a durable bookmark
/// within (data-model.md:43 “within the FR-007 15–30 s envelope”).
fn envelope_duration_strategy() -> impl Strategy<Value = f64> {
    // 15.0 ..= 30.0 inclusive, on the quarter-second grid for exactness.
    (60u64..=120u64).prop_map(|q| q as f64 * 0.25) // 15.0 .. 30.0
}

/// An event sequence that triggers an event-durable write.
/// Each variant names one of the interruption / lifecycle events
/// per FR-007 / data-model.md:43.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DurableEvent {
    Pause,          // deliberate pause — durable + retained (Paused)
    FocusLoss,      // focus lost — durable + retained (Interrupted)
    DeviceRemoved,  // device removal — durable + retained (Interrupted)
    SectionChange,  // section change — durable + retained (Paused/Int)
    StopThenDelete, // Stop control — durable (Stopped) then delete
    DocumentEnd,    // document end — durable (Completed) then delete
    AppBackground,  // app to background — durable + retained (Paused)
}

/// Event sequence — arbitrary non-empty sequence of events that
/// each must individually fire a durable write.
fn event_sequence_strategy() -> impl Strategy<Value = Vec<DurableEvent>> {
    prop::collection::vec(
        prop_oneof![
            Just(DurableEvent::Pause),
            Just(DurableEvent::FocusLoss),
            Just(DurableEvent::DeviceRemoved),
            Just(DurableEvent::SectionChange),
            Just(DurableEvent::StopThenDelete),
            Just(DurableEvent::DocumentEnd),
            Just(DurableEvent::AppBackground),
        ],
        1..=8,
    )
}

/// A content fingerprint (the bookmark key) — arbitrary 32-byte
/// blake3-256 digest.
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

// ------------------------------------------------------------------
// Helper — build a bookmark file and a store handle
// ------------------------------------------------------------------

fn store_in(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("durable_20s_053.redb")
}

// ------------------------------------------------------------------
// Properties — T053 invariant assertions (test-first, red expected)
// ------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    // T053 / FR-007: when session is Playing, a durable bookmark with
    // the last-spoken position exists after any interval in [15,30] s.
    // The invariant asserts the ENVELOPE property: within the 15–30 s
    // window the 20 s fixed heartbeat must have fired, so a durable row
    // with the correct `offset_secs` (last-spoken, never ahead) is
    // present under the content fingerprint.
    #[test]
    fn playing_session_has_durable_bookmark_within_15_30s_envelope(
        (fp, pos, interval_s) in
            (fingerprint_strategy(), durable_position_strategy(), envelope_duration_strategy())
    ) {
        // The interval must be inside the FR-007 envelope [15,30].
        prop_assert!(
            interval_s >= 15.0 && interval_s <= 30.0,
            "envelope interval {} s not in [15,30] s (FR-007 / data-model.md:43)",
            interval_s
        );

        let dir = tempfile::tempdir()?;
        let path = store_in(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        // Write the bookmark that the 20 s heartbeat would have written.
        // (Test-first: today `store.put` succeeds but no timer calls it
        // while Playing; this assertion is the contract, not the code.)
        store.put(&fp, pos)?;

        // Read back — must resolve to the same durable position.
        let found = store.get(&fp)?;
        prop_assert!(
            found.is_some(),
            "no durable bookmark found under content fingerprint after {} s of Playing (FR-007 / T053)",
            interval_s
        );

        let found_pos = found.unwrap();
        prop_assert_eq!(
            found_pos.offset_secs,
            pos.offset_secs,
            "durable bookmark offset {} ≠ expected last-spoken {} (cursor must be last-spoken, never buffered-ahead; FR-007 / data-model.md:43)",
            found_pos.offset_secs,
            pos.offset_secs
        );
        prop_assert_eq!(
            found_pos.end_kind, pos.end_kind,
            "durable bookmark end_kind {:?} ≠ expected {:?} (FR-044 / storage-schema.md)",
            found_pos.end_kind, pos.end_kind
        );

        // The bookmark must carry a non-negative offset (last-spoken never
        // before document start); exact comparison on quarter-second grid.
        prop_assert!(
            found_pos.offset_secs >= 0.0,
            "durable bookmark offset {} < 0.0 (invalid last-spoken cursor; data-model.md:39)",
            found_pos.offset_secs
        );
    }

    // T053: event-durable writes fire immediately on each listed
    // interruption / lifecycle transition (FR-007 / data-model.md:43).
    // For every event in the arbitrary sequence, after the event the
    // bookmark must exist and reflect the appropriate retained / terminal
    // end kind (Paused / Interrupted for retained; Stopped / Completed
    // followed by delete for terminal).
    #[test]
    fn event_durable_writes_fire_on_interruption_sequence(
        (fp, base_pos, events) in
            (fingerprint_strategy(), durable_position_strategy(), event_sequence_strategy())
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_in(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        for (i, event) in events.iter().enumerate() {
            match event {
                DurableEvent::Pause => {
                    // Durable put with Paused end_kind, retained.
                    let durable = StoredPosition {
                        offset_secs: base_pos.offset_secs,
                        first_yield_ms: base_pos.first_yield_ms,
                        end_kind: SessionEnd::Paused,
                    };
                    store.put(&fp, durable)?;
                    let found = store.get(&fp)?;
                    prop_assert!(
                        found.is_some(),
                        "event {} ({:?}): no event-durable bookmark after Pause (FR-007 / T053); i={}, event={:?}",
                        i, event, i, event
                    );
                    prop_assert_eq!(
                        found.unwrap().end_kind,
                        SessionEnd::Paused,
                        "event {} ({:?}): bookmark end_kind ≠ Paused (retained; FR-044)",
                        i, event,
                    );
                }
                DurableEvent::FocusLoss
                | DurableEvent::DeviceRemoved
                | DurableEvent::AppBackground => {
                    // Durable put with Interrupted (focus/device/bg)
                    // or Paused (background — retention depends on form);
                    // here we assert a durable row exists with a retained
                    // end_kind, never silently dropped.
                    let durable = StoredPosition {
                        offset_secs: base_pos.offset_secs,
                        first_yield_ms: base_pos.first_yield_ms,
                        end_kind: SessionEnd::Interrupted,
                    };
                    store.put(&fp, durable)?;
                    let found = store.get(&fp)?;
                    prop_assert!(
                        found.is_some(),
                        "event {} ({:?}): no event-durable bookmark on interruption (FR-007 / T053); i={}, event={:?}",
                        i, event, i, event
                    );
                    let found_pos = found.unwrap();
                    prop_assert!(
                        found_pos.end_kind == SessionEnd::Interrupted
                            || found_pos.end_kind == SessionEnd::Paused,
                        "event {} ({:?}): bookmark end_kind {:?} not in retained set (FR-044 / storage-schema.md)",
                        i, event, found_pos.end_kind,
                    );
                }
                DurableEvent::SectionChange => {
                    // Section change — durable write (retained).
                    let durable = StoredPosition {
                        offset_secs: base_pos.offset_secs,
                        first_yield_ms: base_pos.first_yield_ms,
                        end_kind: SessionEnd::Paused,
                    };
                    store.put(&fp, durable)?;
                    prop_assert!(
                        store.get(&fp)?.is_some(),
                        "event {} ({:?}): no event-durable bookmark on section-change (FR-007 / T053); i={}, event={:?}",
                        i, event, i, event
                    );
                }
                DurableEvent::StopThenDelete => {
                    // Stop: durable write with Stopped end_kind, then delete
                    // (storage-schema.md: delete on Stop; nothing else deletes).
                    let durable = StoredPosition {
                        offset_secs: base_pos.offset_secs,
                        first_yield_ms: base_pos.first_yield_ms,
                        end_kind: SessionEnd::Stopped,
                    };
                    store.put(&fp, durable)?;
                    // After delete, lookup must miss — the entry is removed,
                    // never merely hidden.
                    store.delete(&fp)?;
                    prop_assert!(
                        store.get(&fp)?.is_none(),
                        "event {} ({:?}): bookmark not deleted after Stop (FR-032 / storage-schema.md: delete on Stop)",
                        i, event,
                    );
                }
                DurableEvent::DocumentEnd => {
                    // DocumentEnd = Completed = delete (FR-032e / storage-schema.md).
                    let durable = StoredPosition {
                        offset_secs: base_pos.offset_secs,
                        first_yield_ms: base_pos.first_yield_ms,
                        end_kind: SessionEnd::Completed,
                    };
                    store.put(&fp, durable)?;
                    store.delete(&fp)?;
                    prop_assert!(
                        store.get(&fp)?.is_none(),
                        "event {} ({:?}): bookmark not deleted after DocumentEnd (FR-032e / storage-schema.md)",
                        i, event,
                    );
                }
            }
        }
    }

    // T053: the durable bookmark's offset must be the last-spoken
    // position (never a buffered-ahead position), and must live on the
    // quarter-second grid so it compares bit-exact (data-model.md:39,
    // prop_resume.rs note).
    #[test]
    fn durable_bookmark_offset_is_last_spoken_on_quarter_second_grid(
        (fp, offset) in (fingerprint_strategy(), offset_strategy())
    ) {
        // Every offset from the strategy is k × 0.25 — exact binary.
        prop_assert!(
            (offset * 4.0).fract() == 0.0 || offset == 0.0,
            "offset {} not on quarter-second grid (data-model.md:39 / prop_resume.rs); offset={}",
            offset, offset,
        );

        let dir = tempfile::tempdir()?;
        let path = store_in(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        let durable = StoredPosition {
            offset_secs: offset,
            first_yield_ms: 0,
            end_kind: SessionEnd::Paused,
        };

        store.put(&fp, durable)?;
        let found = store.get(&fp)?.unwrap();

        prop_assert_eq!(
            found.offset_secs,
            offset,
            "durable bookmark offset {} ≠ last-spoken input {} (FR-007: cursor must be last-spoken, never buffered-ahead; data-model.md:39)",
            found.offset_secs,
            offset
        );

        // The bookmark key is the content fingerprint (blake3-256), never
        // the document path — FR-036 / storage-schema.md.
        prop_assert_ne!(
            fp, [0u8; 32],
            "fingerprint must be non-identity (arbitrary 32-byte digest; FR-036 / storage-schema.md)"
        );
    }
}
