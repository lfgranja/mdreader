//! T019 (second half) — refined chunk-split properties for `chunk.rs`.
//!
//! Spec anchors: FR-024a (the adapter splits at paragraph/heading
//! boundaries; a sentence boundary inside a paragraph/heading yields
//! to the paragraph/heading boundary), FR-036 (document identity is
//! the `blake3` fingerprint owned by `session`), R3 (the 5 000-byte
//! budget is measured in bytes, not chars), R6 (resume snaps to the
//! nearest completed paragraph/heading boundary), CHK007 (the resume
//! boundary is the last boundary at or before the yield point),
//! CHK008 (the byte limit is a hard upper bound, not a target),
//! CHK016 (segment-boundary compliance is measurable), CHK029
//! (paragraph/heading precedence over sentence boundaries),
//! `API_CONTRACT.md` §2.1 rule 4 (each segment's content is
//! trimmed of leading and trailing whitespace, and a segment
//! whose trimmed content is empty is not emitted), and
//! `API_CONTRACT.md` §2 — the frozen interface:
//! `split_document(source, document_hash)` and
//! `split_document_within(source, document_hash, max_bytes)` over
//! `MAX_CHUNK_BYTES`, `BoundaryKind`, `SourceRange` and
//! `DocumentSegment`.
//!
//! Written before the implementation exists (implementation protocol
//! rule 1: tests block the implementation wave). This file binds to
//! the frozen contract, not to any current stub.

use adapter_synthesis::chunk::{
    BoundaryKind, DocumentSegment, MAX_CHUNK_BYTES, split_document, split_document_within,
};
use proptest::prelude::*;

/// Hash passed on the split call; per R5 / Assumption 5 (`session`
/// owns the `blake3` fingerprint) it must never be stored on the
/// emitted segments.
const TEST_HASH: &str = "t019-document-hash";

/// Four 2-byte Portuguese characters, used to distinguish bytes from
/// chars (R3).
const MULTIBYTE_CHARS: [char; 4] = ['ã', 'ç', 'é', 'õ'];

/// Whether `kind` is the closed set's `Heading` member.
///
/// Exhaustive over `BoundaryKind` with no wildcard arm (Principle VII
/// / gate 7): a future `Sentence` member fails to compile here.
#[must_use]
const fn is_heading(kind: BoundaryKind) -> bool {
    match kind {
        BoundaryKind::Heading => true,
        BoundaryKind::Paragraph => false,
    }
}

/// Whether `kind` is the closed set's `Paragraph` member.
///
/// Exhaustive over `BoundaryKind` with no wildcard arm (Principle VII
/// / gate 7): a future `Sentence` member fails to compile here.
#[must_use]
const fn is_paragraph(kind: BoundaryKind) -> bool {
    match kind {
        BoundaryKind::Paragraph => true,
        BoundaryKind::Heading => false,
    }
}

/// A single paragraph line of `char_count` multi-byte characters
/// (`char_count * 2` UTF-8 bytes).
#[must_use]
fn multibyte_line(char_count: usize) -> String {
    MULTIBYTE_CHARS
        .iter()
        .copied()
        .cycle()
        .take(char_count)
        .collect()
}

/// A fixed multi-paragraph document: short paragraphs, a multi-byte
/// paragraph, a long paragraph, and a 300-byte run — exercising
/// greedy packing, near-boundary fits, and forced splits at small
/// budgets.
#[must_use]
fn multi_paragraph_document() -> String {
    let paragraphs: [String; 5] = [
        String::from("Título de abertura curto."),
        String::from("Primeira frase do corpo. Segunda frase do corpo."),
        String::from("Ãmbar, çelofane, éter e õ — palavras com acentos."),
        String::from(
            "Uma linha um pouco mais longa para exercitar o empacotamento guloso entre parágrafos.",
        ),
        "a".repeat(300),
    ];
    paragraphs.join("\n\n")
}

