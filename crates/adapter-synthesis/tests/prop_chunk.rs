//! Property tests for `crates/adapter-synthesis` task **T019** — the
//! byte-budget chunk split (`src/chunk.rs`).
//!
//! Spec anchors: **FR-024a** (adapter splits Markdown into ≤ 5000-byte
//! chunks at paragraph/heading boundaries before synthesis), data-model
//! rules **R3** (`DocumentSegment.byte_length <= 5000`, measured in
//! bytes, not characters), **R4** (the adapter owns the split; the
//! engine never resamples or splits independently), **R6** (resume at
//! the nearest completed `Paragraph`/`Heading` boundary — no
//! `Sentence` member at v1), checklist **CHK016** (adapter-interface:
//! `byte_length <= 5000` is a hard bound, split at paragraph/heading),
//! and **CHK001/CHK002/CHK008** (segment-resume: splitting fully
//! specified, `DocumentSegment` fields, the 5000-byte limit is a hard
//! upper bound, not a target). All invariants are normative per
//! **`API_CONTRACT` §2** (frozen interface, §2.1 rules 1–12) and §0
//! global rules.
//!
//! These tests bind to the frozen contract, not to the current stub:
//! the implementation does not exist yet (rule 1 — tests BLOCK the
//! core wave), so a red run is the expected state. They exercise the
//! *real* crate surface
//! (`adapter_synthesis::chunk::{split_document, split_document_within,
//! DocumentSegment, BoundaryKind, SourceRange, MAX_CHUNK_BYTES}`); no
//! local re-declaration of any contract type and no local split
//! implementation (the F1/F2 root cause), and no
//! `unwrap`/`expect`/`panic!`/`println!` — all denied by the
//! workspace lint config, in tests too (`constitution.md` Rust Code
//! Rigor).
//!
//! The proptest generator deliberately composes documents that exceed
//! the 5000-byte budget (2..40 blocks of 0..1200 bytes each,
//! multi-byte words, blank-line separated), because a property that
//! only ever observes sub-budget documents is vacuous — exactly the
//! defect being fixed here (the old file mapped `lines()` locally and
//! could never approach the budget, and measured bytes as if they were
//! characters).

use proptest::prelude::*;

use adapter_synthesis::chunk::{
    BoundaryKind, DocumentSegment, MAX_CHUNK_BYTES, SourceRange, split_document,
    split_document_within,
};

// The hard budget is 5000 UTF-8 **bytes** (`constitution.md:353`,
// R3, CHK008, API_CONTRACT §2). Not 5000 characters, and not
// a target.
//
// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block, so `///` here would raise
// `unused_doc_comments`, which verification task V2 promotes
// to an error via `-D warnings`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]
    #[test]
    fn max_chunk_bytes_is_exactly_5000(() in any::<()>()) {
        prop_assert_eq!(MAX_CHUNK_BYTES, 5_000);
    }
}

