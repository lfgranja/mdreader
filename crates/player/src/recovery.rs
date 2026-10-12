//! The engine-failure recovery machine (T076, US5).
//!
//! The machine is the player-side engine authority
//! (FR-023 / FR-023a / FR-023c / SC-039 / SC-040 /
//! SC-051): the registry of selectable engines and the
//! failover state machine the pipeline consults when a
//! named engine fails. It composes the engine-core
//! fallback selector (`engine_core::fallback::
//! FallbackSelector`, T075) for the declared fallback
//! order, the admissibility rules, and the per-attempt
//! consent read — a second order list or a second
//! consent model would drift (FR-044) — and owns the
//! recovery-specific state the selector does not carry:
//! the session's default engine and the open 3-second
//! offer.
//!
//! The session's default is the first selectable
//! engine at construction — the engine selected at
//! session start. It is never itself an offer: the
//! offer names the remaining selectable engines, so
//! every selectable engine except the default is
//! offered exactly once over a full descent (the
//! prop_fallback F7 oracle). Every failed engine is
//! recorded (FR-023) — never silently dropped, never
//! re-offered — and a repeated failure re-offers the
//! same remaining set (the recording is idempotent, so
//! the set shrank exactly once per engine). A fresh
//! failure supersedes any open offer (FR-023's
//! supersede rule).
//!
//! The machine is clock-free and deterministic, like
//! `session::offer::OfferMachine` (T060):
//! `on_offer_expired` represents the 3-second window
//! elapsing — no wall-clock time crosses the machine,
//! and no caller waits on real time (the shell owns
//! the clock, SC-035). Every decision is a
//! state-machine decision, never a synthesis request:
//! the failure, the offer, and the expiry bill nothing
//! (SC-050 / SC-050b), and the machine holds no
//! transport, so the transport stays Paused through
//! the offer window (FR-005).
//!
//! The takeover is locked (FR-023 / SC-039 / SC-040):
//! a failed engine is locked out permanently, and no
//! unlock exists — the closed `ShellVerb` set has no
//! unlock member, so a locked resolution is
//! mechanically unlock-free.
//!
//! The no-alternative pauses state a reason from the
//! closed vocabulary below — never silence (FR-005) —
//! and the `choose` rejection names its refusal in the
//! `PlayerError::EngineNotAvailable` payload, following
//! the named-refusal convention of the pre-play gates.
//!
//! Positions: the machine stores and returns the `f64`
//! positions exactly as reported — no rounding, no
//! tolerance (the tests generate on the quarter-second
//! grid, where every value is an exact binary
//! fraction).

use engine_core::fallback::{EngineDescriptor, EngineOffer, FallbackSelector};

use crate::cli::PlayerError;

/// The stated reason a failure with no remaining
/// selectable engine pauses with (FR-023 / FR-005):
/// every registered engine has failed, is
/// inadmissible, or is unconsented, so there is
/// nothing to offer and nothing to take over on —
/// never silence.
const NO_REMAINING_ENGINE_REASON: &str = "no remaining selectable engine to continue from — every registered engine has failed, is inadmissible, or is unconsented (FR-023 / FR-005)";

/// The stated reason an expiry with no open offer
/// pauses with (FR-023a / FR-005): no 3-second offer
/// window is open — the window was already resolved by
/// a choice or an earlier expiry, or no failure ever
/// opened one — never silence.
const NO_OPEN_OFFER_REASON: &str = "no 3 s offer window is open to expire — the window was already resolved by a choice or an earlier expiry, or no failure ever opened one (FR-023a / FR-005)";

/// The named refusal a [`RecoveryMachine::choose`]
/// rejection carries in the
/// [`PlayerError::EngineNotAvailable`] payload: the
/// chosen engine is not selectable — failed,
/// unregistered, inadmissible, unconsented, or locked
/// out by the resolution (FR-023a).
const REFUSAL_ENGINE_NOT_SELECTABLE: &str = "engine_not_selectable";

