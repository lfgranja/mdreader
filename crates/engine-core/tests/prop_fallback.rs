//! T069 — fallback-order + consent property tests (US5).
//!
//! Invariant under test (tasks.md T069, verbatim):
//! "Property test: fallback-order + consent in
//! `crates/engine-core/tests/prop_fallback.rs` (proptest:
//! order = ascending `EngineTier::ORDER` (remaining
//! same-tier → cloud_default → cloud_escalation →
//! test_double → evaluated_fallback last, never default);
//! `test_double` tiers are reachable only while no
//! non-`test_double` adapter is registered, asserted as
//! an invariant in both directions; consent for a
//! `test_double` is granted by construction (no egress)
//! and asserted as such; descent only behind fresh 3 s
//! offer naming lower tier; unconsented engine NEVER
//! selected; offer names exactly selectable engines;
//! expired fallback continues from last spoken position
//! never restart; no-alternative expires to
//! paused-with-reason; repeated failure re-offers
//! remaining + each recorded per FR-023, FR-023c; FAIL
//! before implementation)"
//!
//! Spec anchors: FR-023 (a named engine failure produces
//! a named failure and an offer of alternatives; the
//! user's choice is honoured, else the fallback
//! auto-takes-over from the last spoken position — never
//! a restart; a fresh 3 s offer window; no alternative →
//! paused-with-reason, never silence), FR-023a (the
//! offer names the alternatives), FR-023c (the fallback
//! order is the declared `EngineTier::ORDER`, ascending:
//! offline_default → cloud_default → cloud_escalation →
//! test_double → evaluated_fallback; consent is read per
//! attempt from app-private storage; the Edge endpoint
//! stays last and is never a default), FR-005 (the
//! no-alternative expiry pauses with a stated reason —
//! never silence), the 2026-10-05 clarification
//! (test_double engines — the stand-ins — are admissible
//! only while no non-test_double adapter is registered,
//! asserted in both directions; consent for a test_double
//! is granted by construction because it carries no
//! provider/egress field at all), contracts/engine-handoff.md
//! §"Fallback order" ("Remaining engines of the failing
//! tier → higher tiers in registry order → Edge evaluated
//! fallback last, never default. Unconsented engines are
//! never selected. Offers name exactly the engines the
//! fallback may select. Descent to a lower tier is
//! permitted only behind a fresh 3-second offer in the
//! shape of FR-023a naming the lower-tier engines"), and
//! SC-051 (the mid-document failover and forced-double-
//! failure drills the selector serves).
//!
//! FAILS before `crates/engine-core/src/fallback.rs`
//! (T075) exists: the `engine_core::fallback` module is
//! unresolved, so this test target does not build. That
//! build failure is the required pre-implementation red
//! state (T072).
//!
//! The canonical surface this file is the contract for
//! (T075 implements exactly this API, and nothing else):
//!
//! ```text
//! engine_core::fallback::{
//!     EngineDescriptor { pub id: String, pub tier: EngineTier },
//!     EngineOffer { pub engine_id: String, pub tier: EngineTier },
//!     Consent::{Granted, Denied},
//!     FallbackExpiry::{
//!         ContinueFromLastPosition { engine_id: String },
//!         PausedWithReason { reason: String },
//!     },
//!     FallbackSelector::{
//!         new(Vec<EngineDescriptor>) -> Self,
//!         selectable(&self) -> Vec<EngineOffer>,
//!         consent(&self, engine_id: &str) -> Consent,
//!         on_failure(&mut self, engine_id: &str) -> Option<EngineOffer>,
//!         expire_offer(&mut self) -> FallbackExpiry,
//!     },
//!     Default for FallbackSelector (the empty registry),
//! }
//! ```
//!
//! Property legs (F1–F8), each asserted by the property
//! named for it:
//!
//! - F1 [`selectable_engines_follow_the_declared_ascending_order`] —
//!   the selectable engines come back in ascending
//!   `EngineTier::ORDER`: every engine of a lower tier
//!   precedes every engine of a higher tier, and within
//!   a tier the remaining same-tier engines keep registry
//!   order; evaluated_fallback is last when present and
//!   never a default.
//! - F2 [`test_double_engines_are_admissible_only_while_no_other_adapter_is_registered`] —
//!   both directions: a test_double is selectable when the
//!   registry holds only test_doubles, and is NOT
//!   selectable when any non-test_double adapter is
//!   registered.
//! - F3 [`test_double_consent_is_granted_by_construction`] —
//!   consent for a test_double is always Granted: a
//!   test_double carries no provider/egress field at all,
//!   so there is no egress to gate.
//! - F4 [`an_unconsented_engine_is_never_selectable`] —
//!   whatever the per-attempt consent read yields, a
//!   denied engine is never selected.
//! - F5 [`a_failure_descends_behind_a_fresh_offer_and_names_exactly_the_selectable_engines`]
//!   and [`a_repeated_failure_re_offers_the_remaining_engines`] —
//!   every failure descends only behind a fresh 3 s offer
//!   naming the next lower tier: the next offer is the
//!   first remaining engine in the declared order, so the
//!   descent never skips a tier that still has a
//!   remaining engine and never offers the failed engine
//!   again; a repeated failure re-offers the same
//!   remaining set, each failed engine recorded (the
//!   remaining set shrank exactly once), never silently
//!   dropped.
//! - F6 [`an_expired_offer_continues_from_the_last_spoken_position_or_pauses_with_reason`] —
//!   the 3 s window elapsing with an alternative remaining
//!   continues from the last spoken position — the expiry
//!   names the offered engine and carries no position
//!   payload at all, so the position stays the caller's,
//!   never a restart; with no alternative it pauses with
//!   a non-empty stated reason.
//! - F7 (asserted inside the F5 descent property) — the
//!   offer sequence names exactly the selectable engines:
//!   every offered engine is in the registry with a
//!   tier-accurate tier, and no selectable engine is
//!   omitted from the offers.
//! - F8 [`the_fixture_pool_covers_every_declared_tier`] and
//!   [`tier_order_index_agrees_with_the_authoritative_order`] —
//!   the properties enumerate all five tiers explicitly;
//!   `tier_order_index` is a five-arm exhaustive match
//!   with no catch-all, so a sixth tier is a compile
//!   error until the properties cover it (Principle VII).

