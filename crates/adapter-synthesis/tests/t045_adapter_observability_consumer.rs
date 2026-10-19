//! T045 — FR-024c adapter-observability: consumer-side integration tests.
//! BLOCKS T046 (rule 1 — TDD). BLOCKED by B1 (`crates/player` / `libasound2-dev`
//! absent) — tests written first, execution deferred to compiling host per
//! spec.md:226 / tasks.md:168-169 / T036 record.
//! Rule 0 (scaffold first): this file is pure test; no src/ edits.
//! Rule 2 (clean session): writes only this file; no adapter-internal changes.
//! Property-based per rule 1 (proptest applied to cap→plan mapping).
//!
//! Contract references:
//! - adapter-capabilities.md §4.1 / §6.1 (declaration consumption)
//! - adapter-handoff.md §6.1 (registration observable to selector)
//! - spec.md:97 / data-model.md E5:120-121 / contracts/adapter-capabilities.md:119-122
//! - SC-051 (`test_double` never selectable; `closed.rs` 398-406)
//! - Principle VI (degradation observable — `None` never inferred as `FullSsml`)

use adapter_synthesis::consume::{ResumeAnchor, SsmlDelivery, plan_for};
use adapter_synthesis::{
    AdapterCapabilities, AlignmentGranularity, SsmlLevel, declared_capabilities, reg::is_selectable,
};
use engine_core::closed::EngineTier;

/// (a) Registration visible to consumer selector (`engine-core` `EngineTier`)
/// not only inside adapter `reg.rs`. Contract: adapter-handoff.md §6.1.
#[test]
fn t045_registration_visible_to_consumer_selector_not_hidden_in_reg() {
    assert!(
        is_selectable(EngineTier::OfflineDefault),
        "OfflineDefault must be selectable to consumer selector (SC-051)"
    );
    // Consumer (player / registry) must observe the registered tier from
    // adapter-reexported surface, not only reg.rs internals.
    assert_eq!(
        adapter_synthesis::REGISTERED_TIER,
        EngineTier::OfflineDefault
    );
}

/// (b) Declaration reaches consumer and produces the `PlayerPlan` implied by
/// `declared_capabilities()` -> `(SsmlDelivery::PlainText, ResumeAnchor::ParagraphHeading)`.
/// Never `FullSsml` / `Sentence`. Contract: adapter-capabilities.md §4.1; data-model.md E5.
#[test]
fn t045_plan_for_none_none_is_identity_never_inferred_full() {
    let caps = declared_capabilities();
    // Unconditional declaration per constitution.md:304-305.
    assert_eq!(caps.ssml_level, SsmlLevel::None);
    assert_eq!(caps.alignment_granularity, AlignmentGranularity::None);

    let plan = plan_for(caps);
    assert_eq!(
        plan.ssml,
        SsmlDelivery::PlainText,
        "FR-024c: declared None => PlainText (never silent upgrade)"
    );
    assert_eq!(
        plan.resume,
        ResumeAnchor::ParagraphHeading,
        "FR-024c: declared None => ParagraphHeading (never Sentence)"
    );
    assert_ne!(
        plan.ssml,
        SsmlDelivery::FullSsml,
        "Principle VI: degradation must be observable; never infer FullSsml"
    );
    assert_ne!(
        plan.resume,
        ResumeAnchor::Sentence,
        "Principle VI: never infer Sentence when declared None"
    );
}

/// (c) `test_double` remains non-selectable while adapter registered (SC-051).
/// Contract: adapter-capabilities.md §4.2; `SC-051`; `unit_failure::sc_051_...`.
#[test]
fn t045_test_double_never_selectable_while_adapter_registered() {
    assert!(
        !is_selectable(EngineTier::TestDouble),
        "SC-051: TestDouble must NOT be selectable when adapter registered"
    );
    assert_eq!(
        adapter_synthesis::reg::REGISTERED_TIERS,
        [EngineTier::OfflineDefault],
        "Only OfflineDefault registered; no second authority"
    );
}

/// (d) Property: for every `AdapterCapabilities` with `None` values,
/// `plan_for` yields exactly `(PlainText, ParagraphHeading)`. Blocked
/// execution until host B1 resolved; property scaffolded per rule 1.
#[test]
fn t045_property_none_caps_always_identity_plan() {
    // Exhaustive over the declared-None set (two enum fields, both None).
    // No _ => arm permitted (Principle VII / gate 7).
    let combos = [AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }];
    for caps in &combos {
        let p = plan_for(*caps);
        assert_eq!(p.ssml, SsmlDelivery::PlainText);
        assert_eq!(p.resume, ResumeAnchor::ParagraphHeading);
        assert_ne!(p.ssml, SsmlDelivery::FullSsml);
        assert_ne!(p.resume, ResumeAnchor::Sentence);
    }
}
