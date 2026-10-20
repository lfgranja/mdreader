//! Interruption reasons — closed 8-reason set (FR-005 / data-model.md).
//!
//! Measurement note (T001b):
//!   Fixture byte size: `tests/device/fixtures/fixture-pt-2h.md`
//!   Measured via `stat -c %s` → 62424 bytes.
//!   `MAX_DOC_BYTES` = 62424 × 1.5 = 93636 (headroom for ~3h at 1.0x).
//!   Wire reference: FR-018 ingest refusal → `FormatUndeliverable` when exceeded.

/// Fixture-scale document byte cap (FR-018).
///
/// Fixed at plan time from the measured fixture (`fixture-pt-2h.md`:
/// 62424 bytes via `stat -c %s`). Headroom factor 1.5 yields support
/// for ~3 hours at 1.0x speed. Oversize documents refuse before
/// synthesis with `FormatUndeliverable`.
pub const MAX_DOC_BYTES: usize = 93636;

/// The closed interruption-reason set (no `Other` / `Unknown`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptionReason {
    /// Audio focus refused (e.g. call demands silence) → `Refused`.
    FocusDenied,
    /// Focus acquisition delayed; zero-audio hold → `Deferred` (no line).
    FocusDelayed,
    /// Permanent audio-focus loss (e.g. headset removed, call won) → pause + 3s.
    PermanentLoss,
    /// Transient interruption; may duck or pause depending on class.
    /// `DuckableTransient` → duck ~20%, restore on GAIN; `SilenceDemanding` → pause + 3s.
    TransientPaused,
    /// Output device removed; pauses with latest reason (latest-wins).
    DeviceRemoved,
    /// Voice/synthesis unavailable before playback starts → never start + persistent.
    VoiceUnavailable,
    /// Document format or size exceeds deliverable bounds → never start + persistent.
    FormatUndeliverable,
    /// Engine decode failure; never resumes from this state → persistent.
    DecodeFailed,
}

impl InterruptionReason {
    /// Exhaustive match — no catch-all arm (gate 7).
    /// Returns the ≤8-word pt-BR line per contracts/surface.md reason-line table.
    /// `FocusDelayed` returns empty string (Deferred = zero-audio hold, no line).
    /// Undeliverables (`VoiceUnavailable`, `FormatUndeliverable`, `DecodeFailed`) are persistent (`reason_ttl_ms` = null).
    ///
    /// Every line also fits the ≤40-char hard limit of the same contract
    /// (the Android notification content-text limit), which wins over the
    /// ≤8-word soft target whenever the two conflict.
    ///
    /// The conflict is resolved, not merely recorded: the three lines
    /// that once stood at 47 / 43 / 43 chars were shortened rather than
    /// the bound relaxed (T101), so every line now satisfies BOTH bounds
    /// at once — 30 / 23 / 32 / 25 / 37 / 38 / 37 chars and 6 / 4 / 5 /
    /// 4 / 6 / 7 / 6 words — and the tie-break rule is needed for no
    /// line at all. The contract table (`contracts/surface.md`, "Fixed
    /// reason lines") is the normative source of these strings — the
    /// code states them once, and the table restates them once.
    #[must_use]
    pub const fn reason_line(&self) -> &'static str {
        match self {
            Self::FocusDenied => "Foco de áudio negado — pausado",
            Self::FocusDelayed => "",
            Self::PermanentLoss => "Áudio perdido — pausado",
            Self::TransientPaused => "Pausado por interrupção — retome",
            Self::DeviceRemoved => "Fones removidos — pausado",
            Self::VoiceUnavailable => "Voz indisponível — leitura não inicia",
            Self::FormatUndeliverable => "Formato do documento não pode ser lido",
            Self::DecodeFailed => "Falha ao decodificar — leitura parada",
        }
    }

    /// Returns true if this reason is persistent (`reason_ttl_ms` = null).
    /// Undeliverable failures: `VoiceUnavailable`, `FormatUndeliverable`, `DecodeFailed`.
    #[must_use]
    pub const fn is_persistent(&self) -> bool {
        matches!(
            self,
            Self::VoiceUnavailable | Self::FormatUndeliverable | Self::DecodeFailed
        )
    }

    /// Returns the reason TTL in milliseconds, or `None` for persistent.
    /// Transient reasons: 3000 ms. Persistent: None.
    #[must_use]
    pub const fn reason_ttl_ms(&self) -> Option<u64> {
        if self.is_persistent() {
            None
        } else {
            Some(3000)
        }
    }
}

