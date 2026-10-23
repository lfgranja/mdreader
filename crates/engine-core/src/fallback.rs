//! The fallback selector (T075, US5).
//!
//! The selector owns the declared fallback order (FR-023c,
//! contracts/engine-handoff.md §"Fallback order"): the
//! remaining engines of the failing tier first, then the
//! higher tiers in registry order, then the Edge
//! evaluated fallback — last, never a default. The order
//! is the closed [`EngineTier::ORDER`] (T014), consumed
//! directly; a second order list would drift (FR-044).
//!
//! Admissibility (the 2026-10-05 clarification): a
//! `test_double` engine — the `standin_a`/`standin_b`
//! stand-ins — is admissible only while no
//! non-`test_double` adapter is registered, an
//! invariant in both directions: the selector never
//! offers a `test_double` while a non-`test_double` is
//! registered, and the `test_double`s are reachable
//! when only `test_double`s are registered (the SC-051
//! failover drills' configuration).
//!
//! Consent (FR-023c): read per attempt from app-private
//! storage — never cached at construction. The read goes
//! through [`consent_from`], which takes the modeled
//! store as its parameter — the injection point the
//! contract names — so a real app-private-storage read
//! slots in without touching the selection logic. An
//! unconsented engine is never selected. A `test_double`
//! is consented by construction: it carries no
//! provider/egress field at all, so there is no egress
//! to gate.
//!
//! Descent (FR-023 / FR-023a): a named engine failure
//! produces a fresh 3-second offer naming the remaining
//! selectable engines — the first remaining engine in
//! the declared order, so the descent never skips a
//! tier that still has a remaining engine and never
//! offers the failed engine again. A repeated failure
//! re-offers the same remaining set (each failed engine
//! is recorded; the exclusion is idempotent). The fresh
//! offer supersedes any open offer (FR-023's supersede
//! rule). When the 3 s window elapses with an
//! alternative remaining, the fallback auto-takes over
//! from the last spoken position — never a restart: the
//! continuation names the offered engine and carries no
//! position payload at all, so the position stays the
//! caller's. With no alternative, the expiry pauses
//! with a stated reason — never silence (FR-005).
//!
//! The 3-second window is modeled structurally: an offer
//! is open from [`FallbackSelector::on_failure`] until
//! [`FallbackSelector::expire_offer`] consumes it (or
//! the next failure supersedes it). No wall-clock time
//! crosses the core; the shell owns the clock (SC-035).

use crate::closed::EngineTier;

/// The stated reason a no-alternative expiry pauses
/// with (FR-005): never silence, never empty.
const NO_ALTERNATIVE_REASON: &str =
    "no alternative engine remains to continue from — the fallback offer expired (FR-005)";

/// One engine of the fallback registry (FR-023c): the
/// engine's identifier and its declared tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineDescriptor {
    /// The engine's identifier: a constitution-registry
    /// name and version for every tier except
    /// [`EngineTier::TestDouble`], which is restricted
    /// to the `standin_a`/`standin_b` stand-ins (the
    /// 2026-10-05 clarification).
    pub id: String,
    /// The engine's declared tier in the closed fallback
    /// order (FR-023c).
    pub tier: EngineTier,
}

/// One engine a fallback offer names (FR-023a): an
/// offer names exactly the engines the fallback may
/// select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOffer {
    /// The offered engine's identifier.
    pub engine_id: String,
    /// The offered engine's declared tier.
    pub tier: EngineTier,
}

/// The per-attempt consent read from app-private
/// storage (FR-023c): an unconsented engine is never
/// selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consent {
    /// Consent is on file in app-private storage: the
    /// engine is selectable, consent-wise.
    Granted,
    /// No consent is on file (or it was revoked): the
    /// engine is never selected (FR-023c).
    Denied,
}

/// The outcome of an expired fallback offer (FR-023a /
/// FR-005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackExpiry {
    /// The offer window elapsed with an alternative
    /// remaining: the fallback auto-takes over from the
    /// last spoken position — never a restart (FR-023a).
    /// The variant carries no position payload at all:
    /// the position stays the caller's last spoken
    /// position.
    ContinueFromLastPosition {
        /// The engine the expired offer named.
        engine_id: String,
    },
    /// No alternative remained when the window elapsed:
    /// the session pauses with a stated reason — never
    /// silence (FR-005).
    PausedWithReason {
        /// The stated pause reason (never empty).
        reason: String,
    },
}

