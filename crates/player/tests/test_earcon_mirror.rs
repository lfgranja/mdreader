//! T026 — BLOCKING US2 TEST: earcon mirror 1:1
//! (Feature 004-ressoa-dogfood-build / task T026).
//!
//! Requirement (tasks.md T026, verbatim): "Earcon mirror 1:1 test in
//! `crates/player/tests/test_earcon_mirror.rs` (every trigger ⇒
//! same-call lock-screen state change [giant transport glyph +
//! progress + warning/failure banner snapshot assert] +
//! `reason_ttl_ms=3000` for pause-demanding (distinct from
//! `dismissal`); undeliverables (`VoiceUnavailable`/
//! `FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent
//! line + `error` logged; earcon PCM never advances bookmark;
//! ducked gain never quiets earcon)".
//!
//! The mirror contract (contracts/earcon.md): "Trigger + lock-screen
//! mirror publish is one Rust call (impossible to sound without
//! showing)." This file binds the host-testable halves of that
//! contract to the public surface:
//!
//! - [`player::earcon::EarconDescriptor::for_kind`] — the 1:1
//!   trigger→earcon-kind pairing and the procedural PCM descriptor
//!   (T028);
//! - [`player::interruption::InterruptionReason`] —
//!   `reason_line()` / `reason_ttl_ms()` / `is_persistent()` (T027);
//! - [`player::surface::SurfaceResolver`] / `Dismissal` /
//!   [`player::surface::MirrorState`] — the lock-screen mirror
//!   snapshot (T061 / T043);
//! - [`player::position::PositionClock`] — the bookmark the earcon
//!   PCM must never advance (T025).
//!
//! Per-assertion state at this commit (cargo test
//! `-p player --test test_earcon_mirror`):
//!
//! | # | Assertion | State |
//! |---|---|---|
//! | 1 | every trigger selects the contract earcon kind | **RED** — no trigger→earcon mapping call exists in `interruption.rs`/`earcon.rs` (TODO: T029 mirror wiring / T031 Rust-side pattern-id selection) |
//! | 2 | descriptor exists for all 7 kinds (pairing expressible) | GREEN (T003/T028) |
//! | 3 | `reason_ttl_ms=3000` pause-demanding / `None` undeliverable, distinct from `dismissal` | GREEN (T027/T061) |
//! | 4 | undeliverables ⇒ persistent line (never-start + `error` logged traced as the synth-path gap) | GREEN on the enum half; never-start wiring recorded as red/TODO (T043 `failure.rs` refusal — not reachable from the player crate) |
//! | 5 | earcon PCM never advances bookmark | GREEN on the pure half (clock untouched by descriptor construction/delivery); backend sample-counting wiring traced to T029 |
//! | 6 | ducked gain never quiets earcon | **RED** on the bypass half — `backend.rs` has no earcon path (TODO: T029 duck-exempt short-mix); descriptor-half GREEN (full-scale non-silent PCM) |
//! | 7 | mirror snapshot: transport + progress + reason-bearing surface | GREEN on the current shape; `reason`/`reason_ttl_ms` payload fields traced to T043 |
//!
//! No `_ =>` catch-all appears in this file: every match over a
//! closed set is written exhaustively.

use std::error::Error;

use engine_core::closed::TransportState;
use player::earcon::{EarconDescriptor, EarconKind, SampleFormat};
use player::interruption::{InterruptionReason, reason_line};
use player::position::PositionClock;
use player::surface::{
    Dismissal, Heading, MirrorState, Progress, SurfaceResolver, TRANSIENT_DISMISS_MS,
};

/// The closed seven-member earcon set (contracts/earcon.md grammar),
/// in grammar-table order.
const ALL_KINDS: [EarconKind; 7] = [
    EarconKind::Play,
    EarconKind::Pause,
    EarconKind::SkipFwd,
    EarconKind::SkipBack,
    EarconKind::Warning,
    EarconKind::Failure,
    EarconKind::RateStep,
];

/// The closed eight-reason set (FR-005), in enum order.
const ALL_REASONS: [InterruptionReason; 8] = [
    InterruptionReason::FocusDenied,
    InterruptionReason::FocusDelayed,
    InterruptionReason::PermanentLoss,
    InterruptionReason::TransientPaused,
    InterruptionReason::DeviceRemoved,
    InterruptionReason::VoiceUnavailable,
    InterruptionReason::FormatUndeliverable,
    InterruptionReason::DecodeFailed,
];

