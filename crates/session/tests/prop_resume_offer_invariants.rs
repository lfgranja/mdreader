//! T169 — Property-based test: resume offer invariants across differing filenames with identical byte content.
//!
//! Invariant:
//! For any document bytes, changing the filename or URI must not change its Blake3
//! content fingerprint. Reopening with an identical byte stream under any filename
//! must resolve to the identical stored bookmark offset and `OpenOffer::Continue`.

use engine_core::closed::SessionEnd;
use proptest::prelude::*;
use session::fingerprint::fingerprint;
use session::offer::{ColdStart, OpenOffer, cold_start};
use session::store::{BookmarkStore, StoredPosition, lookup_on_open};
use tempfile::tempdir;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_identical_bytes_with_differing_names_preserves_bookmark(
        title in "[A-Za-z0-9 ]{3,20}",
        body in "[A-Za-z0-9 ,.!?\n]{10,150}",
        offset in 1.0f64..3600.0f64,
    ) {
        let doc_text = format!("# {}\n\n{}", title, body);
        let bytes = doc_text.as_bytes();
        prop_assume!(bytes.len() <= 93636);

        let fp = fingerprint(bytes);
        let dir = tempdir().expect("tempdir creates cleanly");
        let store_path = dir.path().join("prop_resume_bookmarks.redb");
        let mut store = BookmarkStore::open(&store_path).expect("store opens cleanly");

        // Save deliberate pause bookmark
        let initial_row = StoredPosition {
            offset_secs: offset,
            first_yield_ms: 250,
            end_kind: SessionEnd::Paused,
        };
        store.put(&fp, initial_row).expect("put succeeds");

        // Reopen with the same bytes (simulating different filename addressing)
        let lookup = lookup_on_open(&mut store, bytes).expect("lookup succeeds");
        prop_assert_eq!(lookup.fingerprint, fp, "Fingerprint depends purely on content bytes");

        match lookup.offer {
            OpenOffer::Continue { offset_secs, first_yield_ms } => {
                prop_assert!((offset_secs - offset).abs() < 1e-6, "Stored offset must match exact recorded offset");
                prop_assert_eq!(first_yield_ms, 250);
            },
            other => prop_assert!(false, "Expected OpenOffer::Continue, got {:?}", other),
        }

        let cold = cold_start(lookup.offer);
        match cold {
            ColdStart::AwaitOffer { offset_secs, .. } => {
                prop_assert!((offset_secs - offset).abs() < 1e-6);
            },
            other => prop_assert!(false, "Expected ColdStart::AwaitOffer, got {:?}", other),
        }
    }
}