/// Concatenation of every segment's content, in order.
#[must_use]
fn reassembled_contents(segments: &[DocumentSegment]) -> String {
    segments
        .iter()
        .map(|segment| segment.content.as_str())
        .collect()
}

/// `segment_index` is 0-based and strictly increasing by 1
/// (contract §2.1 rule 9).
fn assert_consecutive_indices(segments: &[DocumentSegment]) {
    let mut expected_index: u64 = 0;
    for segment in segments {
        assert_eq!(
            segment.segment_index, expected_index,
            "segment_index must be 0-based and strictly increasing by 1"
        );
        expected_index = expected_index.saturating_add(1);
    }
}

/// Every non-blank source line is reproduced, in order (contract
/// §2.1 rule 6).
///
/// Whitespace is squashed before comparison because segment contents
/// are trimmed (rule 4) and forced sub-pieces may cut through
/// interior whitespace.
fn assert_lossless(source: &str, segments: &[DocumentSegment]) {
    let reassembled = reassembled_contents(segments);
    let source_squashed: String = source.chars().filter(|ch| !ch.is_whitespace()).collect();
    let reassembled_squashed: String = reassembled
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert_eq!(
        source_squashed, reassembled_squashed,
        "every non-blank source line must be reproduced, in order"
    );
}

/// A non-blank content of exactly `MAX_CHUNK_BYTES` bytes is one
/// segment of exactly `MAX_CHUNK_BYTES` bytes (contract §2.1 rules 2
/// and 5).
#[test]
fn exact_fit_at_max_chunk_bytes_yields_exactly_one_segment() {
    let source = "a".repeat(MAX_CHUNK_BYTES);
    let segments = split_document(&source, TEST_HASH);
    assert_eq!(
        segments.len(),
        1,
        "an exact fit must yield exactly one segment"
    );
    let segment = &segments[0];
    assert_eq!(segment.byte_length(), MAX_CHUNK_BYTES);
    assert_eq!(segment.content, source);
    assert_eq!(segment.segment_index, 0);
}

/// One byte over the budget forces a hard split into exactly two
/// segments, each within the budget (contract §2.1 rules 3 and 5).
#[test]
fn one_byte_over_max_chunk_bytes_yields_two_bounded_segments() {
    let source = "a".repeat(MAX_CHUNK_BYTES + 1);
    let segments = split_document(&source, TEST_HASH);
    assert_eq!(
        segments.len(),
        2,
        "one byte over the budget must force exactly two segments"
    );
    assert_consecutive_indices(&segments);
    for segment in &segments {
        assert!(
            segment.byte_length() <= MAX_CHUNK_BYTES,
            "segment of {} bytes exceeds the {MAX_CHUNK_BYTES} byte budget",
            segment.byte_length()
        );
    }
    assert_eq!(
        reassembled_contents(&segments),
        source,
        "forced sub-pieces must reassemble the original exactly"
    );
}

/// An oversized single block with no interior boundary is still
/// hard-split (contract §2.1 rule 3): more than one segment is
/// produced, every segment stays within the budget, forced
/// sub-pieces carry `BoundaryKind::Paragraph`, and the concatenation
/// reassembles the original exactly.
#[test]
fn oversized_ascii_block_with_no_interior_boundary_is_split_char_safe() {
    let source = "a".repeat(12_000);
    let segments = split_document(&source, TEST_HASH);
    assert!(
        segments.len() > 1,
        "a 12 000-byte block must be hard-split, not dropped"
    );
    for segment in &segments {
        assert!(
            segment.byte_length() <= MAX_CHUNK_BYTES,
            "segment of {} bytes exceeds the {MAX_CHUNK_BYTES} byte budget",
            segment.byte_length()
        );
        assert!(
            is_paragraph(segment.boundary_kind),
            "forced sub-pieces carry BoundaryKind::Paragraph"
        );
        assert!(
            std::str::from_utf8(segment.content.as_bytes()).is_ok(),
            "segment content must be valid UTF-8"
        );
    }
    assert_consecutive_indices(&segments);
    assert_eq!(
        reassembled_contents(&segments),
        source,
        "the concatenation must reassemble the original exactly"
    );
}

