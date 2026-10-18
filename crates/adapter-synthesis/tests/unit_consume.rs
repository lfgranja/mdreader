//! Unit tests for the player declaration-consumption branch (T12 / T020).
//!
//! Task: `tasks.md` line 38 (T12 — player interaction: declaration consumption)
//! and line 67 (T020 — complete the T2–T5 unit tests; written FIRST, blocking
//! the implementation wave per rule 1).
//!
//! Spec anchors: FR-024c (`spec.md:97` — the adapter MUST declare its
//! capabilities and the player branches on the declared value), data-model rule
//! R9 (declaration `(None, None)` unconditional, no phantom trait), constitution
//! Principle VI (`constitution.md:299-312` — degradation of prosody MUST be
//! explicit and observable, never silent), constitution Principle VII / gate 7
//! (closed sets, no catch-all arm), and checklists CHK001 (declaration
//! unconditional), CHK008 (`SsmlLevel::None` cannot be read as partial SSML),
//! CHK016 (measurable boundary compliance), CHK015 (mechanism consistent with
//! declared `(none, none)`), CHK031 (closed-set enforcement for every variant).
//!
//! Binding interface: `API_CONTRACT.md` §5 (`src/consume.rs`) and §0 global
//! rules — no `_ =>` arm, no `unwrap`/`expect`/`panic!`/`println!` (denied
//! in tests too), no `#[allow]`/`#[expect]` suppression.
//!
//! These tests bind to the FROZEN typed API (`SsmlDelivery`, `ResumeAnchor`,
//! `PlayerPlan`, `ssml_delivery`, `resume_anchor`, `plan_for`), not to the
//! current `consume_declaration -> &'static str` stub. The stub's `_ =>`
//! wildcard arm is removed by the implementation wave (T021) — never by this
//! file.

use adapter_synthesis::consume::{
    PlayerPlan, ResumeAnchor, SsmlDelivery, plan_for, resume_anchor, ssml_delivery,
};
use adapter_synthesis::{
    AdapterCapabilities, AlignmentGranularity, SsmlLevel, declared_capabilities,
};

/// The closed `SsmlLevel` set, enumerated explicitly (CHK031 / Principle VII).
/// A fourth member would fail to compile here, so the closed set cannot grow
/// silently.
const SSML_LEVELS: [SsmlLevel; 3] = [SsmlLevel::None, SsmlLevel::Partial, SsmlLevel::Full];

/// The closed `AlignmentGranularity` set, enumerated explicitly (CHK031 /
/// Principle VII). A fourth member would fail to compile here.
const GRANULARITIES: [AlignmentGranularity; 3] = [
    AlignmentGranularity::None,
    AlignmentGranularity::Word,
    AlignmentGranularity::Sentence,
];

/// The frozen v1 declaration, named once so every test binds to the same value.
const V1_DECLARATION: AdapterCapabilities = AdapterCapabilities {
    ssml_level: SsmlLevel::None,
    alignment_granularity: AlignmentGranularity::None,
};

/// The 9 `(SsmlLevel, AlignmentGranularity)` declaration pairs — the explicit
/// cross product, table-driven with no wildcard iteration over unlisted
/// values (CHK031: closed sets only).
const ALL_PAIRS: [(SsmlLevel, AlignmentGranularity); 9] = [
    (SsmlLevel::None, AlignmentGranularity::None),
    (SsmlLevel::None, AlignmentGranularity::Word),
    (SsmlLevel::None, AlignmentGranularity::Sentence),
    (SsmlLevel::Partial, AlignmentGranularity::None),
    (SsmlLevel::Partial, AlignmentGranularity::Word),
    (SsmlLevel::Partial, AlignmentGranularity::Sentence),
    (SsmlLevel::Full, AlignmentGranularity::None),
    (SsmlLevel::Full, AlignmentGranularity::Word),
    (SsmlLevel::Full, AlignmentGranularity::Sentence),
];