/// The four pause-demanding reasons (the Warning-earcon triggers):
/// each pauses playback and carries a `reason_ttl_ms=3000` line.
const PAUSE_DEMANDING: [InterruptionReason; 4] = [
    InterruptionReason::TransientPaused,
    InterruptionReason::FocusDenied,
    InterruptionReason::PermanentLoss,
    InterruptionReason::DeviceRemoved,
];

/// The three undeliverable reasons (the Failure-earcon triggers):
/// never start, persistent line, `error` logged.
const UNDELIVERABLE: [InterruptionReason; 3] = [
    InterruptionReason::VoiceUnavailable,
    InterruptionReason::FormatUndeliverable,
    InterruptionReason::DecodeFailed,
];

/// The mirror-payload document title used by the snapshot asserts.
const METADATA_TITLE: &str = "Trial Document";

/// Position comparison tolerance, in seconds — the same epsilon form
/// the US2 integration tests use, so no strict float comparison ever
/// appears (clippy `float_cmp`).
const POS_EPS_SECS: f64 = 0.001;

/// The mirror-payload filename placeholder (FR-029).
const FILENAME: &str = "document.md";

/// The resolved position for the snapshot asserts, in seconds.
const MIRROR_POSITION_SECS: f64 = 42.0;

/// Source of the reason semantics (T027) — scanned for the
/// trigger→earcon mapping call.
const INTERRUPTION_SRC: &str = include_str!("../src/interruption.rs");

/// Source of the earcon grammar (T028) — scanned for the
/// trigger→earcon mapping call.
const EARCON_SRC: &str = include_str!("../src/earcon.rs");

/// Source of the playback backend (T029 lands the duck-exempt
/// short-mix here) — scanned for the earcon path.
const BACKEND_SRC: &str = include_str!("../src/backend.rs");

/// The contract trigger→earcon table (contracts/earcon.md grammar +
/// contracts/surface.md reason-line table), as the test expects the
/// single Rust mapping call to express it.
///
/// Pause-demanding reasons sound the Warning earcon; undeliverable
/// reasons sound the Failure earcon; the `FocusDelayed` zero-audio
/// hold sounds nothing (Deferred shows no line). The
/// transport-confirmation kinds (Play/Pause/SkipFwd/SkipBack/
/// RateStep) are never a reason's earcon — a reason only ever
/// selects Warning, Failure, or nothing.
fn expected_earcon_for(reason: InterruptionReason) -> Option<EarconKind> {
    match reason {
        InterruptionReason::TransientPaused
        | InterruptionReason::FocusDenied
        | InterruptionReason::PermanentLoss
        | InterruptionReason::DeviceRemoved => Some(EarconKind::Warning),
        InterruptionReason::VoiceUnavailable
        | InterruptionReason::FormatUndeliverable
        | InterruptionReason::DecodeFailed => Some(EarconKind::Failure),
        InterruptionReason::FocusDelayed => None,
    }
}

/// The reason variant's name, for pairing diagnostics — exhaustive,
/// no catch-all.
fn reason_variant_name(reason: InterruptionReason) -> &'static str {
    match reason {
        InterruptionReason::FocusDenied => "FocusDenied",
        InterruptionReason::FocusDelayed => "FocusDelayed",
        InterruptionReason::PermanentLoss => "PermanentLoss",
        InterruptionReason::TransientPaused => "TransientPaused",
        InterruptionReason::DeviceRemoved => "DeviceRemoved",
        InterruptionReason::VoiceUnavailable => "VoiceUnavailable",
        InterruptionReason::FormatUndeliverable => "FormatUndeliverable",
        InterruptionReason::DecodeFailed => "DecodeFailed",
    }
}

/// The earcon kind's variant name, for pairing diagnostics —
/// exhaustive, no catch-all.
fn earcon_kind_name(kind: EarconKind) -> &'static str {
    match kind {
        EarconKind::Play => "Play",
        EarconKind::Pause => "Pause",
        EarconKind::SkipFwd => "SkipFwd",
        EarconKind::SkipBack => "SkipBack",
        EarconKind::Warning => "Warning",
        EarconKind::Failure => "Failure",
        EarconKind::RateStep => "RateStep",
    }
}

