//! T033 — segmentation-shape properties for `chunk.rs`:
//! the Markdown constructs the frozen generators
//! deliberately omit.
//!
//! Spec anchors: **FR-024a** (the adapter splits Markdown
//! into ≤ 5000-byte chunks at paragraph/heading boundaries
//! before synthesis), **R3** (the byte budget is measured
//! in UTF-8 bytes, not characters), **R6** (resume snaps
//! to the nearest completed paragraph/heading boundary;
//! there is no `Sentence` member at v1), checklist
//! **CHK008** (the 5000-byte limit is a hard upper bound,
//! not a target) and **CHK016** (segment-boundary
//! compliance is measurable), and `API_CONTRACT.md` §2.1
//! rule 1 — the normative line scan recognises a heading
//! block ONLY when a line's trimmed form starts with 1–6
//! `#` followed by a space or end-of-line (at most 3
//! leading spaces).
//!
//! The frozen generators (`tests/prop_chunk.rs` and
//! `tests/prop_chunk_refined.rs`) emit ATX `#` headings
//! only, so four constructs are completely unobserved:
//! setext headings (a title line over a `=====` or `-----`
//! underline), `#` lines inside fenced code blocks (``` or
//! ~~~), list items (`- item`, `1. item`) and block
//! quotes (`> text`). Under the §2.1 rule 1 line scan
//! each is a paragraph block classified
//! `BoundaryKind::Paragraph`, never `Heading`. This file
//! pins each classification, contrasts it with a real ATX
//! heading — which IS `BoundaryKind::Heading` and absorbs
//! its following non-blank body lines into the same
//! segment — and re-asserts the universal §2.1 invariants
//! over every case.
//!
//! Written before the frozen surface exists (rule 1 —
//! tests block the core wave; the surface lands with
//! T028), so the file is red with unresolved imports for
//! the not-yet-existing `adapter_synthesis::chunk` items —
//! the expected state, and the pin T034 must reconcile. It
//! binds to the real crate surface only: no contract type
//! and no split function is re-declared locally (the F1/F2
//! root cause), and no `unwrap`/`expect`/`panic!`/`dbg!`/
//! `println!` appears (all denied workspace-wide, in tests
//! too, per the constitution's Rust Code Rigor).

use adapter_synthesis::chunk::{
    BoundaryKind, DocumentSegment, MAX_CHUNK_BYTES, SourceRange, split_document,
    split_document_within,
};
use proptest::prelude::*;

/// Hash passed on every split call; per R5 / Assumption 5
/// (`session` owns the `blake3` fingerprint) it must never
/// be stored on the emitted segments.
const TEST_HASH: &str = "t033-segmentation-shapes-hash";

/// Whether `kind` is the closed set's `Heading` member.
///
/// Exhaustive over `BoundaryKind` with no wildcard arm
/// (Principle VII / gate 7, `constitution.md:614`): a
/// future third member — a `Sentence`, say — fails to
/// compile right here. This match is the structural
/// assertion that the closed set has exactly two members
/// at v1.
#[must_use]
const fn is_heading(kind: BoundaryKind) -> bool {
    match kind {
        BoundaryKind::Heading => true,
        BoundaryKind::Paragraph => false,
    }
}

/// Whether `kind` is the closed set's `Paragraph` member.
///
/// Exhaustive over `BoundaryKind` with no wildcard arm
/// (Principle VII / gate 7, `constitution.md:614`): a
/// future third member — a `Sentence`, say — fails to
/// compile right here. This match is the structural
/// assertion that the closed set has exactly two members
/// at v1.
#[must_use]
const fn is_paragraph(kind: BoundaryKind) -> bool {
    match kind {
        BoundaryKind::Paragraph => true,
        BoundaryKind::Heading => false,
    }
}

/// A single plain text line of 1..=4 multi-byte Portuguese
/// words, free of leading and trailing whitespace (so
/// exact non-blank-line losslessness holds at the default
/// budget).
#[must_use]
fn text_line_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just("ação"),
            Just("çães"),
            Just("ãó"),
            Just("título"),
            Just("seção"),
            Just("próxima"),
            Just("frase"),
            Just("corpo"),
            Just("texto"),
            Just("citação"),
        ],
        1..=4,
    )
    .prop_map(|words| words.join(" "))
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

