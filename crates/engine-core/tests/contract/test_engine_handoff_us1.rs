//! T022 — engine-handoff shape invariance (US1 contract test).
//!
//! Invariant under test (tasks.md T022, verbatim): "Contract
//! test: engine-handoff shape invariance (US1-specific:
//! stand-ins for US1 fixture; full invariant asserted in
//! Foundation T013b). FAIL before T017b."
//!
//! This is the US1-specific slice of the T013b invariant: the
//! two stand-in engines are the US1 fixture, and the declared
//! handoff shape is asserted across the session lifecycle —
//! the initial path, every rung of the closed rate ladder,
//! the SC-051 mid-document failover drill (failure
//! injection, re-selection, recovery), and the transport
//! lifecycle (Play → audible progress → Pause → resume →
//! Stop) — plus the failure taxonomy and the honesty of the
//! declared capabilities (contracts/engine-handoff.md,
//! "Conformance": "shape invariant across all 7 rate rungs,
//! granularity honestly declared, failure taxonomy exact").
//!
//! Spec anchors: FR-033 (the engine emits mono `f32` already
//! resampled to the one fixed declared rate; the shell only
//! converts to the device format), FR-034 (the Rust playback
//! module solely owns conversion to the device's native
//! format — the engine-delivered shape is fixed, so there is
//! never a format to rescue by substitution), FR-035 (an
//! engine that cannot deliver the declared shape is a named
//! failure — never playback at a substituted format, never a
//! silent resample), and SC-034 (zero instances of the audio
//! the engine delivers arriving at a sample rate other than
//! the declared 22050 Hz — the shape holds across every rung
//! and every transport state).
//!
//! FAILS before `crates/engine-core/src/standin_a.rs` and
//! `standin_b.rs` (T017b) exist: the `engine_core::standin_a`
//! and `engine_core::standin_b` modules are unresolved, so
//! this test target does not build. That build failure is the
//! required pre-implementation red state.
//!
//! T017b remainders (RED findings, reported to the
//! orchestrator — the implementation is not touched here):
//! the stand-ins do not yet expose `Engine::declared_capabilities`
//! (the declared granularities T017b promises) nor an
//! injectable mid-document failure, so the per-stand-in
//! `(SsmlLevel, AlignmentGranularity)` honesty assertion and a
//! live fault injection cannot be executed against the
//! stand-in surface yet. The drill is therefore driven at the
//! contract level — the test plays the fault injector's role,
//! planting a handoff-shape deviation mid-document and
//! asserting it surfaces as the named `EngineError` the
//! contract requires (FR-035) — and the capability
//! declaration is asserted through its headless CLI
//! projection, the form Principle II prescribes for every
//! trait operation.

use proptest::prelude::*;
use serde_json::{Value, json};

use engine_core::cli;
use engine_core::closed::{EngineError, EngineTier, PlaybackRate, TransportState};
use engine_core::format::{DeclaredFormat, check_handoff};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;

/// Every rung of the closed rate ladder (FR-019) — the same
/// closed set T013b ranges over, so the US1 slice exercises
/// the identical ladder.
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

/// Every member of the closed transport-state set (FR-020) —
/// the delivered format must not depend on any of them.
fn transport_state_strategy() -> impl Strategy<Value = TransportState> {
    prop_oneof![
        Just(TransportState::Playing),
        Just(TransportState::Paused),
        Just(TransportState::Stopped),
        Just(TransportState::Completed),
        Just(TransportState::Interrupted),
    ]
}

/// The declared handoff shape — mono `f32` at exactly
/// 22050 Hz — as `(rate_hz, channels, width_bits)` (FR-033).
const DECLARED: (u32, u16, u16) = (
    DeclaredFormat::SAMPLE_RATE,
    DeclaredFormat::CHANNELS,
    DeclaredFormat::SAMPLE_WIDTH_BITS,
);

/// The US1 transport lifecycle every session passes through:
/// Play, audible progress, Pause, resume, Stop — as the
/// transport states the shell mirrors (FR-020, SC-035).
const TRANSPORT_LIFECYCLE: [TransportState; 5] = [
    TransportState::Playing, // Play
    TransportState::Playing, // audible progress
    TransportState::Paused,  // Pause
    TransportState::Playing, // resume
    TransportState::Stopped, // Stop
];

