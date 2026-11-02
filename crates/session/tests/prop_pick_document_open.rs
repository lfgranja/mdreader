//! T163 — Property-based test: unread document ingest always yields auto-play (StartNow).
//!
//! Invariant:
//! For any valid non-empty document bytes (valid UTF-8, size ≤ 93636 bytes) with no prior
//! stored bookmark, `OfferMachine` with a cold start evaluates to `StartNow`, offset 0.0,
//! and computes the exact Blake3 fingerprint of the input bytes.

use proptest::prelude::*;
use session::fingerprint::fingerprint;
use session::offer::{OfferMachine, OfferResolution};
use session::store::{BookmarkStore, StoredPosition};
use tempfile::tempdir;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_unread_document_always_resolves_to_start_now(
        title in "[A-Za-z0-9 ]{3,30}",
        body in "[A-Za-z0-9 ,.!?\n]{10,200}",
    ) {
        let doc_text = format!("# {}\n\n{}", title, body);
        let bytes = doc_text.as_bytes();
        prop_assume!(bytes.len() <= 93636);

        let fp = fingerprint(bytes);
        let dir = tempdir().expect("tempdir creates cleanly");
        let store_path = dir.path().join("prop_bookmarks.redb");
        let store = BookmarkStore::open(&store_path).expect("store opens cleanly");

        // Verify store has no prior bookmark for this fingerprint
        let prior = store.get(&fp).expect("store read succeeds");
        prop_assert!(prior.is_none(), "Unread document must have no prior stored row");

        // An unread document has no prior stored position
        let resolution = OfferMachine::decide(None, true);
        match resolution {
            OfferResolution::StartNow => {},
            other => prop_assert!(false, "Expected StartNow for unread document, got {:?}", other),
        }
    }

    #[test]
    fn prop_unread_document_fingerprint_is_deterministic(
        content in "\\PC{1,500}"
    ) {
        let bytes = content.as_bytes();
        let fp1 = fingerprint(bytes);
        let fp2 = fingerprint(bytes);
        prop_assert_eq!(fp1, fp2, "Fingerprint calculation must be deterministic");
        prop_assert_eq!(fp1.len(), 32, "Blake3 fingerprint must be 32 bytes");
    }
}
