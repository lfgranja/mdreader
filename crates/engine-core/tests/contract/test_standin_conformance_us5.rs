//! T071 — [US5] Contract test: stand-in conformance.
//!
//! Invariant under test (tasks.md T071, verbatim):
//! "Contract test: stand-in conformance in
//! `crates/engine-core/tests/contract/test_standin_conformance_us5.rs`
//! (stand-ins A offline-like + B cloud-like pass
//! handoff-conformance: shape invariant all 7 rungs,
//! granularity honestly declared `Sentence|Word|None`,
//! failure taxonomy exact per
//! contracts/engine-handoff.md; FAIL before
//! implementation)"
//!
//! Spec anchors: contracts/engine-handoff.md
//! §"Conformance" ("Adapters pass handoff-conformance
//! tests: shape invariant across all 7 rate rungs,
//! granularity honestly declared, failure taxonomy
//! exact. Stand-in engines A (offline-like) and B
//! (cloud-like) demonstrate mid-document failover
//! from the last spoken position and
//! forced-double-failure ⇒ paused-with-reason"),
//! §"Engine trait" (`AlignmentGranularity`:
//! `Sentence | Word | None`; `set_rate` is a
//! pitch-preserving time-stretch **in the engine**
//! that MUST NOT alter the delivered sample rate
//! (FR-043, SC-034); `stream` is a pre-roll source,
//! ≤60 s outstanding, position never advances on
//! buffered-only audio; failures surface as the
//! closed `EngineError` set — never silence),
//! §"Fallback order" ("Remaining engines of the
//! failing tier → higher tiers in registry order →
//! Edge evaluated fallback last, never default.
//! Unconsented engines are never selected. Offers
//! name exactly the engines the fallback may
//! select."), FR-019 (the discrete rate ladder —
//! exactly the seven rungs), FR-033 (the engine
//! emits mono `f32` already resampled to the one
//! fixed declared rate), FR-035 (an engine that
//! cannot deliver the declared shape is a named
//! failure — never playback at a substituted format,
//! never a silent resample), FR-023 (a named engine
//! failure produces a named failure and an offer of
//! alternatives; the fallback auto-takes-over from
//! the last spoken position — never a restart; no
//! alternative ⇒ paused-with-reason, never silence),
//! FR-005 (the no-alternative expiry pauses with a
//! stated reason), FR-023c (the fallback order is
//! the declared `EngineTier::ORDER`), SC-034 (zero
//! instances of the audio the engine delivers
//! arriving at a sample rate other than the declared
//! 22050 Hz), SC-051 (the mid-document failover and
//! forced-double-failure drills), the 2026-10-05
//! clarification (the stand-ins are `test_double`
//! fixtures, admissible only while no
//! non-`test_double` adapter is registered),
//! Principle VI (the engine-neutral trait every
//! adapter implements) and Principle VII (closed
//! sets, exhaustive matches, no `_ =>` arms).
//!
//! FAILS in two independent ways before the US5
//! implementation (T075/T076) exists — either one
//! alone keeps this target red (T072):
//!
//! 1. The `engine_core::fallback` module is
//!    unresolved (E0432): T075 implements the
//!    fallback selector the SC-051 drill runs
//!    through. The canonical surface is the one
//!    `tests/prop_fallback.rs` documents; this file
//!    consumes only `FallbackSelector::new`,
//!    `selectable`, `on_failure`, `expire_offer`,
//!    `EngineDescriptor`, `EngineOffer` and
//!    `FallbackExpiry` from it.
//! 2. The `StandinA: Engine` and `StandinB: Engine`
//!    trait bounds are not satisfied (E0277): T076
//!    implements `engine_core::trait_::Engine` for
//!    the stand-ins. Every trait operation below is
//!    invoked QUALIFIED — `Engine::set_rate(&mut
//!    engine, rate)`, `Engine::stream(&mut engine,
//!    position_secs, max_secs)`,
//!    `Engine::declared_capabilities(&engine)` — on
//!    purpose: a plain `engine.set_rate(rate)` would
//!    silently resolve to the stand-ins' INHERENT
//!    `set_rate` (which returns `()`), and the red
//!    signal would be lost; the qualified call is
//!    what forces the precise E0277.
//!
//! Conformance legs (H1–H6), each asserted below:
//!
//! - H1 — shape invariant across all 7 rate rungs:
//!   after `Engine::set_rate` at any rung, both
//!   stand-ins deliver the declared mono-`f32`-at-
//!   22050 Hz shape, and the shape passes its own
//!   `check_handoff` (FR-043, SC-034).
//! - H2 — granularity honestly declared: each
//!   stand-in's declared `(SsmlLevel,
//!   AlignmentGranularity)` is a closed-set member
//!   that round-trips its wire form, and the
//!   declaration is stable across calls.
//! - H3 — a mid-document stream inside the pre-roll
//!   bound starts at the requested position and
//!   delivers boundary events at exactly the declared
//!   granularity, in ascending position order.
//! - H4 — the failure taxonomy is exact: a pre-roll
//!   budget above the 60 s bound is rejected before
//!   any synthesis runs, surfacing as a named member
//!   of the closed `EngineError` set whose wire form
//!   agrees with the exhaustive-match form, and the
//!   rejection never substitutes the delivered
//!   format (FR-035).
//! - H5 — the contract role split and the SC-051
//!   drills: A is the offline-like stand-in and B
//!   the cloud-like one; the mid-document failover
//!   offers B from the last spoken position
//!   (`FallbackExpiry::ContinueFromLastPosition`,
//!   never a restart), and the forced double failure
//!   pauses with a stated reason
//!   (`FallbackExpiry::PausedWithReason`, never
//!   silence).
//! - H6 — the pre-roll bound is inclusive: exactly
//!   60.0 s is the largest outstanding pre-roll the
//!   contract permits (`MAX_PREROLL_SECS`).