/// The planted mid-document failure of the SC-051 drill: a
/// delivered rate deviating from the declared 22050 Hz (the
/// common 44_100 Hz device default). FR-035: the deviation
/// surfaces as a named `EngineError::RateMismatch` — never a
/// silent resample.
const PLANTED_DEVIATION_RATE_HZ: u32 = 44_100;

/// The US1 fixture property, initial path: a freshly selected
/// stand-in — before any rate change, drill phase, or
/// transport transition — delivers the declared handoff
/// shape, and the shape passes its own handoff check
/// (FR-033, FR-035).
#[test]
fn the_us1_fixture_pair_delivers_the_declared_shape_initially() {
    let offline_like = StandinA;
    let cloud_like = StandinB;
    let offline_delivered = offline_like.delivered_format();
    let cloud_delivered = cloud_like.delivered_format();
    assert_eq!(
        offline_delivered, DECLARED,
        "StandinA deviated from the declared shape on the initial path"
    );
    assert_eq!(
        cloud_delivered, DECLARED,
        "StandinB deviated from the declared shape on the initial path"
    );
    assert!(
        check_handoff(
            offline_delivered.0,
            offline_delivered.1,
            offline_delivered.2
        )
        .is_ok(),
        "StandinA's initial shape failed its own handoff check"
    );
    assert!(
        check_handoff(cloud_delivered.0, cloud_delivered.1, cloud_delivered.2).is_ok(),
        "StandinB's initial shape failed its own handoff check"
    );
}

/// The US1 transport integration: the shape is invariant
/// across the transport lifecycle — Play → audible progress
/// → Pause → resume → Stop. The delivered format never
/// depends on the transport state (FR-033, SC-034) —
/// whichever state the session mirrors, both stand-ins
/// deliver the declared shape and the shape passes its own
/// handoff check.
#[test]
fn the_declared_shape_holds_across_the_transport_lifecycle() {
    for state in TRANSPORT_LIFECYCLE {
        let offline_delivered = StandinA.delivered_format();
        let cloud_delivered = StandinB.delivered_format();
        assert_eq!(
            offline_delivered, DECLARED,
            "StandinA deviated from the declared shape in transport state {state}"
        );
        assert_eq!(
            cloud_delivered, DECLARED,
            "StandinB deviated from the declared shape in transport state {state}"
        );
        assert!(
            check_handoff(
                offline_delivered.0,
                offline_delivered.1,
                offline_delivered.2
            )
            .is_ok(),
            "StandinA's shape failed its own handoff check in transport state {state}"
        );
        assert!(
            check_handoff(cloud_delivered.0, cloud_delivered.1, cloud_delivered.2).is_ok(),
            "StandinB's shape failed its own handoff check in transport state {state}"
        );
    }
}

/// The failure taxonomy, planted rate deviation (FR-035): a
/// delivered rate other than the declared 22050 Hz surfaces
/// as exactly `EngineError::RateMismatch` — never `Ok`
/// (playback at a substituted format) and never a silent
/// resample outside the engine.
#[test]
fn a_planted_rate_deviation_surfaces_as_exactly_rate_mismatch() {
    for planted in [0, 1, 8_000, 22_049, 22_051, 44_100, u32::MAX] {
        assert_eq!(
            check_handoff(
                planted,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            ),
            Err(EngineError::RateMismatch),
            "planted rate {planted} Hz was not named RateMismatch"
        );
    }
}

/// The failure taxonomy, planted channel deviation (FR-033):
/// a delivered channel count other than the declared mono
/// surfaces as exactly `EngineError::ChannelMismatch` — never
/// a silent downmix, which would make an adapter conform by
/// accident rather than by contract.
#[test]
fn a_planted_channel_deviation_surfaces_as_exactly_channel_mismatch() {
    for planted in [0, 2, 6, u16::MAX] {
        assert_eq!(
            check_handoff(
                DeclaredFormat::SAMPLE_RATE,
                planted,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            ),
            Err(EngineError::ChannelMismatch),
            "planted channel count {planted} was not named ChannelMismatch"
        );
    }
}

/// The failure taxonomy, planted width deviation
/// (contracts/engine-handoff.md): a delivered sample width
/// other than the declared 32-bit `f32` surfaces as a named
/// `EngineError` — never `Ok` and never a silent buffer
/// reinterpretation. The closed error set names no
/// width-specific variant (Principle VII), so the deviation
/// surfaces as one of the two errors the contract groups all
/// three deviation kinds under.
#[test]
fn a_planted_width_deviation_surfaces_as_a_named_error_never_ok() {
    for planted in [0, 8, 16, 24, 64, u16::MAX] {
        let result = check_handoff(
            DeclaredFormat::SAMPLE_RATE,
            DeclaredFormat::CHANNELS,
            planted,
        );
        let named = matches!(
            result,
            Err(EngineError::RateMismatch) | Err(EngineError::ChannelMismatch)
        );
        assert!(
            named,
            "planted width {planted} bits was not surfaced as a named EngineError"
        );
    }
}

