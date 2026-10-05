//! T010 — closed-set exhaustiveness property tests (Phase 2, BLOCKING).
//!
//! Invariant under test (tasks.md T010, verbatim): "every closed enum — rate
//! rungs `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, transport
//! `Playing|Paused|Stopped|Completed|Interrupted`, focus outcomes
//! `DuckableTransient|SilenceDemanding|PermanentLoss` + `regained` direction
//! event, `EngineError Unreachable|InitFailed|RateMismatch|ChannelMismatch|
//! DeviceRefused|DeviceLost|StartRefused`, session-end
//! `Paused|Stopped|Completed|Interrupted` — has exactly the specified variants,
//! no `Other/Unknown`, and `serde` round-trips every variant"
//!
//! Spec anchors: FR-044 (closed sets; extending one is a coordinated edit
//! across every file that enumerates it, with a semantic-version bump and a
//! documentation update), FR-020 (transport-state reflection), FR-019 (the
//! discrete rate ladder), FR-023c (closed fallback tier order), Principle VII
//! (closed sets everywhere, no `_ =>` arms, no `Other`/`Unknown`).
//!
//! FAILS before `crates/engine-core/src/closed.rs` exists (T014): the
//! `engine_core::closed` module is unresolved, so this test target does not
//! build. That build failure is the required pre-implementation red state.
//!
//! Expected API contract for T014 (`crates/engine-core/src/closed.rs`), all
//! types with `Debug+Clone+Copy+PartialEq+Eq+serde`:
//! - `PlaybackRate` — exactly `R0_75 | R1_0 | R1_25 | R1_5 | R1_75 | R2_0 |
//!   R4_0`; serde wire form is the JSON number `0.75 | 1.0 | 1.25 | 1.5 |
//!   1.75 | 2.0 | 4.0` (contracts/shell-ipc.md case-mapping table)
//! - `TransportState` — exactly `Playing | Paused | Stopped | Completed |
//!   Interrupted`; wire `playing | paused | stopped | completed | interrupted`
//! - `FocusOutcome` — exactly `DuckableTransient | SilenceDemanding |
//!   PermanentLoss`; wire `duckable_transient | silence_demanding |
//!   permanent_loss`
//! - `FocusRegained` — the `regained` direction event (exactly one variant,
//!   `Regained`; wire `regained`). Regained focus is an event of direction,
//!   not a fourth outcome classification (spec.md, session 2026-10-02)
//! - `EngineError` — exactly `Unreachable | InitFailed | RateMismatch |
//!   ChannelMismatch | DeviceRefused | DeviceLost | StartRefused`; wire
//!   `engine_unreachable | engine_init_failed | rate_mismatch |
//!   channel_mismatch | device_refused | device_lost | start_refused`
//! - `SessionEnd` — exactly `Paused | Stopped | Completed | Interrupted`;
//!   wire `paused | stopped | completed | interrupted` (never `playing`)
//! - `EngineTier` — exactly `OfflineDefault | CloudDefault | CloudEscalation |
//!   TestDouble | EvaluatedFallback`, with `pub const ORDER: [EngineTier; 5]`
//!   in ascending fallback order; wire `offline_default | cloud_default |
//!   cloud_escalation | test_double | evaluated_fallback`
//!
//! Exhaustiveness mechanism: every enum is consumed by a wildcard-free
//! exhaustive `match` (the `*_wire_of` helpers below). Such a match compiles
//! only when the enum has exactly the named variants — an added or removed
//! variant breaks this test target's build, which makes "exactly the specified
//! variants, no `Other`/`Unknown`" mechanically decidable rather than a
//! matter of judgement (Principle V).

use proptest::prelude::*;
use serde_json::{Value, json};

use engine_core::closed::{
    EngineError, EngineTier, FocusOutcome, FocusRegained, PlaybackRate, SessionEnd, TransportState,
};

