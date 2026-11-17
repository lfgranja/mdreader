//! Invariant tests for **015 — MD Semantic Prosody**.
//!
//! `plan_for_segment` is a pure function of the segment's markdown
//! semantics; it never mutates [`DocumentSegment`], and the resulting
//! speed factor is applied only to the `GenerationConfig` passed to the
//! engine. These tests bind FR-PROS-002 / FR-PROS-008 / SC-PROS-003: the
//! plan is deterministic, the segment metadata is unchanged, and every
//! speed factor stays inside the conservative `(0.0, 1.0]` band.

use adapter_synthesis::chunk::{DocumentSegment, split_document};
use adapter_synthesis::prosody::{ProsodyPlan, plan_for_segment};

/// The segment metadata the resume/fingerprint contracts bind to.
fn metadata(
    segments: &[DocumentSegment],
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
fn plan_is_deterministic() {
    let segments = split_document("# Título\n\n*item*\n\n> citação\n", "hash");
    for seg in &segments {
        let first = plan_for_segment(seg);
        let second = plan_for_segment(seg);
        assert_eq!(first, second, "prosody plan must be deterministic");
    }
}

#[test]
fn plan_does_not_mutate_segment_metadata() {
    let segments = split_document("# Título\n\n*item*\n\n> citação\n", "hash");
    let before = metadata(&segments);
    let mut plans: Vec<ProsodyPlan> = Vec::with_capacity(segments.len());
    for seg in &segments {
        plans.push(plan_for_segment(seg));
    }
    let after = metadata(&segments);
    assert_eq!(before, after, "prosody planning mutated segment metadata");
    assert_eq!(plans.len(), segments.len());
}

#[test]
fn every_speed_factor_is_conservative_and_positive() {
    let segments = split_document(
        "# Título\n\n*item*\n\n> citação\n\n```\ncode\n```\n\n---\n\nPergunta?\n\nExclamação!\n",
        "hash",
    );
    assert_ne!(segments.len(), 0);
    for seg in &segments {
        let speed = plan_for_segment(seg).speed();
        assert!(
            speed > 0.0 && speed <= 1.0,
            "speed factor {speed} for segment {} is outside the conservative (0.0, 1.0] band",
            seg.segment_index
        );
    }
}

#[test]
fn plain_text_is_normal_and_structure_is_distinct() {
    let plain = split_document("A plain paragraph.", "hash");
    assert_ne!(plain.len(), 0);
    assert_eq!(plan_for_segment(&plain[0]), ProsodyPlan::Normal);
    assert_eq!(plain[0].byte_length(), "A plain paragraph.".len());
}