/// The same hard split over 12 000 multi-byte characters (24 000
/// bytes): the budget is measured in bytes, not chars (R3), and no
/// piece splits a `char`.
#[test]
fn oversized_multibyte_block_with_no_interior_boundary_is_split_char_safe() {
    let source = multibyte_line(12_000);
    assert_eq!(source.chars().count(), 12_000);
    assert_eq!(
        source.len(),
        24_000,
        "the fixture is 24 000 bytes, not 12 000 characters"
    );
    let segments = split_document(&source, TEST_HASH);
    assert!(
        segments.len() > 1,
        "a 24 000-byte block must be hard-split, not dropped"
    );
    for segment in &segments {
        assert!(
            segment.byte_length() <= MAX_CHUNK_BYTES,
            "segment of {} bytes exceeds the {MAX_CHUNK_BYTES} byte budget",
            segment.byte_length()
        );
        assert!(
            is_paragraph(segment.boundary_kind),
            "forced sub-pieces carry BoundaryKind::Paragraph"
        );
        assert!(
            std::str::from_utf8(segment.content.as_bytes()).is_ok(),
            "segment content must be valid UTF-8"
        );
    }
    assert_consecutive_indices(&segments);
    assert_eq!(
        reassembled_contents(&segments),
        source,
        "the concatenation must reassemble the original exactly"
    );
}

/// A multi-byte-only document: the concatenation of all segment
/// contents equals the trimmed source, proving no piece splits a
/// `char`.
#[test]
fn multibyte_only_document_reassembles_exactly_without_splitting_chars() {
    let source = multibyte_line(9_000);
    let trimmed = source.trim();
    let segments = split_document(&source, TEST_HASH);
    assert_eq!(
        reassembled_contents(&segments),
        trimmed,
        "the concatenation of segment contents must equal the trimmed source"
    );
}

/// Empty and whitespace-only sources yield zero segments (contract
/// §2.1 rule 8; `spec.md:102`: no segments ⇒ no `AudioChunk`,
/// indeterminate progress).
#[test]
fn empty_and_whitespace_only_sources_yield_zero_segments() {
    for source in ["", "   \n\t\n  ", "\n\n\n"] {
        let segments = split_document(source, TEST_HASH);
        assert!(
            segments.is_empty(),
            "source {source:?} must yield zero segments"
        );
    }
}

/// §2.1 rule 4 — trimming: every emitted segment's `content`
/// is trimmed of leading and trailing whitespace. A segment
/// whose trimmed content is empty is not emitted (rule 4's
/// second clause, exercised below and by the rule-8 test
/// above), so the trim assertion never demands an empty
/// segment.
///
/// The fixture's first line opens with two spaces and its
/// last line closes with three, so the packed segment's
/// content carries whitespace on both edges; at the small
/// budget each paragraph becomes its own segment, and each
/// one's edge whitespace must be trimmed too.
#[test]
fn segment_content_is_trimmed_of_leading_and_trailing_whitespace() {
    let packed = "  primeira linha com recuo  \n\nsegunda linha com espaço à direita   ";
    let segments = split_document(packed, TEST_HASH);
    assert_eq!(
        segments.len(),
        1,
        "the two fitting paragraphs pack into one segment"
    );
    for segment in &segments {
        assert_eq!(
            segment.content,
            segment.content.trim(),
            "segment content must be trimmed of leading and trailing whitespace (§2.1 rule 4)"
        );
    }

    let separated = "  primeira  \n\nsegunda  \n\nterceira   ";
    let segments = split_document_within(separated, TEST_HASH, 20);
    assert_eq!(
        segments.len(),
        3,
        "the 20-byte budget gives each paragraph its own segment"
    );
    for segment in &segments {
        assert_eq!(
            segment.content,
            segment.content.trim(),
            "segment content must be trimmed of leading and trailing whitespace (§2.1 rule 4)"
        );
    }

    for source in ["   ", "\t\n  \n\t", " "] {
        let segments = split_document(source, TEST_HASH);
        assert!(
            segments.is_empty(),
            "whitespace-only source {source:?} must yield zero segments"
        );
    }
}