/// The modeled app-private consent store (FR-023c).
///
/// The shell reads per-provider consent from app-private
/// storage; the selector reads it on every attempt,
/// never caching the read at construction. The read goes
/// through [`consent_from`], which takes the store as a
/// parameter — the injection point the contract names —
/// so a real app-private-storage read slots in without
/// touching the selection logic. The model here is the
/// storage state the engine-core fixtures run against:
/// consent is on file for every registered engine (the
/// SC-051 drills run consented).
#[derive(Debug, Clone, Default)]
struct AppPrivateConsent {
    /// The engine identifiers the modeled storage holds
    /// consent on file for.
    consented: Vec<String>,
}

/// The fallback selector (FR-023c): the declared engine
/// registry plus the recorded failures, driving the
/// closed declared fallback order.
///
/// The selector is the engine-core half of the SC-051
/// failover drills: a named engine failure
/// ([`Self::on_failure`]) opens a fresh 3-second offer
/// naming exactly the remaining selectable engines, and
/// the offer's expiry ([`Self::expire_offer`]) resolves
/// it — auto-takeover from the last spoken position, or
/// a paused-with-reason session when nothing remains.
#[derive(Debug, Clone)]
pub struct FallbackSelector {
    /// The declared engine registry, in registry order.
    registry: Vec<EngineDescriptor>,
    /// The engines recorded as failed (FR-023): every
    /// failed engine is recorded, never silently dropped.
    failed: Vec<String>,
    /// The fresh 3-second offer currently open (FR-023a),
    /// if any. A new failure supersedes it (FR-023); the
    /// expiry consumes it.
    pending_offer: Option<EngineOffer>,
    /// The modeled app-private consent store (FR-023c).
    consent_store: AppPrivateConsent,
}