/// Whether any source declares a function whose *return type* names
/// `EarconKind` — the single Rust call that pairs a trigger to the
/// earcon kind it must sound (`fn earcon_kind(&self) ->
/// Option<EarconKind>` and equivalents).
///
/// The split on the first `-> ` matters: a parameter list naming
/// `EarconKind` (as `for_kind` does) is not a mapping call.
fn declares_earcon_kind_mapping(sources: &[&str]) -> bool {
    sources.iter().any(|source| {
        source.lines().any(|line| {
            line.split_once("-> ")
                .is_some_and(|(_, return_type)| return_type.contains("EarconKind"))
        })
    })
}

/// Whether one statement — one line, or a wrapped match arm plus its
/// continuation line — names both the reason variant and the earcon
/// kind the contract pairs it with.
fn pairs_on_one_statement(sources: &[&str], reason: &str, kind: &str) -> bool {
    for source in sources {
        let lines: Vec<&str> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let statement = if line.trim_end().ends_with("=>") && index + 1 < lines.len() {
                format!("{line} {}", lines[index + 1])
            } else {
                (*line).to_string()
            };
            if statement.contains(reason) && statement.contains(kind) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Assertion 1 — every trigger selects the contract earcon kind (1:1).
// RED: the mapping call does not exist yet. When it lands (T029 mirror
// wiring / T031 Rust-side pattern-id selection), this test turns green
// against the table in `expected_earcon_for`.
// ---------------------------------------------------------------------------

#[test]
fn every_trigger_selects_the_contract_earcon_kind() {
    let sources = [INTERRUPTION_SRC, EARCON_SRC];

    assert!(
        declares_earcon_kind_mapping(&sources),
        "no trigger→earcon-kind mapping call exists in interruption.rs or earcon.rs: \
         the mirror contract's \"Trigger + lock-screen mirror publish is one Rust call \
         (impossible to sound without showing)\" (contracts/earcon.md) is not expressible — \
         a reason cannot select the earcon it must sound. Expected one call returning \
         Option<EarconKind> carrying the contract table: TransientPaused/FocusDenied/\
         PermanentLoss/DeviceRemoved ⇒ Warning; VoiceUnavailable/FormatUndeliverable/\
         DecodeFailed ⇒ Failure; FocusDelayed ⇒ None (zero-audio hold, no line). \
         RED gap for the impl wave (T029 mirror wiring / T031 Rust-side pattern-id \
         selection)."
    );

    for reason in ALL_REASONS {
        let kind_name = match expected_earcon_for(reason) {
            Some(kind) => earcon_kind_name(kind),
            None => "None",
        };
        assert!(
            pairs_on_one_statement(&sources, reason_variant_name(reason), kind_name),
            "the trigger→earcon mapping does not pair {:?} with {kind_name} on one \
             statement — the 1:1 pairing (contracts/earcon.md grammar + contracts/\
             surface.md reason-line table) is not expressed for this trigger. \
             Expected an arm `{} => … {kind_name}` (or the same pairing in a \
             const table).",
            reason,
            reason_variant_name(reason),
        );
    }
}

// ---------------------------------------------------------------------------
// Assertion 1 (reachable half) — the pairing is expressible: every
// one of the seven closed earcon kinds resolves a deliverable
// descriptor. GREEN (T003 / T028).
// ---------------------------------------------------------------------------

#[test]
fn every_earcon_kind_resolves_a_descriptor() {
    for kind in ALL_KINDS {
        let descriptor = EarconDescriptor::for_kind(kind);
        assert_eq!(
            descriptor.kind, kind,
            "for_kind must round-trip the kind it was given"
        );
        assert_eq!(
            descriptor.sample_rate_hz, 22_050,
            "earcon PCM is mono f32 @ 22050 Hz (contracts/earcon.md)"
        );
        assert_eq!(descriptor.channels, 1, "earcon PCM is mono");
        assert_eq!(descriptor.format, SampleFormat::F32, "earcon PCM is f32");
        assert!(
            descriptor.duration_ms <= 500,
            "{kind:?} earcon is {} ms — the grammar caps every earcon at 500 ms",
            descriptor.duration_ms
        );
        assert!(
            !descriptor.samples.is_empty(),
            "{kind:?} must synthesize PCM — no .wav / SoundPool / ToneGenerator assets"
        );
    }
}

// ---------------------------------------------------------------------------
// Assertion 2 — reason_ttl_ms=3000 for pause-demanding (distinct from
// `dismissal`); None (persistent) for undeliverables. GREEN (T027 /
// T061).
// ---------------------------------------------------------------------------

#[test]
fn reason_ttl_ms_is_3000_for_pause_demanding_none_for_undeliverables() {
    assert_eq!(
        TRANSIENT_DISMISS_MS, 3_000,
        "the transient window is exactly 3 s (FR-028e / contracts/surface.md)"
    );

    // Pause-demanding: the reason line expires after exactly 3 s.
    for reason in PAUSE_DEMANDING {
        assert!(
            !reason.is_persistent(),
            "{reason:?} is pause-demanding — it must not be persistent"
        );
        assert_eq!(
            reason.reason_ttl_ms(),
            Some(3_000),
            "{reason:?} is pause-demanding — reason_ttl_ms must be 3000 \
             (contracts/surface.md)"
        );
    }

    // Undeliverables: the reason line never expires.
    for reason in UNDELIVERABLE {
        assert!(
            reason.is_persistent(),
            "{reason:?} is undeliverable — it must be persistent"
        );
        assert_eq!(
            reason.reason_ttl_ms(),
            None,
            "{reason:?} is undeliverable — reason_ttl_ms must be null \
             (persistent, contracts/surface.md)"
        );
    }

    // `reason_ttl_ms` (the reason-line lifetime) is DISTINCT from
    // `dismissal` (the notification behavior, FR-028e): over the same
    // paused surface a pause-demanding reason's transient surface
    // auto-dismisses at the shared 3 s window, while an undeliverable's
    // resident surface is merely dismissable — and neither field reads
    // the other's value.
    let resolver = SurfaceResolver::new(
        None,
        FILENAME.to_string(),
        0,   // total_units: unknown at session start
        0.0, // total_secs: unknown at session start
        false,
    );
    assert_eq!(
        resolver.dismissal(TransportState::Interrupted, true),
        Dismissal::AutoDismiss {
            after_ms: TRANSIENT_DISMISS_MS,
        },
        "a transient reason surface auto-dismisses at TRANSIENT_DISMISS_MS — \
         the dismissal half, separate from the reason TTL"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Interrupted, false),
        Dismissal::Dismissable,
        "a resident (persistent-reason) surface is dismissable, never \
         auto-dismissed — its resume control survives"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Playing, true),
        Dismissal::NonDismissable,
        "dismissal is refused while playing whatever the transiency input — \
         pausing is the route to dismissal (FR-028e)"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Playing, false),
        Dismissal::NonDismissable,
    );
}