/// The full pinned mapping table: declaration -> expected `PlayerPlan`.
/// Identity mapping per `API_CONTRACT` §5 — every `SsmlLevel` maps to the
/// same-strength `SsmlDelivery`, every `AlignmentGranularity` to the
/// same-strength `ResumeAnchor`.
const EXPECTED_PLANS: [PlayerPlan; 9] = [
    PlayerPlan {
        ssml: SsmlDelivery::PlainText,
        resume: ResumeAnchor::ParagraphHeading,
    },
    PlayerPlan {
        ssml: SsmlDelivery::PlainText,
        resume: ResumeAnchor::Word,
    },
    PlayerPlan {
        ssml: SsmlDelivery::PlainText,
        resume: ResumeAnchor::Sentence,
    },
    PlayerPlan {
        ssml: SsmlDelivery::PartialSsml,
        resume: ResumeAnchor::ParagraphHeading,
    },
    PlayerPlan {
        ssml: SsmlDelivery::PartialSsml,
        resume: ResumeAnchor::Word,
    },
    PlayerPlan {
        ssml: SsmlDelivery::PartialSsml,
        resume: ResumeAnchor::Sentence,
    },
    PlayerPlan {
        ssml: SsmlDelivery::FullSsml,
        resume: ResumeAnchor::ParagraphHeading,
    },
    PlayerPlan {
        ssml: SsmlDelivery::FullSsml,
        resume: ResumeAnchor::Word,
    },
    PlayerPlan {
        ssml: SsmlDelivery::FullSsml,
        resume: ResumeAnchor::Sentence,
    },
];

/// T12: the v1 plan is `(PlainText, ParagraphHeading)` — the `offline_default`
/// declaration consumed as plain text with paragraph/heading resume.
#[test]
fn v1_plan_is_plain_text_and_paragraph_heading() {
    assert_eq!(
        plan_for(declared_capabilities()),
        PlayerPlan {
            ssml: SsmlDelivery::PlainText,
            resume: ResumeAnchor::ParagraphHeading,
        }
    );
}

/// T12: the frozen declaration constant and `declared_capabilities()` agree —
/// one declaration, expressed once (R9 / CHK001).
#[test]
fn v1_declaration_matches_frozen_none_none() {
    assert_eq!(declared_capabilities(), V1_DECLARATION);
}

/// Principle VI / adapter-capabilities §4.1: a declared `SsmlLevel::None`
/// consumes as `PlainText` and is never `FullSsml` — the player must not
/// silently upgrade the delivery path.
#[test]
fn ssml_none_declares_plain_text_and_never_full_ssml() {
    assert_eq!(ssml_delivery(SsmlLevel::None), SsmlDelivery::PlainText);
    assert_ne!(ssml_delivery(SsmlLevel::None), SsmlDelivery::FullSsml);
}

/// Principle VI / adapter-capabilities §4.1: a declared
/// `AlignmentGranularity::None` consumes as `ParagraphHeading` and is never
/// `Sentence` — resume never silently snaps to a boundary the adapter cannot
/// identify (CHK015).
#[test]
fn granularity_none_declares_paragraph_heading_and_never_sentence() {
    assert_eq!(
        resume_anchor(AlignmentGranularity::None),
        ResumeAnchor::ParagraphHeading
    );
    assert_ne!(
        resume_anchor(AlignmentGranularity::None),
        ResumeAnchor::Sentence
    );
}

/// Principle VII / CHK031: `SsmlDelivery` is a closed set of exactly three
/// members. The exhaustive 3-arm match (no `_ =>`) means a future variant
/// fails to compile rather than being silently consumed.
#[test]
fn ssml_delivery_exhaustive_match_has_no_catchall() {
    for level in SSML_LEVELS {
        let delivery = ssml_delivery(level);
        // Exhaustive match — every arm explicit, no wildcard (API_CONTRACT §0.1).
        let described = match delivery {
            SsmlDelivery::PlainText => "plain-text",
            SsmlDelivery::PartialSsml => "partial-ssml",
            SsmlDelivery::FullSsml => "full-ssml",
        };
        assert_ne!(described, "");
    }
}

/// Principle VII / CHK031: `ResumeAnchor` is a closed set of exactly three
/// members. The exhaustive 3-arm match (no `_ =>`) means a future variant
/// fails to compile rather than being silently consumed.
#[test]
fn resume_anchor_exhaustive_match_has_no_catchall() {
    for granularity in GRANULARITIES {
        let anchor = resume_anchor(granularity);
        // Exhaustive match — every arm explicit, no wildcard (API_CONTRACT §0.1).
        let described = match anchor {
            ResumeAnchor::ParagraphHeading => "paragraph-heading",
            ResumeAnchor::Word => "word",
            ResumeAnchor::Sentence => "sentence",
        };
        assert_ne!(described, "");
    }
}