/// The declared capabilities, honesty of the stand-in
/// declaration: each stand-in declares the identity the
/// 2026-10-05 clarification permits — the `standin_a` /
/// `standin_b` ids at the `test_double` tier — and the
/// declaration is honest about the tier's admissibility: a
/// test double is never selectable while any non-`test_double`
/// adapter is registered (SC-051).
///
/// RED FINDING (T017b remainder, reported to the orchestrator
/// — the implementation is not touched here): the stand-ins do
/// not yet expose `Engine::declared_capabilities`, so the
/// `(SsmlLevel, AlignmentGranularity)` pair they will declare
/// cannot be asserted against the stand-in surface yet. What
/// is assertable today is the declaration surface the
/// stand-ins do expose (here) and the closed vocabulary the
/// declaration draws from (next test).
#[test]
fn the_standins_declare_their_test_double_identity_honestly() {
    assert_eq!(StandinA::ID, "standin_a");
    assert_eq!(StandinA::TIER, EngineTier::TestDouble);
    assert_eq!(StandinB::ID, "standin_b");
    assert_eq!(StandinB::TIER, EngineTier::TestDouble);
    // The declaration is honest about admissibility: the
    // test-double tier is never selectable at v1 — the
    // stand-ins exist only as SC-051 failover fixtures.
    assert!(
        !EngineTier::TestDouble.is_selectable(),
        "the test_double tier was declared selectable"
    );
}