use proptest::prelude::*;

use engine_core::closed::EngineTier;
use engine_core::fallback::{
    Consent, EngineDescriptor, EngineOffer, FallbackExpiry, FallbackSelector,
};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;

/// The fixture engine pool: every tier of the closed
/// five-member set, explicitly (F8), with multiple
/// engines per tier so the same-tier registry order is
/// observable. The ids mirror the constitution's TTS
/// Engine Registry (`.specify/memory/constitution.md`,
/// "TTS Engine Registry") for the four registry tiers;
/// the test_double ids are the only two the 2026-10-05
/// clarification permits at that tier.
const ENGINE_POOL: [(&str, EngineTier); 9] = [
    ("sherpa-onnx-piper-pt-br", EngineTier::OfflineDefault),
    ("google-cloud-tts-wavenet", EngineTier::CloudDefault),
    ("google-cloud-tts-neural2", EngineTier::CloudEscalation),
    ("google-cloud-tts-chirp-3-hd", EngineTier::CloudEscalation),
    ("amazon-polly-neural", EngineTier::CloudEscalation),
    ("azure-speech-neural", EngineTier::CloudEscalation),
    (StandinA::ID, EngineTier::TestDouble),
    (StandinB::ID, EngineTier::TestDouble),
    (
        "microsoft-edge-read-aloud-msedge-ts",
        EngineTier::EvaluatedFallback,
    ),
];

/// An arbitrary registry: a subset of [`ENGINE_POOL`], in
/// pool order (which is registry order). The empty subset
/// is included — a registry may hold no engine — and so
/// is every tier combination, both all-`test_double` and
/// mixed.
fn registry_strategy() -> impl Strategy<Value = Vec<EngineDescriptor>> {
    (0u32..(1u32 << ENGINE_POOL.len())).prop_map(|mask| {
        let mut registry: Vec<EngineDescriptor> = Vec::new();
        for (index, (id, tier)) in ENGINE_POOL.iter().enumerate() {
            if (mask & (1u32 << index)) != 0 {
                registry.push(EngineDescriptor {
                    id: String::from(*id),
                    tier: *tier,
                });
            }
        }
        registry
    })
}