impl Default for FallbackSelector {
    /// The default selector is the empty registry:
    /// nothing is selectable, a failure offers nothing,
    /// and an expired offer pauses with a stated reason —
    /// never silence (FR-005).
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl FallbackSelector {
    /// Create a selector over `registry`, in registry
    /// order.
    ///
    /// The registry is the declared engine set (FR-023c):
    /// the fallback order sorts it by [`EngineTier::ORDER`],
    /// with registry order preserved within a tier. The
    /// modeled app-private consent store seeds consent on
    /// file for every declared engine; the read is
    /// re-taken on every attempt ([`Self::consent`]).
    #[must_use]
    pub fn new(registry: Vec<EngineDescriptor>) -> Self {
        let consent_store = AppPrivateConsent {
            consented: registry.iter().map(|entry| entry.id.clone()).collect(),
        };
        Self {
            registry,
            failed: Vec::new(),
            pending_offer: None,
            consent_store,
        }
    }

    /// The engines the fallback may select right now
    /// (FR-023c): the admissible, consented, non-failed
    /// engines, in ascending [`EngineTier::ORDER`] with
    /// registry order preserved within a tier.
    ///
    /// Admissibility (the 2026-10-05 clarification, both
    /// directions): a `test_double` is admissible only
    /// while no non-`test_double` adapter is registered.
    /// Consent is read per attempt: a denied engine is
    /// never selectable (FR-023c).
    /// [`EngineTier::EvaluatedFallback`] (Edge) sorts
    /// last — never a default.
    #[must_use]
    pub fn selectable(&self) -> Vec<EngineOffer> {
        let holds_non_test_double = self
            .registry
            .iter()
            .any(|entry| entry.tier != EngineTier::TestDouble);
        let mut offers: Vec<EngineOffer> = self
            .registry
            .iter()
            .filter(|entry| !(entry.tier == EngineTier::TestDouble && holds_non_test_double))
            .filter(|entry| !self.failed.contains(&entry.id))
            .filter(|entry| self.consent(&entry.id) == Consent::Granted)
            .map(|entry| EngineOffer {
                engine_id: entry.id.clone(),
                tier: entry.tier,
            })
            .collect();
        // Ascending EngineTier::ORDER — the single
        // authoritative list (FR-023c / FR-044). The
        // stable sort preserves registry order within a
        // tier.
        offers.sort_by_key(|offer| tier_order_index(offer.tier));
        offers
    }

    /// The per-attempt consent read from app-private
    /// storage (FR-023c) for `engine_id`.
    ///
    /// A `test_double` is consented by construction — it
    /// carries no provider/egress field at all, so there
    /// is no egress to gate (the 2026-10-05 clarification).
    /// Every other registered engine reads its
    /// per-provider consent record from the store; an id
    /// the registry does not declare reads as denied —
    /// fail-closed, because no consent record exists to
    /// read.
    #[must_use]
    pub fn consent(&self, engine_id: &str) -> Consent {
        consent_from(&self.consent_store, &self.registry, engine_id)
    }

    /// Record the failure of `engine_id` (FR-023) and
    /// return the fresh 3-second offer (FR-023a /
    /// FR-023c): the first remaining engine in the
    /// declared fallback order.
    ///
    /// The descent happens only behind the fresh offer,
    /// which names exactly the remaining selectable
    /// engines — it never offers the failed engine again
    /// and never skips a tier that still has a remaining
    /// engine. A repeated failure re-offers the same
    /// remaining set (each failed engine is recorded; the
    /// exclusion is idempotent, so the remaining set
    /// shrank exactly once per engine). The fresh offer
    /// supersedes any open offer (FR-023's supersede
    /// rule).
    ///
    /// Returns `None` when no engine remains selectable —
    /// the no-alternative case (FR-005).
    #[must_use]
    pub fn on_failure(&mut self, engine_id: &str) -> Option<EngineOffer> {
        // Record the failure (FR-023c): every failed
        // engine is recorded, never silently dropped.
        // Recording an already-recorded engine is
        // idempotent — the exclusion below matches on the
        // recorded set, so a repeated failure shrinks the
        // remaining set exactly once.
        self.failed.push(String::from(engine_id));
        // The fresh offer: the first remaining engine in
        // the declared ascending order, superseding any
        // open offer (FR-023).
        let offer = self.selectable().into_iter().next();
        self.pending_offer.clone_from(&offer);
        offer
    }

    /// Expire the open 3-second offer (FR-023a / FR-005):
    /// the window elapsed with no reader choice.
    ///
    /// With an open offer, the fallback auto-takes over
    /// from the last spoken position — never a restart
    /// (FR-023a): the continuation names the offered
    /// engine and carries no position payload at all, so
    /// the position stays the caller's. With no open
    /// offer — no alternative remained — the session
    /// pauses with a stated reason, never silence
    /// (FR-005).
    ///
    /// The expiry consumes the offer: the window closes
    /// when it expires, and a descent needs a fresh
    /// offer.
    #[must_use]
    pub fn expire_offer(&mut self) -> FallbackExpiry {
        match self.pending_offer.take() {
            Some(offer) => FallbackExpiry::ContinueFromLastPosition {
                engine_id: offer.engine_id,
            },
            None => FallbackExpiry::PausedWithReason {
                reason: String::from(NO_ALTERNATIVE_REASON),
            },
        }
    }
}

/// Read one engine's consent from the modeled app-private
/// store — the per-attempt read (FR-023c).
///
/// The store is the parameter: the read is pure in the
/// store and the registry, so a real app-private-storage
/// read slots in without touching the selection logic. A
/// `test_double` is consented by construction (no egress
/// to gate); every other registered engine reads its
/// per-provider consent record, which the model holds on
/// file for every declared engine; an unregistered id
/// reads as denied — fail-closed, because no consent
/// record exists to read.
#[must_use]
fn consent_from(
    store: &AppPrivateConsent,
    registry: &[EngineDescriptor],
    engine_id: &str,
) -> Consent {
    let registered = registry.iter().find(|entry| entry.id == engine_id);
    match registered {
        // A test_double is consented by construction: it
        // carries no provider/egress field at all, so
        // there is no egress to gate (the 2026-10-05
        // clarification).
        Some(entry) if entry.tier == EngineTier::TestDouble => Consent::Granted,
        // Every other registered engine reads its
        // per-provider consent record from app-private
        // storage; the model holds consent on file for
        // every declared engine (the storage state the
        // SC-051 fixtures run against).
        Some(entry) => {
            let on_file = store.consented.contains(&entry.id);
            if on_file {
                Consent::Granted
            } else {
                Consent::Denied
            }
        }
        // Fail-closed: an id the registry does not declare
        // has no consent record to read (FR-023c).
        None => Consent::Denied,
    }
}

/// The tier's index in the single authoritative
/// [`EngineTier::ORDER`] (FR-023c).
///
/// The index is derived from `ORDER` itself — a second
/// index list would drift (FR-044). Every closed-set
/// tier is a member of `ORDER`, so the position is
/// always `Some`; the impossible non-member case sorts
/// after every tier rather than panicking, which keeps
/// the sort total without a panic path (the closed set
/// makes it unreachable).
#[must_use]
fn tier_order_index(tier: EngineTier) -> usize {
    EngineTier::ORDER
        .iter()
        .position(|&candidate| candidate == tier)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standin_a::StandinA;
    use crate::standin_b::StandinB;

    /// A registry spanning every declared tier, with both
    /// `test_double` stand-ins — the property suite's
    /// fixture pool, in registry order.
    fn registry() -> Vec<EngineDescriptor> {
        vec![
            EngineDescriptor {
                id: String::from("sherpa-onnx-piper-pt-br"),
                tier: EngineTier::OfflineDefault,
            },
            EngineDescriptor {
                id: String::from("google-cloud-tts-wavenet"),
                tier: EngineTier::CloudDefault,
            },
            EngineDescriptor {
                id: String::from(StandinA::ID),
                tier: EngineTier::TestDouble,
            },
            EngineDescriptor {
                id: String::from(StandinB::ID),
                tier: EngineTier::TestDouble,
            },
            EngineDescriptor {
                id: String::from("microsoft-edge-read-aloud-msedge-ts"),
                tier: EngineTier::EvaluatedFallback,
            },
        ]
    }

    #[test]
    fn the_selectable_order_follows_the_declared_ascending_order() {
        let selector = FallbackSelector::new(registry());
        let selectable = selector.selectable();
        // The stand-ins are inadmissible while
        // non-test_double adapters are registered (the
        // 2026-10-05 clarification), so the selectable set
        // is the offline default, the cloud default, and
        // Edge — last, never a default (FR-023c).
        let ids: Vec<&str> = selectable
            .iter()
            .map(|offer| offer.engine_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "sherpa-onnx-piper-pt-br",
                "google-cloud-tts-wavenet",
                "microsoft-edge-read-aloud-msedge-ts",
            ]
        );
        // evaluated_fallback is never a default: while any
        // other engine is selectable, it is not the first
        // (FR-023c).
        assert_ne!(
            selectable.first().map(|offer| offer.tier),
            Some(EngineTier::EvaluatedFallback)
        );
    }

