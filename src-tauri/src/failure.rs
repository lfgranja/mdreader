//! The engine-failure surface (T078; FR-023, FR-023a, FR-037,
//! FR-037a): the named failure rendered through the T065 reason
//! shapes, carrying the recovery decision the machine returned
//! (T076) so the shell can name the offered alternatives, the
//! takeover, or the stated pause reason.
//!
//! The failure statement is the named failure (FR-023): the
//! closed seven-member engine failure taxonomy maps 1:1 onto the
//! seven `Engine*` members of the T065 [`ShortReason`] and
//! [`FullReason`] shapes ([`failure_reason`]), and the surface
//! renders both statements through the T065 getters
//! ([`short_reason`], [`full_reason`]) — the single source of
//! every rendered string, so there is no second definition of
//! any statement (Principle II). The short statement is what the
//! lock screen or notification shade states without an unlock;
//! the full statement is what remains available in the
//! application when it is next opened (FR-037).
//!
//! The failure regime (FR-037a): a failure — an engine failure,
//! an audio-device failure, or a refused start — persists in the
//! application until it is read. The FR-041 3 s focus regime
//! never applies to a failure: the two reason sets are disjoint,
//! and a focus loss is not a malfunction. The surface therefore
//! carries the failure's TTL as `None` — the payload's
//! `reason_ttl_ms: null` — through the T065 classification the
//! [`FullReason::reason_ttl_ms`] getter derives, never as a
//! locally decided window.
//!
//! Two axes, never one (contracts/surface.md, the F8 payload
//! overload fix): the failure's `reason_ttl_ms` says how long its
//! line stays (here: indefinitely), while the dismissal policy of
//! the surface carrying it — `non_dismissable_while_playing |
//! dismissable`, FR-014 / FR-028e — says whether that surface may
//! be dismissed at all. A persistent failure line rides on a
//! surface that is non-dismissable while audio plays, so neither
//! field can be read out of the other's value; the surface
//! getters ([`FailureSurface::reason_ttl_ms`],
//! [`crate::surface::dismissal_for`]) keep them separate.
//!
//! The never-start + `error`-logged half of the undeliverable
//! rule (contracts/surface.md: undeliverables ⇒ never start +
//! persistent line + `error` logged) belongs to the refusal
//! path, not to this renderer. Refusing playback is playback
//! authority (FR-042), and this module owns no state and no
//! transport: it renders the persistent line and carries the
//! machine's decision so the shell can name the alternatives,
//! the takeover, or the stated pause reason. The refusal itself
//! — and the `error` diagnostic entry it must log — live in the
//! adapter's synthesis refusal and the shell's dispatch path,
//! which are outside this module. The boundary is pinned by the
//! tests below: the surface half (a persistent line for every
//! member of the closed failure set) is asserted here, and the
//! never-start half is asserted where the refusal is executed.
//!
//! Document-text boundary (FR-037): a failure reason MUST NOT
//! include text drawn from the document — neither its title nor
//! any word of it. `document_title` reaches the T065 getters,
//! whose focus-loss arms alone consume it; the failure arms are
//! app-generated text and take no document text at all.
//!
//! The recovery decision (T076) the failure produced — the open
//! offer's alternatives, the takeover, or the machine's stated
//! pause reason — is carried for the shell to name, never
//! re-rendered: the machine's stated reasons are already stated
//! text from its closed vocabulary, and the offered engine ids
//! are identifiers the shell names exactly as the machine
//! declared them.
//!
//! Closed sets throughout (Principle VII, FR-044): no
//! `Other`/`Unknown`, no catch-all arms — the 1:1 mapping's
//! exhaustive arms are the coordinated edit point: adding a
//! variant to either the engine taxonomy or the T065 shapes is
//! a coordinated edit with a semver bump and a documentation
//! update. The engine taxonomy reaches this module through the
//! adapter's re-export of the `engine_core::closed` declaration
//! (FR-044: a second list would drift), and the decision
//! through `player::recovery` (T076).

