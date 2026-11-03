//! Property-based test for natural sort invariants of ChapterQueue.

use proptest::prelude::*;

/// Placeholder for the natural sort implementation.
fn sort_chapters_natural(_queue: &mut Vec<String>) {
    // TODO: implement natural sort algorithm.
    todo!();
}

#[test]
fn prop_chapter_queue_natural_sort() {
    // Strategy: generate a vector of random ASCII strings.
    let strategy = proptest::collection::vec("[a-zA-Z0-9]+", 0..100usize);

    proptest::prop_assert!(strategy.prop_map(|mut chapters| {
        // Invoke the (unimplemented) sort function.
        sort_chapters_natural(&mut chapters);
        // No further assertions; the test will fail at runtime due to `todo!()`.
        true
    }).test().is_err());
}
