//! Property tests for the capability declaration — task T3
//! (`tasks.md:23`) and task T020 (`tasks.md:67`) of feature
//! `002-fr-024-synthesis-adapter`.
//!
//! Spec anchors: FR-024c (`spec.md:97` — the adapter MUST declare
//! `(SsmlLevel::None, AlignmentGranularity::None)` via
//! `declared_capabilities()`); `constitution.md:304-305` (every
//! adapter MUST declare its SSML capability level; degradation is
//! explicit and observable, never silent); `adapter-interface.md`
//! CHK001/CHK003/CHK008/CHK025 and `mechanism-gate.md`
//! CHK003/CHK031 (closed sets: no `Other`/`Unknown`, no
//! catch-all); `API_CONTRACT.md` §1 (the frozen `src/lib.rs`
//! declaration surface) and §8 (the single declaration expressed
//! once in `src/capabilities.rs`).
//!
//! Rule 1: tests are written FIRST and block the implementation
//! wave (T021, implemented by a different agent). This file binds
//! to the real crate — it re-declares nothing. The previous
//! version of this file was defective (finding F3): it declared
//! local copies of `SsmlLevel`, `AlignmentGranularity`,
//! `AdapterCapabilities` and `declared_capabilities`, so it only
//! ever asserted that its own copy was `(None, None)` and never
//! touched the crate. Those copies are deleted here.

use proptest::prelude::*;

use adapter_synthesis::capabilities::capabilities;
use adapter_synthesis::chunk::split_document;
use adapter_synthesis::{
    AdapterCapabilities, AlignmentGranularity, SsmlLevel, declared_capabilities,
};

/// Every `SsmlLevel` member, mapped to its own `u8` discriminant.
///
/// The match is exhaustive with **no `_ =>` arm**: it compiles only
/// while `SsmlLevel` has exactly the three members it names. A
/// fourth member (an `Other`/`Unknown` catch-all) would fail to
/// compile right here — that compile failure is the assertion
/// (constitution Principle VII, `constitution.md:314-317`;
/// `mechanism-gate.md` CHK031; `API_CONTRACT.md` §0 rule 1).
const fn ssml_discriminant(level: SsmlLevel) -> u8 {
    match level {
        SsmlLevel::None => 0,
        SsmlLevel::Partial => 1,
        SsmlLevel::Full => 2,
    }
}

/// Every `AlignmentGranularity` member, mapped to its own `u8`
/// discriminant. Exhaustive with **no `_ =>` arm**: a fourth member
/// would fail to compile here — that is the point of the test
/// (constitution Principle VII; `mechanism-gate.md` CHK031).
const fn granularity_discriminant(granularity: AlignmentGranularity) -> u8 {
    match granularity {
        AlignmentGranularity::None => 0,
        AlignmentGranularity::Word => 1,
        AlignmentGranularity::Sentence => 2,
    }
}

/// The closed `SsmlLevel` set, enumerated once. Exactly three
/// members — the exhaustive match above refuses a fourth.
const SSML_LEVELS: [SsmlLevel; 3] = [SsmlLevel::None, SsmlLevel::Partial, SsmlLevel::Full];

/// The closed `AlignmentGranularity` set, enumerated once. Exactly
/// three members — the exhaustive match above refuses a fourth.
const ALIGNMENT_GRANULARITIES: [AlignmentGranularity; 3] = [
    AlignmentGranularity::None,
    AlignmentGranularity::Word,
    AlignmentGranularity::Sentence,
];

/// Count of distinct values in `values` — the closed-set size check
/// for the `u8` discriminants of every member.
fn distinct_count(values: &[u8]) -> usize {
    let mut count = 0;
    for (index, value) in values.iter().enumerate() {
        if !values[..index].contains(value) {
            count += 1;
        }
    }
    count
}