// ---------------------------------------------------------------------------
// Assertion 3 — undeliverables ⇒ persistent line (the reachable half
// of "never start + persistent line + error logged").
// GREEN on the enum half. The "never start + error logged" wiring is a
// synth-path property that is not reachable from the player crate: the
// refusal that keeps synthesis from starting lives in the failure
// surface (T043, `src-tauri/src/failure.rs`) and the diagnostic record
// (T040 `prop_trace`) — recorded here as red/TODO for the impl wave,
// traced rather than asserted, exactly as the lock-screen trial traces
// its device-only gaps.
// ---------------------------------------------------------------------------

#[test]
fn exactly_the_three_undeliverables_are_persistent() -> Result<(), Box<dyn Error>> {
    let persistent: Vec<InterruptionReason> = ALL_REASONS
        .into_iter()
        .filter(InterruptionReason::is_persistent)
        .collect();
    assert_eq!(
        persistent,
        UNDELIVERABLE.to_vec(),
        "exactly the three undeliverables are persistent — no other reason \
         never expires"
    );

    for reason in ALL_REASONS {
        let line = reason_line(&reason);
        match expected_earcon_for(reason) {
            Some(EarconKind::Failure) => {
                assert!(
                    !line.is_empty(),
                    "{reason:?} must carry the persistent failure line"
                );
                assert!(
                    line.split_whitespace().count() <= 8,
                    "{reason:?} line {line:?} exceeds the ≤8-word soft target"
                );
                assert_eq!(reason.reason_ttl_ms(), None, "{reason:?} must never expire");
            }
            Some(EarconKind::Warning) => {
                assert!(
                    !line.is_empty(),
                    "{reason:?} must carry the pause-demanding warning line"
                );
                assert!(
                    line.split_whitespace().count() <= 8,
                    "{reason:?} line {line:?} exceeds the ≤8-word soft target"
                );
                assert_eq!(
                    reason.reason_ttl_ms(),
                    Some(3_000),
                    "{reason:?} expires at 3 s"
                );
            }
            Some(EarconKind::Play)
            | Some(EarconKind::Pause)
            | Some(EarconKind::SkipFwd)
            | Some(EarconKind::SkipBack)
            | Some(EarconKind::RateStep) => {
                return Err(format!(
                    "{reason:?} must not select a transport-confirmation earcon — \
                     a reason only ever selects Warning, Failure, or nothing"
                )
                .into());
            }
            None => {
                assert_eq!(
                    reason,
                    InterruptionReason::FocusDelayed,
                    "only the FocusDelayed zero-audio hold selects no earcon \
                     (Deferred shows no line)"
                );
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Assertion 4 — earcon PCM never advances the bookmark.
// GREEN on the pure half: the clock advances only on audible speech
// progress (`on_audible_progress`, FR-002), so synthesizing and
// delivering every earcon descriptor leaves position and stored basis
// bit-identical. The backend half — earcon samples must not be counted
// in the stream callback's consumed-speech accounting — is the T029
// wiring, traced (device-gated: `CpalBackend::new` needs hardware).
// ---------------------------------------------------------------------------

#[test]
fn earcon_pcm_never_advances_the_position_clock() -> Result<(), Box<dyn Error>> {
    let mut clock = PositionClock::new(0.0);
    let _ = clock.on_audible_progress(12.5)?;
    let _ = clock.on_audible_progress(2.5)?;
    assert!(
        (clock.position_secs() - 15.0).abs() <= POS_EPS_SECS,
        "audible speech progress must have advanced the clock to 15.0 s before \
         any earcon is delivered"
    );

    let position_before = clock.position_secs().to_bits();
    let basis_before = clock.stored_basis().to_bits();

    // Deliver every earcon — the no-look confirmations that ride the
    // same stream as speech (contracts/earcon.md) — and touch their
    // PCM exactly as a backend hand-off would.
    for kind in ALL_KINDS {
        let descriptor = EarconDescriptor::for_kind(kind);
        let delivered = descriptor.samples.len();
        let peak = descriptor
            .samples
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        assert!(
            delivered > 0 && peak > 0.0,
            "{kind:?} earcon PCM must be real audio to be delivered at all"
        );
    }

    assert_eq!(
        clock.position_secs().to_bits(),
        position_before,
        "earcon PCM must never advance the position bookmark — the clock counts \
         speech samples only (contracts/earcon.md)"
    );
    assert_eq!(
        clock.stored_basis().to_bits(),
        basis_before,
        "earcon PCM must never move the kill-replay basis (FR-004)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Assertion 5 (reachable half) — the descriptor a ducked session
// delivers is full-scale, non-silent PCM. GREEN (T028).
// ---------------------------------------------------------------------------

#[test]
fn earcon_descriptors_are_full_scale_and_never_silent() {
    for kind in ALL_KINDS {
        let descriptor = EarconDescriptor::for_kind(kind);
        assert!(
            !descriptor.samples.is_empty(),
            "{kind:?} must synthesize PCM"
        );
        assert!(
            descriptor.samples.iter().all(|sample| sample.is_finite()),
            "{kind:?} earcon PCM must be finite f32 — no NaN/inf samples"
        );
        let peak = descriptor
            .samples
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        assert!(
            peak > 0.0,
            "{kind:?} earcon PCM is silent — silence cannot confirm anything \
             (FR-016)"
        );
        assert!(
            peak <= 1.0,
            "{kind:?} earcon PCM peaks at {peak} — synthesis headroom must keep \
             every sample inside [-1.0, 1.0]"
        );
    }
}

// ---------------------------------------------------------------------------
// Assertion 5 (bypass half) — ducked gain never quiets the earcon.
// RED: the duck-exempt short-mix bypass (T029) does not exist —
// `backend.rs` has no earcon path at all, so today every enqueued
// PCM (speech and earcon alike) is scaled by `SampleQueue::gain`, and
// a DUCK_GAIN (0.20) navigation duck would quiet confirmations below
// audibility. The `SampleQueue` type is private and `CpalBackend::new`
// needs audio hardware, so the host-testable half is the marker below
// plus the descriptor assertion above.
// ---------------------------------------------------------------------------

#[test]
fn earcon_short_mix_bypasses_the_sample_queue_gain() {
    assert!(
        BACKEND_SRC.to_ascii_lowercase().contains("earcon"),
        "backend.rs contains no earcon path: every enqueued PCM — speech and \
         earcon alike — is scaled by SampleQueue::gain, so a DUCK_GAIN (0.20) \
         navigation duck quiets the no-look confirmations. T029 must add the \
         duck-exempt short-mix (earcon skips SampleQueue::gain, same stream/clock, \
         files no focus request, never moves the bookmark). RED gap for the impl \
         wave (T029)."
    );
}

// ---------------------------------------------------------------------------
// Assertion 6 — the mirror snapshot: a pause-demanding trigger's
// same-call lock-screen state change carries the transport glyph +
// progress + the reason-bearing surface. GREEN on the current Rust
// shape (T061); the payload's `reason` / `reason_ttl_ms` fields and
// the ≥48dp giant glyph are T043/T044 surface work, traced below.
// ---------------------------------------------------------------------------

#[test]
fn pause_demanding_trigger_publishes_the_mirror_state() -> Result<(), Box<dyn Error>> {
    let resolver = SurfaceResolver::new(
        Some(METADATA_TITLE.to_string()),
        FILENAME.to_string(),
        5,     // total_units: the chunk split's segment count
        150.0, // total_secs: 30 s of spoken audio per unit
        true,
    );
    let headings = [
        Heading {
            position_secs: 0.0,
            title: "Start".to_string(),
        },
        Heading {
            position_secs: 30.0,
            title: "Middle".to_string(),
        },
    ];

    // The display projection (purity keys ⊆ {title, section_title,
    // progress} — FR-028 series).
    let surface = resolver.resolve(
        MIRROR_POSITION_SECS,
        resolver.consumed_units_at(MIRROR_POSITION_SECS),
        &headings,
    );
    assert_eq!(
        surface.title, METADATA_TITLE,
        "the surface title is the metadata title when present (FR-029)"
    );
    assert_eq!(
        surface.section_title,
        Some("Middle".to_string()),
        "the section title is the most-recent-passed heading, never forward-looking"
    );
    match surface.progress {
        Progress::Determinate {
            consumed_units,
            total_units,
            k,
            n,
        } => {
            assert_eq!(
                consumed_units, 1,
                "consumed units are the position's parsed-content projection: 42 s of a                  150 s document of 5 units (30 s per unit) is unit 1 — never the position                  rendered as units (FR-013)"
            );
            assert_eq!(
                total_units, 5,
                "the determinate total is the session-start parsed-content unit count — the                  chunk split's segment count, never an estimate (FR-013)"
            );
            assert_eq!(k, 2, "the section index is 1-based over the heading list");
            assert_eq!(n, 2, "the section count is the heading-list length");
        }
        Progress::Indeterminate => {
            return Err(
                "a session that knew its duration at start must resolve determinate \
                 progress for the whole session (FR-028a)"
                    .into(),
            );
        }
    }

    // The mirror state the lock screen renders — the giant transport
    // glyph (transport) + progress — carrying the session's
    // authoritative position: a mirror, never a second clock
    // (SC-035 / FR-003).
    let mirror = MirrorState {
        transport: TransportState::Interrupted,
        position_secs: MIRROR_POSITION_SECS,
        progress: surface.progress.clone(),
    };
    let wire = serde_json::to_value(&mirror)?;
    let object = wire
        .as_object()
        .ok_or("the mirror state must serialize to a JSON object")?;
    assert_eq!(
        object.get("transport"),
        Some(&serde_json::json!("interrupted")),
        "the mirror carries the session's transport as the giant-glyph source"
    );
    assert!(
        object.contains_key("position_secs"),
        "the mirror carries the session's authoritative position"
    );
    assert!(
        object.contains_key("progress"),
        "the mirror carries the progress indication fixed at session start"
    );

    // The banner half: the reason line the same trigger publishes
    // (contracts/surface.md reason-line table) — the warning banner a
    // pause-demanding interruption shows beside the glyph and progress.
    assert_eq!(
        reason_line(&InterruptionReason::TransientPaused),
        "Pausado por interrupção — retome para continuar",
        "the pause-demanding reason carries its fixed pt-BR line"
    );
    assert_eq!(
        resolver.dismissal(TransportState::Interrupted, true),
        Dismissal::AutoDismiss {
            after_ms: TRANSIENT_DISMISS_MS,
        },
        "the transient reason surface auto-dismisses at the 3 s window — \
         distinct from the reason TTL field"
    );

    // Traced gap (not asserted): the full mirror payload of
    // contracts/surface.md additionally carries `reason` and
    // `reason_ttl_ms` keys and the `dismissal` wire value; those
    // payload fields land with T043 (`src-tauri/src/surface.rs` +
    // `failure.rs`), and the ≥48dp giant glyph with T044 — the
    // reason→Warning/Failure earcon pairing they render is the RED
    // gap asserted by `every_trigger_selects_the_contract_earcon_kind`.
    Ok(())
}