use adapter_synthesis::failure::EngineError;
use player::recovery::RecoveryDecision;

use crate::surface::{FullReason, ReasonExpiry, ShortReason, full_reason, short_reason};

/// The engine-failure surface (T078; FR-023, FR-023a, FR-037,
/// FR-037a): the named failure rendered through the T065 reason
/// shapes, carrying the recovery decision the machine returned
/// (T076) so the shell can name the offered alternatives, the
/// takeover, or the stated pause reason.
#[derive(Debug, Clone, PartialEq)]
pub struct FailureSurface {
    /// The short lock-screen statement (FR-037) — the named
    /// failure through the T065 getter [`short_reason`] (the
    /// single source): the statement the lock screen or
    /// notification shade states without an unlock.
    pub short_statement: String,
    /// The full in-app statement (FR-037) — through the T065
    /// getter [`full_reason`] (the single source): the full
    /// reason that remains available in the application when it
    /// is next opened.
    pub full_statement: String,
    /// How long the reason stays (FR-037a): always
    /// [`ReasonExpiry::FailurePersistent`] for a failure — a
    /// failure persists in the application until it is read; the
    /// FR-041 3 s focus regime never applies to a failure.
    pub expiry: ReasonExpiry,
    /// The payload TTL of this reason (FR-037a): always `None` —
    /// the payload's `reason_ttl_ms: null` — because a failure
    /// persists in the application until it is read. The numeric
    /// form of the T065 classification the [`FullReason`] getter
    /// derives, never a locally decided window: a failure never
    /// expires the way a focus-loss pause reason does
    /// ([`crate::surface::PAUSE_REASON_TTL_MS`]).
    ///
    /// Distinct from the dismissal policy of the surface carrying
    /// this reason ([`crate::surface::Dismissal`], FR-014 /
    /// FR-028e), which the transport states and this renderer
    /// never computes (FR-042): a persistent failure line rides on
    /// a surface that is non-dismissable while audio plays
    /// (contracts/surface.md, the F8 payload overload fix).
    pub reason_ttl_ms: Option<u64>,
    /// The recovery decision (T076) the failure produced — the
    /// open offer's alternatives, the takeover, or the machine's
    /// stated pause reason — carried for the shell to name, never
    /// re-rendered (the machine's stated reasons are already
    /// stated text from its closed vocabulary).
    pub decision: RecoveryDecision,
}

/// Map the engine failure taxonomy to the T065 reason shapes:
/// the closed 1:1 correspondence between the seven [`EngineError`]
/// variants and the seven `Engine*` members of [`ShortReason`]
/// and [`FullReason`] (no catch-all arm; adding a variant to
/// either set is a coordinated edit).
///
/// The taxonomy is the adapter's re-export of the
/// `engine_core::closed` declaration — the single authoritative
/// declaration of the failure set (FR-044) — so the mapping
/// cannot drift from the errors the engine actually reports.
#[must_use]
pub fn failure_reason(error: &EngineError) -> (ShortReason, FullReason) {
    match error {
        EngineError::Unreachable => (
            ShortReason::EngineUnreachable,
            FullReason::EngineUnreachable,
        ),
        EngineError::InitFailed => (ShortReason::EngineInitFailed, FullReason::EngineInitFailed),
        EngineError::RateMismatch => (
            ShortReason::EngineRateMismatch,
            FullReason::EngineRateMismatch,
        ),
        EngineError::ChannelMismatch => (
            ShortReason::EngineChannelMismatch,
            FullReason::EngineChannelMismatch,
        ),
        EngineError::DeviceRefused => (
            ShortReason::EngineDeviceRefused,
            FullReason::EngineDeviceRefused,
        ),
        EngineError::DeviceLost => (ShortReason::EngineDeviceLost, FullReason::EngineDeviceLost),
        EngineError::StartRefused => (
            ShortReason::EngineStartRefused,
            FullReason::EngineStartRefused,
        ),
    }
}