use proptest::prelude::*;
use serde_json::Value;

use engine_core::closed::{EngineError, EngineTier, PlaybackRate};
use engine_core::fallback::{EngineDescriptor, EngineOffer, FallbackExpiry, FallbackSelector};
use engine_core::format::{DeclaredFormat, check_handoff};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;
use engine_core::trait_::{
    AlignmentGranularity, BoundaryEvent, BoundaryKind, Engine, MAX_PREROLL_SECS, SsmlLevel,
};

/// Every rung of the closed rate ladder (FR-019) —
/// the same closed set the other engine-core property
/// files range over.
fn rate_strategy() -> impl Strategy<Value = PlaybackRate> {
    prop_oneof![
        Just(PlaybackRate::R0_75),
        Just(PlaybackRate::R1_0),
        Just(PlaybackRate::R1_25),
        Just(PlaybackRate::R1_5),
        Just(PlaybackRate::R1_75),
        Just(PlaybackRate::R2_0),
        Just(PlaybackRate::R4_0),
    ]
}

/// The declared handoff shape — mono `f32` at exactly
/// 22050 Hz — as `(rate_hz, channels, width_bits)`
/// (FR-033).
const DECLARED: (u32, u16, u16) = (
    DeclaredFormat::SAMPLE_RATE,
    DeclaredFormat::CHANNELS,
    DeclaredFormat::SAMPLE_WIDTH_BITS,
);

/// A mid-document position for the stream leg (H3):
/// past the start of the document, well inside any
/// session.
const MIDDOCUMENT_POSITION_SECS: f64 = 12.5;

/// A pre-roll budget comfortably inside the 60 s
/// bound (H3).
const PREROLL_BUDGET_SECS: f64 = 1.0;

/// The closed `SsmlLevel` set with its wire forms —
/// the three-member set the stand-ins declare from
/// (Principle VII; the wire form is the `snake_case`
/// level name).
const SSML_LEVELS: [(SsmlLevel, &str); 3] = [
    (SsmlLevel::None, "none"),
    (SsmlLevel::Partial, "partial"),
    (SsmlLevel::Full, "full"),
];

/// The closed `AlignmentGranularity` set with its
/// wire forms — the `Sentence|Word|None` set T071
/// requires the stand-ins to declare from (the wire
/// form is the `snake_case` granularity name).
const ALIGNMENT_GRANULARITIES: [(AlignmentGranularity, &str); 3] = [
    (AlignmentGranularity::None, "none"),
    (AlignmentGranularity::Word, "word"),
    (AlignmentGranularity::Sentence, "sentence"),
];

