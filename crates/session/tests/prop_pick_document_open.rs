//! T163 — Property-based test: unread document ingest always yields auto-play (StartNow).
//!
//! Invariant:
//! For any valid non-empty document bytes (valid UTF-8, size ≤ 93636 bytes) with no prior
//! stored bookmark, `cold_start(lookup.offer)` evaluates to `ColdStart::StartNow`,
//! starting at the first spoken position with no offer dialog, and computes the exact
//! Blake3 fingerprint of the input bytes.

use proptest::prelude::*;
use session::fingerprint::fingerprint;
use session::offer::{ColdStart, OpenOffer, cold_start};
use session::store::{BookmarkStore, lookup_on_open};
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
        let mut store = BookmarkStore::open(&store_path).expect("store opens cleanly");

        // Verify store has no prior bookmark for this fingerprint
        let prior = store.get(&fp).expect("store read succeeds");
        prop_assert!(prior.is_none(), "Unread document must have no prior stored row");

        // Lookup on open with empty store yields OpenOffer::NoOffer
        let lookup = lookup_on_open(&mut store, bytes).expect("lookup_on_open succeeds");
        prop_assert_eq!(lookup.offer, OpenOffer::NoOffer);
        prop_assert_eq!(lookup.fingerprint, fp);

        // Cold start decision maps OpenOffer::NoOffer directly to ColdStart::StartNow
        let start_decision = cold_start(lookup.offer);
        prop_assert_eq!(start_decision, ColdStart::StartNow, "Unread document must map to ColdStart::StartNow (auto-play)");
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