/// Rate rungs with their exact wire-form numbers (the FR-019 ladder).
const RATE_RUNGS: [(PlaybackRate, f64); 7] = [
    (PlaybackRate::R0_75, 0.75),
    (PlaybackRate::R1_0, 1.0),
    (PlaybackRate::R1_25, 1.25),
    (PlaybackRate::R1_5, 1.5),
    (PlaybackRate::R1_75, 1.75),
    (PlaybackRate::R2_0, 2.0),
    (PlaybackRate::R4_0, 4.0),
];

/// Transport states with their exact wire strings.
const TRANSPORT_STATES: [(TransportState, &str); 5] = [
    (TransportState::Playing, "playing"),
    (TransportState::Paused, "paused"),
    (TransportState::Stopped, "stopped"),
    (TransportState::Completed, "completed"),
    (TransportState::Interrupted, "interrupted"),
];

/// Focus outcomes with their exact wire strings — three outcomes; `regained`
/// is a separate direction event, not a fourth outcome classification.
const FOCUS_OUTCOMES: [(FocusOutcome, &str); 3] = [
    (FocusOutcome::DuckableTransient, "duckable_transient"),
    (FocusOutcome::SilenceDemanding, "silence_demanding"),
    (FocusOutcome::PermanentLoss, "permanent_loss"),
];

/// Engine error kinds with their exact wire strings.
const ENGINE_ERRORS: [(EngineError, &str); 7] = [
    (EngineError::Unreachable, "engine_unreachable"),
    (EngineError::InitFailed, "engine_init_failed"),
    (EngineError::RateMismatch, "rate_mismatch"),
    (EngineError::ChannelMismatch, "channel_mismatch"),
    (EngineError::DeviceRefused, "device_refused"),
    (EngineError::DeviceLost, "device_lost"),
    (EngineError::StartRefused, "start_refused"),
];

/// Session-end reasons with their exact wire strings — the closed-4 shared
/// with the diagnostic record (FR-044); `playing` is never among them.
const SESSION_ENDS: [(SessionEnd, &str); 4] = [
    (SessionEnd::Paused, "paused"),
    (SessionEnd::Stopped, "stopped"),
    (SessionEnd::Completed, "completed"),
    (SessionEnd::Interrupted, "interrupted"),
];

/// Engine tiers with their exact wire strings, in ascending fallback order.
const ENGINE_TIERS: [(EngineTier, &str); 5] = [
    (EngineTier::OfflineDefault, "offline_default"),
    (EngineTier::CloudDefault, "cloud_default"),
    (EngineTier::CloudEscalation, "cloud_escalation"),
    (EngineTier::TestDouble, "test_double"),
    (EngineTier::EvaluatedFallback, "evaluated_fallback"),
];

// Wildcard-free exhaustive matches. Each compiles only when its enum has
// exactly the named variants — no `Other`/`Unknown`/`_ =>` arm can be added
// without breaking this test target (Principle VII, FR-044).

fn rate_wire_of(rate: PlaybackRate) -> f64 {
    match rate {
        PlaybackRate::R0_75 => 0.75,
        PlaybackRate::R1_0 => 1.0,
        PlaybackRate::R1_25 => 1.25,
        PlaybackRate::R1_5 => 1.5,
        PlaybackRate::R1_75 => 1.75,
        PlaybackRate::R2_0 => 2.0,
        PlaybackRate::R4_0 => 4.0,
    }
}

fn transport_wire_of(state: TransportState) -> &'static str {
    match state {
        TransportState::Playing => "playing",
        TransportState::Paused => "paused",
        TransportState::Stopped => "stopped",
        TransportState::Completed => "completed",
        TransportState::Interrupted => "interrupted",
    }
}

fn focus_wire_of(outcome: FocusOutcome) -> &'static str {
    match outcome {
        FocusOutcome::DuckableTransient => "duckable_transient",
        FocusOutcome::SilenceDemanding => "silence_demanding",
        FocusOutcome::PermanentLoss => "permanent_loss",
    }
}

fn focus_regained_wire_of(event: FocusRegained) -> &'static str {
    match event {
        FocusRegained::Regained => "regained",
    }
}

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