/// The closed `EngineError` set with its wire forms —
/// the seven-member failure taxonomy (FR-023, FR-035,
/// FR-037).
const ENGINE_ERRORS: [(EngineError, &str); 7] = [
    (EngineError::Unreachable, "engine_unreachable"),
    (EngineError::InitFailed, "engine_init_failed"),
    (EngineError::RateMismatch, "rate_mismatch"),
    (EngineError::ChannelMismatch, "channel_mismatch"),
    (EngineError::DeviceRefused, "device_refused"),
    (EngineError::DeviceLost, "device_lost"),
    (EngineError::StartRefused, "start_refused"),
];

/// The wire form of a failure-taxonomy member.
///
/// The exhaustive seven-arm match — no catch-all —
/// is the compile-time half of "failure taxonomy
/// exact": it compiles only while `EngineError` has
/// exactly the seven named members (Principle VII,
/// FR-044).
fn engine_error_wire_of(error: EngineError) -> &'static str {
    match error {
        EngineError::Unreachable => "engine_unreachable",
        EngineError::InitFailed => "engine_init_failed",
        EngineError::RateMismatch => "rate_mismatch",
        EngineError::ChannelMismatch => "channel_mismatch",
        EngineError::DeviceRefused => "device_refused",
        EngineError::DeviceLost => "device_lost",
        EngineError::StartRefused => "start_refused",
    }
}

/// Whether two `f64` values are exactly equal.
///
/// Every `f64` in this file is an exact binary
/// fraction (12.5, 1.0, 60.0, 60.5), so `==` is
/// exact — the same reasoning `closed.rs` records
/// for the rate rungs: a tolerance would wrongly
/// admit values that are merely close.
#[expect(
    clippy::float_cmp,
    reason = "every f64 in this file is an exact binary fraction (12.5, 1.0, 60.0, 60.5), so == is exact; a tolerance would wrongly admit off-ladder values that are merely close"
)]
#[must_use]
fn exactly_equal(left: f64, right: f64) -> bool {
    left == right
}

/// A declared capability pair is a closed-set member:
/// both halves round-trip through their wire forms, so
/// whatever a stand-in declares is a member of the
/// closed vocabulary — the declaration is the
/// pipeline's source of truth (contract "Engine
/// trait"; Principle VII).
fn assert_declared_pair_round_trips(ssml_level: SsmlLevel, granularity: AlignmentGranularity) {
    assert_eq!(
        serde_json::to_value(ssml_level).ok(),
        Some(Value::String(String::from(ssml_level.as_wire_str())))
    );
    assert_eq!(
        serde_json::to_value(granularity).ok(),
        Some(Value::String(String::from(granularity.as_wire_str())))
    );
    assert_eq!(
        serde_json::from_value::<SsmlLevel>(Value::String(String::from(ssml_level.as_wire_str())))
            .ok(),
        Some(ssml_level)
    );
    assert_eq!(
        serde_json::from_value::<AlignmentGranularity>(Value::String(String::from(
            granularity.as_wire_str()
        )))
        .ok(),
        Some(granularity)
    );
}

/// Boundary honesty (contract "Conformance":
/// "granularity honestly declared"): every delivered
/// boundary event is at exactly the declared
/// granularity — a `None` declaration is made
/// observable by the absence of events, and a
/// `Word`/`Sentence` declaration by events of that
/// kind only. A coarser event than declared would be
/// silent degradation, which Principle VI forbids:
/// degradation is visible in the declaration, never
/// in the delivered events.
fn assert_boundaries_respect_declared_granularity(
    boundaries: &[BoundaryEvent],
    granularity: AlignmentGranularity,
) {
    match granularity {
        AlignmentGranularity::None => assert!(
            boundaries.is_empty(),
            "a None alignment granularity delivered boundary events"
        ),
        AlignmentGranularity::Word => {
            for event in boundaries {
                assert_eq!(
                    event.kind,
                    BoundaryKind::Word,
                    "a word alignment granularity delivered a sentence boundary at {} s",
                    event.position_secs
                );
            }
        }
        AlignmentGranularity::Sentence => {
            for event in boundaries {
                assert_eq!(
                    event.kind,
                    BoundaryKind::Sentence,
                    "a sentence alignment granularity delivered a word boundary at {} s",
                    event.position_secs
                );
            }
        }
    }
}