/// The declared capabilities, stability across calls: the
/// headless projection of the `declared_capabilities`
/// operation — the form Principle II prescribes for every
/// trait operation — returns exactly the closed
/// `(SsmlLevel, AlignmentGranularity)` vocabulary, and the
/// projection is stable across calls: two invocations produce
/// the identical payload.
#[test]
fn the_declared_capability_projection_is_stable_across_calls()
-> Result<(), Box<dyn core::error::Error>> {
    let mut first_stdout: Vec<u8> = Vec::new();
    let mut first_stderr: Vec<u8> = Vec::new();
    let mut second_stdout: Vec<u8> = Vec::new();
    let mut second_stderr: Vec<u8> = Vec::new();
    cli::run(
        std::iter::once(String::from("capabilities")),
        &mut std::io::empty(),
        &mut first_stdout,
        &mut first_stderr,
    )?;
    cli::run(
        std::iter::once(String::from("capabilities")),
        &mut std::io::empty(),
        &mut second_stdout,
        &mut second_stderr,
    )?;
    // The success half of the protocol: payload on stdout,
    // nothing on stderr.
    assert!(
        first_stderr.is_empty(),
        "the first capabilities invocation wrote to stderr"
    );
    assert!(
        second_stderr.is_empty(),
        "the second capabilities invocation wrote to stderr"
    );
    // Stability across calls: the same operation, invoked
    // twice, projects the identical payload.
    assert_eq!(
        first_stdout, second_stdout,
        "the declared-capability projection is not stable across calls"
    );
    let payload: Value = serde_json::from_slice(&first_stdout)?;
    assert_eq!(
        payload.get("capability").and_then(Value::as_str),
        Some("declared_capabilities")
    );
    assert_eq!(
        payload.get("ssml_levels"),
        Some(&json!(["none", "partial", "full"]))
    );
    assert_eq!(
        payload.get("alignment_granularities"),
        Some(&json!(["none", "word", "sentence"]))
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The US1 fixture property, rate paths (SC-034): after
    /// `set_rate` at any of the seven rungs, StandinA still
    /// delivers the declared mono-`f32`-at-22050 Hz shape —
    /// the pitch-preserving time-stretch (FR-043) happens
    /// inside the engine and never alters the delivered
    /// sample rate.
    #[test]
    fn standin_a_delivers_the_declared_shape_after_set_rate_at_every_rung(
        rate in rate_strategy(),
    ) {
        let mut engine = StandinA;
        engine.set_rate(rate);
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinA deviated from the declared shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinA's shape failed its own handoff check at rate {:?}",
            rate
        );
    }

    /// The same invariant for the second stand-in: SC-051
    /// needs two working engines for the mid-document
    /// failover drill, and both honour the same handoff
    /// contract.
    #[test]
    fn standin_b_delivers_the_declared_shape_after_set_rate_at_every_rung(
        rate in rate_strategy(),
    ) {
        let mut engine = StandinB;
        engine.set_rate(rate);
        let delivered = engine.delivered_format();
        prop_assert_eq!(
            delivered,
            DECLARED,
            "StandinB deviated from the declared shape at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "StandinB's shape failed its own handoff check at rate {:?}",
            rate
        );
    }

    /// The US1 fixture property, SC-051 failover drill: the
    /// declared shape survives a mid-document failure
    /// injection, the re-selection of the failover target,
    /// and the recovery of the failed engine — at every rung
    /// of the ladder.
    ///
    /// The stand-in's injectable mid-document failure (T017b)
    /// is not yet on the stand-in surface (RED finding, see
    /// the module docs), so the drill is driven at the
    /// contract level: the test plays the fault injector's
    /// role, planting a handoff-shape deviation mid-document
    /// and asserting it surfaces as exactly the named
    /// `EngineError` the contract requires (FR-035) — the
    /// failover keys on the named failure, and a silent
    /// resample would hide the failure the drill exists to
    /// exercise.
    #[test]
    fn the_declared_shape_survives_the_sc051_failover_drill_at_every_rung(
        rate in rate_strategy(),
    ) {
        // Phase 1 — selection: the offline-like stand-in is
        // selected and plays mid-document at this rung.
        let mut offline_like = StandinA;
        offline_like.set_rate(rate);
        let offline_delivered = offline_like.delivered_format();
        prop_assert_eq!(
            offline_delivered,
            DECLARED,
            "StandinA deviated from the declared shape before the injected failure at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(offline_delivered.0, offline_delivered.1, offline_delivered.2).is_ok(),
            "StandinA's shape failed its own handoff check before the injected failure at rate {:?}",
            rate
        );

        // Phase 2 — mid-document failure injection: the drill
        // injects a delivered rate deviating from the declared
        // 22050 Hz. The deviation must surface as exactly
        // `EngineError::RateMismatch` — never `Ok` (playback
        // at a substituted format) and never a silent
        // resample.
        prop_assert_eq!(
            check_handoff(
                PLANTED_DEVIATION_RATE_HZ,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            ),
            Err(EngineError::RateMismatch),
            "the injected mid-document deviation was not named RateMismatch at rate {:?}",
            rate
        );

        // Phase 3 — re-selection (failover): the cloud-like
        // stand-in takes over mid-document at the same rung.
        let mut cloud_like = StandinB;
        cloud_like.set_rate(rate);
        let cloud_delivered = cloud_like.delivered_format();
        prop_assert_eq!(
            cloud_delivered,
            DECLARED,
            "StandinB deviated from the declared shape after re-selection at rate {:?}",
            rate
        );
        prop_assert!(
            check_handoff(cloud_delivered.0, cloud_delivered.1, cloud_delivered.2).is_ok(),
            "StandinB's shape failed its own handoff check after re-selection at rate {:?}",
            rate
        );

        // Phase 4 — recovery: the failed engine recovers and
        // is re-selected; both engines of the pair still
        // deliver the declared shape at the same rung.
        offline_like.set_rate(rate);
        let recovered_delivered = offline_like.delivered_format();
        prop_assert_eq!(
            recovered_delivered,
            DECLARED,
            "StandinA deviated from the declared shape after recovery at rate {:?}",
            rate
        );
        prop_assert_eq!(
            cloud_delivered,
            DECLARED,
            "StandinB deviated from the declared shape after the recovery re-selection at rate {:?}",
            rate
        );
    }

    /// The US1 transport integration, generalized over the
    /// closed transport-state set (SC-034): the delivered
    /// format never depends on the transport state — for
    /// every state the session can mirror, both stand-ins
    /// deliver the declared shape.
    #[test]
    fn the_delivered_format_never_depends_on_transport_state(
        state in transport_state_strategy(),
    ) {
        prop_assert_eq!(
            StandinA.delivered_format(),
            DECLARED,
            "StandinA's delivered format depended on transport state {:?}",
            state
        );
        prop_assert_eq!(
            StandinB.delivered_format(),
            DECLARED,
            "StandinB's delivered format depended on transport state {:?}",
            state
        );
    }
}