fn session_end_wire_of(end: SessionEnd) -> &'static str {
    match end {
        SessionEnd::Paused => "paused",
        SessionEnd::Stopped => "stopped",
        SessionEnd::Completed => "completed",
        SessionEnd::Interrupted => "interrupted",
    }
}

fn engine_tier_wire_of(tier: EngineTier) -> &'static str {
    match tier {
        EngineTier::OfflineDefault => "offline_default",
        EngineTier::CloudDefault => "cloud_default",
        EngineTier::CloudEscalation => "cloud_escalation",
        EngineTier::TestDouble => "test_double",
        EngineTier::EvaluatedFallback => "evaluated_fallback",
    }
}

// Strategies that generate exactly the closed sets — every case is drawn from
// the enumerated variants, so no arbitrary value can enter a test.

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

fn transport_strategy() -> impl Strategy<Value = TransportState> {
    prop_oneof![
        Just(TransportState::Playing),
        Just(TransportState::Paused),
        Just(TransportState::Stopped),
        Just(TransportState::Completed),
        Just(TransportState::Interrupted),
    ]
}

fn focus_strategy() -> impl Strategy<Value = FocusOutcome> {
    prop_oneof![
        Just(FocusOutcome::DuckableTransient),
        Just(FocusOutcome::SilenceDemanding),
        Just(FocusOutcome::PermanentLoss),
    ]
}

fn focus_regained_strategy() -> impl Strategy<Value = FocusRegained> {
    prop_oneof![Just(FocusRegained::Regained),]
}

fn engine_error_strategy() -> impl Strategy<Value = EngineError> {
    prop_oneof![
        Just(EngineError::Unreachable),
        Just(EngineError::InitFailed),
        Just(EngineError::RateMismatch),
        Just(EngineError::ChannelMismatch),
        Just(EngineError::DeviceRefused),
        Just(EngineError::DeviceLost),
        Just(EngineError::StartRefused),
    ]
}

fn session_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![
        Just(SessionEnd::Paused),
        Just(SessionEnd::Stopped),
        Just(SessionEnd::Completed),
        Just(SessionEnd::Interrupted),
    ]
}