/// Every non-blank source line is reproduced, in order
/// (contract §2.1 rule 6).
///
/// Whitespace is squashed before comparison because segment
/// contents are trimmed (rule 4) and forced sub-pieces may
/// cut through interior whitespace (rule 3).
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

/// The universal §2.1 invariants over any split result: the
/// byte budget (rules 2 and 5), `byte_length()` equal to
/// the content's UTF-8 byte count (rule 5 / R3), 0-based
/// consecutive `segment_index` (rule 9), a 1-based
/// `source_range` with `line_end >= line_start` (rule 10),
/// `resume_point == Some(source_range)` (rule 11 / R6), and
/// losslessness over non-blank lines (rule 6).
fn assert_universal_invariants(source: &str, segments: &[DocumentSegment], max_bytes: usize) {
    assert_consecutive_indices(segments);
    for segment in segments {
        assert!(
            segment.byte_length() <= max_bytes,
            "segment of {} bytes exceeds the {max_bytes} byte budget",
            segment.byte_length()
        );
        assert_eq!(
            segment.byte_length(),
            segment.content.as_bytes().len(),
            "byte_length() must equal the content's UTF-8 byte count"
        );
        let range = segment.source_range;
        assert!(range.line_start >= 1, "source_range must be 1-based");
        assert!(
            range.line_end >= range.line_start,
            "line_end must not precede line_start"
        );
        assert!(
            range.contains_line(range.line_start),
            "source_range must contain its own line_start"
        );
        assert_eq!(
            segment.resume_point,
            Some(segment.source_range),
            "resume_point must be Some(source_range)"
        );
    }
    assert_lossless(source, segments);
}

/// A setext heading — a title line over a `=====` or
/// `-----` underline — opens no heading block under the
/// §2.1 rule 1 line scan (only 1–6 `#` followed by a space
/// or end-of-line does): the pair is one paragraph block,
/// classified `BoundaryKind::Paragraph`, never `Heading`,
/// and its whole text survives in one segment's content.
#[test]
fn setext_headings_are_paragraphs_not_headings() {
    for source in [
        "Título da Seção\n=================",
        "Subtítulo Menor\n-----------------",
    ] {
        let segments = split_document(source, TEST_HASH);
        assert_eq!(
            segments.len(),
            1,
            "the setext pair is a single paragraph block, hence one segment"
        );
        let segment = &segments[0];
        assert!(
            is_paragraph(segment.boundary_kind),
            "a setext heading must be classified Paragraph, not Heading"
        );
        assert!(
            !is_heading(segment.boundary_kind),
            "a setext heading must never be classified Heading"
        );
        assert_eq!(
            segment.content, source,
            "the whole setext pair must survive verbatim in the one segment"
        );
        assert_universal_invariants(source, &segments, MAX_CHUNK_BYTES);
    }
}

/// A `#` line inside a fenced code block (``` or ~~~) is
/// code text, not a heading: no segment of the document is
/// classified `Heading` — every segment is a `Paragraph` —
/// and the fenced text survives verbatim. The same
/// document under a small explicit budget, where the
/// fence-marker paragraph would close early and let the
/// `#` line open its own segment under a heading-labelling
/// scan, still yields no `Heading` segment.
#[test]
fn hash_lines_inside_fenced_code_blocks_are_paragraphs_not_headings() {
    for source in [
        "```rust\n# Not a heading\nlet x = 1;\n```",
        "~~~text\n# Still not a heading\ny = 2;\n~~~",
    ] {
        let segments = split_document(source, TEST_HASH);
        assert_eq!(
            segments.len(),
            1,
            "the fenced block is a single paragraph block, hence one segment"
        );
        for segment in &segments {
            assert!(
                is_paragraph(segment.boundary_kind),
                "a `#` line inside a fenced code block must be classified Paragraph, never Heading"
            );
        }
        assert_eq!(
            segments[0].content, source,
            "the whole fenced block must survive verbatim in the one segment"
        );
        assert_universal_invariants(source, &segments, MAX_CHUNK_BYTES);

        // Small budget: the fenced block alone exceeds it,
        // so it is hard-split (§2.1 rule 3); every forced
        // sub-piece is still a `Paragraph` sub-piece, never
        // a `Heading`.
        let bounded = split_document_within(source, TEST_HASH, 20);
        for segment in &bounded {
            assert!(
                is_paragraph(segment.boundary_kind),
                "a hard-split sub-piece of a fenced code block must still be a Paragraph"
            );
        }
        assert_universal_invariants(source, &bounded, 20);
    }
}

