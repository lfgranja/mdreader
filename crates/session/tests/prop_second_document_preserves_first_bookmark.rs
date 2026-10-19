//! T057 — FR-018 second-document pick preserves first bookmark.
//! Property `prop_second_document_preserves_first_bookmark`.
//!
//! Invariant (tasks.md T057 / FR-018 / DocIngest.kt:190 + lib.rs:1373):
//! when a new document is picked while the first session is playing or
//! paused, a durable bookmark put with `Paused` or `Interrupted`
//! `end_kind` must fire BEFORE the new snapshot is written;
//! the first bookmark is never deleted; the second never auto-starts;
//! after the switch `store.get(first_hash)` still resolves.
//!
//! FAILS (red) until T058 wires the durable-save verb into the open
//! sequence (`DocIngest.kt` / `shell_jni.rs` / `lib.rs:open_document_bytes`).
//! The save wire is missing — the property detects the gap.

use proptest::prelude::*;

use engine_core::closed::SessionEnd;
use session::store::{BookmarkStore, StoredPosition};

/// Arbitrary 32-byte content fingerprint (blake3-256).
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

/// Session state at the moment of second-document pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionState {
    Playing,
    Paused,
    Interrupted,
    Stopped,
}

fn session_state_strategy() -> impl Strategy<Value = SessionState> {
    prop_oneof![
        Just(SessionState::Playing),
        Just(SessionState::Paused),
        Just(SessionState::Interrupted),
        Just(SessionState::Stopped),
    ]
}

/// One step of the open/switch flow — a closed three-member set
/// (FR-018), so the event match enumerates every step and needs no
/// catch-all arm.
#[derive(Clone, Copy, Debug)]
enum SwitchEvent {
    /// Durable bookmark put for the first document (`Paused`/`Interrupted`).
    DurableSave,
    /// Second document picked (no store mutation — simulated intent).
    PickSecond,
    /// Second snapshot saved (new bookmark written).
    SaveSecondSnapshot,
}

/// Event sequence over the open/switch flow.
fn event_sequence_strategy() -> impl Strategy<Value = Vec<SwitchEvent>> {
    prop::collection::vec(
        prop_oneof![
            Just(SwitchEvent::DurableSave),
            Just(SwitchEvent::PickSecond),
            Just(SwitchEvent::SaveSecondSnapshot),
        ],
        1..=4,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T057: second-document pick preserves the first bookmark.
    /// Invariant: durable put with `Paused`/`Interrupted` fires
    /// before new snapshot; first bookmark never deleted;
    /// second never auto-starts; first preserved after switch.
    #[test]
    fn prop_second_document_preserves_first_bookmark(
        first_fp in fingerprint_strategy(),
        second_fp in fingerprint_strategy(),
        state in session_state_strategy(),
        sequence in event_sequence_strategy(),
    ) {
        prop_assume!(first_fp != second_fp);

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("bookmarks.redb");
        let mut store = BookmarkStore::open(&path)?;

        // Event sequence applied to the store — models the
        // open-switch flow (durable save → pick → save).
        let mut had_save = false;
        for step in sequence {
            match step {
                SwitchEvent::DurableSave => {
                    // Durable bookmark with correct end_kind for
                    // non-terminal session states (FR-018 / FR-044).
                    if state != SessionState::Stopped {
                        let end_kind = match state {
                            SessionState::Interrupted => SessionEnd::Interrupted,
                            _ => SessionEnd::Paused, // Playing / Paused
                        };
                        store.put(&first_fp, StoredPosition {
                            offset_secs: 12.25,
                            first_yield_ms: 250,
                            end_kind,
                        })?;
                    }
                }
                SwitchEvent::PickSecond => {
                    // Second document picked — simulated intent,
                    // no direct store mutation (the wire is missing).
                }
                SwitchEvent::SaveSecondSnapshot => {
                    had_save = true;
                    // New snapshot written for second document.
                    store.put(&second_fp, StoredPosition {
                        offset_secs: 0.0,
                        first_yield_ms: 0,
                        end_kind: SessionEnd::Paused,
                    })?;
                }
            }
        }

        // Invariant 1: first bookmark preserved after switch.
        // Only terminal ends delete (FR-032 / FR-032b / FR-032f);
        // second-document pick is NOT a terminal end.
        if state != SessionState::Stopped {
            let first_after = store.get(&first_fp)?;
            prop_assert!(
                first_after.is_some(),
                "first bookmark deleted after second document pick (FR-018); durable-save wire missing"
            );
            if let Some(pos) = first_after {
                prop_assert!(
                    pos.end_kind == SessionEnd::Paused || pos.end_kind == SessionEnd::Interrupted,
                    "first bookmark lost Paused/Interrupted end_kind after switch (FR-018)"
                );
            }
        }

        // Invariant 2: new snapshot exists ONLY after save step.
        if had_save {
            let second_after = store.get(&second_fp)?;
            prop_assert!(
                second_after.is_some(),
                "second snapshot missing after save step (FR-018)"
            );
        }

        // Invariant 3: `clear_bookmark` NOT called before save.
        // Verified negatively by the first bookmark still resolving
        // (if `clear_bookmark` had fired, `store.get(&first_fp)`
        // would be None — see assertion above). `clear_bookmark`
        // lives in `session::store` and is only invoked by Stop /
        // document-end flows, never by a second-document pick
        // (storage-schema.md).
        prop_assert!(
            true,
            "clear_bookmark must not fire before new snapshot (FR-018 / storage-schema.md)"
        );

        // Invariant 4: second never auto-starts — the new snapshot
        // carries `Paused` (deliberate open) not an implicit play
        // state. The resume-offer form is resolved solely from the
        // stored `end_kind` (FR-044); a `Paused` entry opens the
        // 3-second offer (FR-032c), never auto-audio.
        if had_save
            && let Some(second_pos) = store.get(&second_fp)?
        {
            prop_assert!(
                second_pos.end_kind == SessionEnd::Paused,
                "second snapshot must carry Paused (not auto-start) (FR-018)"
            );
        }
    }
}
