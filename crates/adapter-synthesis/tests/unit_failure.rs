//! T5 / T020 — unit tests for the failure-mapping, SC-051 mechanism
//! and adapter-registration invariants of the FR-024 synthesis
//! adapter (`crates/adapter-synthesis`).
//!
//! Written against the frozen interface *before* the implementation
//! exists (implementation protocol rule 1: tests BLOCK the core
//! wave). These tests bind to the contract, not to the current
//! stub: this test target does not compile until T021 lands the
//! contract exactly as frozen — that unresolved-import red state is
//! the required pre-implementation position.
//!
//! Spec anchors: FR-023a (named failure with a bounded 3 s offer),
//! FR-035 (any format deviation surfaces a named `EngineError`),
//! FR-037 (the notification carries the failure reason), FR-041 (a
//! stated reason expires after 3 s), SC-035 (never a silent
//! resample, never a silent failure), SC-051 (the engine-failure
//! mechanism preserved continuously as a selector invariant), R7
//! (failure is a named member of the 7-member `EngineError` closed
//! set, condition→variant mapped, no `AdapterFailed`), R11
//! (`EngineTier::OfflineDefault` is the sole registered tier at v1;
//! `test_double` is not selectable while this adapter is
//! registered).
//!
//! Checklist anchors: mechanism-gate CHK003 (the 7 `EngineError`
//! variants with the condition→variant table, `DeviceLost` row
//! present, retired names absent), CHK004 (SC-035: named failure,
//! playback never starts on deviation), CHK015 (the failure
//! mechanism is consistent with the `(none, none)` declaration —
//! resume re-anchors to the segment boundary, never adapter
//! state), CHK030 (Rust rigor: no `unwrap`/`expect`/`panic`),
//! CHK031 (closed-set enforcement, no `Other`/`Unknown`/catch-all);
//! adapter-interface CHK005 (all seven failure variants mapped),
//! CHK011 (`offline_default` placement consistent with
//! `EngineTier::ORDER`), CHK024 (dependency failure maps to a named
//! error), CHK025 (mono f32 @ 22050 Hz as the validated shape);
//! audio-safety CHK023 (device refusal at open → `DeviceRefused`;
//! `BECOMING_NOISY` or `onAudioDevicesRemoved` → `DeviceLost`),
//! CHK034 (engine dependency failure → `InitFailed`); segment-resume
//! CHK030 (`EngineFailure::session_position` is an echo of the
//! delivered `DocumentSegment` boundary, never adapter-computed —
//! the player is the sole playback authority,
//! `constitution.md:486-490`).
//!
//! Contract anchors: API_CONTRACT.md §3 (`failure.rs` — the
//! condition→variant table, `EngineFailure::report`,
//! `MAX_OFFER_MS`, `MAX_CONDITION_DETAIL_CHARS`), §4 (`reg.rs` —
//! registration and selectability, the SC-051 selector invariant),
//! §7 (`synth.rs` — the pipeline wrapper that fails closed).
//!
//! Rust rigor (`constitution.md:460-475`): `unwrap` / `expect` /
//! `panic!` / `todo!` / `unimplemented!` / `dbg!` / `println!` /
//! `eprintln!` are denied — including in tests — and no
//! `#[allow]` suppression appears anywhere in this file. The
//! single `#[expect]` is the data-model E2 duration derivation
//! (`buffer.len() as f64`) in §7 — the identical, sanctioned
//! computation the delivery-unit tests freeze
//! (`tests/audio_chunk.rs`; API_CONTRACT.md §0.1 entry 4).
//! Every closed-set enum below is consumed by a wildcard-free
//! exhaustive `match` (no `_ =>`): an added or removed variant
//! breaks this test target's build instead of passing silently
//! (Principle VII; `constitution.md:314-322`; constitution gate 7;
//! mechanism-gate CHK031).

use adapter_synthesis::chunk::{
    BoundaryKind, DocumentSegment, SourceRange, split_document, split_document_within,
};
use adapter_synthesis::failure::{
    EngineError, EngineFailure, FailureCondition, FailureReportError, MAX_CONDITION_DETAIL_CHARS,
    MAX_OFFER_MS,
};
use adapter_synthesis::format_check::{
    DECLARED_CHANNELS, DECLARED_RATE_HZ, DECLARED_WIDTH_BITS, check_delivery,
};
use adapter_synthesis::reg::{REGISTERED_TIERS, is_selectable, registered_tiers};
use adapter_synthesis::synth::synthesize;
use engine_core::closed::EngineTier;