/// Free-function wrapper for external use (tests, FFI).
#[must_use]
pub const fn reason_line(reason: &InterruptionReason) -> &'static str {
    reason.reason_line()
}

/// Maps a `FocusOutcome` to an `InterruptionReason` for the pause branch.
/// `DuckableTransient` ducks (no `InterruptionReason`); `SilenceDemanding`/`PermanentLoss` pause.
#[must_use]
pub const fn from_focus_outcome(outcome: crate::focus::FocusOutcome) -> Option<InterruptionReason> {
    match outcome {
        crate::focus::FocusOutcome::DuckableTransient => None, // ducks, no reason line
        crate::focus::FocusOutcome::SilenceDemanding => Some(InterruptionReason::TransientPaused),
        crate::focus::FocusOutcome::PermanentLoss => Some(InterruptionReason::PermanentLoss),
    }
}

/// Maps a `FocusGrant` to an `InterruptionReason`.
/// Granted → None; Deferred → `FocusDelayed`; Refused → `FocusDenied`.
#[must_use]
pub const fn from_focus_grant(grant: crate::focus::FocusGrant) -> Option<InterruptionReason> {
    match grant {
        crate::focus::FocusGrant::Granted => None,
        crate::focus::FocusGrant::Deferred => Some(InterruptionReason::FocusDelayed),
        crate::focus::FocusGrant::Refused => Some(InterruptionReason::FocusDenied),
    }
}

/// Maps a `RoutingDecision` pause to an `InterruptionReason`.
///
/// Both `BecomingNoisy` and `DeviceDisappeared` while playing → `DeviceRemoved`.
/// Privacy-precedence pause (private→public switch) → `DeviceRemoved`.
#[must_use]
pub const fn from_routing_pause(
    decision: &crate::routing::RoutingDecision,
) -> Option<InterruptionReason> {
    match decision {
        crate::routing::RoutingDecision::Pause { .. } => Some(InterruptionReason::DeviceRemoved),
        crate::routing::RoutingDecision::Follow { .. }
        | crate::routing::RoutingDecision::NoChange => None,
    }
}

/// The earcon kind a reason sounds (contracts/earcon.md mirror
/// contract: "Trigger + lock-screen mirror publish is one Rust
/// call — impossible to sound without showing").
///
/// The N:1 mapping onto the closed 7-kind set (Principle VII —
/// the match is exhaustive, no catch-all): the four
/// pause-demanding reasons sound `Warning` (the
/// abstract-dissonant confirmation that a pause stands); the
/// three undeliverables sound `Failure` (the E4 transposed
/// dissonant pair, persistent on the surface); `FocusDelayed`
/// sounds nothing (the zero-audio hold states no line and
/// confirms nothing — data-model.md interruption table).
// §3/§5-style explicitness: every trigger pairs with its earcon on its own
// arm so the mirror contract's 1:1 pairing is expressed once, per trigger —
// grouping the arms would weaken the per-trigger mapping the test pins
// (test_earcon_mirror.rs assertion 1); sanctioned per API_CONTRACT.md §0.1
// entry 1 convention.
#[expect(
    clippy::match_same_arms,
    reason = "contracts/earcon.md mirror contract pairs every trigger with its earcon on its own arm (test_earcon_mirror.rs assertion 1 pins the pairing); the four pause-demanding reasons legitimately share the Warning arm and the three undeliverables the Failure arm — grouping them into or-patterns would hide the per-trigger pairing"
)]
#[must_use]
pub const fn earcon_for(reason: InterruptionReason) -> Option<crate::earcon::EarconKind> {
    match reason {
        InterruptionReason::TransientPaused => Some(crate::earcon::EarconKind::Warning),
        InterruptionReason::FocusDenied => Some(crate::earcon::EarconKind::Warning),
        InterruptionReason::PermanentLoss => Some(crate::earcon::EarconKind::Warning),
        InterruptionReason::DeviceRemoved => Some(crate::earcon::EarconKind::Warning),
        InterruptionReason::VoiceUnavailable => Some(crate::earcon::EarconKind::Failure),
        InterruptionReason::FormatUndeliverable => Some(crate::earcon::EarconKind::Failure),
        InterruptionReason::DecodeFailed => Some(crate::earcon::EarconKind::Failure),
        InterruptionReason::FocusDelayed => None,
    }
}
