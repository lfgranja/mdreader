//! Closed-set vocabulary for the engine core (T014).
//!
//! Every set in this module is closed by construction (Principle
//! VII, FR-044): the variants below are the complete sets, there
//! is no `Other`/`Unknown` catch-all, and every consumer matches
//! exhaustively — an added or removed variant is a coordinated
//! edit across every file that enumerates it, with a
//! semantic-version bump and a documentation update.
//!
//! The wire forms are fixed by the contracts: rate rungs serialize
//! as their JSON number (contracts/shell-ipc.md case-mapping
//! table); every other set serializes as its `snake_case` wire
//! string. An unknown wire value never deserializes, because there
//! is no variant for it to land in.
//!
//! Engine identity: every tier except [`EngineTier::TestDouble`]
//! refers to an engine of the constitution's TTS Engine Registry
//! by name and version. [`EngineTier::TestDouble`] is the sole
//! exception (2026-10-05 clarification) and is restricted to the
//! `standin_a`/`standin_b` stand-ins, which exist only as
//! fixtures for SC-051 failover drills.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The discrete playback-rate ladder (FR-019): exactly the seven
/// rungs `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`.
///
/// The wire form is the JSON number of the rung itself
/// (contracts/shell-ipc.md case-mapping table), so the enum
/// round-trips as a bare number rather than as a string or a
/// tagged object. All seven rungs are exact binary fractions, so
/// wire-form comparison is exact — no tolerance is involved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackRate {
    /// `0.75×` — the slowest declared rung.
    R0_75,
    /// `1.0×` — normal speed; the default rung.
    R1_0,
    /// `1.25×`.
    R1_25,
    /// `1.5×`.
    R1_5,
    /// `1.75×`.
    R1_75,
    /// `2.0×` — the pitch-tolerance reference rung (within
    /// [`crate::format::PITCH_TOLERANCE_CENTS`], SC-015,
    /// contracts/engine-handoff.md).
    R2_0,
    /// `4.0×` — the fastest declared rung.
    R4_0,
}

impl PlaybackRate {
    /// The rung's numeric value: the exact `f64` carried on the
    /// wire.
    const fn value(self) -> f64 {
        match self {
            Self::R0_75 => 0.75,
            Self::R1_0 => 1.0,
            Self::R1_25 => 1.25,
            Self::R1_5 => 1.5,
            Self::R1_75 => 1.75,
            Self::R2_0 => 2.0,
            Self::R4_0 => 4.0,
        }
    }

    /// The rung for a numeric value, or `None` when the value is
    /// not one of the seven declared rungs (the ladder is closed:
    /// nothing between or beyond the rungs exists, FR-019).
    #[expect(
        clippy::float_cmp,
        reason = "all seven rungs are exact binary fractions (multiples of 1/4), so == is exact; a tolerance would wrongly admit off-ladder values that are merely close, breaking the closed ladder (FR-019)"
    )]
    fn from_value(value: f64) -> Option<Self> {
        if value == 0.75 {
            Some(Self::R0_75)
        } else if value == 1.0 {
            Some(Self::R1_0)
        } else if value == 1.25 {
            Some(Self::R1_25)
        } else if value == 1.5 {
            Some(Self::R1_5)
        } else if value == 1.75 {
            Some(Self::R1_75)
        } else if value == 2.0 {
            Some(Self::R2_0)
        } else if value == 4.0 {
            Some(Self::R4_0)
        } else {
            None
        }
    }
}

impl Serialize for PlaybackRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.value())
    }
}

impl<'de> Deserialize<'de> for PlaybackRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Visitor for the rate ladder: accepts the rung as a JSON
        /// number, in float or integer notation.
        struct RungVisitor;

        impl serde::de::Visitor<'_> for RungVisitor {
            type Value = PlaybackRate;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a playback-rate rung: 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, or 4.0")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                PlaybackRate::from_value(value)
                    .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Float(value), &self))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == 1 {
                    Ok(PlaybackRate::R1_0)
                } else if value == 2 {
                    Ok(PlaybackRate::R2_0)
                } else if value == 4 {
                    Ok(PlaybackRate::R4_0)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Unsigned(value),
                        &self,
                    ))
                }
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == 1 {
                    Ok(PlaybackRate::R1_0)
                } else if value == 2 {
                    Ok(PlaybackRate::R2_0)
                } else if value == 4 {
                    Ok(PlaybackRate::R4_0)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Signed(value),
                        &self,
                    ))
                }
            }
        }

        deserializer.deserialize_f64(RungVisitor)
    }
}