// ---------------------------------------------------------------------------
// Wildcard-free exhaustive matches over the closed sets.
//
// Each helper compiles only while its enum has exactly the named
// variants — no `_ =>` arm exists, so a variant added to (or
// removed from) a closed set fails this test target's build. This
// is the structural mechanism that keeps the closed sets closed
// (Principle VII; mechanism-gate CHK031), and it is the same
// exhaustiveness pattern as `engine-core`'s `prop_closed_sets.rs`.
// ---------------------------------------------------------------------------

/// The closed condition set as ordinals 1..=10. The 10-arm match
/// has no `_ =>`: an eleventh `FailureCondition` member fails to
/// compile here.
fn condition_ordinal(condition: FailureCondition) -> u8 {
    match condition {
        FailureCondition::RateDeviation => 1,
        FailureCondition::WidthDeviation => 2,
        FailureCondition::PitchToleranceViolation => 3,
        FailureCondition::ChannelDeviation => 4,
        FailureCondition::VoiceInitFailure => 5,
        FailureCondition::ProcessDeath => 6,
        FailureCondition::TransportUnreachable => 7,
        FailureCondition::DeviceRefusedAtOpen => 8,
        FailureCondition::DeviceRemoved => 9,
        FailureCondition::StartRefusedAtStart => 10,
    }
}

/// The closed `EngineError` set as ordinals 1..=7. The 7-arm match
/// has no `_ =>`: an eighth member — including the `AdapterFailed`
/// member retired by clarification C4-A — fails to compile here, so
/// the forbidden names `format_mismatch`, `engine_crash`,
/// `refusal` and `deferred_grant` have no variant to be spelled
/// as and can never be reported.
fn engine_error_ordinal(variant: EngineError) -> u8 {
    match variant {
        EngineError::Unreachable => 1,
        EngineError::InitFailed => 2,
        EngineError::RateMismatch => 3,
        EngineError::ChannelMismatch => 4,
        EngineError::DeviceRefused => 5,
        EngineError::DeviceLost => 6,
        EngineError::StartRefused => 7,
    }
}

/// The closed tier set as its declared wire name. The 5-arm match
/// has no `_ =>`: a sixth `EngineTier` member fails to compile
/// here.
fn engine_tier_wire_of(tier: EngineTier) -> &'static str {
    match tier {
        EngineTier::OfflineDefault => "offline_default",
        EngineTier::CloudDefault => "cloud_default",
        EngineTier::CloudEscalation => "cloud_escalation",
        EngineTier::TestDouble => "test_double",
        EngineTier::EvaluatedFallback => "evaluated_fallback",
    }
}

/// The closed report-refusal set as ordinals 1..=2. The 2-arm
/// match has no `_ =>`: a third `FailureReportError` member fails
/// to compile here.
fn failure_report_error_marker(error: FailureReportError) -> u8 {
    match error {
        FailureReportError::DetailTooLong { .. } => 1,
        FailureReportError::OfferTooLong { .. } => 2,
    }
}

// ---------------------------------------------------------------------------
// 1. The condition→variant table, member by member (spec.md:86,
//    API_CONTRACT §3).
// ---------------------------------------------------------------------------