/// A heading block extends over the heading line and its body: they
/// are narrated together and the split never lands between them
/// (contract §2.1 rule 1).
#[test]
fn heading_absorbs_its_body_and_is_never_split_from_it() {
    let source = "## Título\n\nPrimeira frase do corpo.\n\nSegundo parágrafo.";
    let segments = split_document(source, TEST_HASH);
    assert_eq!(
        segments.len(),
        2,
        "the heading block and the trailing paragraph are two segments"
    );
    let heading = &segments[0];
    assert!(
        is_heading(heading.boundary_kind),
        "the opening block is a heading block"
    );
    assert!(
        heading.content.starts_with("## Título"),
        "the heading block opens with its heading line"
    );
    assert!(
        heading.content.contains("Primeira frase do corpo."),
        "the heading block carries its body"
    );
    let paragraph = &segments[1];
    assert!(
        is_paragraph(paragraph.boundary_kind),
        "the trailing block is a paragraph block"
    );
    assert!(paragraph.content.contains("Segundo parágrafo."));
}

/// A paragraph of several sentences that fits the budget is one
/// segment: the paragraph/heading boundary takes precedence over any
/// sentence boundary inside it (FR-024a; `segment-resume.md`
/// CHK029).
#[test]
fn paragraph_boundary_takes_precedence_over_sentence_boundaries_within_it() {
    let source = "Primeira frase. Segunda frase. Terceira frase.";
    let segments = split_document(source, TEST_HASH);
    assert_eq!(
        segments.len(),
        1,
        "a fitting paragraph is one segment, never split at sentence boundaries"
    );
    assert!(is_paragraph(segments[0].boundary_kind));
    for sentence in ["Primeira frase.", "Segunda frase.", "Terceira frase."] {
        assert!(
            segments[0].content.contains(sentence),
            "sentence {sentence:?} must survive inside the single segment"
        );
    }
}

/// Small custom budgets via `split_document_within`: the bound,
/// losslessness, determinism, and consecutive indices all hold at
/// `max_bytes` of 64 and 200.
#[test]
fn small_custom_budgets_keep_bound_losslessness_determinism_and_indices() {
    let source = multi_paragraph_document();
    for max_bytes in [64, 200] {
        let first = split_document_within(&source, TEST_HASH, max_bytes);
        let second = split_document_within(&source, TEST_HASH, max_bytes);
        assert_eq!(
            first, second,
            "the same arguments must yield identical segments"
        );
        assert_consecutive_indices(&first);
        for segment in &first {
            assert!(
                segment.byte_length() <= max_bytes,
                "segment of {} bytes exceeds the {max_bytes} byte budget",
                segment.byte_length()
            );
        }
        assert_lossless(&source, &first);
    }
}

/// `document_hash` is accepted on the call but never stored on the
/// segment (Assumption 5 / R5: the `blake3` fingerprint is owned by
/// `session`): two splits of the same source with different hashes
/// produce byte-identical segments.
#[test]
fn document_hash_is_accepted_but_never_stored_on_segments() {
    let source = multi_paragraph_document();
    let with_alpha = split_document(&source, "hash-alpha");
    let with_beta = split_document(&source, "hash-beta");
    assert_eq!(
        with_alpha, with_beta,
        "segments must not embed the document hash"
    );
    let within_alpha = split_document_within(&source, "hash-alpha", 128);
    let within_beta = split_document_within(&source, "hash-beta", 128);
    assert_eq!(
        within_alpha, within_beta,
        "the bounded variant must not embed the document hash either"
    );
}