/// List items — unordered (`- item`) and ordered (`1.
/// item`) — are not heading lines under the §2.1 rule 1
/// line scan: a run of them is one paragraph block,
/// classified `BoundaryKind::Paragraph`, never `Heading`,
/// with every item surviving verbatim in one segment's
/// content.
#[test]
fn list_items_are_paragraphs_not_headings() {
    for source in [
        "- Primeiro item\n- Segundo item\n- Terceiro item",
        "1. Primeiro\n2. Segundo\n3. Terceiro",
    ] {
        let segments = split_document(source, TEST_HASH);
        assert_eq!(
            segments.len(),
            1,
            "a run of list items is a single paragraph block, hence one segment"
        );
        let segment = &segments[0];
        assert!(
            is_paragraph(segment.boundary_kind),
            "list items must be classified Paragraph, not Heading"
        );
        assert_eq!(
            segment.content, source,
            "every list item must survive verbatim in the one segment"
        );
        assert_universal_invariants(source, &segments, MAX_CHUNK_BYTES);
    }
}

/// Block-quote lines (`> text`) are not heading lines
/// under the §2.1 rule 1 line scan: a quote run is one
/// paragraph block, classified `BoundaryKind::Paragraph`,
/// never `Heading`, with every quoted line surviving
/// verbatim in one segment's content.
#[test]
fn block_quote_lines_are_paragraphs_not_headings() {
    let source = "> Primeira citação\n> Segunda citação\n> Terceira citação";
    let segments = split_document(source, TEST_HASH);
    assert_eq!(
        segments.len(),
        1,
        "a block-quote run is a single paragraph block, hence one segment"
    );
    let segment = &segments[0];
    assert!(
        is_paragraph(segment.boundary_kind),
        "block quotes must be classified Paragraph, not Heading"
    );
    assert_eq!(
        segment.content, source,
        "every quoted line must survive verbatim in the one segment"
    );
    assert_universal_invariants(source, &segments, MAX_CHUNK_BYTES);
}

/// A real ATX heading — `# Title` or `###### Title` (1–6
/// `#` followed by a space) — IS a heading block under
/// §2.1 rule 1, classified `BoundaryKind::Heading`, and
/// the block absorbs its following non-blank body lines
/// into the same segment: the split never lands between a
/// heading and its text. This is the explicit contrast
/// with the four constructs pinned above.
#[test]
fn atx_headings_are_headings_and_absorb_their_body_lines() {
    let fixtures: [(&str, &str, &str); 3] = [
        (
            "# Título Principal\n\nPrimeira frase do corpo. Segunda frase.",
            "# Título Principal",
            "Primeira frase do corpo. Segunda frase.",
        ),
        (
            "###### Título de Nível Seis\n\nCorpo do título de nível seis.",
            "###### Título de Nível Seis",
            "Corpo do título de nível seis.",
        ),
        (
            "# Título\nLinha de corpo imediata",
            "# Título",
            "Linha de corpo imediata",
        ),
    ];
    for (source, heading_line, body_line) in fixtures {
        let segments = split_document(source, TEST_HASH);
        assert_eq!(
            segments.len(),
            1,
            "a heading block and its absorbed body form one segment"
        );
        let heading_segment = &segments[0];
        assert!(
            is_heading(heading_segment.boundary_kind),
            "an ATX heading block must be classified Heading"
        );
        assert!(
            heading_segment.content.starts_with(heading_line),
            "the heading segment must open with its heading line"
        );
        assert!(
            heading_segment.content.contains(body_line),
            "the heading block must absorb its following body line into the same segment"
        );
        let body_segment_count = segments
            .iter()
            .filter(|segment| segment.content.contains(body_line))
            .count();
        assert_eq!(
            body_segment_count, 1,
            "the absorbed body line must appear in exactly one segment — the heading's own"
        );
        assert_universal_invariants(source, &segments, MAX_CHUNK_BYTES);
    }
}