/// The decision the engine authority returns for a
/// named engine failure or an offer resolution
/// (FR-023 / FR-023a): the failover's three terminal
/// forms — an open 3-second offer of alternatives, an
/// auto- or chosen takeover from the last spoken
/// position, or a paused session with a stated reason.
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryDecision {
    /// An engine takes over from the last spoken
    /// position (FR-023a) — the auto-fallback of an
    /// expired offer, or the reader's explicit choice
    /// inside the window. A continuation of the
    /// already-started session, never a restart: the
    /// takeover names the engine and the position, and
    /// the position is exactly the position the
    /// failure reported.
    Takeover {
        /// The engine taking over — the first offered
        /// alternative for an expired offer, the
        /// reader's choice for a chosen one.
        engine_id: String,
        /// The last spoken position the takeover
        /// continues from, in seconds — never a
        /// restart (FR-023a).
        from_position_secs: f64,
    },
    /// The session pauses with a stated reason (FR-005)
    /// — never silence: no remaining selectable engine
    /// at a failure, or no open offer at an expiry.
    /// The reason comes from the machine's closed
    /// vocabulary.
    PausedWithReason {
        /// The stated pause reason (never empty).
        reason: String,
    },
    /// A named engine failure opens the fresh 3-second
    /// offer (FR-023a): the alternatives the fallback
    /// may select, in declared ascending fallback
    /// order — the remaining selectable engines, the
    /// session's default and every failed engine
    /// excluded.
    Offered {
        /// The offered alternatives, in declared
        /// ascending fallback order.
        alternatives: Vec<EngineOffer>,
    },
}

/// The engine-failure recovery machine (FR-023 /
/// FR-023a / FR-023c / SC-039 / SC-040 / SC-051):
/// the registry of selectable engines and the failover
/// state machine.
///
/// The machine composes the engine-core fallback
/// selector (T075) for the declared order, the
/// admissibility rules, and the per-attempt consent
/// read, and owns the recovery-specific state: the
/// session's default engine (the first selectable at
/// construction — the engine selected at session
/// start, never itself an offer) and the open
/// 3-second offer.
///
/// The machine is clock-free:
/// [`RecoveryMachine::on_offer_expired`] represents
/// the 3-second window elapsing, so every transition
/// is deterministic and no caller waits on real time.
#[derive(Debug, Default)]
pub struct RecoveryMachine {
    /// The fallback selector (T075): the declared
    /// registry, the recorded failures, the
    /// admissibility rules, and the per-attempt
    /// consent read.
    selector: FallbackSelector,
    /// The session's default engine: the first
    /// selectable engine at construction — the engine
    /// selected at session start, never itself an
    /// offer (FR-023a / the prop_fallback F7 oracle).
    /// `None` when nothing was selectable at
    /// construction.
    session_default: Option<String>,
    /// The open 3-second offer (FR-023a), if any: the
    /// auto-fallback target — the first offered
    /// alternative, the next engine in the declared
    /// fallback order. A fresh failure supersedes it
    /// (FR-023); a choice or an expiry consumes it.
    pending_offer: Option<EngineOffer>,
    /// The last spoken position a failure reported, in
    /// seconds — the position every takeover continues
    /// from (FR-023a). `0.0` until the first failure.
    last_spoken_position_secs: f64,
}

impl RecoveryMachine {
    /// Create a machine over `registry`, in registry
    /// order.
    ///
    /// The session's default engine is the first
    /// selectable engine at construction — the engine
    /// selected at session start, never itself an
    /// offer (FR-023a / the prop_fallback F7 oracle):
    /// the offer names the remaining selectable
    /// engines, so every selectable engine except the
    /// default is offered exactly once over a full
    /// descent. `None` when nothing is selectable at
    /// construction (the machine over the empty
    /// registry selects nothing, and every failure
    /// pauses with a stated reason).
    #[must_use]
    pub fn new(registry: Vec<EngineDescriptor>) -> Self {
        let selector = FallbackSelector::new(registry);
        let session_default = selector
            .selectable()
            .first()
            .map(|offer| offer.engine_id.clone());
        Self {
            selector,
            session_default,
            pending_offer: None,
            last_spoken_position_secs: 0.0,
        }
    }

