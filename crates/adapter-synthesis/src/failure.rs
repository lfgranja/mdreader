//! Failure signaling — the closed 7-member `EngineError`
//! set (FR-023, FR-035, FR-037; spec.md:85;
//! contracts/adapter-handoff.md §3.1, "Failure Signaling
//! Contract").
//!
//! The set is re-exported from `engine_core::closed` — the
//! single authoritative declaration (FR-044: a second list
//! would drift). The closed set is exactly `Unreachable |
//! InitFailed | RateMismatch | ChannelMismatch |
//! DeviceRefused | DeviceLost | StartRefused` — no
//! `Other`/`Unknown`, and no adapter-specific member (C4-A:
//! no `AdapterFailed`). Invented members such as
//! `FormatMismatch`, `ChunkTooLarge`, `SynthesisFailed`, or
//! `ReservationFailed` are contract violations: a format,
//! rate, or width deviation is `RateMismatch`; a channel
//! deviation is `ChannelMismatch` (spec.md:85;
//! contracts/adapter-handoff.md §3.1 condition→variant
//! table).
//!
//! The condition set is closed the same way: exactly ten
//! members, each reporting exactly one `EngineError` member
//! through an exhaustive ten-arm `match` — there is no
//! "unmapped" arm (`API_CONTRACT.md` §3). Construction fails
//! closed: an `observed` detail over [`MAX_CONDITION_DETAIL_CHARS`]
//! is refused with [`FailureReportError::DetailTooLong`] —
//! never truncated, because a truncated identifier would be a
//! silent misreport (R7; SC-035) — and an offer over
//! [`MAX_OFFER_MS`] is refused with
//! [`FailureReportError::OfferTooLong`] (FR-023a / FR-032c /
//! FR-041). Bad Markdown is a document-layer concern, not a
//! [`FailureCondition`] (R8): it is rejected by the document
//! layer before the adapter is invoked, so no member exists
//! for it.

use engine_core::closed::EngineTier;

use crate::chunk::SourceRange;

pub use engine_core::closed::EngineError;

/// Maximum lifetime of the failure offer, in milliseconds
/// (FR-023a, FR-032c, FR-041).
pub const MAX_OFFER_MS: u32 = 3_000;

/// Maximum length of the identifier-pattern detail attached to
/// a failure.
pub const MAX_CONDITION_DETAIL_CHARS: usize = 200;

/// The closed set of conditions the adapter can observe. Every
/// member maps to exactly one `EngineError` member — there is
/// no "unmapped" arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureCondition {
    /// The delivered rate deviates from the declared 22 050 Hz
    /// handoff shape → [`EngineError::RateMismatch`].
    RateDeviation,
    /// The delivered sample width deviates from the declared
    /// 32-bit `f32` shape → [`EngineError::RateMismatch`]: the
    /// closed set names no width-specific variant, because a
    /// width deviation is rate-shape per Principle VII.
    WidthDeviation,
    /// The pitch tolerance
    /// ([`engine_core::format::PITCH_TOLERANCE_CENTS`], the shared
    /// SC-015 bound, 50 cents) is violated →
    /// [`EngineError::RateMismatch`].
    PitchToleranceViolation,
    /// The delivered channel count deviates from the declared
    /// mono handoff shape → [`EngineError::ChannelMismatch`].
    ChannelDeviation,
    /// The voice model, its loader, or initialisation is
    /// corrupt → [`EngineError::InitFailed`].
    VoiceInitFailure,
    /// The engine process died (killed / OOM) →
    /// [`EngineError::Unreachable`].
    ProcessDeath,
    /// The engine transport is unreachable →
    /// [`EngineError::Unreachable`].
    TransportUnreachable,
    /// `AAudio` refused the rate/format at open →
    /// [`EngineError::DeviceRefused`].
    DeviceRefusedAtOpen,
    /// The audio device was removed — `BECOMING_NOISY` or
    /// `onAudioDevicesRemoved`, either sufficient →
    /// [`EngineError::DeviceLost`].
    DeviceRemoved,
    /// Playback start was refused —
    /// `ForegroundServiceStartNotAllowed`, or consent missing
    /// at start → [`EngineError::StartRefused`].
    StartRefusedAtStart,
}

impl FailureCondition {
    /// Every member, in declaration order — a permutation of
    /// the whole closed set: no member missing, none
    /// duplicated.
    pub const ALL: [Self; 10] = [
        Self::RateDeviation,
        Self::WidthDeviation,
        Self::PitchToleranceViolation,
        Self::ChannelDeviation,
        Self::VoiceInitFailure,
        Self::ProcessDeath,
        Self::TransportUnreachable,
        Self::DeviceRefusedAtOpen,
        Self::DeviceRemoved,
        Self::StartRefusedAtStart,
    ];

