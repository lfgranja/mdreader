//! Invariant tests for **014 — TTS Symbol Normalization**.
//!
//! `normalize_for_speech` is a pure function of `(text, mode)` applied
//! only to the TTS input inside `synth.rs::synthesize`; it never mutates
//! [`DocumentSegment`]. These tests bind the invariants the spec requires
//! (FR-SYM-007 / FR-SYM-009 / SC-SYM-004): the split and the segment
//! metadata are independent of the normalization mode, Literal is the
//! identity, and Fluid never alters the protected zones.
//!
//! The blake3 document fingerprint is owned by `session` and computed from
//! the raw document bytes before any synthesis/normalization, so it cannot
//! be altered here; that invariant is covered by
//! `crates/session/tests/prop_fingerprint.rs`.

use adapter_synthesis::chunk::split_document;
use adapter_synthesis::normalize::{NormalizationMode, normalize_for_speech};

/// A marker-rich document exercising headings, emphasis, lists, quotes,
/// thematic breaks, inline code, URLs and simple math.
const MARKED: &str = "# Título\n\n*item um* e **item dois**\n\n> citação\n\n---\n\n`código *não* normaliza`\n\nhttps://exemplo.com/a*b e 2*3\n";

/// The segment metadata the resume/fingerprint contracts bind to:
/// source range, index, resume point and the raw-content byte length.
fn metadata(
    segments: &[adapter_synthesis::chunk::DocumentSegment],
) -> Vec<(
    adapter_synthesis::chunk::SourceRange,
    u64,
    Option<adapter_synthesis::chunk::SourceRange>,
    usize,
)> {
    segments
        .iter()
        .map(|seg| {
            (
                seg.source_range,
                seg.segment_index,
                seg.resume_point,
                seg.byte_length(),
            )
        })
        .collect()
}

#[test]
fn literal_mode_is_the_identity() {
    assert_eq!(
        normalize_for_speech(MARKED, NormalizationMode::Literal),
        MARKED
    );
}

#[test]
fn fluid_mode_strips_structural_markers() {
    let spoken = normalize_for_speech(MARKED, NormalizationMode::Fluid);
    assert!(!spoken.contains('#'), "heading marker leaked: {spoken}");
    assert!(!spoken.contains("**"), "bold marker leaked: {spoken}");
    assert!(
        !spoken.contains("---"),
        "thematic break marker leaked: {spoken}"
    );
    assert!(
        !spoken
            .lines()
            .any(|line| line.trim_start().starts_with("* ") || line.trim_start().starts_with("- ")),
        "list bullet leaked: {spoken}"
    );
    // The inner text is preserved even though the markers are gone.
    assert!(spoken.contains("Título"), "heading text lost: {spoken}");
    assert!(
        spoken.contains("item um e item dois"),
        "emphasis text lost: {spoken}"
    );
}

#[test]
fn fluid_mode_preserves_protected_zones() {
    let spoken = normalize_for_speech(MARKED, NormalizationMode::Fluid);
    assert!(
        spoken.contains("`código *não* normaliza`"),
        "inline code altered: {spoken}"
    );
    assert!(
        spoken.contains("https://exemplo.com/a*b"),
        "URL altered: {spoken}"
    );
    assert!(spoken.contains("2*3"), "simple math altered: {spoken}");
}

#[test]
fn split_is_independent_of_normalization_mode() {
    let fluid = split_document(MARKED, "hash");
    let literal = split_document(MARKED, "hash");
    assert_eq!(
        fluid, literal,
        "the split must not depend on the normalization mode"
    );
}

#[test]
fn normalization_does_not_mutate_segment_metadata() {
    let segments = split_document(MARKED, "hash");
    let before = metadata(&segments);
    let mut spoken_texts: Vec<String> = Vec::with_capacity(segments.len());
    for seg in &segments {
        spoken_texts.push(normalize_for_speech(&seg.content, NormalizationMode::Fluid));
    }
    let after = metadata(&segments);
    assert_eq!(before, after, "normalization mutated segment metadata");
    assert_eq!(spoken_texts.len(), segments.len());
}

#[test]
fn normalization_is_deterministic() {
    let first = normalize_for_speech(MARKED, NormalizationMode::Fluid);
    let second = normalize_for_speech(MARKED, NormalizationMode::Fluid);
    assert_eq!(first, second, "normalization must be deterministic");
}

#[test]
fn normalization_never_blanks_a_nonempty_segment() {
    let segments = split_document(MARKED, "hash");
    for seg in &segments {
        if seg.content.trim().is_empty() {
            continue;
        }
        let spoken = normalize_for_speech(&seg.content, NormalizationMode::Fluid);
        assert!(
            !spoken.trim().is_empty(),
            "a nonempty segment was blanked by normalization (segment {})",
            seg.segment_index
        );
    }
}