// Byte budget holds unconditionally: every emitted segment's
// `byte_length()` is `<= MAX_CHUNK_BYTES`, for arbitrary
// headings/paragraphs/blank runs built from multi-byte text (API_CONTRACT
// §2.1 rule 5). The generator composes 2..40 blocks of 0..1200 bytes
// each — a plausible mid-sized Portuguese document — so cases routinely
// exceed 5000 bytes and the property is not vacuous.
//
// Also defended, per case: `byte_length()` equals the content's byte
// count (rule 2 of this task); 0-based strictly-increasing
// `segment_index` (§2.1 rule 9); 1-based ordered non-overlapping
// `source_range` with `contains_line` (§2.1 rule 10);
// `resume_point == Some(source_range)` (§2.1 rule 11, R6); losslessness
// (§2.1 rule 6); the closed two-member `boundary_kind` set (no
// `Sentence`, R6 / Principle VII); determinism (§2.1 rule 7); and
// `document_hash` accepted but not stored (§2.1 rule 12, R5).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn every_segment_stays_within_the_5000_byte_budget(document in document_strategy()) {
        let segments: Vec<DocumentSegment> = split_document(&document, "budget-hash-a");

        // Rule 5 — the budget is a hard bound, never a target, and
        // `byte_length()` is derived from the content's byte count.
        for segment in &segments {
            prop_assert!(
                segment.byte_length() <= MAX_CHUNK_BYTES,
                "segment {index} is {length} bytes, over the {budget}-byte budget",
                index = segment.segment_index,
                length = segment.byte_length(),
                budget = MAX_CHUNK_BYTES
            );
            prop_assert_eq!(
                segment.byte_length(),
                segment.content.len()
            );
        }

        // Rule 9 — 0-based, strictly increasing by exactly 1.
        for (position, segment) in segments.iter().enumerate() {
            prop_assert_eq!(
                segment.segment_index,
                position as u64,
                "segment_index at vector position {position} is not 0-based sequential",
                position = position
            );
        }

        // Rule 10 — 1-based inclusive lines, ordered, non-overlapping,
        // and self-consistent with `contains_line`.
        let mut previous: Option<SourceRange> = None;
        for segment in &segments {
            let range = segment.source_range;
            prop_assert!(range.line_start >= 1, "source_range is not 1-based");
            prop_assert!(
                range.line_end >= range.line_start,
                "line_end precedes line_start"
            );
            prop_assert!(
                range.contains_line(range.line_start),
                "range does not contain its own line_start"
            );
            if let Some(prior) = previous {
                prop_assert!(
                    range.line_start >= prior.line_start,
                    "source ranges are not non-decreasing in line order"
                );
                // Non-overlapping: with non-decreasing starts, this
                // range must begin strictly after the prior one ends
                // (inclusive bounds), so the two share no line.
                prop_assert!(
                    range.line_start > prior.line_end,
                    "source ranges overlap"
                );
            }
            previous = Some(range);
        }

        // Rule 11 / R6 — every segment carries its resume boundary.
        for segment in &segments {
            prop_assert_eq!(
                segment.resume_point,
                Some(segment.source_range),
                "resume_point is not Some(source_range)"
            );
        }

        // Closed set (Principle VII / R6): an exhaustive match with no
        // `_ =>` arm, so a third `BoundaryKind` member (e.g.
        // `Sentence`) would not compile.
        for segment in &segments {
            match segment.boundary_kind {
                BoundaryKind::Paragraph | BoundaryKind::Heading => {}
            }
        }

        // Rule 6 — losslessness: every non-blank source
        // line is reproduced, in order, none dropped,
        // duplicated, or reordered. Whitespace is
        // squashed before comparison because segment
        // contents are trimmed (§2.1 rule 4) and
        // forced sub-pieces may cut through interior
        // whitespace (§2.1 rule 3) — the same
        // measurement as `assert_lossless` in
        // `prop_chunk_refined.rs` and
        // `prop_segmentation_shapes.rs`. The padded
        // generator stays: it keeps the byte budget
        // and losslessness exercised together, and any
        // dropped, duplicated, or reordered line still
        // changes the squashed sequence.
        let source_squashed: String =
            document.chars().filter(|ch| !ch.is_whitespace()).collect();
        let reassembled: String = segments
            .iter()
            .map(|segment| segment.content.as_str())
            .collect();
        let produced_squashed: String = reassembled
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        prop_assert_eq!(
            produced_squashed, source_squashed,
            "segmentation is not lossless over non-blank lines"
        );

        // Rule 7 — determinism: same input, same vector (all fields
        // compared; `DocumentSegment` derives `PartialEq`).
        let repeated = split_document(&document, "budget-hash-a");
        prop_assert_eq!(&repeated, &segments, "split is not deterministic");

        // Rule 12 / R5 — the document hash is accepted but never stored
        // on the segment: a distinct hash per call leaves the emitted
        // segments untouched (the `blake3` fingerprint is owned by
        // `session`, Assumption 5).
        let other = split_document(&document, "budget-hash-b");
        prop_assert_eq!(
            other, segments,
            "document_hash leaked into the emitted segments"
        );
    }
}