    #[test]
    fn a_test_double_only_registry_makes_the_stand_ins_selectable() {
        let registry = vec![
            EngineDescriptor {
                id: String::from(StandinA::ID),
                tier: EngineTier::TestDouble,
            },
            EngineDescriptor {
                id: String::from(StandinB::ID),
                tier: EngineTier::TestDouble,
            },
        ];
        let selector = FallbackSelector::new(registry);
        let selectable = selector.selectable();
        let ids: Vec<&str> = selectable
            .iter()
            .map(|offer| offer.engine_id.as_str())
            .collect();
        assert_eq!(ids, vec![StandinA::ID, StandinB::ID]);
        // A test_double is consented by construction (the
        // 2026-10-05 clarification).
        assert_eq!(selector.consent(StandinA::ID), Consent::Granted);
        assert_eq!(selector.consent(StandinB::ID), Consent::Granted);
    }

    #[test]
    fn a_failure_descends_behind_a_fresh_offer_and_the_expiry_continues_in_place() {
        let mut selector = FallbackSelector::new(registry());
        // The offline default fails: the fresh offer names
        // the first remaining engine in the declared order.
        assert_eq!(
            selector.on_failure("sherpa-onnx-piper-pt-br"),
            Some(EngineOffer {
                engine_id: String::from("google-cloud-tts-wavenet"),
                tier: EngineTier::CloudDefault,
            })
        );
        // The window elapses: the fallback auto-takes over
        // from the last spoken position — never a restart,
        // and the continuation carries no position payload.
        assert_eq!(
            selector.expire_offer(),
            FallbackExpiry::ContinueFromLastPosition {
                engine_id: String::from("google-cloud-tts-wavenet"),
            }
        );
        // The failed engine is never re-offered: the
        // descent continues from the remaining engines.
        assert_eq!(
            selector.on_failure("sherpa-onnx-piper-pt-br"),
            Some(EngineOffer {
                engine_id: String::from("google-cloud-tts-wavenet"),
                tier: EngineTier::CloudDefault,
            })
        );
    }

    #[test]
    fn a_repeated_failure_re_offers_the_same_remaining_engines() {
        let mut selector = FallbackSelector::new(registry());
        let first_offer = selector.on_failure("sherpa-onnx-piper-pt-br");
        let repeated_offer = selector.on_failure("sherpa-onnx-piper-pt-br");
        assert_eq!(first_offer, repeated_offer);
    }

    #[test]
    fn a_no_alternative_failure_pauses_with_a_stated_reason() {
        let registry = vec![EngineDescriptor {
            id: String::from(StandinA::ID),
            tier: EngineTier::TestDouble,
        }];
        let mut selector = FallbackSelector::new(registry);
        assert_eq!(selector.on_failure(StandinA::ID), None);
        // Never silence: the expiry states a reason, and
        // never a continuation, because no engine is left
        // to continue on.
        let expiry = selector.expire_offer();
        assert!(
            matches!(&expiry, FallbackExpiry::PausedWithReason { reason } if !reason.is_empty()),
            "a no-alternative expiry was {expiry:?} instead of a pause with a stated reason"
        );
    }

    #[test]
    fn the_default_selector_selects_nothing() {
        let mut selector = FallbackSelector::default();
        assert_eq!(selector.selectable(), []);
        assert_eq!(selector.on_failure("any-engine"), None);
    }
}