/// Ascending order: the `StreamChunk` contract carries
/// boundary events "in ascending position order". The
/// check orders the positions with `partial_cmp`
/// rather than `<=` (which would need a
/// `clippy::float_cmp` suppression for no benefit):
/// an unordered (`NaN`) position is a delivery
/// defect, not a pass.
fn assert_ascending_boundary_positions(boundaries: &[BoundaryEvent]) {
    for (earlier, later) in boundaries.iter().zip(boundaries.iter().skip(1)) {
        let ascending = !matches!(
            earlier.position_secs.partial_cmp(&later.position_secs),
            Some(core::cmp::Ordering::Greater)
        );
        assert!(
            ascending,
            "boundary positions are not ascending: {} s before {} s",
            earlier.position_secs, later.position_secs
        );
    }
}

/// The declared capabilities, honestly and stably
/// (contract "Conformance": "granularity honestly
/// declared"): the closed vocabulary the declaration
/// draws from round-trips its wire forms — every
/// `SsmlLevel` and `AlignmentGranularity` member,
/// and every `EngineError` member with its
/// exhaustive-match wire form — and each stand-in
/// declares a stable pair from that vocabulary: two
/// invocations of `declared_capabilities` produce
/// the identical pair, and the declared pair is a
/// closed-set member whose wire form round-trips
/// back to it.
#[test]
fn the_standins_declare_closed_set_capabilities_honestly_and_stably() {
    // The closed capability vocabulary (Principle
    // VII).
    for (level, wire) in SSML_LEVELS {
        assert_eq!(level.as_wire_str(), wire);
        assert_eq!(
            serde_json::to_value(level).ok(),
            Some(Value::String(String::from(wire)))
        );
        assert_eq!(
            serde_json::from_value::<SsmlLevel>(Value::String(String::from(wire))).ok(),
            Some(level)
        );
    }
    for (granularity, wire) in ALIGNMENT_GRANULARITIES {
        assert_eq!(granularity.as_wire_str(), wire);
        assert_eq!(
            serde_json::to_value(granularity).ok(),
            Some(Value::String(String::from(wire)))
        );
        assert_eq!(
            serde_json::from_value::<AlignmentGranularity>(Value::String(String::from(wire))).ok(),
            Some(granularity)
        );
    }
    // The closed failure taxonomy: the serde wire
    // forms agree with the exhaustive-match wire
    // forms, member for member (contract
    // "Conformance": "failure taxonomy exact").
    for (error, wire) in ENGINE_ERRORS {
        assert_eq!(engine_error_wire_of(error), wire);
        assert_eq!(
            serde_json::to_value(error).ok(),
            Some(Value::String(String::from(wire)))
        );
    }
    // Stand-in A (offline-like) declares from the
    // closed vocabulary, stably across calls.
    let (a_first_level, a_first_granularity) = Engine::declared_capabilities(&StandinA);
    let (a_second_level, a_second_granularity) = Engine::declared_capabilities(&StandinA);
    assert_eq!(
        (a_first_level, a_first_granularity),
        (a_second_level, a_second_granularity),
        "StandinA's declared capabilities are not stable across calls"
    );
    assert_declared_pair_round_trips(a_first_level, a_first_granularity);
    // Stand-in B (cloud-like), the same honesty.
    let (b_first_level, b_first_granularity) = Engine::declared_capabilities(&StandinB);
    let (b_second_level, b_second_granularity) = Engine::declared_capabilities(&StandinB);
    assert_eq!(
        (b_first_level, b_first_granularity),
        (b_second_level, b_second_granularity),
        "StandinB's declared capabilities are not stable across calls"
    );
    assert_declared_pair_round_trips(b_first_level, b_first_granularity);
}