#[test]
fn every_failure_condition_reports_its_frozen_contract_variant() {
    // spec.md:86 / API_CONTRACT §3 — the frozen condition→variant
    // table. The match is exhaustive over the closed condition set
    // (no `_ =>`): an eleventh member fails to compile here, so the
    // table can never silently skip a condition (Principle VII;
    // gate 7; mechanism-gate CHK003/CHK031). Each arm names its
    // condition so a failure is diagnosable without unwrap/expect.
    for condition in FailureCondition::ALL {
        let variant = condition.variant();
        match condition {
            FailureCondition::RateDeviation => assert_eq!(
                variant,
                EngineError::RateMismatch,
                "RateDeviation -> RateMismatch (delivered rate != 22050 Hz)"
            ),
            FailureCondition::WidthDeviation => assert_eq!(
                variant,
                EngineError::RateMismatch,
                "WidthDeviation -> RateMismatch (width is rate-shape per Principle VII)"
            ),
            FailureCondition::PitchToleranceViolation => assert_eq!(
                variant,
                EngineError::RateMismatch,
                "PitchToleranceViolation -> RateMismatch (pitch tolerance +/-3 cents @ 2.0x)"
            ),
            FailureCondition::ChannelDeviation => assert_eq!(
                variant,
                EngineError::ChannelMismatch,
                "ChannelDeviation -> ChannelMismatch (delivered channels != 1)"
            ),
            FailureCondition::VoiceInitFailure => assert_eq!(
                variant,
                EngineError::InitFailed,
                "VoiceInitFailure -> InitFailed (voice model / loader / init corruption)"
            ),
            FailureCondition::ProcessDeath => assert_eq!(
                variant,
                EngineError::Unreachable,
                "ProcessDeath -> Unreachable (killed / OOM)"
            ),
            FailureCondition::TransportUnreachable => assert_eq!(
                variant,
                EngineError::Unreachable,
                "TransportUnreachable -> Unreachable (transport unreachable)"
            ),
            FailureCondition::DeviceRefusedAtOpen => assert_eq!(
                variant,
                EngineError::DeviceRefused,
                "DeviceRefusedAtOpen -> DeviceRefused (AAudio rate/format refusal at open)"
            ),
            FailureCondition::DeviceRemoved => assert_eq!(
                variant,
                EngineError::DeviceLost,
                "DeviceRemoved -> DeviceLost (BECOMING_NOISY or onAudioDevicesRemoved)"
            ),
            FailureCondition::StartRefusedAtStart => assert_eq!(
                variant,
                EngineError::StartRefused,
                "StartRefusedAtStart -> StartRefused (ForegroundServiceStartNotAllowed; consent missing at start)"
            ),
        }
    }
    assert_eq!(
        FailureCondition::ALL.len(),
        10,
        "FailureCondition::ALL must hold exactly the ten contract conditions"
    );
}

#[test]
fn failure_condition_all_is_exactly_the_closed_ten_member_set() {
    // `ALL` has exactly 10 members, and every member of the closed
    // set appears in it exactly once. The exhaustive 10-arm match in
    // `condition_ordinal` (no `_ =>`) fixes the set's cardinality
    // at compile time; the ordinal coverage below proves `ALL` is a
    // permutation of the whole set — no member missing, none
    // duplicated (Principle VII; mechanism-gate CHK031).
    assert_eq!(
        FailureCondition::ALL.len(),
        10,
        "FailureCondition::ALL must hold exactly the ten contract conditions"
    );
    let mut seen = [false; 10];
    for condition in FailureCondition::ALL {
        let ordinal = usize::from(condition_ordinal(condition) - 1);
        assert!(
            !seen[ordinal],
            "FailureCondition::ALL repeats the member with ordinal {}",
            ordinal + 1
        );
        seen[ordinal] = true;
    }
    for (index, member_present) in seen.iter().enumerate() {
        assert!(
            member_present,
            "closed-set member #{} is absent from FailureCondition::ALL",
            index + 1
        );
    }
}

// ---------------------------------------------------------------------------
// 2. The forbidden names are absent (clarification batch C4-A).
// ---------------------------------------------------------------------------

