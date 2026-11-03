//! T204 — Property-based test for directory resume invariants (`prop_directory_resume`).
//!
//! Invariants under test (tasks.md T204, data-model.md §2, spec.md FR-007 / FR-008 / FR-009):
//! 1. Deliberate pause stores `(chapter_index, offset_secs, ts, Paused)` in `DIRECTORY_BOOKMARKS`
//!    and `lookup_directory_on_open` routes to `ColdStart::AwaitOffer` with `offer_route: "in_app_dialog"`.
//! 2. Interrupted / hard kill stores `(chapter_index, offset_secs, ts, Interrupted)` and
//!    `lookup_directory_on_open` resumes immediately (`ColdStart::StartNow`, `offer_route: "none"`)
//!    without prompt while preserving the stored `CompositeBookmark` for immediate restoration.
//! 3. Atomic chapter transition (`persist_chapter_transition`, T206): completing chapter `N` and
//!    advancing to `N+1` atomically clears any single-document bookmark for chapter `N` and
//!    persists `(N+1, 0.0, ts, Interrupted)` in `DIRECTORY_BOOKMARKS`.

use engine_core::closed::SessionEnd;
use proptest::prelude::*;
use session::fingerprint::{compute_collection_fingerprint, fingerprint};
use session::offer::{ColdStart, OpenOffer};
use session::store::{
    BookmarkStore, CompositeBookmark, StoredPosition, lookup_directory_on_open,
};
use tempfile::tempdir;

fn arb_chapter_hashes() -> impl Strategy<Value = Vec<[u8; 32]>> {
    prop::collection::vec(prop::array::uniform32(any::<u8>()), 2..=12)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_deliberate_pause_routes_to_await_offer_and_in_app_dialog(
        hashes in arb_chapter_hashes(),
        chapter_offset_ratio in 0usize..12usize,
        quarter_steps in 1u32..14_400u32,
        saved_ts_ms in 1_000_000u64..2_000_000_000_000u64,
    ) {
        let total = hashes.len();
        let chapter_index = u32::try_from(chapter_offset_ratio % total).unwrap_or(0);
        let offset_secs = f64::from(quarter_steps) * 0.25;
        let collection_fp = compute_collection_fingerprint(&hashes);

        let dir = tempdir()?;
        let store_path = dir.path().join("prop_dir_pause.redb");
        let store = BookmarkStore::open(&store_path)?;

        let bookmark = CompositeBookmark {
            collection_fingerprint: collection_fp,
            chapter_index,
            offset_secs,
            saved_ts_ms,
            end_kind: SessionEnd::Paused,
        };
        store.put_directory_bookmark(&bookmark)?;

        let lookup = lookup_directory_on_open(&store, &hashes)?;
        prop_assert_eq!(lookup.collection_fingerprint, collection_fp);
        prop_assert_eq!(lookup.stored, Some(bookmark));
        prop_assert_eq!(
            lookup.offer,
            OpenOffer::Continue {
                offset_secs,
                first_yield_ms: saved_ts_ms,
            }
        );
        prop_assert_eq!(
            lookup.cold_start,
            ColdStart::AwaitOffer {
                offset_secs,
                first_yield_ms: saved_ts_ms,
            }
        );
        prop_assert_eq!(lookup.offer_route, "in_app_dialog");
    }

    #[test]
    fn prop_interrupted_kill_resumes_immediately_without_prompt(
        hashes in arb_chapter_hashes(),
        chapter_offset_ratio in 0usize..12usize,
        quarter_steps in 1u32..14_400u32,
        saved_ts_ms in 1_000_000u64..2_000_000_000_000u64,
    ) {
        let total = hashes.len();
        let chapter_index = u32::try_from(chapter_offset_ratio % total).unwrap_or(0);
        let offset_secs = f64::from(quarter_steps) * 0.25;
        let collection_fp = compute_collection_fingerprint(&hashes);

        let dir = tempdir()?;
        let store_path = dir.path().join("prop_dir_interrupted.redb");
        let store = BookmarkStore::open(&store_path)?;

        let bookmark = CompositeBookmark {
            collection_fingerprint: collection_fp,
            chapter_index,
            offset_secs,
            saved_ts_ms,
            end_kind: SessionEnd::Interrupted,
        };
        store.put_directory_bookmark(&bookmark)?;

        let lookup = lookup_directory_on_open(&store, &hashes)?;
        prop_assert_eq!(lookup.collection_fingerprint, collection_fp);
        prop_assert_eq!(lookup.stored, Some(bookmark));
        prop_assert_eq!(lookup.cold_start, ColdStart::StartNow);
        prop_assert_eq!(lookup.offer_route, "none");
    }

    #[test]
    fn prop_atomic_chapter_transition_persists_next_chapter_at_zero(
        hashes in arb_chapter_hashes(),
        completed_idx_ratio in 0usize..10usize,
        prior_quarter_steps in 4u32..2_000u32,
        transition_ts_ms in 1_000_000u64..2_000_000_000_000u64,
    ) {
        let total = hashes.len();
        let completed_idx = completed_idx_ratio % (total - 1);
        let next_idx = u32::try_from(completed_idx + 1).unwrap_or(1);
        let completed_fp = hashes[completed_idx];
        let collection_fp = compute_collection_fingerprint(&hashes);

        let dir = tempdir()?;
        let store_path = dir.path().join("prop_dir_transition.redb");
        let mut store = BookmarkStore::open(&store_path)?;

        // Seed a prior single-document bookmark for the chapter being completed
        // and a prior directory bookmark on `completed_idx`.
        store.put(
            &completed_fp,
            StoredPosition {
                offset_secs: f64::from(prior_quarter_steps) * 0.25,
                first_yield_ms: transition_ts_ms.saturating_sub(5_000),
                end_kind: SessionEnd::Interrupted,
            },
        )?;
        store.put_directory_bookmark(&CompositeBookmark {
            collection_fingerprint: collection_fp,
            chapter_index: u32::try_from(completed_idx).unwrap_or(0),
            offset_secs: f64::from(prior_quarter_steps) * 0.25,
            saved_ts_ms: transition_ts_ms.saturating_sub(5_000),
            end_kind: SessionEnd::Interrupted,
        })?;

        // Execute atomic chapter transition (T206)
        store.persist_chapter_transition(
            &collection_fp,
            &completed_fp,
            next_idx,
            transition_ts_ms,
        )?;

        // Completed chapter's single bookmark is cleared (Completed deletes per storage-schema.md)
        let single_after = store.get(&completed_fp)?;
        prop_assert_eq!(single_after, None);

        // Directory bookmark atomically advances to next_idx at 0.0s
        let dir_after = store.get_directory_bookmark(&collection_fp)?;
        let expected = CompositeBookmark {
            collection_fingerprint: collection_fp,
            chapter_index: next_idx,
            offset_secs: 0.0,
            saved_ts_ms: transition_ts_ms,
            end_kind: SessionEnd::Interrupted,
        };
        prop_assert_eq!(dir_after, Some(expected));

        // Verify content-addressed invariance: single-byte change in any chapter hash misses
        let mut modified_hashes = hashes;
        modified_hashes[0] = fingerprint(b"modified chapter content");
        if compute_collection_fingerprint(&modified_hashes) != collection_fp {
            let miss_lookup = lookup_directory_on_open(&store, &modified_hashes)?;
            prop_assert_eq!(miss_lookup.stored, None);
            prop_assert_eq!(miss_lookup.cold_start, ColdStart::StartNow);
            prop_assert_eq!(miss_lookup.offer_route, "none");
        }
    }
}