/// A fixed document mixing every construct this file pins —
/// a setext heading, a fenced code block containing a `#`
/// line, a list run, a block-quote run, two ATX headings
/// (one with a blank-separated body, one with an
/// immediately-following body line), and multi-byte
/// Portuguese paragraphs — satisfies the universal §2.1
/// invariants: the byte budget, byte-measured lengths,
/// consecutive 0-based indices, 1-based ranges,
/// `resume_point == Some(source_range)`, determinism (rule
/// 7), hash-never-stored (rule 12 / R5), and losslessness
/// over non-blank lines (rule 6).
#[test]
fn universal_invariants_hold_on_a_fixed_mix_of_pinned_constructs() {
    let source = [
        "Título de Abertura",
        "====================",
        "",
        "```rust",
        "# Comentário dentro do bloco",
        "let x = 1;",
        "```",
        "",
        "- Primeiro item",
        "- Segundo item",
        "",
        "> Citação de abertura",
        "",
        "# Seção Um",
        "",
        "Corpo da seção um. Ãmbar, çelofane, éter e õ.",
        "",
        "###### Seção Seis",
        "Corpo imediato do nível seis.",
        "",
        "Parágrafo final: ação, çães e ãos — áêíóú.",
    ]
    .join("\n");
    let segments = split_document(&source, TEST_HASH);
    assert_universal_invariants(&source, &segments, MAX_CHUNK_BYTES);
    // Rule 7 — determinism.
    assert_eq!(
        split_document(&source, TEST_HASH),
        segments,
        "the split must be deterministic"
    );
    // Rule 12 / R5 — the document hash is accepted but
    // never stored on the segment.
    assert_eq!(
        split_document(&source, "t033-alternate-hash"),
        segments,
        "the document hash must not leak into the emitted segments"
    );
    // The closed set: every segment is classified as
    // exactly one of the two members (Principle VII /
    // gate 7).
    for segment in &segments {
        assert!(
            is_heading(segment.boundary_kind) || is_paragraph(segment.boundary_kind),
            "every segment must be classified as Heading or Paragraph"
        );
    }
}

// A generated document mixing setext headings, fenced code
// blocks (with `#`, `-`, `>`, `1.` and plain content
// lines), list runs, block-quote runs, ATX headings with
// absorbed bodies, and multi-byte Portuguese paragraphs,
// split at the default `MAX_CHUNK_BYTES` budget. Every
// universal §2.1 invariant holds, and — because no
// generated block ever exceeds `MAX_CHUNK_BYTES` (so no
// block is ever hard-split) — every non-blank source line
// survives verbatim, in order.
//
// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block, so `///` here would raise
// `unused_doc_comments`, which verification task V2
// promotes to an error via `-D warnings`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn mixed_construct_documents_satisfy_the_universal_invariants(
        document in mixed_document_strategy(),
    ) {
        let segments = split_document(&document, TEST_HASH);

        // Rule 5 — the budget is a hard bound, and
        // `byte_length()` is derived from the content's
        // byte count (R3).
        for segment in &segments {
            prop_assert!(
                segment.byte_length() <= MAX_CHUNK_BYTES,
                "segment of {} bytes exceeds the {MAX_CHUNK_BYTES} byte budget",
                segment.byte_length()
            );
            prop_assert_eq!(
                segment.byte_length(),
                segment.content.as_bytes().len()
            );
        }

        // Rule 9 — 0-based, strictly increasing by exactly 1.
        for (position, segment) in segments.iter().enumerate() {
            prop_assert_eq!(
                segment.segment_index,
                position as u64,
                "segment_index at vector position {} is not 0-based sequential",
                position
            );
        }

        // Rule 10 — 1-based inclusive lines, non-decreasing
        // and non-overlapping. Overlap cannot occur here
        // because no generated block exceeds the budget, so
        // no block is ever hard-split into sub-pieces that
        // would share the block's line range.
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
                prop_assert!(
                    range.line_start > prior.line_end,
                    "source ranges overlap"
                );
            }
            previous = Some(range);
        }

        // Rule 11 / R6 — every segment carries its resume
        // boundary.
        for segment in &segments {
            prop_assert_eq!(
                segment.resume_point,
                Some(segment.source_range),
                "resume_point is not Some(source_range)"
            );
        }

        // Closed set (Principle VII / gate 7): an exhaustive
        // match with no `_ =>` arm, so a third `BoundaryKind`
        // member (e.g. `Sentence`) would not compile.
        for segment in &segments {
            match segment.boundary_kind {
                BoundaryKind::Paragraph => {}
                BoundaryKind::Heading => {}
            }
        }

        // Rule 6 — losslessness: every non-blank source line
        // appears in exactly one segment, in source order.
        // Exact line equality holds because no block is
        // hard-split and no generated line carries leading or
        // trailing whitespace.
        let expected: Vec<&str> = document
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let mut produced: Vec<&str> = Vec::new();
        for segment in &segments {
            for line in segment.content.lines() {
                if !line.trim().is_empty() {
                    produced.push(line);
                }
            }
        }
        prop_assert_eq!(
            produced, expected,
            "segmentation is not lossless over non-blank lines"
        );

        // Rule 7 — determinism: same input, same vector (all
        // fields compared; `DocumentSegment` derives
        // `PartialEq`).
        let repeated = split_document(&document, TEST_HASH);
        prop_assert_eq!(repeated, segments, "the split is not deterministic");

        // Rule 12 / R5 — the document hash is accepted but
        // never stored on the segment: a distinct hash per
        // call leaves the emitted segments untouched (the
        // `blake3` fingerprint is owned by `session`,
        // Assumption 5).
        let other = split_document(&document, "t033-alternate-hash");
        prop_assert_eq!(
            other, segments,
            "document_hash leaked into the emitted segments"
        );
    }
}

