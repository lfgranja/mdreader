//! T012 — blake3 fingerprint stability property tests (Phase 2, BLOCKING).
//!
//! Invariants under test (tasks.md T012, verbatim): "same bytes → same 32-byte
//! hash regardless of path; 1-bit content change → different hash; identical
//! bytes at two paths share position"
//!
//! Spec anchors: FR-036 ("What uniquely identifies a document for the purposes
//! of remembering where the reader stopped? → A fingerprint derived from the
//! document's content itself, so that a stored position is still found after
//! the file is moved or renamed"), FR-036a ("A changed document is a different
//! document. Because FR-036 keys the position by content, an edited file's
//! fingerprint no longer matches, the stored position becomes unreachable, and
//! no resume is offered"), and contracts/storage-schema.md ("`bookmarks`: key
//! `[u8; 32]` (blake3-256 of document bytes)"; "Identical bytes at any path
//! share the fingerprint and the position (accepted, FR-036)").
//!
//! FAILS before `crates/session/src/fingerprint.rs` exists (T059): the
//! `session::fingerprint` module is unresolved, so this test target does not
//! build. That build failure is the required pre-implementation red state.
//!
//! Expected API contract for T059 (`crates/session/src/fingerprint.rs`):
//! - `pub fn fingerprint(content: &[u8]) -> [u8; 32]` — the blake3-256 digest
//!   of the document bytes. A pure function of content: never of path, file
//!   name, or metadata, so a moved or renamed document resolves to the same
//!   bookmark key and therefore the same stored position (FR-036), while any
//!   edited byte yields a different key, making the old position unreachable
//!   (FR-036a).

use proptest::prelude::*;

/// Document bytes up to 4 KiB — enough to exercise multi-block blake3 input
/// while keeping the generated cases fast.
fn document_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..=4096)
}

/// Document bytes plus the index of exactly one bit to flip. The document is
/// never empty (a 1-bit change needs a bit to change) and the bit index is
/// always in range, so no case needs to be skipped.
fn document_with_bit_flip() -> impl Strategy<Value = (Vec<u8>, usize)> {
    proptest::collection::vec(any::<u8>(), 1..=4096).prop_flat_map(|bytes| {
        let bit_count = bytes.len() * 8;
        (Just(bytes), 0usize..bit_count)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn same_bytes_produce_the_same_32_byte_hash(content in document_strategy()) {
        // Invariant: "same bytes → same 32-byte hash regardless of path".
        // The fingerprint is a function of content alone, so hashing the same
        // bytes twice — as happens when a document is read again after a move
        // or rename — yields the identical digest.
        let first = session::fingerprint(&content);
        let second = session::fingerprint(&content);
        prop_assert_eq!(
            first,
            second,
            "identical bytes produced different fingerprints"
        );
        prop_assert_eq!(
            first.len(),
            32,
            "fingerprint is not a blake3-256 (32-byte) digest"
        );
    }

    #[test]
    fn a_one_bit_content_change_produces_a_different_hash(
        (mut content, bit_index) in document_with_bit_flip()
    ) {
        // Invariant: "1-bit content change → different hash" (FR-036a: an
        // edited document is a different document — its fingerprint no longer
        // matches, so the stored position becomes unreachable and no resume
        // is offered for the changed content).
        let original = session::fingerprint(&content);
        content[bit_index / 8] ^= 1 << (bit_index % 8);
        let edited = session::fingerprint(&content);
        prop_assert_ne!(
            original,
            edited,
            "a 1-bit content change left the fingerprint unchanged"
        );
    }
}

/// Invariant: "identical bytes at two paths share position" — the fingerprint
/// is keyed by content, so the same bytes at two different paths (a document
/// moved or renamed on the device, FR-036) resolve to the same bookmark key,
/// and a position stored under one path is found under the other. The bookmark
/// key IS the fingerprint (contracts/storage-schema.md), so key equality is
/// position sharing.
#[test]
fn identical_bytes_at_two_paths_share_the_stored_position() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempfile::tempdir()?;
    let original_path = dir.path().join("comute-estrada.md");
    let renamed_path = dir.path().join("renamed-on-the-device.md");

    let content: &[u8] = b"# Trajeto para o trabalho\n\nmesmo conteudo, outro caminho\n";
    std::fs::write(&original_path, content)?;
    std::fs::write(&renamed_path, content)?;

    let at_original = std::fs::read(&original_path)?;
    let at_renamed = std::fs::read(&renamed_path)?;

    let key_from_original = session::fingerprint(&at_original);
    let key_from_renamed = session::fingerprint(&at_renamed);

    assert_eq!(
        key_from_original, key_from_renamed,
        "identical bytes at two paths produced different bookmark keys"
    );
    assert_eq!(
        key_from_original.len(),
        32,
        "fingerprint is not a blake3-256 (32-byte) digest"
    );
    Ok(())
}

/// A moved document must not merely hash consistently — the digest must be
/// stable across reads of the same path too (the re-read after a rename is
/// the same bytes, so the key is unchanged; this guards against any
/// path-dependent behavior sneaking into the hash input).
#[test]
fn the_fingerprint_of_a_path_is_stable_across_repeated_reads()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("noticia.md");
    std::fs::write(&path, b"conteudo fixo\n")?;

    let first_read = std::fs::read(&path)?;
    let second_read = std::fs::read(&path)?;

    assert_eq!(
        session::fingerprint(&first_read),
        session::fingerprint(&second_read),
        "repeated reads of the same path produced different fingerprints"
    );
    Ok(())
}