// Generated documents (headings, paragraphs, blank runs, multi-byte
// characters) under a randomly chosen budget in `4..=MAX_CHUNK_BYTES`:
// no segment ever exceeds its budget, and `segment_index` stays
// consecutive. The floor is the maximum UTF-8 character width: below
// 4 bytes a single `char` can exceed the budget, and §2.1 rules 3, 5
// and 6 (character-safe hard split, hard bound, losslessness) are
// jointly unsatisfiable — cutting through a `char` would produce
// invalid UTF-8 — so the bound is a precondition that holds from
// this floor upward, never below it.
//
// Plain `//`, not `///`: a doc comment cannot attach to a `proptest!`
// block, so `///` here would raise `unused_doc_comments`, which
// verification task V2 promotes to an error via `-D warnings`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]
    #[test]
    fn no_segment_ever_exceeds_a_randomly_chosen_budget(
        (source, max_bytes) in document_strategy(),
    ) {
        let segments = split_document_within(&source, TEST_HASH, max_bytes);
        let mut expected_index: u64 = 0;
        for segment in &segments {
            prop_assert!(
                segment.byte_length() <= max_bytes,
                "segment of {} bytes exceeds the {max_bytes} byte budget",
                segment.byte_length()
            );
            prop_assert_eq!(segment.segment_index, expected_index);
            expected_index = expected_index.saturating_add(1);
        }
    }
}

// §2.1 rule 4 — trimming, over generated documents whose
// lines carry leading and/or trailing whitespace: every
// emitted segment's content equals its own trim. The
// generator deliberately produces whitespace (an
// unpadded-only generator would make the property
// vacuous — no segment could ever carry edge whitespace).
//
// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block, so `///` here would raise
// `unused_doc_comments`, which verification task V2
// promotes to an error via `-D warnings`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn every_segment_content_is_trimmed(
        document in whitespace_padded_document_strategy(),
    ) {
        let segments = split_document(&document, TEST_HASH);
        for segment in &segments {
            prop_assert!(
                segment.content == segment.content.trim(),
                "segment content carries leading or trailing whitespace: {:?}",
                segment.content
            );
        }
    }
}

// A generated document: heading lines, ASCII and multi-byte
// paragraph lines, and blank runs, paired with a budget in
// `4..=MAX_CHUNK_BYTES` — the floor is the maximum UTF-8
// character width, the smallest budget at which every
// character fits whole (see the property above).
#[must_use]
fn document_strategy() -> impl Strategy<Value = (String, usize)> {
    let line = prop_oneof![
        (1..=6usize, "[a-z]{1,16}")
            .prop_map(|(level, title)| format!("{} {title}", "#".repeat(level))),
        "[a-z]{1,60}",
        "[ãçéõ]{1,60}",
        "[a-zãçé]{1,100}",
        Just(String::new()),
        Just("   ".to_owned()),
    ];
    (
        proptest::collection::vec(line, 0..=16),
        4usize..=MAX_CHUNK_BYTES,
    )
        .prop_map(|(lines, max_bytes)| (lines.join("\n"), max_bytes))
}

/// A generated document whose lines may carry leading and
/// trailing whitespace: each line is an optional run of
/// spaces or tabs, 1–20 ASCII or multi-byte characters,
/// and another optional run, with blank and
/// whitespace-only lines mixed in. The generator makes the
/// §2.1 rule 4 trimming property non-vacuous: a document
/// of only unpadded lines could never expose untrimmed
/// segment content, and the default budget keeps every
/// generated block well under `MAX_CHUNK_BYTES`, so no
/// block is ever hard-split and the property observes the
/// segment-close path alone.
#[must_use]
fn whitespace_padded_document_strategy() -> impl Strategy<Value = String> {
    let line = prop_oneof![
        "[ \\t]{0,3}[a-zãçé]{1,20}[ \\t]{0,3}",
        "[a-zãçé]{1,20}",
        Just(String::new()),
        Just("   ".to_owned()),
    ];
    proptest::collection::vec(line, 0..=16).prop_map(|lines| lines.join("\n"))
}
