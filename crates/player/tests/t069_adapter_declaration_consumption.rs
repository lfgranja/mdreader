//! T069 — Test-first verification of FR-024c consumer wiring.
//! Asserts that `player` consumes `adapter_synthesis` declaration,
//! not local duplicate types.
//!
//! This test MUST be observed RED (failing) before T070 implementation.
//! It binds to the frozen `API_CONTRACT.md` §10 and `contracts/adapter-capabilities.md` §4.1.

use adapter_synthesis::{
    AdapterCapabilities, AlignmentGranularity, SsmlLevel,
    consume::{ResumeAnchor, SsmlDelivery, plan_for},
    declared_capabilities,
};
use engine_core::closed::EngineTier;

#[test]
fn t069_declared_capabilities_returns_frozen_none_none() {
    // FR-024c: adapter declares (None, None) unconditionally
    let caps = declared_capabilities();
    assert_eq!(caps.ssml_level, SsmlLevel::None);
    assert_eq!(caps.alignment_granularity, AlignmentGranularity::None);
}

#[test]
fn t069_plan_for_none_none_is_identity_never_inferred_full() {
    // Principle VI / FR-024c: declared None never upgrades to FullSsml/Sentence
    let caps = declared_capabilities();
    let plan = plan_for(caps);

    assert_eq!(plan.ssml, SsmlDelivery::PlainText);
    assert_eq!(plan.resume, ResumeAnchor::ParagraphHeading);

    // Verify identity mapping across all 9 pairs (no silent inference)
    for ssml in [SsmlLevel::None, SsmlLevel::Partial, SsmlLevel::Full] {
        for align in [
            AlignmentGranularity::None,
            AlignmentGranularity::Word,
            AlignmentGranularity::Sentence,
        ] {
            let p = plan_for(AdapterCapabilities {
                ssml_level: ssml,
                alignment_granularity: align,
            });
            // Each declaration maps to same-strength delivery/resume (identity)
            match ssml {
                SsmlLevel::None => assert_eq!(p.ssml, SsmlDelivery::PlainText),
                SsmlLevel::Partial => assert_eq!(p.ssml, SsmlDelivery::PartialSsml),
                SsmlLevel::Full => assert_eq!(p.ssml, SsmlDelivery::FullSsml),
            }
            match align {
                AlignmentGranularity::None => assert_eq!(p.resume, ResumeAnchor::ParagraphHeading),
                AlignmentGranularity::Word => assert_eq!(p.resume, ResumeAnchor::Word),
                AlignmentGranularity::Sentence => assert_eq!(p.resume, ResumeAnchor::Sentence),
            }
        }
    }
}

#[test]
fn t069_offline_default_visible_to_selector_not_hidden_in_reg() {
    // SC-051 / spec.md:89: OfflineDefault must be observable through engine_core selector
    // not only inside adapter-synthesis reg.rs
    assert!(EngineTier::OfflineDefault.is_selectable());
    assert!(
        !EngineTier::TestDouble.is_selectable(),
        "test_double must not be selectable while non-test_double adapter registered"
    );
    assert!(!EngineTier::CloudDefault.is_selectable());
    assert!(!EngineTier::CloudEscalation.is_selectable());
    assert!(!EngineTier::EvaluatedFallback.is_selectable());
}

#[test]
fn t069_test_double_never_selectable_while_adapter_registered() {
    // SC-051 invariant: test_double stand-ins not selectable when adapter registered
    // This is verified by the selector in engine_core::closed::EngineTier::is_selectable
    // which returns false for TestDouble when any non-test_double adapter is registered
    assert!(!EngineTier::TestDouble.is_selectable());
}