// The same generated documents under an explicit budget
// chosen at random from `1..=MAX_CHUNK_BYTES`: the budget
// is a hard bound even when a single construct block
// exceeds it (§2.1 rule 3 — the block is hard-split,
// snapped down to a UTF-8 character boundary, into
// `BoundaryKind::Paragraph` sub-pieces), `segment_index`
// stays consecutive, ranges stay 1-based with
// `line_end >= line_start`, and the reassembly still
// reproduces every non-blank source line (whitespace
// squashed, because forced sub-pieces may cut through
// interior whitespace).
//
// Plain `//`, not `///`: a doc comment cannot attach to a
// `proptest!` block (see the previous block).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn mixed_construct_documents_stay_bounded_under_random_budgets(
        (document, max_bytes) in mixed_document_and_budget_strategy(),
    ) {
        let segments = split_document_within(&document, TEST_HASH, max_bytes);
        let mut expected_index: u64 = 0;
        for segment in &segments {
            prop_assert!(
                segment.byte_length() <= max_bytes,
                "segment of {} bytes exceeds the {max_bytes} byte budget",
                segment.byte_length()
            );
            prop_assert_eq!(
                segment.byte_length(),
                segment.content.as_bytes().len()
            );
            prop_assert_eq!(segment.segment_index, expected_index);
            expected_index = expected_index.saturating_add(1);
            let range = segment.source_range;
            prop_assert!(range.line_start >= 1, "source_range is not 1-based");
            prop_assert!(
                range.line_end >= range.line_start,
                "line_end precedes line_start"
            );
            prop_assert_eq!(
                segment.resume_point,
                Some(segment.source_range),
                "resume_point is not Some(source_range)"
            );
        }

        // Rule 7 — determinism under an explicit budget.
        let repeated = split_document_within(&document, TEST_HASH, max_bytes);
        prop_assert_eq!(
            repeated, segments,
            "the bounded split is not deterministic"
        );

        // Rule 6 — losslessness, whitespace-squashed because
        // forced sub-pieces may cut through interior
        // whitespace.
        let reassembled = reassembled_contents(&segments);
        let source_squashed: String =
            document.chars().filter(|ch| !ch.is_whitespace()).collect();
        let reassembled_squashed: String = reassembled
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        prop_assert_eq!(
            source_squashed, reassembled_squashed,
            "every non-blank source line must be reproduced, in order"
        );
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Builds a document as a random number of blocks (0..=48),
/// each one an ATX heading block (1–6 `#` plus 0..=2
/// absorbed body lines), a setext heading pair (a title
/// over a `=====`/`-----` underline), a fenced code block
/// (``` or ~~~, with content lines that may themselves
/// begin with `#`, `-`, `>`, `1.` or nothing), a run of
/// unordered or ordered list items, a block-quote run, a
/// multi-byte Portuguese paragraph, or an occasional empty
/// block (blank-run separation). Blocks are joined with
/// blank lines.
///
/// Every block stays far below `MAX_CHUNK_BYTES` (at most
/// ~160 bytes), so the default-budget property is never
/// vacuous — up to 48 blocks routinely exceed 5000 bytes
/// in total — yet no block is ever hard-split at the
/// default budget, which keeps exact non-blank-line
/// losslessness provable.
#[must_use]
fn mixed_document_strategy() -> impl Strategy<Value = String> {
    // An ATX heading block: 1–6 `#`, a space, a title,
    // then 0..=2 non-blank body lines absorbed into the
    // block.
    let atx_heading = (
        1..=6usize,
        text_line_strategy(),
        proptest::collection::vec(text_line_strategy(), 0..=2),
    )
        .prop_map(|(level, title, body)| {
            let mut block = format!("{} {title}", "#".repeat(level));
            for line in body {
                block.push('\n');
                block.push_str(&line);
            }
            block
        });

    // A setext heading pair: a title line over a `=====` or
    // `-----` underline — a paragraph block under the
    // §2.1 rule 1 line scan, never a heading block.
    let setext_heading = (
        text_line_strategy(),
        prop_oneof![Just('='), Just('-')],
        1..=20usize,
    )
        .prop_map(|(title, marker, width)| {
            format!("{title}\n{}", marker.to_string().repeat(width))
        });

    // A fenced code block: ``` or ~~~ (optionally carrying a
    // language tag), 0..=4 content lines — some beginning
    // with `#`, `-`, `>` or `1.`, so the generator exercises
    // exactly the constructs that must not open a heading
    // block inside a fence — closed by the matching fence
    // marker.
    let fenced_block = (
        prop_oneof![Just("```"), Just("~~~")],
        prop_oneof![Just("rust"), Just("text"), Just("")],
        proptest::collection::vec(
            prop_oneof![
                text_line_strategy().prop_map(|line| format!("# {line}")),
                text_line_strategy().prop_map(|line| format!("- {line}")),
                text_line_strategy().prop_map(|line| format!("1. {line}")),
                text_line_strategy().prop_map(|line| format!("> {line}")),
                text_line_strategy(),
            ],
            0..=4,
        ),
    )
        .prop_map(|(fence, lang, content)| {
            let opener = if lang.is_empty() {
                fence.to_owned()
            } else {
                format!("{fence} {lang}")
            };
            let mut block = opener.clone();
            for line in content {
                block.push('\n');
                block.push_str(&line);
            }
            block.push('\n');
            block.push_str(&opener);
            block
        });

    // A run of unordered (`- item`) or ordered (`1. item`)
    // list items — paragraph lines under the §2.1 rule 1
    // line scan.
    let list_block = (
        any::<bool>(),
        proptest::collection::vec(text_line_strategy(), 1..=4),
    )
        .prop_map(|(ordered, items)| {
            let mut block = String::new();
            for (index, item) in items.iter().enumerate() {
                if !block.is_empty() {
                    block.push('\n');
                }
                if ordered {
                    block.push_str(&format!("{}. {item}", index + 1));
                } else {
                    block.push_str("- {item}");
                }
            }
            block
        });

    // A block-quote run (`> text`) — paragraph lines under
    // the §2.1 rule 1 line scan.
    let quote_block = proptest::collection::vec(text_line_strategy(), 1..=3).prop_map(|lines| {
        lines
            .into_iter()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    });

    // A plain multi-byte Portuguese paragraph.
    let paragraph =
        proptest::collection::vec(text_line_strategy(), 1..=4).prop_map(|lines| lines.join("\n"));

    let block = prop_oneof![
        1 => atx_heading,
        1 => setext_heading,
        1 => fenced_block,
        1 => list_block,
        1 => quote_block,
        3 => paragraph,
        1 => Just(String::new()),
    ];

    proptest::collection::vec(block, 0..=48).prop_map(|blocks| blocks.join("\n\n"))
}

/// A generated document paired with an explicit budget
/// chosen at random from `1..=MAX_CHUNK_BYTES` — small
/// enough that single construct blocks are routinely
/// hard-split (§2.1 rule 3).
#[must_use]
fn mixed_document_and_budget_strategy() -> impl Strategy<Value = (String, usize)> {
    (mixed_document_strategy(), 1usize..=MAX_CHUNK_BYTES)
}