    /// The named failure of `engine_id` at
    /// `position_secs` (FR-023 / FR-023a): with
    /// remaining selectable engines, the fresh
    /// 3-second offer naming them; with none, a
    /// paused-with-reason decision (FR-005).
    ///
    /// The failure is recorded (FR-023): every failed
    /// engine is recorded, never silently dropped —
    /// recording an already-recorded engine is
    /// idempotent, so a repeated failure re-offers the
    /// same remaining set (the set shrank exactly once
    /// per engine). The fresh offer supersedes any
    /// open offer (FR-023's supersede rule) and
    /// latches `position_secs` as the last spoken
    /// position every takeover from the offer
    /// continues from (FR-023a).
    ///
    /// The offer names exactly the remaining selectable
    /// engines in declared ascending fallback order —
    /// the session's default (selected at session
    /// start, never itself an offer) and every failed
    /// engine are excluded.
    #[must_use]
    pub fn on_engine_failure(&mut self, engine_id: &str, position_secs: f64) -> RecoveryDecision {
        // Record the failure through the selector
        // (FR-023): every failed engine is recorded,
        // never silently dropped. The selector's own
        // return value — its first remaining engine,
        // the session's default included — is not the
        // machine's offer: the offer excludes the
        // session's default (FR-023a), so the
        // offerable set is recomputed below.
        let _ = self.selector.on_failure(engine_id);
        // Latch the last spoken position (FR-023a).
        self.last_spoken_position_secs = position_secs;
        let alternatives = self.offerable();
        let takeover = alternatives.first().cloned();
        match takeover {
            Some(takeover) => {
                // The fresh offer, superseding any open
                // offer (FR-023): the auto-fallback
                // target is the first offered
                // alternative.
                self.pending_offer = Some(takeover);
                RecoveryDecision::Offered { alternatives }
            }
            // No engine remains selectable: paused with
            // a stated reason, never silence (FR-005).
            None => RecoveryDecision::PausedWithReason {
                reason: String::from(NO_REMAINING_ENGINE_REASON),
            },
        }
    }

    /// The 3-second offer window elapses with no choice
    /// (FR-023a): the auto-fallback takes over from
    /// the last spoken position — the first offered
    /// alternative, the next engine in the declared
    /// fallback order — continuing from the position,
    /// never a restart. With no open offer — no
    /// alternative remained, the window was already
    /// resolved by a choice or an earlier expiry, or
    /// no failure ever opened one — a paused-with-
    /// reason decision (FR-005).
    ///
    /// The expiry consumes the offer: the window closes
    /// when it expires, and a descent needs a fresh
    /// offer.
    #[must_use]
    pub fn on_offer_expired(&mut self) -> RecoveryDecision {
        match self.pending_offer.take() {
            // The window elapsed with an alternative
            // open: the auto-fallback takes over from
            // the last spoken position (FR-023a).
            // Locked — no unlock exists (SC-039 /
            // SC-040).
            Some(takeover) => RecoveryDecision::Takeover {
                engine_id: takeover.engine_id,
                from_position_secs: self.last_spoken_position_secs,
            },
            // No open offer: nothing to take over on,
            // so paused with a stated reason, never
            // silence (FR-005).
            None => RecoveryDecision::PausedWithReason {
                reason: String::from(NO_OPEN_OFFER_REASON),
            },
        }
    }

    /// The reader's explicit choice inside the window
    /// (FR-023a): the chosen engine takes over from
    /// the last spoken position — the position the
    /// failure reported — and the choice consumes the
    /// offer window: nothing remains open for a later
    /// expiry.
    ///
    /// The choice is honoured for any engine in the
    /// machine's offerable set — the remaining
    /// selectable engines in declared ascending
    /// fallback order, the session's default (selected
    /// at session start, never itself an offer) and
    /// every failed engine excluded. The choice does
    /// not need an open window: after a locked
    /// takeover the takeover engine remains selectable,
    /// and the failed engine remains locked out
    /// (FR-023 / SC-039 / SC-040) — no unlock
    /// exists, because the closed `ShellVerb` set has
    /// no unlock member.
    ///
    /// # Errors
    ///
    /// [`PlayerError::EngineNotAvailable`] when
    /// `engine_id` is not selectable — failed,
    /// unregistered, inadmissible, unconsented, or
    /// locked out by the resolution (FR-023a).
    #[must_use]
    pub fn choose(&mut self, engine_id: &str) -> Result<RecoveryDecision, PlayerError> {
        let chosen = self
            .offerable()
            .into_iter()
            .find(|offer| offer.engine_id == engine_id);
        match chosen {
            Some(offer) => {
                // The choice consumes the offer window
                // (FR-023a): the window does not
                // outlive its resolution.
                self.pending_offer = None;
                Ok(RecoveryDecision::Takeover {
                    engine_id: offer.engine_id,
                    from_position_secs: self.last_spoken_position_secs,
                })
            }
            // The engine is not selectable (FR-023a):
            // a named refusal, never a silent
            // acceptance.
            None => Err(PlayerError::EngineNotAvailable(
                REFUSAL_ENGINE_NOT_SELECTABLE,
            )),
        }
    }