/// An arbitrary registry holding only `test_double`
/// engines — the 2026-10-05 clarification's admissible
/// fixture case (direction (a) of F2).
fn test_double_only_registry_strategy() -> impl Strategy<Value = Vec<EngineDescriptor>> {
    let test_doubles: Vec<(&str, EngineTier)> = ENGINE_POOL
        .iter()
        .copied()
        .filter(|(_, tier)| *tier == EngineTier::TestDouble)
        .collect();
    let subsets = 1u32 << test_doubles.len();
    (0u32..subsets).prop_map(move |mask| {
        let mut registry: Vec<EngineDescriptor> = Vec::new();
        for (index, (id, tier)) in test_doubles.iter().enumerate() {
            if (mask & (1u32 << index)) != 0 {
                registry.push(EngineDescriptor {
                    id: String::from(*id),
                    tier: *tier,
                });
            }
        }
        registry
    })
}

/// The tier's index in the single authoritative
/// `EngineTier::ORDER` (FR-023c) — five explicit arms
/// and no catch-all, so a sixth tier is a compile error
/// until the properties cover it (F8, Principle VII).
fn tier_order_index(tier: EngineTier) -> usize {
    match tier {
        EngineTier::OfflineDefault => 0,
        EngineTier::CloudDefault => 1,
        EngineTier::CloudEscalation => 2,
        EngineTier::TestDouble => 3,
        EngineTier::EvaluatedFallback => 4,
    }
}

/// The engines the declared fallback order makes
/// selectable from `registry` once the engines in `failed`
/// have failed: the admissible and consented entries, in
/// ascending `EngineTier::ORDER` with registry order
/// preserved within a tier. This is the properties'
/// oracle — the declared order itself (FR-023c,
/// contracts/engine-handoff.md §"Fallback order", the
/// 2026-10-05 clarification).
///
/// Admissibility: a `test_double` entry is admissible
/// only while no non-`test_double` entry is registered.
/// Consent: an entry the `consent_of` read denies is
/// never selectable (FR-023c).
fn expected_selectable(
    registry: &[EngineDescriptor],
    failed: &[String],
    consent_of: impl Fn(&str) -> Consent,
) -> Vec<EngineOffer> {
    let has_non_test_double = registry
        .iter()
        .any(|entry| entry.tier != EngineTier::TestDouble);
    let mut expected: Vec<EngineOffer> = Vec::new();
    for entry in registry {
        // The 2026-10-05 clarification: a test_double is
        // admissible only while no non-test_double adapter
        // is registered.
        if entry.tier == EngineTier::TestDouble && has_non_test_double {
            continue;
        }
        // A failed engine is no longer remaining.
        if failed.iter().any(|failed_id| *failed_id == entry.id) {
            continue;
        }
        // FR-023c: an unconsented engine is never selected.
        if consent_of(&entry.id) == Consent::Denied {
            continue;
        }
        expected.push(EngineOffer {
            engine_id: entry.id.clone(),
            tier: entry.tier,
        });
    }
    // Ascending EngineTier::ORDER; the stable sort keeps
    // registry order within a tier (FR-023c).
    expected.sort_by_key(|offer| tier_order_index(offer.tier));
    expected
}