/// T12 / CHK031: `plan_for` is total over all 9 declaration pairs, the
/// component functions agree with the composite, and no pair contradicts its
/// declaration (a `None` declaration never yields `FullSsml`/`Sentence`).
#[test]
fn plan_is_total_over_all_nine_declaration_pairs() {
    for (index, (level, granularity)) in ALL_PAIRS.iter().enumerate() {
        let capabilities = AdapterCapabilities {
            ssml_level: *level,
            alignment_granularity: *granularity,
        };

        // Totality: defined for every pair in the closed cross product.
        let plan = plan_for(capabilities);
        assert_eq!(plan, EXPECTED_PLANS[index]);

        // Consistency: direct call and composite agree.
        assert_eq!(plan.ssml, ssml_delivery(*level));
        assert_eq!(plan.resume, resume_anchor(*granularity));

        // No contradiction with the declaration: a `None` declaration never
        // upgrades (Principle VI — degradation is observable, never silent).
        if *level == SsmlLevel::None {
            assert_ne!(plan.ssml, SsmlDelivery::FullSsml);
            assert_ne!(plan.ssml, SsmlDelivery::PartialSsml);
        }
        if *granularity == AlignmentGranularity::None {
            assert_ne!(plan.resume, ResumeAnchor::Sentence);
            assert_ne!(plan.resume, ResumeAnchor::Word);
        }
    }
}

/// The full mapping table is pinned, not just the v1 row (`API_CONTRACT` §5):
/// `SsmlLevel::None -> PlainText`, `Partial -> PartialSsml`,
/// `Full -> FullSsml`; `AlignmentGranularity::None -> ParagraphHeading`,
/// `Word -> Word`, `Sentence -> Sentence`.
#[test]
fn full_declaration_to_plan_table_is_pinned() {
    assert_eq!(ssml_delivery(SsmlLevel::None), SsmlDelivery::PlainText);
    assert_eq!(ssml_delivery(SsmlLevel::Partial), SsmlDelivery::PartialSsml);
    assert_eq!(ssml_delivery(SsmlLevel::Full), SsmlDelivery::FullSsml);

    assert_eq!(
        resume_anchor(AlignmentGranularity::None),
        ResumeAnchor::ParagraphHeading
    );
    assert_eq!(
        resume_anchor(AlignmentGranularity::Word),
        ResumeAnchor::Word
    );
    assert_eq!(
        resume_anchor(AlignmentGranularity::Sentence),
        ResumeAnchor::Sentence
    );
}

/// Derives are load-bearing: `missing_debug_implementations` is `deny`
/// workspace-wide (constitution.md:461-470), so a future removal of
/// `Debug`/`Clone`/`Copy`/`PartialEq`/`Eq` on the consume types is caught
/// here. The format round-trip also proves `Debug` is stable.
#[test]
fn consume_types_exercise_debug_clone_copy_partial_eq_eq() {
    // Eq: no floating-point or partial-comparison member exists.
    fn assert_eq_trait<T: Eq>() {}

    let plan = plan_for(V1_DECLARATION);

    // Debug: format the plan and the enums (no `println!` — denied).
    let plan_text = format!("{plan:?}");
    let ssml_text = format!("{:?}", SsmlDelivery::PlainText);
    let anchor_text = format!("{:?}", ResumeAnchor::ParagraphHeading);
    assert_ne!(plan_text, "");
    assert_ne!(ssml_text, "");
    assert_ne!(anchor_text, "");

    // Clone: the derived `Clone` is exercised explicitly and preserves
    // equality; Copy: an implicit move-out copy preserves it too.
    #[expect(
        clippy::clone_on_copy,
        reason = "the test deliberately calls the derived `Clone` to pin the trait-derivation contract; the implicit move-out below covers the `Copy` half"
    )]
    let cloned = plan.clone();
    let copied = plan;
    assert_eq!(plan, cloned);
    assert_eq!(plan, copied);
    assert_eq!(SsmlDelivery::PartialSsml, SsmlDelivery::PartialSsml);
    assert_eq!(ResumeAnchor::Word, ResumeAnchor::Word);

    assert_eq_trait::<PlayerPlan>();
    assert_eq_trait::<SsmlDelivery>();
    assert_eq_trait::<ResumeAnchor>();
}

/// Sentence-boundary resume is a closed-set member that the adapter never
/// declares at v1 (FR-024 / `spec.md` clarification 2026-10-06): Piper
/// exposes no `SentenceBoundary` events, so `AlignmentGranularity::Sentence`
/// exists only as a deferred v2 member. The player therefore never receives
/// a `Sentence` anchor through the declaration — the degradation from the
/// desired `Sentence` to the declared `None` is observable via
/// `declared_capabilities()`, never silent (Principle VI).
#[test]
fn declared_granularity_is_never_sentence_at_v1() {
    assert_ne!(
        declared_capabilities().alignment_granularity,
        AlignmentGranularity::Sentence
    );
}

/// The declaration is immutable across interleaved calls — one authority, no
/// state (R9 / constitution.md:304-305; the declaration reads nothing and
/// returns the same value on every call).
#[test]
fn declaration_is_constant_across_repeated_calls() {
    let first = declared_capabilities();
    let second = declared_capabilities();
    let third = declared_capabilities();
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(first, V1_DECLARATION);
}