/// The declaration is `(SsmlLevel::None, AlignmentGranularity::None)`
/// — the `offline_default` adapter declares plain-text delivery and
/// paragraph/heading resume (FR-024c; `adapter-capabilities.md`
/// §3.1/§3.2; `adapter-interface.md` CHK001/CHK008).
#[test]
fn declared_capabilities_is_always_none_none() {
    let declared = declared_capabilities();
    assert_eq!(declared.ssml_level, SsmlLevel::None);
    assert_eq!(declared.alignment_granularity, AlignmentGranularity::None);
}

/// The declaration is immutable: 64 repeated calls return the same
/// value — no state exists that could change it
/// (`constitution.md:304-305`; `API_CONTRACT.md` §1).
#[test]
fn declaration_is_constant_across_64_repeated_calls() {
    let first = declared_capabilities();
    for _ in 0..64 {
        assert_eq!(declared_capabilities(), first);
    }
    assert_eq!(first.ssml_level, SsmlLevel::None);
    assert_eq!(first.alignment_granularity, AlignmentGranularity::None);
}

// The declaration is unconditional: no input, document, or
// operation can change it. Interleaves 64 declaration calls with
// arbitrary work — string allocation and a real `split_document`
// on a small document — and the declaration is unaffected
// (`constitution.md:304-305`; `API_CONTRACT.md` §1: "unconditional
// declaration requires no synthesis"; `adapter-capabilities.md` §5:
// declaration can be called at registration time, before the first
// chunk is delivered).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn declaration_never_changes_under_interleaved_arbitrary_work(
        document in "[a-z\\s]{0,256}",
        document_hash in "[a-f0-9]{0,64}",
    ) {
        let first = declared_capabilities();
        for _ in 0..64 {
            // Arbitrary other work: allocate strings ...
            let scratch = format!("interleaved-{document_hash}-{document}");
            // ... and split a small document through the real chunker.
            let segments = split_document(&document, &document_hash);
            let _segment_count = segments.len();
            // The declaration is unaffected by all of it.
            prop_assert_eq!(declared_capabilities(), first);
            prop_assert_eq!(declared_capabilities().ssml_level, SsmlLevel::None);
            prop_assert_eq!(
                declared_capabilities().alignment_granularity,
                AlignmentGranularity::None
            );
            // The interleaved work really happened (the allocations
            // and the split are live, not optimized away).
            prop_assert!(scratch.starts_with("interleaved-"));
        }
    }
}

/// One declaration, expressed once: `capabilities::capabilities()`
/// and `declared_capabilities()` agree exactly (`API_CONTRACT.md`
/// §8 — there is no second authority over the declaration).
#[test]
fn capabilities_module_agrees_with_declared_capabilities() {
    assert_eq!(capabilities(), declared_capabilities());
    assert_eq!(capabilities().ssml_level, SsmlLevel::None);
    assert_eq!(
        capabilities().alignment_granularity,
        AlignmentGranularity::None
    );
}

/// Both enums are exhaustively matched with a distinct value per
/// arm and no `_ =>` arm: the matches above compile only while
/// each enum has exactly the three members exercised here
/// (constitution Principle VII; `adapter-interface.md` CHK003).
#[test]
fn ssml_level_and_alignment_granularity_are_exhaustively_matched() {
    assert_eq!(ssml_discriminant(SsmlLevel::None), 0);
    assert_eq!(ssml_discriminant(SsmlLevel::Partial), 1);
    assert_eq!(ssml_discriminant(SsmlLevel::Full), 2);
    assert_eq!(granularity_discriminant(AlignmentGranularity::None), 0);
    assert_eq!(granularity_discriminant(AlignmentGranularity::Word), 1);
    assert_eq!(granularity_discriminant(AlignmentGranularity::Sentence), 2);
}