/// The outcome of driving the complete failure descent
/// from the session's default engine (the first
/// selectable engine): every offer the selector produced,
/// paired with the offer the declared order requires at
/// that step. The descent fails the active engine, takes
/// the fresh offer, fails that engine, and so on until
/// nothing remains.
fn drive_full_descent(
    registry: &[EngineDescriptor],
    selector: &mut FallbackSelector,
) -> (Vec<Option<EngineOffer>>, Vec<Option<EngineOffer>>) {
    let mut failed: Vec<String> = Vec::new();
    let initial = selector.selectable();
    let mut current: Option<EngineOffer> = initial.first().cloned();
    let mut actual: Vec<Option<EngineOffer>> = Vec::new();
    let mut expected: Vec<Option<EngineOffer>> = Vec::new();
    while let Some(active) = current {
        // The active engine failed; descent continues only
        // behind a fresh 3 s offer (FR-023c).
        failed.push(active.engine_id.clone());
        let offer = selector.on_failure(&active.engine_id);
        // The oracle: the first remaining engine in the
        // declared order — admissible, consented, and
        // never failed.
        let oracle = expected_selectable(registry, &failed, |id| selector.consent(id))
            .into_iter()
            .next();
        expected.push(oracle);
        actual.push(offer.clone());
        current = offer;
    }
    (actual, expected)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// F1: the selectable engines come back in ascending
    /// `EngineTier::ORDER` — every engine of a lower tier
    /// precedes every engine of a higher tier, and within
    /// a tier the remaining same-tier engines keep registry
    /// order. The oracle below is the declared order
    /// itself: the admissible (test_double only while no
    /// non-test_double is registered), consented,
    /// non-failed engines, stable-sorted by their `ORDER`
    /// index. evaluated_fallback is last when present and
    /// never a default: it is never the first selectable
    /// engine while any other engine is selectable
    /// (FR-023c — the Edge endpoint stays last and is
    /// never a default).
    #[test]
    fn selectable_engines_follow_the_declared_ascending_order(
        registry in registry_strategy(),
    ) {
        let mut selector = FallbackSelector::new(registry.clone());
        let selectable = selector.selectable();
        let expected = expected_selectable(&registry, &[], |id| selector.consent(id));
        prop_assert_eq!(
            selectable.clone(),
            expected,
            "the selectable engines did not follow the declared ascending order"
        );
        // The pairwise form of the same invariant: every
        // engine of a lower tier precedes every engine of
        // a higher tier.
        for (i, earlier) in selectable.iter().enumerate() {
            for later in selectable.iter().skip(i + 1) {
                prop_assert!(
                    tier_order_index(earlier.tier) <= tier_order_index(later.tier),
                    "a {later:?} offer preceded a {earlier:?} offer, inverting the declared order"
                );
            }
        }
        // evaluated_fallback is never a default: while any
        // other engine is selectable, it is not the first
        // selectable engine (FR-023c).
        if selectable.len() > 1 {
            prop_assert_ne!(
                selectable[0].tier,
                EngineTier::EvaluatedFallback,
                "the evaluated_fallback tier was selected as a default"
            );
        }
    }

    /// F2, asserted in both directions for arbitrary
    /// registries: (a) a registry holding ONLY test_double
    /// engines makes the test_doubles selectable — every
    /// entry is admissible (no non-test_double adapter is
    /// registered) and consented by construction, so the
    /// selectable set is exactly the registry, in registry
    /// order; (b) a registry holding ANY non-test_double
    /// engine makes NO test_double selectable (the
    /// 2026-10-05 clarification).
    #[test]
    fn test_double_engines_are_admissible_only_while_no_other_adapter_is_registered(
        registry in registry_strategy(),
        test_double_only in test_double_only_registry_strategy(),
    ) {
        // Direction (a): only test_doubles registered.
        {
            let mut selector = FallbackSelector::new(test_double_only.clone());
            let selectable = selector.selectable();
            let expected: Vec<EngineOffer> = test_double_only
                .iter()
                .map(|entry| EngineOffer {
                    engine_id: entry.id.clone(),
                    tier: entry.tier,
                })
                .collect();
            prop_assert_eq!(
                selectable,
                expected,
                "the test_double engines were not selectable while only test_doubles are registered"
            );
        }
        // Direction (b): any non-test_double registered.
        {
            let mut selector = FallbackSelector::new(registry.clone());
            let selectable = selector.selectable();
            let holds_non_test_double = registry
                .iter()
                .any(|entry| entry.tier != EngineTier::TestDouble);
            if holds_non_test_double {
                for offer in &selectable {
                    prop_assert_ne!(
                        offer.tier,
                        EngineTier::TestDouble,
                        "a test_double engine was selectable while a non-test_double adapter is registered"
                    );
                }
            }
        }
    }

    /// F3: consent for a test_double is granted by
    /// construction — a test_double carries no
    /// provider/egress field at all (`EngineDescriptor`
    /// holds only an id and a tier), so there is no egress
    /// to gate (the 2026-10-05 clarification). The
    /// test_double ids are exactly the two the
    /// clarification permits: `standin_a` and `standin_b`.
    #[test]
    fn test_double_consent_is_granted_by_construction(
        registry in registry_strategy(),
    ) {
        let mut selector = FallbackSelector::new(registry.clone());
        for entry in &registry {
            if entry.tier == EngineTier::TestDouble {
                prop_assert_eq!(
                    selector.consent(&entry.id),
                    Consent::Granted,
                    "consent for the test_double {} was not granted by construction",
                    entry.id
                );
                prop_assert!(
                    entry.id == StandinA::ID || entry.id == StandinB::ID,
                    "the test_double id {} is neither of the two ids the 2026-10-05 clarification permits",
                    entry.id
                );
            }
        }
    }

    /// F4: consent for a cloud-tier engine is read per
    /// attempt from app-private storage, and the selector
    /// exposes the read. Whatever the read yields, a
    /// denied engine is NEVER selected (FR-023c): the
    /// oracle for "denied" is the selector's own consent
    /// read, so the invariant holds regardless of the
    /// storage state the read resolves to.
    #[test]
    fn an_unconsented_engine_is_never_selectable(
        registry in registry_strategy(),
    ) {
        let mut selector = FallbackSelector::new(registry.clone());
        let selectable = selector.selectable();
        for entry in &registry {
            if selector.consent(&entry.id) == Consent::Denied {
                prop_assert!(
                    !selectable.iter().any(|offer| offer.engine_id == entry.id),
                    "the unconsented engine {} was selectable",
                    entry.id
                );
            }
        }
    }

    /// F5 + F7: every failure descends only behind a fresh
    /// 3 s offer naming the next lower tier — the next
    /// offer is the first remaining engine in ascending
    /// `EngineTier::ORDER`, so the descent never skips a
    /// tier that still has a remaining engine, never
    /// offers the failed engine again, and never offers an
    /// engine outside the selectable set. The offer
    /// sequence names exactly the selectable engines (F7):
    /// every offered engine is in the registry with a
    /// tier-accurate tier, and every selectable engine
    /// except the session's default — selected at session
    /// start, never itself an offer — is offered exactly
    /// once. The descent ends because nothing remained.
    #[test]
    fn a_failure_descends_behind_a_fresh_offer_and_names_exactly_the_selectable_engines(
        registry in registry_strategy(),
    ) {
        let mut selector = FallbackSelector::new(registry.clone());
        let initial = selector.selectable();
        let (actual, expected) = drive_full_descent(&registry, &mut selector);
        prop_assert_eq!(
            actual.clone(),
            expected.clone(),
            "the descent offer sequence diverged from the declared fallback order"
        );
        // The descent ended because nothing remained: the
        // last failure exhausted the selectable set.
        match expected.last() {
            Some(last) => {
                prop_assert!(
                    last.is_none(),
                    "the descent ended while an offer was still remaining"
                );
            }
            None => {
                // Nothing was selectable, so the descent
                // never began.
            }
        }
        // F7 soundness: every offered engine is in the
        // registry, with a tier-accurate tier. Both
        // checks assert data-derived booleans: the
        // failing path is a real property outcome, not
        // dead code, so a literal assertion would be a
        // constant assertion rather than a check.
        let offered: Vec<EngineOffer> = actual.into_iter().flatten().collect();
        for offer in &offered {
            let registered = registry
                .iter()
                .any(|entry| entry.id == offer.engine_id);
            prop_assert!(
                registered,
                "the offer named engine {}, which is not in the registry",
                offer.engine_id
            );
            let tier_accurate = registry
                .iter()
                .find(|entry| entry.id == offer.engine_id)
                .is_some_and(|entry| offer.tier == entry.tier);
            prop_assert!(
                tier_accurate,
                "the offer for engine {} named an inaccurate tier",
                offer.engine_id
            );
        }
        // F7 completeness: no selectable engine is omitted
        // from the offers — the offer sequence is exactly
        // the selectable set minus the session's default.
        let expected_offers: Vec<EngineOffer> = initial.into_iter().skip(1).collect();
        prop_assert_eq!(
            offered,
            expected_offers,
            "the offers did not name exactly the selectable engines"
        );
    }

    /// F5: a second failure of the same engine re-offers
    /// the remaining engines — the failing engine is
    /// recorded once (the remaining set shrank exactly
    /// once; each failed engine is recorded, never
    /// silently dropped), and the failed engine itself is
    /// never re-offered. The oracle agrees: after exactly
    /// one recorded failure, the next offer is the first
    /// remaining engine in the declared order.
    #[test]
    fn a_repeated_failure_re_offers_the_remaining_engines(
        registry in registry_strategy(),
    ) {
        let mut selector = FallbackSelector::new(registry.clone());
        let initial = selector.selectable();
        match initial.first().cloned() {
            Some(default_engine) => {
                let first_offer = selector.on_failure(&default_engine.engine_id);
                let repeated_offer = selector.on_failure(&default_engine.engine_id);
                prop_assert_eq!(
                    first_offer,
                    repeated_offer.clone(),
                    "the repeated failure did not re-offer the same remaining engines"
                );
                let oracle = expected_selectable(
                    &registry,
                    &[default_engine.engine_id.clone()],
                    |id| selector.consent(id),
                )
                .into_iter()
                .next();
                prop_assert_eq!(
                    repeated_offer.clone(),
                    oracle,
                    "the re-offer did not name the first remaining engine in the declared order"
                );
            }
            None => {
                // Nothing is selectable: the failure of any
                // registered engine offers nothing, because
                // nothing remains.
                match registry.first() {
                    Some(entry) => {
                        prop_assert_eq!(
                            selector.on_failure(&entry.id),
                            None,
                            "a failure offered an engine while nothing is selectable"
                        );
                    }
                    None => {
                        // An empty registry holds no engine
                        // to fail.
                    }
                }
            }
        }
    }

    /// F6: a registered engine failed, and descent is only
    /// behind a fresh 3 s offer naming the next lower tier
    /// (FR-023c). When the 3 s window elapses with no
    /// choice: an alternative remaining auto-takes-over
    /// from the last spoken position — never a restart.
    /// The expiry names the offered engine and carries no
    /// position payload at all, so the position stays the
    /// caller's last spoken position (FR-023a). With no
    /// alternative, the expiry pauses with a stated
    /// reason — never silence (FR-005).
    #[test]
    fn an_expired_offer_continues_from_the_last_spoken_position_or_pauses_with_reason(
        (registry, failure_index) in registry_strategy().prop_flat_map(|registry| {
            let len = registry.len();
            (Just(registry), 0usize..len.max(1))
        }),
    ) {
        prop_assume!(failure_index < registry.len());
        let failed_entry = registry[failure_index].clone();
        let mut selector = FallbackSelector::new(registry.clone());
        let offer = selector.on_failure(&failed_entry.id);
        match offer {
            Some(offered) => {
                // An alternative remains: the expiry
                // continues from the last spoken position.
                let expiry = selector.expire_offer();
                prop_assert_eq!(
                    expiry,
                    FallbackExpiry::ContinueFromLastPosition {
                        engine_id: offered.engine_id
                    },
                    "the expired offer did not continue from the last spoken position"
                );
            }
            None => {
                // No alternative remains: the expiry pauses
                // with a stated reason — never silence, and
                // never a continuation, because no engine is
                // left to continue on.
                let expiry = selector.expire_offer();
                prop_assert!(
                    matches!(
                        &expiry,
                        FallbackExpiry::PausedWithReason { reason } if !reason.is_empty()
                    ),
                    "the no-alternative expiry was {expiry:?} instead of a pause with a stated reason"
                );
            }
        }
    }
}