#[test]
fn engine_error_is_closed_at_seven_members_with_no_adapter_failed() {
    // The exhaustive 7-arm match in `engine_error_ordinal` compiles
    // only while `EngineError` is exactly the seven contract
    // members — `Unreachable | InitFailed | RateMismatch |
    // ChannelMismatch | DeviceRefused | DeviceLost | StartRefused`
    // (`crates/engine-core/src/closed.rs:235-270`). That
    // compilation is the assertion: the `AdapterFailed` member
    // retired by clarification C4-A cannot exist (an eighth member
    // would fail this test target's build), and the retired wire
    // names `format_mismatch`, `engine_crash`, `refusal` and
    // `deferred_grant` have no variant to be spelled as. Bad
    // Markdown is a document-layer concern, not a member (R8).
    let members = [
        EngineError::Unreachable,
        EngineError::InitFailed,
        EngineError::RateMismatch,
        EngineError::ChannelMismatch,
        EngineError::DeviceRefused,
        EngineError::DeviceLost,
        EngineError::StartRefused,
    ];
    assert_eq!(members.len(), 7, "the closed set is exactly seven members");
    let mut seen = [false; 7];
    for member in members {
        let ordinal = usize::from(engine_error_ordinal(member) - 1);
        assert!(
            !seen[ordinal],
            "the closed set repeats the member with ordinal {}",
            ordinal + 1
        );
        seen[ordinal] = true;
    }
    for (index, member_present) in seen.iter().enumerate() {
        assert!(
            member_present,
            "closed-set member #{} is absent from the enumerated set",
            index + 1
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Fail-closed reporting (API_CONTRACT §3; FR-023a / FR-032c /
//    FR-041; diagnostic record rule).
// ---------------------------------------------------------------------------

#[test]
fn the_offer_and_detail_bounds_are_the_frozen_contract_constants() {
    // The failure offer must expire within 3 s (FR-023a / FR-032c /
    // FR-041), and the diagnostic record carries at most 200
    // identifier-pattern chars (AGENTS.md diagnostic-record rule).
    assert_eq!(
        MAX_OFFER_MS, 3_000,
        "the failure offer must expire within 3 s (FR-023a / FR-032c / FR-041)"
    );
    assert_eq!(
        MAX_CONDITION_DETAIL_CHARS, 200,
        "the diagnostic record allows at most 200 identifier-pattern chars"
    );
}

#[test]
fn report_accepts_exactly_the_contract_bounds() {
    // An `observed` string of exactly `MAX_CONDITION_DETAIL_CHARS`
    // chars succeeds, and an offer of exactly `MAX_OFFER_MS`
    // succeeds — the bounds are inclusive (API_CONTRACT §3).
    let position = SourceRange::whole_lines(1, 1);
    let observed_exact = "d".repeat(MAX_CONDITION_DETAIL_CHARS);
    let result = EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        &observed_exact,
        position,
        MAX_OFFER_MS,
    );
    assert_eq!(
        result,
        Ok(EngineFailure {
            variant: EngineError::InitFailed,
            condition: FailureCondition::VoiceInitFailure,
            observed: observed_exact,
            session_position: position,
            offer_expires_in_ms: MAX_OFFER_MS,
            tier_at_failure: EngineTier::OfflineDefault,
        }),
        "report must accept an observed string of exactly MAX_CONDITION_DETAIL_CHARS chars with an offer of exactly MAX_OFFER_MS"
    );
}

#[test]
fn report_refuses_an_observed_string_over_the_detail_bound() {
    // Fail closed (API_CONTRACT §3): one char over
    // `MAX_CONDITION_DETAIL_CHARS` is refused with
    // `DetailTooLong`, never truncated — the diagnostic record's
    // identifier-pattern bound is a hard bound, and a truncated
    // identifier would be a silent misreport (R7; SC-035).
    let observed_too_long = "d".repeat(MAX_CONDITION_DETAIL_CHARS + 1);
    let result = EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        &observed_too_long,
        SourceRange::whole_lines(1, 1),
        MAX_OFFER_MS,
    );
    assert_eq!(
        result,
        Err(FailureReportError::DetailTooLong {
            len: MAX_CONDITION_DETAIL_CHARS + 1,
            max: MAX_CONDITION_DETAIL_CHARS,
        }),
        "report must fail closed on an observed string over MAX_CONDITION_DETAIL_CHARS"
    );
}

#[test]
fn report_refuses_an_offer_over_the_three_second_bound() {
    // Fail closed (FR-023a / FR-032c / FR-041): an offer one ms
    // over `MAX_OFFER_MS` is refused with `OfferTooLong` — a
    // failure whose offer cannot expire within 3 s must not be
    // reported at all, because the stated reason would outlive its
    // presentation bound.
    let result = EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        "voice_init_failure",
        SourceRange::whole_lines(1, 1),
        MAX_OFFER_MS + 1,
    );
    assert_eq!(
        result,
        Err(FailureReportError::OfferTooLong {
            offer_ms: MAX_OFFER_MS + 1,
            max_ms: MAX_OFFER_MS,
        }),
        "report must fail closed on an offer over MAX_OFFER_MS"
    );
}

#[test]
fn failure_report_error_is_closed_at_two_refusal_kinds() {
    // The exhaustive 2-arm match in `failure_report_error_marker`
    // (no `_ =>`) compiles only while `FailureReportError` is
    // exactly `DetailTooLong | OfferTooLong` — a third refusal kind
    // would fail this test target's build (Principle VII; gate 7;
    // mechanism-gate CHK031).
    let detail_too_long = FailureReportError::DetailTooLong {
        len: MAX_CONDITION_DETAIL_CHARS + 1,
        max: MAX_CONDITION_DETAIL_CHARS,
    };
    let offer_too_long = FailureReportError::OfferTooLong {
        offer_ms: MAX_OFFER_MS + 1,
        max_ms: MAX_OFFER_MS,
    };
    assert_eq!(failure_report_error_marker(detail_too_long), 1);
    assert_eq!(failure_report_error_marker(offer_too_long), 2);
    assert_ne!(
        failure_report_error_marker(detail_too_long),
        failure_report_error_marker(offer_too_long),
        "the two report-refusal kinds must be distinct"
    );
}

// ---------------------------------------------------------------------------
// 4. `tier_at_failure` is always `EngineTier::OfflineDefault`
//    (R11; the sole registered tier at v1; SC-044 deferred).
// ---------------------------------------------------------------------------

#[test]
fn tier_at_failure_is_always_the_registered_offline_default() {
    // R11 / SC-051: the adapter is the sole registered tier at v1,
    // so every failure it reports — whatever the condition —
    // carries `EngineTier::OfflineDefault`. No second real engine
    // exists to register (SC-044 is deferred), and the
    // `test_double` stand-ins are fixtures, never a failure's
    // tier.
    for condition in FailureCondition::ALL {
        let result = EngineFailure::report(
            condition,
            condition.detail(),
            SourceRange::whole_lines(1, 1),
            MAX_OFFER_MS,
        );
        match result {
            Ok(failure) => assert_eq!(
                failure.tier_at_failure,
                EngineTier::OfflineDefault,
                "a failure for condition #{:?} must carry the sole registered tier",
                condition_ordinal(condition)
            ),
            Err(error) => assert_eq!(
                failure_report_error_marker(error),
                0,
                "report must accept the identifier-pattern detail of every condition"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. `session_position` is an echo, never adapter-computed
//    (segment-resume CHK030; the player is the sole playback
//    authority, constitution.md:486-490).
// ---------------------------------------------------------------------------

#[test]
fn failure_session_position_echoes_the_segment_source_range() {
    // segment-resume CHK030: an `EngineFailure` reported at a
    // segment's `source_range` carries exactly that range. The
    // adapter reports the segment boundary it delivered — it never
    // invents a position; position/buffering/transport belong to
    // the player alone (constitution.md:486-490). The 40-byte
    // budget splits the fixture into two segments (a heading block
    // absorbing its body, then a paragraph), so the echo is checked
    // at more than one boundary.
    let source = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n";
    let segments = split_document_within(source, "document-hash", 40);
    assert_eq!(
        segments.len(),
        2,
        "the fixture splits into exactly two segments under the 40-byte budget"
    );
    for segment in segments {
        let position = segment.source_range;
        let result = EngineFailure::report(
            FailureCondition::VoiceInitFailure,
            "voice_init_failure",
            position,
            MAX_OFFER_MS,
        );
        match result {
            Ok(failure) => assert_eq!(
                failure.session_position, position,
                "session_position must echo the segment's source_range exactly (segment #{})",
                segment.segment_index
            ),
            Err(error) => assert_eq!(
                failure_report_error_marker(error),
                0,
                "report must accept a segment source_range as the session position"
            ),
        }
    }

    // The same echo holds under the default 5000-byte budget: the
    // whole document is one segment, and the failure still reports
    // that segment's boundary.
    let default_segments = split_document(source, "document-hash");
    assert!(
        !default_segments.is_empty(),
        "a non-empty document yields at least one segment"
    );
    for segment in default_segments {
        let position = segment.source_range;
        let result = EngineFailure::report(
            FailureCondition::DeviceRemoved,
            "device_removed",
            position,
            MAX_OFFER_MS,
        );
        match result {
            Ok(failure) => assert_eq!(
                failure.session_position, position,
                "session_position must echo the segment's source_range exactly (segment #{})",
                segment.segment_index
            ),
            Err(error) => assert_eq!(
                failure_report_error_marker(error),
                0,
                "report must accept a segment source_range as the session position"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// 6. The SC-051 mechanism: registration makes `offline_default`
//    the sole selectable tier (API_CONTRACT §4; R11; SC-051;
//    adapter-handoff §6).
// ---------------------------------------------------------------------------

#[test]
fn sc_051_registration_makes_offline_default_the_sole_selectable_tier() {
    // With this adapter registered, the registry holds exactly one
    // tier and the function and the constant agree (API_CONTRACT
    // §4; R11).
    assert_eq!(
        registered_tiers(),
        &[EngineTier::OfflineDefault],
        "the adapter registers in offline_default only"
    );
    assert_eq!(
        registered_tiers(),
        &REGISTERED_TIERS,
        "registered_tiers() and REGISTERED_TIERS must agree"
    );
    assert_eq!(
        REGISTERED_TIERS,
        [EngineTier::OfflineDefault],
        "REGISTERED_TIERS is exactly the offline_default tier"
    );

    // The selectability expectation is an exhaustive 5-arm match
    // over the closed tier set (no `_ =>`): a sixth tier member
    // fails to compile here, so the SC-051 expectation can never
    // silently skip a tier (Principle VII; gate 7; mechanism-gate
    // CHK031). Only `offline_default` is selectable: the
    // `test_double` stand-ins are fixtures and are never selectable
    // while a non-`test_double` adapter is registered
    // (001/spec.md:89; closed.rs:305-312), and the cloud and
    // evaluated tiers are unregistered until SC-044 lands a second
    // real engine.
    for tier in EngineTier::ORDER {
        let tier_wire = engine_tier_wire_of(tier);
        let expectation = tier_wire == "offline_default";
        assert_eq!(
            is_selectable(tier),
            expectation,
            "SC-051: is_selectable({tier_wire}) must be {expectation} while the adapter is the sole registered tier"
        );
    }

    // Named per-tier expectations (the stand-ins and the
    // unregistered tiers are all unselectable).
    assert!(
        is_selectable(EngineTier::OfflineDefault),
        "the registered offline_default tier is selectable"
    );
    assert!(
        !is_selectable(EngineTier::TestDouble),
        "test_double stand-ins are fixtures, never selectable while this adapter is registered (001/spec.md:89)"
    );
    assert!(
        !is_selectable(EngineTier::CloudDefault),
        "cloud_default is not registered at v1 (SC-044 deferred)"
    );
    assert!(
        !is_selectable(EngineTier::CloudEscalation),
        "cloud_escalation is not registered at v1 (SC-044 deferred)"
    );
    assert!(
        !is_selectable(EngineTier::EvaluatedFallback),
        "evaluated_fallback is not registered at v1 (SC-044 deferred)"
    );

    // The fallback chain is a single tier: iterating the declared
    // order and keeping the selectable tiers yields exactly
    // `[offline_default]`, so a mid-document active switch is
    // unavailable by construction (SC-051 preserved as an
    // invariant; active takeover deferred to SC-044).
    let selectable_tiers: Vec<EngineTier> = EngineTier::ORDER
        .iter()
        .copied()
        .filter(|tier| is_selectable(*tier))
        .collect();
    assert_eq!(
        selectable_tiers,
        vec![EngineTier::OfflineDefault],
        "the fallback chain is the single offline_default tier, so a mid-document active switch is unavailable by construction"
    );
}

// ---------------------------------------------------------------------------
// 7. Synthesis delivers real audio through the bound engine,
//    and a reachable failure still names a closed-set member
//    (API_CONTRACT §7; R2 / R7; SC-035; T044).
// ---------------------------------------------------------------------------

#[test]
fn synthesis_delivers_real_audio_and_a_reachable_failure_names_its_condition() {
    // T044 bound the real `offline_default` engine — `sherpa-onnx`
    // 1.13.8 driving the embedded Piper pt-BR `faber` voice — so a
    // well-formed segment synthesizes to a real, non-empty, mono-`f32`
    // buffer at the declared 22 050 Hz. The design-phase pin (unbound
    // model → unconditional `InitFailed`) does not survive binding: an
    // `Err` for a well-formed segment would now be the silent branch
    // this crate exists to prevent ("a silent branch is worse than a
    // failed build", constitution.md:314-322).
    let source = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n";
    let segments = split_document(source, "document-hash");
    assert!(
        !segments.is_empty(),
        "a non-empty document yields at least one segment"
    );
    for segment in &segments {
        match synthesize(segment) {
            Ok(buffer) => {
                assert!(
                    !buffer.is_empty(),
                    "the bound engine must deliver a non-empty buffer for a well-formed segment"
                );
                // The delivered buffer round-trips through the derived
                // duration (data-model E2): the delivery gate accepts
                // exactly the duration derived from the buffer's own
                // length, so the delivered audio conforms to the
                // declared mono-`f32`-at-22 050 Hz shape.
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "data-model E2 mandates deriving duration_ms as pcm_data.len() / rate_hz * 1000; f64 has no lossless From<usize>, so the usize -> f64 cast is unavoidable — the identical computation is frozen by the delivery-unit tests (tests/audio_chunk.rs:231-232, :516-518); sanctioned by API_CONTRACT.md §0.1 entry 4"
                )]
                let duration_ms = buffer.len() as f64 / f64::from(DECLARED_RATE_HZ) * 1000.0;
                assert_eq!(
                    check_delivery(
                        DECLARED_RATE_HZ,
                        DECLARED_CHANNELS,
                        DECLARED_WIDTH_BITS,
                        buffer.len(),
                        duration_ms
                    ),
                    Ok(()),
                    "the delivered buffer must conform to the declared mono-f32-at-22 050 Hz shape"
                );
            }
            Err(failure) => {
                // Negative assertion (R2 / R7 / SC-035): a well-formed
                // segment cannot fail through the bound engine. The
                // predicate is a runtime value, not a constant, so the
                // assertion always fails without tripping the
                // constant-assertion lint.
                let well_formed_segment_may_fail = failure.offer_expires_in_ms <= MAX_OFFER_MS;
                assert!(
                    !well_formed_segment_may_fail,
                    "a well-formed segment must synthesize through the bound engine, not fail closed: {failure:?}"
                );
            }
        }
    }

    // The fail-closed mechanism itself stays reachable: a segment
    // carrying an interior NUL cannot be marshalled across the C FFI
    // boundary, so synthesis refuses with the named `Unreachable`
    // (condition `TransportUnreachable`) — a closed-set member with a
    // bounded offer and the segment's echoed position — rather than
    // letting the dependency panic on `CString::new` (R2/R7; SC-035).
    let unreachable_segment = DocumentSegment {
        content: "Parágrafo com\0um NUL interior.".to_owned(),
        source_range: SourceRange::whole_lines(1, 1),
        boundary_kind: BoundaryKind::Paragraph,
        segment_index: 0,
        resume_point: Some(SourceRange::whole_lines(1, 1)),
    };
    let expected_position = match unreachable_segment.resume_point {
        Some(resume_point) => resume_point,
        None => unreachable_segment.source_range,
    };
    match synthesize(&unreachable_segment) {
        Ok(buffer) => {
            // Negative assertion (R2 / R7 / SC-035): `Ok` here —
            // with or without samples — is a defect. An interior NUL
            // must be refused before the FFI boundary; a success would
            // mean unmarshalable text crossed it. The predicate is a
            // runtime value, not a constant, so the assertion always
            // fails without tripping the constant-assertion lint.
            let silent_transport_success = buffer.len() < usize::MAX;
            assert!(
                !silent_transport_success,
                "an interior NUL must be refused with a named failure, not carried across the FFI boundary"
            );
        }
        Err(failure) => {
            assert_eq!(
                failure.variant,
                EngineError::Unreachable,
                "an interior NUL is a named Unreachable, never silence"
            );
            assert_eq!(
                failure.condition,
                FailureCondition::TransportUnreachable,
                "the named failure carries its observing condition"
            );
            assert!(
                failure.offer_expires_in_ms <= MAX_OFFER_MS,
                "the failure offer expires within MAX_OFFER_MS (FR-023a / FR-032c / FR-041)"
            );
            assert_eq!(
                failure.tier_at_failure,
                EngineTier::OfflineDefault,
                "the failure carries the sole registered tier (R11)"
            );
            assert_eq!(
                failure.session_position, expected_position,
                "session_position is the segment's resume_point, or its source_range when none is declared — echoed, never computed"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Every one of the 7 `EngineError` members is reachable through
//    the classification table, so no member of the closed set is
//    dead (R7; mechanism-gate CHK003).
// ---------------------------------------------------------------------------

#[test]
fn every_engine_error_member_is_reachable_through_the_classification_table() {
    // Each of the seven closed-set members is the image of at least
    // one observable condition, so no member is dead: a dead member
    // would be an unreachable arm in the taxonomy — a closed set
    // that cannot be exercised cannot be verified (R7;
    // mechanism-gate CHK003).
    let mut reachable = [false; 7];
    for condition in FailureCondition::ALL {
        let ordinal = usize::from(engine_error_ordinal(condition.variant()) - 1);
        reachable[ordinal] = true;
    }
    for (index, reached) in reachable.iter().enumerate() {
        assert!(
            reached,
            "EngineError member #{} is unreachable through the condition->variant table",
            index + 1
        );
    }
}