/// The transport-state reflection (FR-020): exactly
/// `Playing | Paused | Stopped | Completed | Interrupted`.
///
/// The shell mirrors this state; it never computes position or
/// transport transitions itself (SC-035). The wire form is the
/// `snake_case` state name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    /// Audio is playing.
    #[serde(rename = "playing")]
    Playing,
    /// Playback is paused; position is retained.
    #[serde(rename = "paused")]
    Paused,
    /// Playback is stopped; the session has ended at user request.
    #[serde(rename = "stopped")]
    Stopped,
    /// Playback reached the end of the document.
    #[serde(rename = "completed")]
    Completed,
    /// Playback was interrupted (for example by audio focus loss);
    /// the interruption reason travels alongside, not inside, this
    /// state.
    #[serde(rename = "interrupted")]
    Interrupted,
}

impl std::fmt::Display for TransportState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The wire form is the human-readable form too, so a
        // state interpolated into a failure message can never
        // drift from its diagnostic-record form (SC-005).
        f.write_str(match self {
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
            Self::Completed => "completed",
            Self::Interrupted => "interrupted",
        })
    }
}

/// The audio-focus outcome classification (FR-023 series): exactly
/// `DuckableTransient | SilenceDemanding | PermanentLoss`.
///
/// `*_MAY_DUCK` events classify as [`FocusOutcome::DuckableTransient`]
/// (duck to about 20% and keep reading); call/alarm/permanent-loss
/// events classify as [`FocusOutcome::SilenceDemanding`] or
/// [`FocusOutcome::PermanentLoss`] (pause with an expiring reason).
/// Regained focus is not an outcome — it is the separate direction
/// event [`FocusRegained`] (spec.md, session 2026-10-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusOutcome {
    /// A transient request that permits ducking: duck to about
    /// 20% and keep reading.
    #[serde(rename = "duckable_transient")]
    DuckableTransient,
    /// A demand that requires silence (call, alarm): pause now,
    /// with a reason that expires.
    #[serde(rename = "silence_demanding")]
    SilenceDemanding,
    /// Permanent loss of the audio route: pause now, with a reason
    /// that expires.
    #[serde(rename = "permanent_loss")]
    PermanentLoss,
}

/// The focus `regained` direction event (spec.md, session
/// 2026-10-02).
///
/// Regained focus is an event of direction, not a fourth outcome
/// classification: it names the moment focus returns so the shell
/// can act on the resume policy, and it carries no outcome of its
/// own. The wire form is `"regained"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusRegained {
    /// Audio focus was regained.
    #[serde(rename = "regained")]
    Regained,
}

/// The engine failure taxonomy (FR-023, FR-035, FR-037): exactly
/// `Unreachable | InitFailed | RateMismatch | ChannelMismatch |
/// DeviceRefused | DeviceLost | StartRefused`.
///
/// Failures surface as one of these named errors — never as
/// silence, never as a substituted format (FR-023: "a failure that
/// cannot be named cannot be reported"). The wire form is the
/// `snake_case` error name prefixed with `engine_` where it names a
/// transport-level condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineError {
    /// The engine is unreachable at the transport level.
    #[serde(rename = "engine_unreachable")]
    Unreachable,
    /// The engine failed to initialize.
    #[serde(rename = "engine_init_failed")]
    InitFailed,
    /// The delivered (or requested) rate deviates from the declared
    /// handoff shape (FR-035) — including a pitch-tolerance
    /// violation at 2.0× (contracts/engine-handoff.md).
    #[serde(rename = "rate_mismatch")]
    RateMismatch,
    /// The delivered channel count deviates from the declared mono
    /// handoff shape (FR-019, FR-035) (FR-033, superseded by FR-019) — never a silent downmix.
    #[serde(rename = "channel_mismatch")]
    ChannelMismatch,
    /// The audio device refused the playback request.
    #[serde(rename = "device_refused")]
    DeviceRefused,
    /// The audio device was lost during playback.
    #[serde(rename = "device_lost")]
    DeviceLost,
    /// Playback start was refused.
    #[serde(rename = "start_refused")]
    StartRefused,
}

/// The session-end reason (FR-044): exactly
/// `Paused | Stopped | Completed | Interrupted` — one closed set of
/// four values shared with the diagnostic record.
///
/// `Playing` is a [`TransportState`] and is never a session-end
/// reason: a session cannot end while still playing. The wire form
/// is the `snake_case` reason name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionEnd {
    /// The session ended because playback was paused.
    #[serde(rename = "paused")]
    Paused,
    /// The session ended because playback was stopped.
    #[serde(rename = "stopped")]
    Stopped,
    /// The session ended because the document completed.
    #[serde(rename = "completed")]
    Completed,
    /// The session ended because playback was interrupted.
    #[serde(rename = "interrupted")]
    Interrupted,
}

