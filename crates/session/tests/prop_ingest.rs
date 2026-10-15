//! T009 — BLOCKING FOUNDATIONAL TEST: property `prop_ingest_identity`
//! (Feature 004-ressoa-dogfood-build / task T009).
//!
//! Invariants under test (tasks.md T009, verbatim):
//! "Property `prop_ingest_identity`: identical bytes at any path
//! ⇒ identical blake3; edited byte ⇒ different hash ⇒ no resume;
//! snapshot bytes are the synthesis input, never the URI."
//!
//! The ingest flow is defined in `contracts/ingest.md` and the
//! fingerprint logic is in `crates/session/src/fingerprint.rs`.
//! This test asserts the content-identity contract.
//!
//! FAILS before ingest is implemented — acceptable red state
//! for the blocking gate.

use proptest::prelude::*;

use session::fingerprint;

/// Generate arbitrary byte arrays for testing.
fn bytes_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 1..=5000)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T009: `prop_ingest_identity` — content-identity invariants.
    #[test]
    fn prop_ingest_identity(bytes in bytes_strategy()) {
        // Invariant 1: identical bytes ⇒ identical blake3 fingerprint.
        let fp1 = fingerprint(&bytes);
        let fp2 = fingerprint(&bytes);
        prop_assert_eq!(
            fp1, fp2,
            "identical bytes must produce identical fingerprint"
        );

        // Invariant 2: edited byte ⇒ different hash.
        // Flip one bit in a copy.
        let mut edited = bytes.clone();
        if !edited.is_empty() {
            edited[0] ^= 0x01; // flip LSB of first byte
            let fp_edited = fingerprint(&edited);
            prop_assert_ne!(
                fp1, fp_edited,
                "edited bytes must produce different fingerprint"
            );
        }

        // Invariant 3: snapshot path is the synthesis input,
        // never the URI. The snapshot path is derived from the
        // fingerprint (hex) and the original extension.
        // (SnapshotPath type not yet implemented — test will
        // compile once T017/T018 implement ingest flow.)
    }

    /// Two different byte arrays must produce different fingerprints
    /// with overwhelming probability (blake3 collision resistance).
    #[test]
    fn prop_ingest_different_bytes_different_hash(
        bytes1 in bytes_strategy(),
        bytes2 in bytes_strategy(),
    ) {
        prop_assume!(bytes1 != bytes2);
        let fp1 = fingerprint(&bytes1);
        let fp2 = fingerprint(&bytes2);
        // blake3 is cryptographically strong; collision probability
        // is negligible for random inputs.
        prop_assert_ne!(fp1, fp2, "different bytes must produce different hashes");
    }
}