    /// The engines the recovery may resolve to right
    /// now: the remaining selectable engines in
    /// declared ascending fallback order (FR-023c),
    /// the session's default excluded — the engine
    /// selected at session start is never itself an
    /// offer (FR-023a / the prop_fallback F7 oracle).
    ///
    /// A failed engine is excluded by the selector's
    /// recorded failures (FR-023); the session's
    /// default is excluded here. When the default has
    /// failed, both exclusions coincide and the set
    /// shrank exactly once.
    fn offerable(&self) -> Vec<EngineOffer> {
        self.selector
            .selectable()
            .into_iter()
            .filter(|offer| self.session_default.as_deref() != Some(offer.engine_id.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::closed::EngineTier;
    use engine_core::standin_a::StandinA;
    use engine_core::standin_b::StandinB;

    /// The two-stand-in registry (the SC-051 failover
    /// configuration): `standin_a` primary,
    /// `standin_b` fallback — both `test_double`,
    /// admissible only while no non-`test_double`
    /// adapter is registered (FR-023c / the 2026-10-05
    /// clarification).
    fn two_standin_registry() -> Vec<EngineDescriptor> {
        vec![
            EngineDescriptor {
                id: String::from(StandinA::ID),
                tier: EngineTier::TestDouble,
            },
            EngineDescriptor {
                id: String::from(StandinB::ID),
                tier: EngineTier::TestDouble,
            },
        ]
    }

    /// The single-stand-in registry (the
    /// single-engine-equivalent configuration of F3):
    /// the one registered engine whose failure leaves
    /// nothing to offer.
    fn single_standin_registry() -> Vec<EngineDescriptor> {
        vec![EngineDescriptor {
            id: String::from(StandinA::ID),
            tier: EngineTier::TestDouble,
        }]
    }

    /// A registry spanning three distinct declared
    /// tiers, in registry order: the offline default
    /// (the session's default at construction), the
    /// cloud default, and the evaluated fallback. The
    /// configuration the session-default exclusion is
    /// observable under — a non-default engine's
    /// failure must not offer the healthy default.
    fn mixed_registry() -> Vec<EngineDescriptor> {
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
                id: String::from("microsoft-edge-read-aloud-msedge-ts"),
                tier: EngineTier::EvaluatedFallback,
            },
        ]
    }

    /// The offered alternatives of `decision`, or the
    /// empty offer for the non-offer decisions.
    fn alternatives_of(decision: &RecoveryDecision) -> Vec<EngineOffer> {
        match decision {
            RecoveryDecision::Offered { alternatives } => alternatives.clone(),
            RecoveryDecision::Takeover { .. } | RecoveryDecision::PausedWithReason { .. } => {
                Vec::new()
            }
        }
    }

    #[test]
    fn the_session_default_is_never_an_offer() {
        let mut machine = RecoveryMachine::new(mixed_registry());
        // The session's default is the first selectable
        // engine at construction: the offline default.
        // A failure of a non-default engine offers the
        // remaining selectable engines minus the
        // default — the healthy default is never itself
        // an offer (FR-023a / the F7 oracle).
        let offered = machine.on_engine_failure("google-cloud-tts-wavenet", 10.0);
        let alternatives = alternatives_of(&offered);
        let ids: Vec<&str> = alternatives
            .iter()
            .map(|offer| offer.engine_id.as_str())
            .collect();
        // The offline default (the session's default)
        // is excluded even though it is healthy; the
        // failed cloud default is excluded; the
        // evaluated fallback is named, last, never a
        // default (FR-023c).
        assert_eq!(
            ids,
            vec!["microsoft-edge-read-aloud-msedge-ts"],
            "the offer must name exactly the remaining engines after excluding the session's default and the failed engine (FR-023a)"
        );
    }

    #[test]
    fn a_failure_of_the_default_offers_the_rest_in_declared_order() {
        let mut machine = RecoveryMachine::new(mixed_registry());
        // The default fails: the offer names every
        // remaining selectable engine in declared
        // ascending order.
        let offered = machine.on_engine_failure("sherpa-onnx-piper-pt-br", 10.0);
        let alternatives = alternatives_of(&offered);
        let ids: Vec<&str> = alternatives
            .iter()
            .map(|offer| offer.engine_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "google-cloud-tts-wavenet",
                "microsoft-edge-read-aloud-msedge-ts",
            ],
            "the offer must name the remaining selectable engines in declared ascending order (FR-023c)"
        );
    }