/// The contract role split and the SC-051 drills
/// (contract "Conformance": "Stand-in engines A
/// (offline-like) and B (cloud-like) demonstrate
/// mid-document failover from the last spoken
/// position and forced-double-failure ⇒
/// paused-with-reason"): A is the offline-like
/// stand-in and B the cloud-like one, both at the
/// `test_double` tier — admissible while no
/// non-`test_double` adapter is registered (the
/// 2026-10-05 clarification) — and the failover
/// drill runs through the fallback selector (T075):
/// with both stand-ins registered, both are
/// selectable in registry order; the offline-like
/// stand-in's mid-document failure produces a fresh
/// offer naming exactly the remaining selectable
/// engine; expiring the offer auto-takes-over from
/// the last spoken position (FR-023 — never a
/// restart); and the forced double failure, with no
/// alternative remaining, pauses with a stated
/// reason (FR-005 — never silence).
#[test]
fn the_standins_satisfy_the_contract_role_split() {
    assert_eq!(StandinA::ID, "standin_a");
    assert_eq!(StandinA::TIER, EngineTier::TestDouble);
    assert_eq!(StandinB::ID, "standin_b");
    assert_eq!(StandinB::TIER, EngineTier::TestDouble);

    // The drill registry: both stand-ins, in registry
    // order. With no non-`test_double` adapter
    // registered, both are admissible and consented by
    // construction, so both are selectable — ascending
    // `EngineTier::ORDER`, registry order within the
    // tier.
    let registry = vec![
        EngineDescriptor {
            id: String::from(StandinA::ID),
            tier: StandinA::TIER,
        },
        EngineDescriptor {
            id: String::from(StandinB::ID),
            tier: StandinB::TIER,
        },
    ];
    let mut selector = FallbackSelector::new(registry);
    assert_eq!(
        selector.selectable(),
        vec![
            EngineOffer {
                engine_id: String::from(StandinA::ID),
                tier: StandinA::TIER,
            },
            EngineOffer {
                engine_id: String::from(StandinB::ID),
                tier: StandinB::TIER,
            },
        ],
        "the selectable set did not name exactly the two stand-ins in registry order"
    );

    // Mid-document: the offline-like stand-in fails.
    // The fresh offer names exactly the remaining
    // selectable engine — the cloud-like stand-in.
    assert_eq!(
        selector.on_failure(StandinA::ID),
        Some(EngineOffer {
            engine_id: String::from(StandinB::ID),
            tier: StandinB::TIER,
        }),
        "the failover offer did not name the cloud-like stand-in"
    );

    // The offer expires: the fallback auto-takes over
    // from the last spoken position — never a restart
    // (FR-023). The variant carries no position payload
    // at all, so the position stays the caller's last
    // spoken position.
    assert_eq!(
        selector.expire_offer(),
        FallbackExpiry::ContinueFromLastPosition {
            engine_id: String::from(StandinB::ID),
        },
        "the expired offer did not continue from the last spoken position on the cloud-like stand-in"
    );

    // Forced double failure: the cloud-like stand-in
    // fails too, and no alternative remains.
    assert_eq!(
        selector.on_failure(StandinB::ID),
        None,
        "a second failure with no remaining engine still produced an offer"
    );

    // The no-alternative expiry pauses with a stated
    // reason — never silence (FR-005), and never a
    // continuation, because no engine is left to
    // continue on.
    let expiry = selector.expire_offer();
    assert!(
        matches!(
            &expiry,
            FallbackExpiry::PausedWithReason { reason } if !reason.is_empty()
        ),
        "the no-alternative expiry was {expiry:?} instead of a pause with a stated reason"
    );
}