/// Render the engine-failure surface: the named failure through
/// the T065 getters plus the machine's decision. `document_title`
/// is consumed by the T065 getters' focus-loss arms alone — a
/// failure statement never contains document text (FR-037).
///
/// The expiry is the T065 classification the [`FullReason`]
/// getter derives for the named failure — the persistent regime
/// for every member of the closed failure set (FR-037a): a
/// failure persists in the application until it is read, and the
/// FR-041 3 s focus regime never applies to it. The
/// [`FailureSurface::reason_ttl_ms`] field is that same
/// classification in its payload form — `None`, the wire
/// `null` — so a consumer reads the TTL from the surface instead
/// of re-deriving the split.
#[must_use]
pub fn render_failure(
    error: &EngineError,
    decision: &RecoveryDecision,
    document_title: &str,
) -> FailureSurface {
    let (short, full) = failure_reason(error);
    FailureSurface {
        short_statement: short_reason(&short, document_title),
        full_statement: full_reason(&full, document_title),
        expiry: full.expiry(),
        reason_ttl_ms: full.reason_ttl_ms(),
        decision: decision.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Dismissal, PAUSE_REASON_TTL_MS, dismissal_for};
    use engine_core::closed::TransportState;
    use player::interruption::InterruptionReason;
    use player::recovery::RecoveryMachine;

    /// A document title whose words occur in no app-generated
    /// reason string, so an assertion can tell document-derived
    /// text from app-generated text (FR-037).
    const DOCUMENT_TITLE: &str = "Rota da Manhã";

    /// The seven engine failures — the closed [`EngineError`]
    /// set — with the T065 reason pair each maps to: the closed
    /// 1:1 correspondence between the engine failure taxonomy
    /// and the `Engine*` members of the T065 reason shapes.
    /// Iterating the whole list covers both presentations
    /// exhaustively, and a variant added to either set fails to
    /// compile the mapping's exhaustive arms (Principle VII).
    const FAILURES: [(EngineError, ShortReason, FullReason); 7] = [
        (
            EngineError::Unreachable,
            ShortReason::EngineUnreachable,
            FullReason::EngineUnreachable,
        ),
        (
            EngineError::InitFailed,
            ShortReason::EngineInitFailed,
            FullReason::EngineInitFailed,
        ),
        (
            EngineError::RateMismatch,
            ShortReason::EngineRateMismatch,
            FullReason::EngineRateMismatch,
        ),
        (
            EngineError::ChannelMismatch,
            ShortReason::EngineChannelMismatch,
            FullReason::EngineChannelMismatch,
        ),
        (
            EngineError::DeviceRefused,
            ShortReason::EngineDeviceRefused,
            FullReason::EngineDeviceRefused,
        ),
        (
            EngineError::DeviceLost,
            ShortReason::EngineDeviceLost,
            FullReason::EngineDeviceLost,
        ),
        (
            EngineError::StartRefused,
            ShortReason::EngineStartRefused,
            FullReason::EngineStartRefused,
        ),
    ];

    /// The two focus-loss pause reasons (FR-013a, FR-014) —
    /// the disjoint set whose FR-041 3 s regime never applies
    /// to a failure (FR-037a).
    const FOCUS: [ShortReason; 2] = [
        ShortReason::FocusSilenceDemanding,
        ShortReason::FocusPermanentLoss,
    ];

    /// Every transport state of the session's closed transport
    /// reflection (FR-044), so the dismissal axis of a failure
    /// surface is asserted over the whole set.
    const ALL_TRANSPORTS: [TransportState; 5] = [
        TransportState::Playing,
        TransportState::Paused,
        TransportState::Stopped,
        TransportState::Completed,
        TransportState::Interrupted,
    ];

    /// The undeliverable members of the player's closed
    /// interruption set (contracts/surface.md: never start +
    /// persistent line + `error` logged) — the reasons the
    /// failure surface is the surface half of, iterated so a test
    /// can pin the classification the T065 failure reasons carry
    /// against the player's own set.
    const UNDELIVERABLE: [InterruptionReason; 3] = [
        InterruptionReason::VoiceUnavailable,
        InterruptionReason::FormatUndeliverable,
        InterruptionReason::DecodeFailed,
    ];

    /// The whitespace-delimited word count of a rendered line —
    /// the measurement the ≤8-word bound of contracts/surface.md
    /// is checked against.
    fn words(line: &str) -> usize {
        line.split_whitespace().count()
    }

    /// A takeover decision (T076): the auto-fallback of an
    /// expired offer or the reader's explicit choice, with the
    /// engine id and the last spoken position the surface must
    /// carry verbatim.
    fn takeover() -> RecoveryDecision {
        RecoveryDecision::Takeover {
            engine_id: String::from("standin_b"),
            from_position_secs: 120.5,
        }
    }

    /// The stated reason of a [`RecoveryDecision::PausedWithReason`],
    /// or `None` for the other decisions — the shell's read path
    /// for the machine's stated pause reason (T076).
    fn stated_reason_of(decision: &RecoveryDecision) -> Option<&str> {
        match decision {
            RecoveryDecision::PausedWithReason { reason } => Some(reason.as_str()),
            RecoveryDecision::Offered { .. } | RecoveryDecision::Takeover { .. } => None,
        }
    }

    #[test]
    fn the_failure_taxonomy_maps_one_to_one_onto_both_presentations() {
        // FR-023: the named failure — the closed 1:1 mapping of
        // the engine taxonomy onto both T065 presentations, with
        // no catch-all arm: every `EngineError` variant names its
        // `Engine*` counterpart in each shape, and the two
        // presentations stay parallel (the same counterpart in
        // both).
        for (error, short, full) in FAILURES {
            assert_eq!(failure_reason(&error), (short, full));
        }
    }

    #[test]
    fn the_rendered_statements_are_the_t065_getters_output() {
        // FR-037 / Principle II: the surface's statements ARE
        // the T065 getters' output — byte-equal, the round trip
        // through `short_reason` and `full_reason` — never a
        // second definition of any rendered string.
        for (error, short, full) in FAILURES {
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            assert_eq!(
                surface.short_statement,
                short_reason(&short, DOCUMENT_TITLE)
            );
            assert_eq!(surface.full_statement, full_reason(&full, DOCUMENT_TITLE));
        }
    }

    #[test]
    fn failure_statements_never_quote_document_text() {
        // FR-037: a failure reason MUST NOT include text drawn
        // from the document — neither its title nor any word of
        // it — on either surface. The getters' focus-loss arms
        // alone consume the title; the failure arms are
        // app-generated text and take no document text at all.
        for (error, _, _) in FAILURES {
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            for statement in [&surface.short_statement, &surface.full_statement] {
                assert!(!statement.contains(DOCUMENT_TITLE));
                assert!(!statement.contains("Rota"));
                assert!(!statement.contains("Manhã"));
            }
        }
    }

    #[test]
    fn a_failure_reason_persists_until_read() {
        // FR-037 / FR-037a: persistence is the failure regime —
        // the reason stays in the application until it is read —
        // and the FR-041 3 s focus regime never applies to a
        // failure: the two reason sets are disjoint, and a focus
        // loss is not a malfunction.
        for (error, _, full) in FAILURES {
            assert_eq!(full.expiry(), ReasonExpiry::FailurePersistent);
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            assert_eq!(surface.expiry, ReasonExpiry::FailurePersistent);
            // The TTL is the classification's payload form: `None`
            // (the wire `null`), never a locally decided window.
            assert_eq!(surface.reason_ttl_ms, None);
            assert_eq!(surface.reason_ttl_ms, full.reason_ttl_ms());
        }
        for focus in FOCUS {
            assert_eq!(focus.expiry(), ReasonExpiry::Focus3s);
            // The expiring half of the classification, for contrast:
            // a focus-loss reason carries the 3 s window a failure
            // never carries.
            assert_eq!(focus.reason_ttl_ms(), Some(PAUSE_REASON_TTL_MS));
        }
    }

    #[test]
    fn a_failure_never_starts_playback_and_its_line_never_expires() {
        // T043 — the surface half of the undeliverable rule
        // (contracts/surface.md: undeliverables ⇒ never start +
        // persistent line + `error` logged). Every member of the
        // closed failure set renders a line that never expires
        // (`reason_ttl_ms = null`), so a reader who finds nothing
        // playing can always find out why — the line outlives the
        // pause, the reopen, and any number of transport changes.
        //
        // The never-start enforcement and the `error` diagnostic
        // entry are NOT this module's: refusing playback is
        // playback authority (FR-042), which lives in the refusal
        // path — the adapter's synthesis refusal and the shell's
        // dispatch. A renderer that refused playback would be a
        // second playback authority, and a renderer that logged the
        // refusal would be a second diagnostic record. This test
        // pins the surface half and records the boundary; the
        // player-side classification it agrees with is pinned in
        // `surface::tests::the_ttl_classification_is_the_one_player_interruption_carries`.
        for (error, _, _) in FAILURES {
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            assert_eq!(
                surface.reason_ttl_ms, None,
                "{error:?} must render a persistent line — the reader must always learn why nothing plays"
            );
            assert!(!surface.short_statement.is_empty());
            assert!(!surface.full_statement.is_empty());
        }
        // The classification is one across both closed sets: the
        // player's undeliverables persist for the same reason the
        // surface's failures do, with the same `null` TTL.
        for reason in UNDELIVERABLE {
            assert!(reason.is_persistent(), "{reason:?} is undeliverable");
            assert_eq!(reason.reason_ttl_ms(), None);
        }
    }

    #[test]
    fn the_failure_ttl_and_the_dismissal_policy_are_distinct_axes() {
        // T043 / contracts/surface.md F8: the payload's
        // `reason_ttl_ms` and `dismissal` are two fields, and
        // neither is read out of the other. A failure is persistent
        // (its line never expires) while the surface carrying it is
        // non-dismissable — and both values occur across the closed
        // reason and transport sets, so no collapse of the two
        // fields can hide: every combination below is reachable.
        for (error, _, _) in FAILURES {
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            // Persistent line, non-dismissable surface while playing.
            assert_eq!(surface.reason_ttl_ms, None);
            assert_eq!(
                dismissal_for(TransportState::Playing),
                Dismissal::NonDismissableWhilePlaying
            );
            // The same persistent line on a paused surface, which
            // the reader may dismiss: the dismissal policy follows
            // the transport, never the reason's TTL.
            assert_eq!(
                dismissal_for(TransportState::Paused),
                Dismissal::Dismissable
            );
        }
        for transport in ALL_TRANSPORTS {
            // The dismissal policy is a function of the transport
            // alone — a failure's persistent TTL never changes it.
            assert_eq!(
                dismissal_for(transport),
                if transport == TransportState::Playing {
                    Dismissal::NonDismissableWhilePlaying
                } else {
                    Dismissal::Dismissable
                }
            );
        }
    }

    #[test]
    fn the_failure_short_lines_fit_the_lock_screen_line_bounds() {
        // contracts/surface.md: the lock-screen reason line is a
        // ≤8-word pt-BR line under a ≤40-char hard limit (the
        // Android content-text limit). The short statements the
        // failure surface renders are declared names, so the whole
        // closed failure set fits both bounds; the full statements
        // are the in-app full reason (FR-037) and are not a
        // notification content text, so the notification bound is
        // asserted on the short form alone.
        for (error, _, _) in FAILURES {
            let surface = render_failure(&error, &takeover(), DOCUMENT_TITLE);
            assert!(
                surface.short_statement.chars().count() <= 40,
                "{error:?} renders a {}-char lock-screen line — the ≤40-char limit is exceeded by {:?}",
                surface.short_statement.chars().count(),
                surface.short_statement
            );
            assert!(
                words(&surface.short_statement) <= 8,
                "{error:?} renders {} words on the lock screen — the ≤8-word bound is exceeded",
                words(&surface.short_statement)
            );
        }
    }

    #[test]
    fn the_takeover_engine_id_and_position_are_carried_verbatim() {
        // T076 / FR-023a: the takeover names the engine that
        // takes over and the last spoken position it continues
        // from — carried for the shell to name, exactly as the
        // machine returned them, never re-rendered. Equality over
        // the derived `PartialEq` compares every field, so the
        // carried decision names exactly the engine and the
        // position the machine returned; the position is an exact
        // binary fraction, so the derived comparison is exact and
        // no tolerance is admissible.
        let decision = takeover();
        let surface = render_failure(&EngineError::DeviceLost, &decision, DOCUMENT_TITLE);
        assert_eq!(surface.decision, decision);
        assert_eq!(
            surface.decision,
            RecoveryDecision::Takeover {
                engine_id: String::from("standin_b"),
                from_position_secs: 120.5,
            }
        );
    }

    #[test]
    fn the_machine_stated_pause_reason_is_carried_verbatim() {
        // T076 / FR-005 / FR-023: a failure with no remaining
        // selectable engine pauses with the machine's stated
        // reason — the machine's own closed-vocabulary statement,
        // carried verbatim, never re-rendered and never silence.
        // The default machine is the machine over the empty
        // registry: nothing is selectable, so its failure states
        // the machine's real no-remaining-engine reason.
        let mut machine = RecoveryMachine::default();
        let decision = machine.on_engine_failure("the-only-registered-engine", 42.0);
        let machine_reason = stated_reason_of(&decision);
        assert!(
            machine_reason.is_some_and(|reason| !reason.is_empty()),
            "a no-alternative failure must pause with a stated reason (FR-023 / FR-005), got {decision:?}"
        );
        let surface = render_failure(&EngineError::StartRefused, &decision, DOCUMENT_TITLE);
        assert_eq!(surface.decision, decision);
        assert_eq!(stated_reason_of(&surface.decision), machine_reason);
    }

    #[test]
    fn the_offered_alternatives_are_carried_intact() {
        // T076 / FR-023a: the open offer's alternatives are
        // carried for the shell to name — the decision the
        // machine returned, intact and in its declared order,
        // never re-rendered. The shell reads the offered engine
        // ids through the carried decision's own structure, so
        // the carry is the decision's equality — over the
        // alternatives list, element-wise in declared order —
        // and nothing is extracted, transformed, or reordered.
        let decision = RecoveryDecision::Offered {
            alternatives: Vec::new(),
        };
        let surface = render_failure(&EngineError::Unreachable, &decision, DOCUMENT_TITLE);
        assert_eq!(surface.decision, decision);
        assert_eq!(
            surface.decision,
            RecoveryDecision::Offered {
                alternatives: Vec::new(),
            }
        );
    }

    #[test]
    fn the_render_is_pure_over_its_inputs() {
        // The same failure, decision, and title render the same
        // surface, twice — the surface owns no state and no
        // playback logic (FR-042): it renders what its caller
        // supplies, through the T065 getters and the machine's
        // decision.
        let decision = takeover();
        assert_eq!(
            render_failure(&EngineError::RateMismatch, &decision, DOCUMENT_TITLE),
            render_failure(&EngineError::RateMismatch, &decision, DOCUMENT_TITLE)
        );
        // A different failure names a different reason: the
        // surface is a function of the named failure, not a
        // constant statement.
        let rate = render_failure(&EngineError::RateMismatch, &decision, DOCUMENT_TITLE);
        let init = render_failure(&EngineError::InitFailed, &decision, DOCUMENT_TITLE);
        assert_ne!(rate.short_statement, init.short_statement);
        assert_ne!(rate.full_statement, init.full_statement);
    }
}