#[test]
fn the_fixture_pool_covers_every_declared_tier() {
    // F8: the properties enumerate all five tiers
    // explicitly — the pool carries at least one engine of
    // every tier in the closed set, so every property
    // exercises each tier, and a sixth tier leaves this
    // assertion naming a tier the properties do not cover.
    for tier in EngineTier::ORDER {
        assert!(
            ENGINE_POOL.iter().any(|(_, pool_tier)| *pool_tier == tier),
            "the fixture pool lacks the {tier:?} tier, so the properties cannot enumerate it"
        );
    }
}

#[test]
fn tier_order_index_agrees_with_the_authoritative_order() {
    // F8: `tier_order_index` is the exhaustive five-arm
    // match the properties sort by; it must agree with
    // the single authoritative `EngineTier::ORDER`
    // (FR-044 — a second list would drift).
    for (index, tier) in EngineTier::ORDER.iter().enumerate() {
        assert_eq!(
            tier_order_index(*tier),
            index,
            "tier_order_index disagrees with EngineTier::ORDER at index {index}"
        );
    }
}

#[test]
fn the_default_selector_selects_nothing_and_pauses_with_reason() {
    // The default selector is the empty registry: nothing
    // is selectable, a failure offers nothing, and the
    // expiry pauses with a stated reason — never silence
    // (FR-005).
    let mut selector = FallbackSelector::default();
    assert!(
        selector.selectable().is_empty(),
        "the default selector selected an engine from an empty registry"
    );
    assert_eq!(
        selector.on_failure("any-engine"),
        None,
        "a failure offered an engine from an empty registry"
    );
    // The expiry pauses with a stated reason — never
    // silence, and never a continuation, because the
    // empty registry holds no engine to continue on.
    let expiry = selector.expire_offer();
    assert!(
        matches!(
            &expiry,
            FallbackExpiry::PausedWithReason { reason } if !reason.is_empty()
        ),
        "the empty registry expired to {expiry:?} instead of a pause with a stated reason"
    );
}