fn engine_tier_strategy() -> impl Strategy<Value = EngineTier> {
    prop_oneof![
        Just(EngineTier::OfflineDefault),
        Just(EngineTier::CloudDefault),
        Just(EngineTier::CloudEscalation),
        Just(EngineTier::TestDouble),
        Just(EngineTier::EvaluatedFallback),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rate_ladder_round_trips_every_declared_rung(rate in rate_strategy()) {
        // Invariant: the rate rungs are exactly `[0.75, 1.0, 1.25, 1.5,
        // 1.75, 2.0, 4.0]` and serde round-trips every variant (FR-044).
        let wire = serde_json::to_value(&rate).ok();
        prop_assert!(wire.is_some(), "PlaybackRate failed to serialize: {rate:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(rate_wire_of(rate)),
            "wire form of {:?} is not its declared rung number",
            rate
        );
        let round_trip = serde_json::from_value::<PlaybackRate>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(rate),
            "serde round-trip failed for rate rung {:?}",
            rate
        );
    }

    #[test]
    fn transport_states_round_trip_every_declared_variant(state in transport_strategy()) {
        // Invariant: transport is exactly `Playing|Paused|Stopped|Completed|
        // Interrupted` and serde round-trips every variant (FR-020, FR-044).
        let wire = serde_json::to_value(&state).ok();
        prop_assert!(wire.is_some(), "TransportState failed to serialize: {state:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(transport_wire_of(state)),
            "wire form of {:?} is not its declared wire string",
            state
        );
        let round_trip = serde_json::from_value::<TransportState>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(state),
            "serde round-trip failed for transport state {:?}",
            state
        );
    }

    #[test]
    fn focus_outcomes_round_trip_every_declared_variant(outcome in focus_strategy()) {
        // Invariant: focus outcomes are exactly `DuckableTransient|
        // SilenceDemanding|PermanentLoss` and serde round-trips every variant.
        let wire = serde_json::to_value(&outcome).ok();
        prop_assert!(wire.is_some(), "FocusOutcome failed to serialize: {outcome:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(focus_wire_of(outcome)),
            "wire form of {:?} is not its declared wire string",
            outcome
        );
        let round_trip = serde_json::from_value::<FocusOutcome>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(outcome),
            "serde round-trip failed for focus outcome {:?}",
            outcome
        );
    }

    #[test]
    fn the_regained_direction_event_round_trips(event in focus_regained_strategy()) {
        // Invariant: `regained` is a direction event, not a fourth outcome
        // classification (spec.md, session 2026-10-02), and it round-trips
        // to the wire string the diagnostic record carries.
        let wire = serde_json::to_value(&event).ok();
        prop_assert!(wire.is_some(), "FocusRegained failed to serialize: {event:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(focus_regained_wire_of(event)),
            "wire form of the regained direction event is not \"regained\""
        );
        let round_trip = serde_json::from_value::<FocusRegained>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(event),
            "serde round-trip failed for the regained direction event {:?}",
            event
        );
    }

    #[test]
    fn engine_errors_round_trip_every_declared_variant(error in engine_error_strategy()) {
        // Invariant: `EngineError` is exactly `Unreachable|InitFailed|
        // RateMismatch|ChannelMismatch|DeviceRefused|DeviceLost|StartRefused`
        // and serde round-trips every variant (FR-023, FR-035, FR-037).
        let wire = serde_json::to_value(&error).ok();
        prop_assert!(wire.is_some(), "EngineError failed to serialize: {error:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(engine_error_wire_of(error)),
            "wire form of {:?} is not its declared wire string",
            error
        );
        let round_trip = serde_json::from_value::<EngineError>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(error),
            "serde round-trip failed for engine error {:?}",
            error
        );
    }

    #[test]
    fn session_end_reasons_round_trip_every_declared_variant(end in session_end_strategy()) {
        // Invariant: session-end is exactly `Paused|Stopped|Completed|
        // Interrupted` — one closed set of four values shared with the record
        // (FR-044) — and serde round-trips every variant.
        let wire = serde_json::to_value(&end).ok();
        prop_assert!(wire.is_some(), "SessionEnd failed to serialize: {end:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(session_end_wire_of(end)),
            "wire form of {:?} is not its declared wire string",
            end
        );
        let round_trip = serde_json::from_value::<SessionEnd>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(end),
            "serde round-trip failed for session-end reason {:?}",
            end
        );
    }

    #[test]
    fn engine_tiers_round_trip_every_declared_variant(tier in engine_tier_strategy()) {
        // Invariant: the engine tier is a closed-5 set in ascending fallback
        // order (FR-023c) and serde round-trips every variant.
        let wire = serde_json::to_value(&tier).ok();
        prop_assert!(wire.is_some(), "EngineTier failed to serialize: {tier:?}");
        let wire = wire.unwrap_or(Value::Null);
        prop_assert_eq!(
            wire,
            json!(engine_tier_wire_of(tier)),
            "wire form of {:?} is not its declared wire string",
            tier
        );
        let round_trip = serde_json::from_value::<EngineTier>(wire).ok();
        prop_assert_eq!(
            round_trip,
            Some(tier),
            "serde round-trip failed for engine tier {:?}",
            tier
        );
    }
}

