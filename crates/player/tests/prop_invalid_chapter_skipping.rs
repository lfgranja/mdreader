//! T212 — Property test: invalid chapter skipping and byte budget (FR-013).
//!
//! Empty, oversize (> `MAX_DOC_BYTES`) and non-UTF-8 chapters are marked
//! `ChapterStatus::Skipped`, map to `FormatUndeliverable`, and never block or
//! alter subsequent valid chapters.

use player::interruption::{InterruptionReason, MAX_DOC_BYTES};
use player::queue::{
    ChapterItem, ChapterQueue, ChapterRefusal, ChapterStatus, MAX_CHAPTER_BYTES,
    validate_chapter_bytes,
};
use proptest::prelude::*;

fn item(name: &str) -> ChapterItem {
    ChapterItem {
        filename: name.to_owned(),
        file_uri: format!("content://vault/{name}"),
        relative_path: name.to_owned(),
        byte_size: 1,
        content_fingerprint: [0; 32],
        playback_status: ChapterStatus::Pending,
    }
}

#[test]
fn budget_is_the_single_doc_cap() {
    assert_eq!(MAX_CHAPTER_BYTES, MAX_DOC_BYTES);
    assert_eq!(MAX_CHAPTER_BYTES, 93_636);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validation_matches_budget_and_utf8(size in 0usize..200_000, bad_utf8 in any::<bool>()) {
        let mut data = vec![b'a'; size];
        if bad_utf8 && let Some(first) = data.first_mut() {
            *first = 0xFF;
        }
        let expected = if size == 0 {
            Some(ChapterRefusal::Empty)
        } else if size > MAX_CHAPTER_BYTES {
            Some(ChapterRefusal::Oversize)
        } else if bad_utf8 {
            Some(ChapterRefusal::InvalidUtf8)
        } else {
            None
        };
        match (validate_chapter_bytes(&data), expected) {
            (Ok(text), None) => prop_assert_eq!(text.len(), size),
            (Err(got), Some(want)) => {
                prop_assert_eq!(got, want);
                prop_assert_eq!(got.reason(), InterruptionReason::FormatUndeliverable);
            }
            (got, want) => prop_assert!(false, "got {:?}, wanted {:?}", got, want),
        }
    }

    #[test]
    fn invalid_chapter_is_skipped_and_later_valid_chapters_unaffected(
        size in 0usize..200_000,
        bad_utf8 in any::<bool>(),
    ) {
        let mut bad = vec![b'a'; size];
        if bad_utf8 && let Some(first) = bad.first_mut() {
            *first = 0xFF;
        }
        let refused = size == 0 || size > MAX_CHAPTER_BYTES || bad_utf8;
        let mut queue = ChapterQueue::new(vec![item("01.md"), item("02.md"), item("03.md")]);

        let first = queue.admit_chapter_bytes(0, &bad);
        let second = queue.admit_chapter_bytes(1, b"valid chapter");
        let third = queue.admit_chapter_bytes(2, "ação válida".as_bytes());

        prop_assert_eq!(first.is_err(), refused);
        prop_assert!(second.is_ok());
        prop_assert!(third.is_ok());
        let statuses: Vec<_> = queue.items().iter().map(|c| c.playback_status).collect();
        let want_first = if refused { ChapterStatus::Skipped } else { ChapterStatus::Pending };
        prop_assert_eq!(statuses, vec![want_first, ChapterStatus::Pending, ChapterStatus::Pending]);
    }
}