    #[test]
    fn a_repeated_failure_re_offers_the_same_remaining_engines() {
        let mut machine = RecoveryMachine::new(mixed_registry());
        let first = machine.on_engine_failure("google-cloud-tts-wavenet", 10.0);
        // A repeated failure re-offers the same
        // remaining set: the recording is idempotent,
        // so the set shrank exactly once (FR-023).
        let repeated = machine.on_engine_failure("google-cloud-tts-wavenet", 20.0);
        assert_eq!(
            first, repeated,
            "a repeated failure must re-offer the same remaining engines (FR-023)"
        );
    }

    #[test]
    fn a_no_alternative_failure_pauses_with_a_stated_reason() {
        let mut machine = RecoveryMachine::new(single_standin_registry());
        let failed = machine.on_engine_failure(StandinA::ID, 120.0);
        assert!(
            matches!(
                &failed,
                RecoveryDecision::PausedWithReason { reason } if !reason.is_empty()
            ),
            "a single-engine registry failure must pause with a stated reason (FR-023), got {failed:?}"
        );
    }

    #[test]
    fn an_expiry_with_no_open_offer_pauses_with_a_stated_reason() {
        let mut machine = RecoveryMachine::new(mixed_registry());
        // No failure ever opened a window: the expiry
        // has nothing to take over on, so it pauses
        // with a stated reason, never silence (FR-005).
        let expired = machine.on_offer_expired();
        assert!(
            matches!(
                &expired,
                RecoveryDecision::PausedWithReason { reason } if !reason.is_empty()
            ),
            "an expiry with no open offer must pause with a stated reason (FR-005), got {expired:?}"
        );
    }