#[test]
fn closed_sets_reject_other_and_unknown_wire_strings() {
    // Invariant: "no `Other/Unknown`" — an unknown wire string must not
    // deserialize into any closed enum, because there is no catch-all variant
    // for it to land in (Principle VII, FR-044).
    let unknowns = [
        "other",
        "unknown",
        "Other",
        "Unknown",
        "UNKNOWN",
        "",
        "unknown_variant",
    ];
    for word in unknowns {
        assert!(
            serde_json::from_value::<TransportState>(json!(word)).is_err(),
            "TransportState accepted the unknown wire string \"{word}\""
        );
        assert!(
            serde_json::from_value::<SessionEnd>(json!(word)).is_err(),
            "SessionEnd accepted the unknown wire string \"{word}\""
        );
        assert!(
            serde_json::from_value::<FocusOutcome>(json!(word)).is_err(),
            "FocusOutcome accepted the unknown wire string \"{word}\""
        );
        assert!(
            serde_json::from_value::<FocusRegained>(json!(word)).is_err(),
            "FocusRegained accepted the unknown wire string \"{word}\""
        );
        assert!(
            serde_json::from_value::<EngineError>(json!(word)).is_err(),
            "EngineError accepted the unknown wire string \"{word}\""
        );
        assert!(
            serde_json::from_value::<EngineTier>(json!(word)).is_err(),
            "EngineTier accepted the unknown wire string \"{word}\""
        );
    }

    // The rate ladder is closed at its seven rungs: nothing between or beyond
    // them deserializes (FR-019).
    let off_ladder = [0.5, 0.8, 0.9, 1.1, 3.0, 5.0, -1.0, 0.0];
    for off in off_ladder {
        assert!(
            serde_json::from_value::<PlaybackRate>(json!(off)).is_err(),
            "PlaybackRate accepted the off-ladder rung {off}"
        );
    }
}

#[test]
fn session_end_never_carries_the_playing_transport_string() {
    // FR-044: the session-end reason and the transport state are one closed
    // set of four values shared with the record — `playing` is a transport
    // state and is never a session-end reason.
    assert!(
        serde_json::from_value::<SessionEnd>(json!("playing")).is_err(),
        "SessionEnd accepted \"playing\", which is a transport state, not a session-end reason"
    );
    for (_, wire) in SESSION_ENDS {
        assert_ne!(wire, "playing");
    }
}

#[test]
fn every_declared_wire_string_is_produced_by_exactly_one_variant() {
    // Cross-check that the exhaustive-match tables above enumerate the declared
    // closed sets with no duplicates: the wire strings are pairwise distinct
    // within each set, which is only possible if each variant maps to its own
    // wire form (the compile-time exhaustive matches already prove the variant
    // sets are exactly as specified).
    fn distinct<T: Copy + PartialEq>(entries: &[(T, &str)]) -> bool {
        for (i, (_, a)) in entries.iter().enumerate() {
            for (_, b) in entries.iter().skip(i + 1) {
                if a == b {
                    return false;
                }
            }
        }
        true
    }

    assert!(distinct(
        &RATE_RUNGS.iter().map(|(r, v)| (*r, "")).collect::<Vec<_>>()
    ));
    assert!(distinct(&TRANSPORT_STATES));
    assert!(distinct(&FOCUS_OUTCOMES));
    assert!(distinct(&ENGINE_ERRORS));
    assert!(distinct(&SESSION_ENDS));
    assert!(distinct(&ENGINE_TIERS));
}

#[test]
fn engine_tier_order_is_the_declared_ascending_fallback_sequence() {
    // FR-023c / contracts/engine-handoff.md: the fallback order is ascending —
    // offline_default < cloud_default < cloud_escalation < test_double <
    // evaluated_fallback — so the Edge endpoint stays last and is never a
    // default. `ORDER` is the single declared sequence; a second list would
    // drift (FR-044).
    assert_eq!(
        EngineTier::ORDER,
        [
            EngineTier::OfflineDefault,
            EngineTier::CloudDefault,
            EngineTier::CloudEscalation,
            EngineTier::TestDouble,
            EngineTier::EvaluatedFallback,
        ]
    );

    // Ascending order implies no tier repeats in the sequence.
    let order = EngineTier::ORDER;
    assert_eq!(
        order.len(),
        5,
        "EngineTier::ORDER does not hold exactly the five tiers"
    );
    for (i, earlier) in order.iter().enumerate() {
        for later in order.iter().skip(i + 1) {
            assert_ne!(
                earlier, later,
                "EngineTier::ORDER repeats a tier, so the fallback order is not a sequence"
            );
        }
    }
}
