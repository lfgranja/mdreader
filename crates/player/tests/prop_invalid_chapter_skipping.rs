//! T212 — Property-based test (TDD RED): invalid chapter skipping and byte budget validation.
//!
//! Invariants (spec.md FR-013, tasks.md T212, contracts/directory-playback.md §1.2):
//! 1. Files > 93_636 bytes (`MAX_DOC_BYTES`) are marked `ChapterStatus::Skipped`.
//! 2. Files with 0 bytes are marked `ChapterStatus::Skipped`.
//! 3. Files with invalid UTF-8 are marked `ChapterStatus::Skipped`.
//! 4. Valid files before and after invalid ones continue to play without transport crash or abort.

use player::queue::{validate_chapter_bytes, ChapterItem, ChapterStatus, MAX_CHAPTER_BYTES};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_empty_or_oversized_or_invalid_utf8_is_skipped(
        size in 0usize..200_000usize,
        is_invalid_utf8 in any::<bool>(),
    ) {
        let is_valid_size = size > 0 && size <= MAX_CHAPTER_BYTES;
        let mut data = vec![b'a'; size];
        if is_invalid_utf8 && !data.is_empty() {
            data[0] = 0xFF; // Invalid UTF-8 lead byte
        }

        let result = validate_chapter_bytes(&data);
        if !is_valid_size || (is_invalid_utf8 && !data.is_empty()) {
            prop_assert!(result.is_err(), "Invalid chapter data must be rejected as an error");
        } else {
            prop_assert!(result.is_ok(), "Valid chapter data must be accepted");
        }
    }
}