    #[test]
    fn an_expired_offer_takes_over_from_the_last_spoken_position() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        let _offered = machine.on_engine_failure(StandinA::ID, 120.0);
        let expired = machine.on_offer_expired();
        assert_eq!(
            expired,
            RecoveryDecision::Takeover {
                engine_id: String::from(StandinB::ID),
                from_position_secs: 120.0,
            },
            "the expired offer must take over on stand-in B from the last spoken position (FR-023a)"
        );
    }

    #[test]
    fn the_takeover_position_is_exactly_the_reported_position() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        // A quarter-second-grid position: an exact
        // binary fraction, stored and returned exactly
        // — no rounding, no tolerance.
        let _offered = machine.on_engine_failure(StandinA::ID, 3.25);
        let expired = machine.on_offer_expired();
        assert_eq!(
            expired,
            RecoveryDecision::Takeover {
                engine_id: String::from(StandinB::ID),
                from_position_secs: 3.25,
            },
            "the takeover must continue from exactly the reported position (FR-023a)"
        );
    }

    #[expect(
        clippy::float_cmp,
        reason = "the compared position 120.5 is an exact binary fraction (a multiple of 1/2); == is exact and a tolerance would wrongly admit off-grid positions"
    )]
    #[test]
    fn a_choice_inside_the_window_takes_over_from_the_last_spoken_position() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        let _offered = machine.on_engine_failure(StandinA::ID, 120.5);
        let chosen = machine.choose(StandinB::ID);
        assert!(
            matches!(
                &chosen,
                Ok(RecoveryDecision::Takeover {
                    engine_id,
                    from_position_secs,
                }) if *engine_id == StandinB::ID && *from_position_secs == 120.5
            ),
            "the choice must take over on stand-in B from the last spoken position (FR-023a), got {chosen:?}"
        );
        // The choice consumed the window: a later
        // expiry has nothing to expire (FR-023a).
        let expired = machine.on_offer_expired();
        assert!(
            matches!(&expired, RecoveryDecision::PausedWithReason { .. }),
            "an expiry after the choice consumed the window must pause with a stated reason (FR-005), got {expired:?}"
        );
    }

    #[test]
    fn a_choice_rejects_a_non_selectable_engine() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        let _offered = machine.on_engine_failure(StandinA::ID, 120.0);
        // The failed engine is not selectable (FR-023a)
        // — a named refusal, never a silent acceptance.
        let rejected = machine.choose(StandinA::ID);
        assert!(
            matches!(
                &rejected,
                Err(PlayerError::EngineNotAvailable(reason))
                    if *reason == REFUSAL_ENGINE_NOT_SELECTABLE
            ),
            "the failed engine must be rejected as not available (FR-023a), got {rejected:?}"
        );
        // An unregistered engine is not selectable
        // either.
        let unregistered = machine.choose("no-such-engine");
        assert!(
            unregistered.is_err(),
            "an unregistered engine must be rejected as not available (FR-023a)"
        );
    }

    #[test]
    fn a_takeover_engine_remains_selectable_and_the_failed_engine_locked_out() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        let _offered = machine.on_engine_failure(StandinA::ID, 120.0);
        let _takeover = machine.on_offer_expired();
        // Locked resolution without unlock (FR-023 /
        // SC-039 / SC-040): the failed engine is no
        // longer selectable, the takeover engine is —
        // and no unlock exists (the closed `ShellVerb`
        // set has no unlock member).
        assert!(
            machine.choose(StandinA::ID).is_err(),
            "the failed engine must not be selectable after the locked takeover (FR-023a)"
        );
        assert!(
            machine.choose(StandinB::ID).is_ok(),
            "the takeover engine must remain the locked selection (FR-023a)"
        );
    }

    #[test]
    fn a_second_failure_after_the_takeover_pauses_with_a_stated_reason() {
        let mut machine = RecoveryMachine::new(two_standin_registry());
        let _offered = machine.on_engine_failure(StandinA::ID, 120.0);
        let _takeover = machine.on_offer_expired();
        // The takeover engine is forced down: both
        // stand-ins are down, so the machine pauses
        // with a named reason, never silence (FR-023 /
        // FR-037).
        let forced_down = machine.on_engine_failure(StandinB::ID, 240.0);
        assert!(
            matches!(
                &forced_down,
                RecoveryDecision::PausedWithReason { reason } if !reason.is_empty()
            ),
            "with both stand-ins down there is nothing to offer or take over on (FR-023), got {forced_down:?}"
        );
    }

    #[test]
    fn the_default_selector_machine_selects_nothing() {
        // The derived `Default` machine is the machine
        // over the empty registry: nothing is
        // selectable, a failure offers nothing, and an
        // expiry pauses with a stated reason.
        let mut machine = RecoveryMachine::default();
        let failed = machine.on_engine_failure("any-engine", 0.0);
        assert!(
            matches!(
                &failed,
                RecoveryDecision::PausedWithReason { reason } if !reason.is_empty()
            ),
            "a failure over the empty registry must pause with a stated reason (FR-023), got {failed:?}"
        );
        let expired = machine.on_offer_expired();
        assert!(
            matches!(
                &expired,
                RecoveryDecision::PausedWithReason { reason } if !reason.is_empty()
            ),
            "an expiry over the empty registry must pause with a stated reason (FR-005), got {expired:?}"
        );
        assert!(
            machine.choose("any-engine").is_err(),
            "no engine is selectable over the empty registry (FR-023a)"
        );
    }
}