// `byte_length()` measures UTF-8 **bytes**, never characters
// (`constitution.md:353-356`: chunked "measured in **bytes** rather
// than characters"). Accented Portuguese (`ã`, `ç`, `õ`, `á`, `ê`)
// encodes to 2 bytes per char in UTF-8, so `char_count < byte_count`
// strictly — an implementation that counts characters would pass a
// char-based budget check and must fail this one.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]
    #[test]
    fn byte_length_counts_bytes_not_chars_on_multibyte_input(() in any::<()>()) {
        let document = "### Título com acentos\n\nAção, çães e ãos: áêíóú, çãõ. Portuguese mid-sized paragraph ãããõõõççç.\n\nPróxima seção: mais caracteres multi-byte para medir bytes, não caracteres.\n";
        let segments = split_document(document, "multibyte-hash");
        let mut saw_multibyte = false;
        for segment in &segments {
            let byte_count = segment.content.len();
            let char_count = segment.content.chars().count();
            prop_assert_eq!(
                segment.byte_length(),
                byte_count,
                "byte_length() disagrees with content.as_bytes().len()"
            );
            if char_count < byte_count {
                saw_multibyte = true;
            }
        }
        prop_assert!(
            saw_multibyte,
            "the multi-byte fixture did not exercise any char < byte segment"
        );
    }
}

// The same byte-vs-char distinction under an explicit small budget via
// `split_document_within` (API_CONTRACT §2): a single multi-byte
// paragraph far over the budget is hard-split into char-safe pieces,
// each still within the budget (§2.1 rule 3 — the budget is a hard
// bound, not a target).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]
    #[test]
    fn split_document_within_enforces_explicit_byte_budget(() in any::<()>()) {
        let word = "Ação_çação_ãõé"; // 13 chars, 23 bytes
        let mut document = String::new();
        for _ in 0..80 {
            document.push_str(word);
            document.push(' ');
        }
        let segments = split_document_within(&document, "small-budget-hash", 64);
        for segment in &segments {
            prop_assert!(
                segment.byte_length() <= 64,
                "piece is {} bytes, over the 64-byte budget",
                segment.byte_length()
            );
            prop_assert_eq!(segment.byte_length(), segment.content.len());
            prop_assert!(segment.content.chars().count() <= segment.byte_length());
        }
        prop_assert!(
            segments.len() > 1,
            "the 64-byte budget never split the multi-byte document"
        );
    }
}

// Determinism, restated as an explicit property: splitting the same
// input twice yields identical vectors including `source_range`,
// `boundary_kind`, `segment_index` and `resume_point` (API_CONTRACT
// §2.1 rule 7). The `document_hash` differs between the two calls,
// proving it is not part of the output (rule 12 / Assumption 5 / R5 —
// the `blake3` fingerprint is owned by `session`, not by the adapter).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn deterministic_split_is_identical_across_distinct_document_hashes(document in document_strategy()) {
        let first = split_document(&document, "determinism-hash-1");
        let second = split_document(&document, "determinism-hash-2");

        prop_assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter()) {
            prop_assert_eq!(left.source_range, right.source_range);
            prop_assert_eq!(left.boundary_kind, right.boundary_kind);
            prop_assert_eq!(left.segment_index, right.segment_index);
            prop_assert_eq!(left.resume_point, right.resume_point);
            prop_assert_eq!(&left.content, &right.content);
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Builds a document as a random number of blocks (2..40), each either a
/// heading line (`### …`) or a paragraph of random length (0..1200
/// bytes) made of multi-byte Portuguese words, separated by blank lines.
/// With up to 40 blocks averaging ~600 bytes, documents routinely reach
/// tens of kilobytes — far beyond `MAX_CHUNK_BYTES` — so the budget
/// property is exercised for real, never vacuously.
fn document_strategy() -> impl Strategy<Value = String> {
    // One heading block: a `###` line with accented text.
    let heading =
        any::<u32>().prop_map(|index| format!("### Seção {index} — título acentuado ãçõ"));
    // One paragraph block: 0..1200 bytes of multi-byte words. Each
    // `aãçõáê ` is 6 chars / 12 bytes, so byte length and char length
    // diverge throughout the document.
    let paragraph = (0u32..1200u32).prop_map(|byte_budget| {
        let words = byte_budget / 12;
        "aãçõáê ".repeat(words as usize)
    });
    // Headings are rarer than paragraphs (1:3).
    let block = prop_oneof![1 => heading, 3 => paragraph];

    proptest::collection::vec(block, 2usize..40usize).prop_map(|blocks| blocks.join("\n\n"))
}