/// The closed engine-tier set (FR-023c) in ascending fallback
/// order: `offline_default < cloud_default < cloud_escalation <
/// test_double < evaluated_fallback`.
///
/// Every tier except [`EngineTier::TestDouble`] refers to engines
/// of the constitution's TTS Engine Registry by name and version
/// (`.specify/memory/constitution.md`, "TTS Engine Registry"):
/// the offline default is `sherpa-onnx` + Piper `pt-BR`; the cloud
/// default is Google Cloud TTS `WaveNet`; the cloud-escalation tier
/// holds the remaining registry engines ranked by free quota; the
/// evaluated fallback is Microsoft Edge read-aloud via `msedge-tts`
/// 0.4.0 — last, never default, consent-gated, and never proxied
/// through the OpenAI-compatible Python wrapper. [`EngineTier::TestDouble`]
/// is the sole exception to registry reference (2026-10-05
/// clarification): it exists only for the `standin_a`/`standin_b`
/// stand-ins, which are fixtures for SC-051 failover drills and
/// are never selectable while any non-`test_double` adapter is
/// registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineTier {
    /// Offline default tier: the on-device engine (constitution
    /// registry: `sherpa-onnx` + Piper `pt-BR`). Admissible without
    /// any network connection (Principle IX).
    #[serde(rename = "offline_default")]
    OfflineDefault,
    /// Cloud default tier (constitution registry: Google Cloud TTS
    /// `WaveNet`). Consent-gated per provider, revocable, and never
    /// a silent default (Principle X).
    #[serde(rename = "cloud_default")]
    CloudDefault,
    /// Cloud escalation tier (constitution registry: the remaining
    /// cloud engines, ranked by free quota).
    #[serde(rename = "cloud_escalation")]
    CloudEscalation,
    /// Test-double tier (2026-10-05 clarification): restricted to
    /// the `standin_a`/`standin_b` stand-in fixtures for SC-051
    /// failover drills. Never selectable while any non-`test_double`
    /// adapter is registered.
    #[serde(rename = "test_double")]
    TestDouble,
    /// Evaluated fallback tier (constitution registry: Microsoft
    /// Edge read-aloud via `msedge-tts` 0.4.0, under the
    /// constitution's XI exception): the Edge endpoint stays last
    /// and is never a default.
    #[serde(rename = "evaluated_fallback")]
    EvaluatedFallback,
}

impl EngineTier {
    /// The single declared ascending fallback sequence (FR-023c,
    /// contracts/engine-handoff.md): `offline_default`,
    /// `cloud_default`, `cloud_escalation`, `test_double`,
    /// `evaluated_fallback`.
    ///
    /// This is the one authoritative list — a second list would
    /// drift (FR-044). The fallback selector consumes this order
    /// directly (T075); the Edge endpoint is last and never a
    /// default.
    pub const ORDER: [Self; 5] = [
        Self::OfflineDefault,
        Self::CloudDefault,
        Self::CloudEscalation,
        Self::TestDouble,
        Self::EvaluatedFallback,
    ];

    /// Whether this tier is selectable while the adapter's
    /// `OfflineDefault` registration is active (`SC-051`;
    /// `adapter-handoff.md` §6.1; `reg.rs`: `REGISTERED_TIERS`).
    ///
    /// Reflects adapter registration — not a second tier list
    /// (`FR-044`; `API_CONTRACT.md` §4). Only `OfflineDefault`
    /// is selectable at v1; `TestDouble` is false because the
    /// stand-ins (`standin_a`/`standin_b`) are fixtures
    /// (`001/spec.md:89`; `closed.rs`:305-312).
    ///
    /// Five explicit arms — never a wildcard or underscore arm
    /// (Principle VII; constitution gate 7). The three unregistered
    /// tiers return false with distinct deferral reasons
    /// (`SC-044` deferred, consent deferred, evaluated-fallback
    /// deferred) recorded in doc comments, not duplicated arm
    /// bodies — sanctioned by `API_CONTRACT.md` §0.1 entry 2
    /// (registered in `API_CONTRACT.md` §0.1).
    #[expect(
        clippy::match_same_arms,
        reason = "API_CONTRACT.md §0.1 entry 2: SC-051 requires five distinct arms; CloudDefault, CloudEscalation, EvaluatedFallback legitimately all false at v1 (unregistered: SC-044 deferred / consent deferred / evaluated-fallback deferred). Reasons live in doc comments above, not duplicated bodies."
    )]
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        match self {
            Self::OfflineDefault => true,
            Self::CloudDefault => false,
            Self::CloudEscalation => false,
            Self::TestDouble => false,
            Self::EvaluatedFallback => false,
        }
    }
}