/// The pre-roll bound (H6): the maximum pre-roll an
/// engine may hold outstanding is exactly 60.0
/// seconds (contracts/engine-handoff.md,
/// "`stream()` pre-roll source; ≤60 s outstanding")
/// — the bound is inclusive, so a budget of exactly
/// 60.0 s is the largest the contract permits and
/// 60.5 s is the smallest the contract rejects.
#[test]
fn the_declared_pre_roll_bound_is_sixty_seconds() {
    assert_eq!(MAX_PREROLL_SECS, 60.0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// H1 + H2 + H3 for the offline-like stand-in:
    /// after `Engine::set_rate` at any of the seven
    /// rungs, StandinA declares its capabilities
    /// honestly, delivers the declared mono-`f32`-at-
    /// 22050 Hz shape — the pitch-preserving
    /// time-stretch (FR-043) happens inside the engine
    /// and never alters the delivered sample rate
    /// (SC-034) — and a mid-document stream inside the
    /// pre-roll bound starts at the requested position
    /// and delivers boundary events at exactly the
    /// declared granularity, in ascending position
    /// order.
    #[test]
    fn standin_a_accepts_every_rung_and_delivers_the_declared_handoff_shape(
        rate in rate_strategy(),
    ) {
        let mut engine = StandinA;
        // H2: the declared capabilities are a closed-set
        // member — both halves round-trip their wire
        // forms (the declaration is the pipeline's
        // source of truth, contract "Engine trait").
        let (ssml_level, granularity) = Engine::declared_capabilities(&engine);
        prop_assert_eq!(
            serde_json::to_value(ssml_level).ok(),
            Some(Value::String(String::from(ssml_level.as_wire_str())))
        );
        prop_assert_eq!(
            serde_json::to_value(granularity).ok(),
            Some(Value::String(String::from(granularity.as_wire_str())))
        );
        // H1: the rate change is a pitch-preserving
        // time-stretch inside the engine — the
        // delivered shape is invariant across all
        // seven rungs.
        prop_assert!(
            Engine::set_rate(&mut engine, rate).is_ok(),
            "the offline-like stand-in refused the rate rung {rate:?}"
        );
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinA deviated from the declared shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinA's shape failed its own handoff check at rate {rate:?}"
        );
        // H3: a mid-document stream inside the pre-roll
        // bound starts at the requested position and
        // delivers boundary events at exactly the
        // declared granularity, in ascending position
        // order.
        match Engine::stream(
            &mut engine,
            MIDDOCUMENT_POSITION_SECS,
            PREROLL_BUDGET_SECS,
        ) {
            Ok(chunk) => {
                prop_assert!(
                    exactly_equal(chunk.start_position_secs, MIDDOCUMENT_POSITION_SECS),
                    "the mid-document chunk started at {} s instead of {position} s",
                    chunk.start_position_secs,
                    position = MIDDOCUMENT_POSITION_SECS
                );
                assert_boundaries_respect_declared_granularity(&chunk.boundaries, granularity);
                assert_ascending_boundary_positions(&chunk.boundaries);
            }
            Err(error) => {
                // A conforming stand-in streams
                // mid-document audio inside the pre-roll
                // bound without failing: reaching this
                // arm is the failure. The named member is
                // mapped through the exhaustive taxonomy
                // match — the compile-time half of
                // "failure taxonomy exact" — and the case
                // fails naming the member's wire form.
                let wire = engine_error_wire_of(error);
                prop_assert!(
                    wire.is_empty(),
                    "a mid-document stream at {position} s inside the {bound} s pre-roll bound failed with {wire}",
                    position = MIDDOCUMENT_POSITION_SECS,
                    bound = MAX_PREROLL_SECS
                );
            }
        }
    }

    /// The same H1 + H2 + H3 invariant for the
    /// cloud-like stand-in: SC-051 needs two working
    /// engines for the mid-document failover drill,
    /// and both honour the same handoff contract.
    #[test]
    fn standin_b_accepts_every_rung_and_delivers_the_declared_handoff_shape(
        rate in rate_strategy(),
    ) {
        let mut engine = StandinB;
        // H2: the declared capabilities are a closed-set
        // member — both halves round-trip their wire
        // forms (the declaration is the pipeline's
        // source of truth, contract "Engine trait").
        let (ssml_level, granularity) = Engine::declared_capabilities(&engine);
        prop_assert_eq!(
            serde_json::to_value(ssml_level).ok(),
            Some(Value::String(String::from(ssml_level.as_wire_str())))
        );
        prop_assert_eq!(
            serde_json::to_value(granularity).ok(),
            Some(Value::String(String::from(granularity.as_wire_str())))
        );
        // H1: the rate change is a pitch-preserving
        // time-stretch inside the engine — the
        // delivered shape is invariant across all
        // seven rungs.
        prop_assert!(
            Engine::set_rate(&mut engine, rate).is_ok(),
            "the cloud-like stand-in refused the rate rung {rate:?}"
        );
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinB deviated from the declared shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinB's shape failed its own handoff check at rate {rate:?}"
        );
        // H3: a mid-document stream inside the pre-roll
        // bound starts at the requested position and
        // delivers boundary events at exactly the
        // declared granularity, in ascending position
        // order.
        match Engine::stream(
            &mut engine,
            MIDDOCUMENT_POSITION_SECS,
            PREROLL_BUDGET_SECS,
        ) {
            Ok(chunk) => {
                prop_assert!(
                    exactly_equal(chunk.start_position_secs, MIDDOCUMENT_POSITION_SECS),
                    "the mid-document chunk started at {} s instead of {position} s",
                    chunk.start_position_secs,
                    position = MIDDOCUMENT_POSITION_SECS
                );
                assert_boundaries_respect_declared_granularity(&chunk.boundaries, granularity);
                assert_ascending_boundary_positions(&chunk.boundaries);
            }
            Err(error) => {
                // A conforming stand-in streams
                // mid-document audio inside the pre-roll
                // bound without failing: reaching this
                // arm is the failure. The named member is
                // mapped through the exhaustive taxonomy
                // match — the compile-time half of
                // "failure taxonomy exact" — and the case
                // fails naming the member's wire form.
                let wire = engine_error_wire_of(error);
                prop_assert!(
                    wire.is_empty(),
                    "a mid-document stream at {position} s inside the {bound} s pre-roll bound failed with {wire}",
                    position = MIDDOCUMENT_POSITION_SECS,
                    bound = MAX_PREROLL_SECS
                );
            }
        }
    }
}

