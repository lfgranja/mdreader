//! T059 — FR-024 synthesis adapter prop/declaration tests (test-only).
//! BLOCKS T060 — do NOT implement; observe RED only.
//! Rule 2 (clean session): writes only this file; no src/ edits.
//! Freeze bound: API_CONTRACT.md §1 / adapter-capabilities.md / adapter-handoff.md.

use adapter_synthesis::{
    AdapterCapabilities, AlignmentGranularity, SsmlLevel,
    declared_capabilities,
};
use adapter_synthesis::consume::{ResumeAnchor, SsmlDelivery, plan_for};
use adapter_synthesis::reg::{is_selectable, REGISTERED_TIERS};
use engine_core::closed::EngineTier;

// (a) OfflineDefault visible to selector, not only reg.rs.
// Contract: adapter-capabilities.md §4.1 / adapter-handoff.md §6.1 / API_CONTRACT.md §1.
// The selector must see the registered tier from the crate root,
// not just via reg.rs internal names.
#[test]
fn t059_offline_default_visible_to_selector_not_only_reg_rs() {
    assert_eq!(adapter_synthesis::REGISTERED_TIER, EngineTier::OfflineDefault);
    assert!(is_selectable(EngineTier::OfflineDefault));
    // The tier is visible to external selectors (player / registry),
    // not hidden inside reg.rs only.
    assert_eq!(REGISTERED_TIERS, [EngineTier::OfflineDefault]);
    assert_eq!(adapter_synthesis::REGISTERED_TIER, EngineTier::OfflineDefault);
}

// (b) plan_for(None, None) -> (PlainText, ParagraphHeading); never FullSsml.
// Contract: adapter-capabilities.md §3.1 / consume.rs identity mapping /
// API_CONTRACT.md §2 declaration consumes as PlainText/ParagraphHeading.
#[test]
fn t059_plan_for_none_none_is_plaintext_paragraph_never_fullssml() {
    let plan = plan_for(AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    });
    assert_eq!(plan.ssml, SsmlDelivery::PlainText);
    assert_eq!(plan.resume, ResumeAnchor::ParagraphHeading);
    // The v1 declaration never silently upgrades to FullSsml or Sentence.
    assert_ne!(plan.ssml, SsmlDelivery::FullSsml,
        "plan_for(None,None) must NEVER yield FullSsml (FR-024c; Principle VI)");
    assert_ne!(plan.resume, ResumeAnchor::Sentence,
        "plan_for(None,None) must NEVER yield Sentence (C1-A deferred)");
}

// (c) test_double non-selectable when adapter registered.
// Contract: adapter-handoff.md §6.2 / adapter-capabilities.md §4.1 /
// closed.rs 305-312 (SC-051 invariant).
#[test]
fn t059_test_double_non_selectable_when_adapter_registered() {
    // Adapter registered at OfflineDefault (not a fixture).
    assert!(is_selectable(EngineTier::OfflineDefault));
    // Fixture tiers must not be selectable while adapter registered.
    assert!(!is_selectable(EngineTier::TestDouble),
        "TestDouble non-selectable when adapter registered (SC-051 / 001/spec.md:89)");
}

// Frozen surface binding: declared_capabilities() is constant (None, None).
// API_CONTRACT.md §1 / adapter-capabilities.md §2.1 / constitution.md:304-305.
#[test]
fn t059_declared_capabilities_is_frozen_none_none() {
    let d = declared_capabilities();
    assert_eq!(d.ssml_level, SsmlLevel::None);
    assert_eq!(d.alignment_granularity, AlignmentGranularity::None);
    // Re-declare binds frozen surface — no change possible.
    assert_eq!(declared_capabilities(), d);
}