    /// The single `EngineError` member this condition reports
    /// (spec.md:86 condition→variant table). The match is
    /// exhaustive over the closed ten-member set — there is no
    /// "unmapped" arm.
    // §3 mandates the exhaustive 10-arm condition→variant
    // match — every condition names its variant explicitly —
    // so five arms legitimately share a body
    // (RateDeviation/WidthDeviation/PitchToleranceViolation
    // → RateMismatch; ProcessDeath/TransportUnreachable →
    // Unreachable). Grouping them would weaken the explicit
    // per-condition mapping the failure taxonomy depends on
    // (spec.md:86) — sanctioned by API_CONTRACT.md §0.1
    // entry 1.
    #[expect(
        clippy::match_same_arms,
        reason = "API_CONTRACT.md §3 mandates the exhaustive 10-arm condition→variant match with every arm explicit; five arms share a body by design (RateDeviation/WidthDeviation/PitchToleranceViolation → RateMismatch, ProcessDeath/TransportUnreachable → Unreachable) — grouping would weaken the explicit per-condition mapping (spec.md:86); sanctioned by API_CONTRACT.md §0.1 entry 1"
    )]
    #[must_use]
    pub const fn variant(self) -> EngineError {
        match self {
            Self::RateDeviation => EngineError::RateMismatch,
            Self::WidthDeviation => EngineError::RateMismatch,
            Self::PitchToleranceViolation => EngineError::RateMismatch,
            Self::ChannelDeviation => EngineError::ChannelMismatch,
            Self::VoiceInitFailure => EngineError::InitFailed,
            Self::ProcessDeath => EngineError::Unreachable,
            Self::TransportUnreachable => EngineError::Unreachable,
            Self::DeviceRefusedAtOpen => EngineError::DeviceRefused,
            Self::DeviceRemoved => EngineError::DeviceLost,
            Self::StartRefusedAtStart => EngineError::StartRefused,
        }
    }

    /// A stable `snake_case` identifier for this condition
    /// (`rate_deviation`, …) — an identifier pattern for the
    /// diagnostic record, never document text (R7).
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::RateDeviation => "rate_deviation",
            Self::WidthDeviation => "width_deviation",
            Self::PitchToleranceViolation => "pitch_tolerance_violation",
            Self::ChannelDeviation => "channel_deviation",
            Self::VoiceInitFailure => "voice_init_failure",
            Self::ProcessDeath => "process_death",
            Self::TransportUnreachable => "transport_unreachable",
            Self::DeviceRefusedAtOpen => "device_refused_at_open",
            Self::DeviceRemoved => "device_removed",
            Self::StartRefusedAtStart => "start_refused_at_start",
        }
    }
}

/// A named failure crossing the adapter → player boundary
/// (data-model E3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineFailure {
    /// The named `EngineError` member this failure reports.
    pub variant: EngineError,
    /// The observing condition from the closed ten-member set.
    pub condition: FailureCondition,
    /// Identifier-pattern detail only (e.g.
    /// `delivered_rate=24000`). Never document text.
    pub observed: String,
    /// Echo of the last delivered segment's `source_range` —
    /// derived from `DocumentSegment`, never adapter-computed
    /// (segment-resume.md CHK030).
    pub session_position: SourceRange,
    /// The offer's remaining lifetime in milliseconds; always
    /// `<= MAX_OFFER_MS`.
    pub offer_expires_in_ms: u32,
    /// Always `EngineTier::OfflineDefault` at v1 (R11) — the
    /// sole registered tier (`crate::REGISTERED_TIER`).
    pub tier_at_failure: EngineTier,
}

/// Why a failure report was refused at construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FailureReportError {
    /// The identifier-pattern detail exceeded the
    /// diagnostic-record bound.
    #[error(
        "condition detail is {len} chars; the diagnostic record allows at most {max} identifier-pattern chars"
    )]
    DetailTooLong {
        /// The refused detail's length, in chars.
        len: usize,
        /// The bound the detail exceeded.
        max: usize,
    },
    /// The failure offer outlived the presentation bound.
    #[error("failure offer is {offer_ms} ms; the offer must expire within {max_ms} ms")]
    OfferTooLong {
        /// The refused offer's lifetime, in milliseconds.
        offer_ms: u32,
        /// The bound the offer exceeded.
        max_ms: u32,
    },
}

impl EngineFailure {
    /// Build a named failure, failing closed on an over-long
    /// detail or an over-long offer. `tier_at_failure` is
    /// always `EngineTier::OfflineDefault`.
    ///
    /// # Errors
    ///
    /// * [`FailureReportError::DetailTooLong`] when `observed`
    ///   is longer than [`MAX_CONDITION_DETAIL_CHARS`] chars —
    ///   the detail is refused, never truncated, because a
    ///   truncated identifier would be a silent misreport (R7;
    ///   SC-035).
    /// * [`FailureReportError::OfferTooLong`] when
    ///   `offer_expires_in_ms` is greater than [`MAX_OFFER_MS`]
    ///   — a stated reason that outlives its presentation bound
    ///   must not be reported at all (FR-023a / FR-032c /
    ///   FR-041).
    pub fn report(
        condition: FailureCondition,
        observed: &str,
        session_position: SourceRange,
        offer_expires_in_ms: u32,
    ) -> Result<Self, FailureReportError> {
        let detail_chars = observed.chars().count();
        if detail_chars > MAX_CONDITION_DETAIL_CHARS {
            return Err(FailureReportError::DetailTooLong {
                len: detail_chars,
                max: MAX_CONDITION_DETAIL_CHARS,
            });
        }
        if offer_expires_in_ms > MAX_OFFER_MS {
            return Err(FailureReportError::OfferTooLong {
                offer_ms: offer_expires_in_ms,
                max_ms: MAX_OFFER_MS,
            });
        }
        Ok(Self {
            variant: condition.variant(),
            condition,
            observed: observed.to_owned(),
            session_position,
            offer_expires_in_ms,
            tier_at_failure: EngineTier::OfflineDefault,
        })
    }
}