/// H4 + H6 for the offline-like stand-in: the
/// failure taxonomy is exact — a pre-roll budget
/// above the 60 s bound is rejected before any
/// synthesis runs, surfacing as a named member of
/// the closed `EngineError` set whose wire form
/// agrees with the exhaustive-match form, and the
/// rejection never substitutes the delivered
/// format (FR-035) — while the bound itself is
/// inclusive: exactly 60.0 s is accepted (H6).
#[test]
fn standin_a_rejects_a_pre_roll_budget_above_sixty_seconds() {
    let mut engine = StandinA;
    let over_bound = MAX_PREROLL_SECS + 0.5;
    match Engine::stream(&mut engine, 0.0, over_bound) {
        Ok(_) => assert!(
            exactly_equal(over_bound, MAX_PREROLL_SECS),
            "a pre-roll budget of {over_bound} s was accepted despite exceeding the {bound} s bound",
            bound = MAX_PREROLL_SECS
        ),
        Err(error) => {
            // The rejection surfaces as a named
            // member of the closed failure taxonomy
            // (contract "Conformance": "failure
            // taxonomy exact") — the exhaustive
            // seven-arm match is the compile-time
            // proof, and the named member's wire
            // form agrees with it.
            let wire = engine_error_wire_of(error);
            assert_eq!(
                serde_json::to_value(error).ok(),
                Some(Value::String(String::from(wire))),
                "the over-bound rejection named {wire}, whose wire form drifted from the closed taxonomy"
            );
        }
    }
    // A rejected budget never substitutes the
    // delivered format (FR-035): the declared
    // shape is unchanged.
    assert_eq!(
        engine.delivered_format(),
        DECLARED,
        "the over-bound rejection substituted StandinA's delivered format"
    );
    // H6: the bound is inclusive — exactly 60.0 s
    // is the largest outstanding pre-roll the
    // contract permits.
    let at_bound = Engine::stream(&mut engine, 0.0, MAX_PREROLL_SECS);
    assert!(
        at_bound.is_ok(),
        "a pre-roll budget of exactly {bound} s was rejected",
        bound = MAX_PREROLL_SECS
    );
}

/// The same H4 + H6 invariant for the cloud-like
/// stand-in: the rejection path is part of the
/// handoff contract for both engines of the
/// SC-051 pair.
#[test]
fn standin_b_rejects_a_pre_roll_budget_above_sixty_seconds() {
    let mut engine = StandinB;
    let over_bound = MAX_PREROLL_SECS + 0.5;
    match Engine::stream(&mut engine, 0.0, over_bound) {
        Ok(_) => assert!(
            exactly_equal(over_bound, MAX_PREROLL_SECS),
            "a pre-roll budget of {over_bound} s was accepted despite exceeding the {bound} s bound",
            bound = MAX_PREROLL_SECS
        ),
        Err(error) => {
            // The rejection surfaces as a named
            // member of the closed failure taxonomy
            // (contract "Conformance": "failure
            // taxonomy exact") — the exhaustive
            // seven-arm match is the compile-time
            // proof, and the named member's wire
            // form agrees with it.
            let wire = engine_error_wire_of(error);
            assert_eq!(
                serde_json::to_value(error).ok(),
                Some(Value::String(String::from(wire))),
                "the over-bound rejection named {wire}, whose wire form drifted from the closed taxonomy"
            );
        }
    }
    // A rejected budget never substitutes the
    // delivered format (FR-035): the declared
    // shape is unchanged.
    assert_eq!(
        engine.delivered_format(),
        DECLARED,
        "the over-bound rejection substituted StandinB's delivered format"
    );
    // H6: the bound is inclusive — exactly 60.0 s
    // is the largest outstanding pre-roll the
    // contract permits.
    let at_bound = Engine::stream(&mut engine, 0.0, MAX_PREROLL_SECS);
    assert!(
        at_bound.is_ok(),
        "a pre-roll budget of exactly {bound} s was rejected",
        bound = MAX_PREROLL_SECS
    );
}
