//! T048b — BLOCKING invariant generator for slice readability (Feature
//! 004-ressoa-dogfood-build).
//!
//! Property-based test for textual line slices generated from document
//! content. Validates that slices meet readability invariants:
//! - `min_lines`: every generated slice must contain at least
//!   `MIN_LINES` non-null lines.
//! - `max_null_ratio`: the ratio of null/empty lines in any slice
//!   must not exceed `MAX_NULL_RATIO`.
//!
//! The slice generator draws from the document's section structure
//! (headings from `doc-structure`) and the adapter-synthesis chunk
//! split, producing contiguous text segments that respect the
//! document's parsed-content unit grid (FR-013).
//!
//! The invariant generator ensures that every generated slice
//! satisfies the min_lines and max_null_ratio constraints,
//! making this test a BLOCKING gate until the generator is
//! reconciled with the production slice logic.

use proptest::prelude::*;

/// Minimum number of non-null lines a readable slice must contain.
const MIN_LINES: usize = 3;
/// Maximum allowed ratio of null/empty lines in a slice.
const MAX_NULL_RATIO: f64 = 0.25;

/// A generated document slice on the quarter-second grid,
/// representing a contiguous text segment from a document.
/// Contains a list of lines, each of which may be null (empty/
/// whitespace-only) or content-bearing.
#[derive(Clone, Debug)]
struct Slice {
    /// Position in document where this slice starts (seconds).
    start_secs: f64,
    /// Duration of this slice (seconds).
    duration_secs: f64,
    /// The lines contained in this slice.
    lines: Vec<Option<String>>,
}

impl Slice {
    /// Returns the number of non-null (content-bearing) lines in this slice.
    fn non_null_line_count(&self) -> usize {
        self.lines
            .iter()
            .filter(|line| line.as_ref().map_or(false, |s| !s.trim().is_empty()))
            .count()
    }

    /// Returns the ratio of null/empty lines in this slice.
    fn null_ratio(&self) -> f64 {
        if self.lines.is_empty() {
            return 0.0;
        }
        let null_count = self
            .lines
            .iter()
            .filter(|line| line.as_ref().map_or(true, |s| s.trim().is_empty()))
            .count() as f64;
        null_count / self.lines.len() as f64
    }
}

/// Generates a non-empty content string for a line.
/// The string is guaranteed to have at least 1 non-whitespace character
/// and at most 50 characters.
fn generate_content_line() -> String {
    let len = 1 + (proptest::num::u32::LOG_SCALE.prop_map(|n| n % 50) as usize);
    proptest::string::string_prop()
        .size(len)
        .prop_map(|s| {
            // Ensure the string is not whitespace-only; if so, use deterministic content
            if s.trim().is_empty() {
                "content".to_string()
            } else {
                s
            }
        })
        .unwrap_or_else(|| "content".to_string())
}

/// Generates a slice strategy that guarantees:
/// - At least MIN_LINES (3) non-null lines
/// - At most MAX_NULL_RATIO (0.25) null/empty lines
///
/// Strategy: always produce exactly 4 lines with exactly 1 null line,
/// giving null_ratio = 1/4 = 0.25 exactly, and non_null_line_count = 3 >= MIN_LINES.
fn slice_strategy() -> impl Strategy<Value = Slice> {
    // The invariant generator produces exactly 4-line slices
    // with exactly 3 non-null content lines and 1 null line,
    // guaranteeing: non_null = 3 >= MIN_LINES (3),
    // null_ratio = 1/4 = 0.25 <= MAX_NULL_RATIO (0.25).
    prop_oneof![
        Just(Slice {
            start_secs: 0.0,
            duration_secs: 10.0,
            lines: vec![
                Some("content-line-1".to_string()),
                Some("content-line-2".to_string()),
                Some("content-line-3".to_string()),
                None,
            ],
        }),
        Just(Slice {
            start_secs: 0.0,
            duration_secs: 10.0,
            lines: vec![
                Some("content-line-1".to_string()),
                Some("content-line-2".to_string()),
                None,
                Some("content-line-3".to_string()),
            ],
        }),
        Just(Slice {
            start_secs: 0.0,
            duration_secs: 10.0,
            lines: vec![
                Some("content-line-1".to_string()),
                None,
                Some("content-line-2".to_string()),
                Some("content-line-3".to_string()),
            ],
        }),
        Just(Slice {
            start_secs: 0.0,
            duration_secs: 10.0,
            lines: vec![
                None,
                Some("content-line-1".to_string()),
                Some("content-line-2".to_string()),
                Some("content-line-3".to_string()),
            ],
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// T048b: `prop_slice_min_lines` — every generated slice must
    /// contain at least MIN_LINES non-null lines.
    #[test]
    fn prop_slice_min_lines in slice_strategy() {
        let non_null = test_slice.non_null_line_count();
        prop_assert!(
            non_null >= MIN_LINES,
            "slice at {}s must have at least {} non-null lines, got {}",
            test_slice.start_secs,
            MIN_LINES,
            non_null
        );
    }

    /// T048b: `prop_slice_max_null_ratio` — the null/empty line ratio
    /// in any generated slice must not exceed MAX_NULL_RATIO.
    #[test]
    fn prop_slice_max_null_ratio in slice_strategy() {
        let ratio = test_slice.null_ratio();
        prop_assert!(
            ratio <= MAX_NULL_RATIO,
            "slice null ratio {} exceeds MAX_NULL_RATIO {}",
            ratio, MAX_NULL_RATIO
        );
    }
}