/// No `Other`/`Unknown` member exists: the closed sets are exactly
/// three members each, and every member maps to its own `u8`
/// discriminant — the count of distinct discriminants is 3
/// (constitution Principle VII, `constitution.md:314-317`;
/// `mechanism-gate.md` CHK031; `adapter-interface.md` CHK003).
#[test]
fn both_enums_are_exactly_three_member_closed_sets() {
    assert_eq!(SSML_LEVELS.len(), 3);
    assert_eq!(ALIGNMENT_GRANULARITIES.len(), 3);

    let ssml_discriminants = [
        ssml_discriminant(SsmlLevel::None),
        ssml_discriminant(SsmlLevel::Partial),
        ssml_discriminant(SsmlLevel::Full),
    ];
    assert_eq!(ssml_discriminants, [0, 1, 2]);
    assert_eq!(distinct_count(&ssml_discriminants), 3);

    let granularity_discriminants = [
        granularity_discriminant(AlignmentGranularity::None),
        granularity_discriminant(AlignmentGranularity::Word),
        granularity_discriminant(AlignmentGranularity::Sentence),
    ];
    assert_eq!(granularity_discriminants, [0, 1, 2]);
    assert_eq!(distinct_count(&granularity_discriminants), 3);
}

// Every member of the closed sets, chosen by index 0..3, is one
// of the three named members and never a catch-all: the
// enumeration is total, so an unnamed member cannot be selected
// or handled silently (constitution Principle VII;
// `mechanism-gate.md` CHK031).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn any_member_chosen_by_index_is_one_of_the_three_never_a_catch_all(index in 0..3usize) {
        let level = SSML_LEVELS[index];
        prop_assert!(
            level == SsmlLevel::None
                || level == SsmlLevel::Partial
                || level == SsmlLevel::Full
        );
        prop_assert!(ssml_discriminant(level) < 3);

        let granularity = ALIGNMENT_GRANULARITIES[index];
        prop_assert!(
            granularity == AlignmentGranularity::None
                || granularity == AlignmentGranularity::Word
                || granularity == AlignmentGranularity::Sentence
        );
        prop_assert!(granularity_discriminant(granularity) < 3);
    }
}

/// The declaration precedes any synthesis: it is callable as the
/// first statement of a fresh test, with no registration, no
/// loaded voice model, no split document and no synthesized audio
/// before it. This tautology defends the ordering — the
/// declaration depends on no engine state (`constitution.md:304-305`;
/// `adapter-capabilities.md` §5: "declaration can be called at
/// registration time, before the first chunk delivered —
/// unconditional declaration requires no synthesis").
#[test]
fn declaration_precedes_any_synthesis_and_requires_no_engine_state() {
    let declared = declared_capabilities();
    assert_eq!(declared.ssml_level, SsmlLevel::None);
    assert_eq!(declared.alignment_granularity, AlignmentGranularity::None);
}

/// `AdapterCapabilities` derives `Debug`, `Clone`, `Copy`,
/// `PartialEq` and `Eq` — exercised here so a future removal is
/// caught in this file rather than at the next consumer
/// (`missing_debug_implementations` is `deny` at workspace level,
/// `constitution.md:462`; `API_CONTRACT.md` §1).
#[expect(
    clippy::clone_on_copy,
    reason = "the test's contract is that the derived Clone impl compiles and yields an equal value — the lint's suggested removal would stop exercising Clone entirely; the Copy impl is separately exercised by the assignment below"
)]
#[test]
fn adapter_capabilities_derives_debug_clone_copy_partial_eq_and_eq() {
    let declared = declared_capabilities();

    // Debug: the derived implementation names the struct and fields.
    let rendered = format!("{declared:?}");
    assert!(rendered.contains("AdapterCapabilities"));
    assert!(rendered.contains("ssml_level"));
    assert!(rendered.contains("alignment_granularity"));

    // Clone: an independent copy compares equal.
    let cloned = declared.clone();
    assert_eq!(cloned, declared);

    // Copy: the assignment copies, leaving `declared` usable.
    let copied = declared;
    assert_eq!(copied, declared);

    // PartialEq / Eq: equal to a fresh declaration, never equal to
    // a stronger, undeclared capability pair.
    assert_eq!(declared, declared_capabilities());
    assert_ne!(
        declared,
        AdapterCapabilities {
            ssml_level: SsmlLevel::Full,
            alignment_granularity: AlignmentGranularity::Sentence,
        }
    );
}
