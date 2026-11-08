//! The real `offline_default` synthesis engine — `sherpa-onnx`
//! 1.13.8 driving the Piper pt-BR `faber` VITS voice
//! (FR-024; `spec.md` Assumptions 1–2; Principle IX).
//!
//! [`synthesize`] loads the embedded voice model exactly once (a
//! `OnceLock`, not per call — the property tests synthesize
//! hundreds of segments) and synthesizes each bounded
//! [`DocumentSegment`] to mono `f32` at the declared 22 050 Hz.
//! The voice assets are embedded in the repo
//! (`assets/voices/pt_BR-faber-medium/`, CC0 — see
//! `PROVENANCE.md`) and loaded through paths resolved at compile
//! time from the voice asset dir — the compile-time
//! `CARGO_MANIFEST_DIR` tree on hosts, or the runtime override
//! set via [`set_asset_dir`] where assets ship packaged (Android);
//! nothing is fetched from the
//! network at synthesis time (Principle IX). The native library
//! itself comes from `sherpa-onnx-sys`'s prebuilt static archive,
//! which bundles ONNX Runtime, `espeak-ng` and
//! `piper_phonemize` — no
//! source build, and no `ort` (see the `ort` deviation record in
//! `API_CONTRACT.md` §7).
//!
//! Every failure path fails closed through [`EngineFailure::report`]
//! naming a member of the closed 7-member [`EngineError`] set
//! (R2/R7; FR-035; SC-035):
//!
//! * voice model load/init/corruption, or an empty (corrupt)
//!   synthesis result → [`FailureCondition::VoiceInitFailure`] →
//!   [`EngineError::InitFailed`];
//! * the engine process dying (killed/OOM) mid-generation →
//!   [`FailureCondition::ProcessDeath`], or an input the C FFI
//!   transport cannot carry → [`FailureCondition::
//!   TransportUnreachable`], both → [`EngineError::Unreachable`];
//! * a delivered rate deviating from 22 050 Hz →
//!   [`FailureCondition::RateDeviation`] → [`EngineError::
//!   RateMismatch`] — never a silent resample (FR-024b).
//!
//! The delivered PCM is verified with [`check_delivery`] against
//! the declared mono-`f32`-at-22 050 Hz shape; any deviation names
//! a closed-set member rather than being silently corrected. The
//! `session_position` a failure carries is an echo of the segment's
//! `resume_point` (falling back to its `source_range`) — never
//! adapter-computed (segment-resume.md CHK030; the `player` is the
//! sole playback authority, `constitution.md:486-490`).

use std::path::PathBuf;
use std::sync::OnceLock;

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig,
    OfflineTtsVitsModelConfig,
};

use crate::chunk::{DocumentSegment, SourceRange};
use crate::failure::{
    EngineError, EngineFailure, FailureCondition, FailureReportError, MAX_OFFER_MS,
};
use crate::format_check::{
    DECLARED_CHANNELS, DECLARED_RATE_HZ, DECLARED_WIDTH_BITS, check_delivery,
};
use crate::normalize::{normalization_mode, normalize_for_speech};
use crate::prosody::plan_for_segment;
use std::time::Instant;
/// The selectable synthesis voice (FR-01).
///
/// `Faber` is the v1 default (Piper pt-BR `faber`, CC0).
/// `Lula` honours Brazilian President Luiz Inácio Lula da Silva
/// with a pt-BR VITS voice obtained separately (see
/// `assets/voices/pt_BR-lula-medium/PROVENANCE.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Voice {
    /// The default Piper pt-BR `faber` voice (CC0).
    #[default]
    Faber,
    /// The `lula` voice honouring President Luiz Inácio Lula da Silva.
    Lula,
}

impl Voice {
    /// The asset directory name for this voice.
    #[must_use]
    pub const fn dir_name(self) -> &'static str {
        match self {
            Self::Faber => "pt_BR-faber-medium",
            Self::Lula => "pt_BR-lula-medium",
        }
    }
}

impl std::fmt::Display for Voice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Faber => write!(f, "faber"),
            Self::Lula => write!(f, "lula"),
        }
    }
}

/// Runtime override for the selected synthesis voice.
///
/// The first call wins; later calls are ignored. Unset means
/// [`Voice::Faber`] (the v1 default).
pub fn set_voice(voice: Voice) {
    let _ignored = SELECTED_VOICE.set(voice);
}

/// Override storage for [`set_voice`].
static SELECTED_VOICE: OnceLock<Voice> = OnceLock::new();

/// The selected voice: the runtime override when set, else
/// [`Voice::Faber`].
fn selected_voice() -> Voice {
    SELECTED_VOICE.get().copied().unwrap_or(Voice::Faber)
}

/// Runtime override for the voice asset directory.
///
/// Hosts that cannot use the compile-time manifest path set this
/// once at startup: on Android the voice files ship inside the APK
/// and resolve through the Tauri resource dir, so the bridge calls
/// this during setup before the first [`synthesize`]. The first
/// call wins; later calls are ignored. Unset means the embedded
/// `assets/voices/pt_BR-<voice>-medium/` tree beside
/// `CARGO_MANIFEST_DIR` (host CLI, tests).
pub fn set_asset_dir(dir: PathBuf) {
    let _ignored = ASSET_DIR_OVERRIDE.set(dir);
}

#[expect(dead_code, reason = "used by asset_dir_for via selected_voice")]
fn asset_dir() -> PathBuf {
    asset_dir_for(selected_voice())
}

/// Override storage for [`set_asset_dir`].
static ASSET_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// The voice asset dir for a given voice: the runtime override
/// when set, else the compile-time manifest tree for that voice.
fn asset_dir_for(voice: Voice) -> PathBuf {
    ASSET_DIR_OVERRIDE.get().cloned().unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/voices/")
            .join(voice.dir_name())
    })
}

/// The Piper VITS model for a voice — mono, 22 050 Hz.
fn model_path(voice: Voice) -> PathBuf {
    asset_dir_for(voice).join(format!("{}.onnx", voice.dir_name()))
}

/// The token map the model for a voice was trained with.
fn tokens_path(voice: Voice) -> PathBuf {
    asset_dir_for(voice).join("tokens.txt")
}

/// The espeak-ng data tree for a voice: resolves pt-BR phonemes
/// against the piper phonemizer at synthesis time.
fn espeak_data_dir(voice: Voice) -> PathBuf {
    asset_dir_for(voice).join("espeak-ng-data")
}
/// Build the `sherpa-onnx` VITS configuration for a voice. The inference
/// parameters mirror the model's own `.onnx.json` (`noise_scale` 0.667,
/// `noise_w` 0.8, `length_scale` 1.0); `provider` stays `None` for the
/// default ONNX Runtime execution provider.
///
/// `num_threads` is pinned to `1`: a single-threaded ONNX Runtime
/// session is resource-predictable for an embedded offline engine —
/// it spawns no uncontrolled per-session thread pool that would
/// contend with the playback, focus and notification threads of the
/// driving shell — and the pre-roll architecture (FR-015a) means
/// synthesis runs ahead of real-time, so single-threaded inference
/// keeps up comfortably within the 60 s pre-roll bound.
fn voice_config(voice: Voice) -> OfflineTtsConfig {
    OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            num_threads: 1,
            vits: OfflineTtsVitsModelConfig {
                model: Some(model_path(voice).to_string_lossy().into_owned()),
                tokens: Some(tokens_path(voice).to_string_lossy().into_owned()),
                data_dir: Some(espeak_data_dir(voice).to_string_lossy().into_owned()),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Lazily-constructed engines, one per voice. Each model loads exactly once
/// — not per call — so the property tests (which synthesize hundreds of
/// segments) pay one load per voice, not hundreds (T044). A failed load is
/// cached as `None`: the cause is deterministic (missing or corrupt voice
/// assets), so every later call re-reports it — with the *current* segment's
/// echoed position, not the first call's.
static ENGINE_FABER: OnceLock<Option<OfflineTts>> = OnceLock::new();
static ENGINE_LULA: OnceLock<Option<OfflineTts>> = OnceLock::new();

/// The shared engine for a voice, loading the voice model on first use.
/// `None` means the model failed to load.
fn engine_for(voice: Voice) -> Option<&'static OfflineTts> {
    match voice {
        Voice::Faber => ENGINE_FABER
            .get_or_init(|| OfflineTts::create(&voice_config(Voice::Faber)))
            .as_ref(),
        Voice::Lula => ENGINE_LULA
            .get_or_init(|| OfflineTts::create(&voice_config(Voice::Lula)))
            .as_ref(),
    }
}

/// Synthesize one bounded segment to mono `f32` at the declared
/// 22 050 Hz, through the selected voice.
///
/// The engine for `voice` is loaded once (see [`engine_for`]) and
/// `segment.content` is synthesized with the VITS model. On success
/// the delivered buffer is non-empty, mono, `f32`, at 22 050 Hz,
/// and is verified against the declared handoff shape with
/// [`check_delivery`] before it is returned — any deviation names a
/// member of the closed [`EngineError`] set instead of being
/// silently resampled or downmixed (FR-024b / SC-035).
///
/// # Errors
///
/// Always a named [`EngineFailure`] — never an empty or silent
/// success (R2/R7):
///
/// * [`FailureCondition::VoiceInitFailure`] → [`EngineError::
///   InitFailed`] when the voice model fails to load, or when the
///   engine yields an empty (corrupt) result for a non-empty
///   segment;
/// * [`FailureCondition::TransportUnreachable`] →
///   [`EngineError::Unreachable`] when the segment text cannot be
///   marshalled across the C FFI boundary (an interior NUL);
/// * [`FailureCondition::ProcessDeath`] → [`EngineError::
///   Unreachable`] when the engine process dies (killed/OOM)
///   mid-generation;
/// * [`FailureCondition::RateDeviation`] → [`EngineError::
///   RateMismatch`] when the delivered rate is not 22 050 Hz.
///
/// The `session_position` the failure carries echoes the segment's
/// `resume_point` (falling back to its `source_range`) — never
/// adapter-computed (segment-resume.md CHK030).
pub fn synthesize(segment: &DocumentSegment, voice: Voice) -> Result<Vec<f32>, EngineFailure> {
    let Some(tts) = engine_for(voice) else {
        return Err(named_failure(
            FailureCondition::VoiceInitFailure,
            FailureCondition::VoiceInitFailure.detail(),
            segment,
        ));
    };
    // The C FFI boundary is the engine's transport: a `&str` with
    // an interior NUL cannot become a `CString`, so the engine is
    // unreachable for this request. Refusing here keeps the
    // named-failure-over-silence rule intact instead of letting the
    // dependency panic on `CString::new` (R2/R7).
    let synth_start = Instant::now();
    if segment.content.contains('\0') {
        return Err(named_failure(
            FailureCondition::TransportUnreachable,
            FailureCondition::TransportUnreachable.detail(),
            segment,
        ));
    }
    let spoken = normalize_for_speech(&segment.content, normalization_mode());
    let Some(audio) = tts.generate_with_config(
        &spoken,
        &GenerationConfig {
            speed: plan_for_segment(segment).speed(),
            ..GenerationConfig::default()
        },
        // No progress callback: the whole buffer is returned
        // at once. The concrete `fn`-pointer type satisfies
        // the `FnMut(&[f32], f32) -> bool` bound so the
        // generic `F` resolves (a bare `None` cannot).
        None::<fn(&[f32], f32) -> bool>,
    ) else {
        // The engine process died (killed/OOM) during generation.
        return Err(named_failure(
            FailureCondition::ProcessDeath,
            FailureCondition::ProcessDeath.detail(),
            segment,
        ));
    };
    let samples = audio.samples();
    // An empty result for a non-empty segment is a corrupt
    // synthesis result — the silent-success defect mode this module
    // exists to prevent. The delivery unit applies the same policy
    // (audio_chunk.rs: an empty delivery is `InitFailed`).
    if samples.is_empty() {
        return Err(named_failure(
            FailureCondition::VoiceInitFailure,
            FailureCondition::VoiceInitFailure.detail(),
            segment,
        ));
    }
    let sample_count = samples.len();
    // The delivered rate, verified against the declared 22 050 Hz
    // before the conformance gate: a deviation names `RateMismatch`
    // with the observed rate, never a silent resample (FR-024b).
    // The Piper `lula`/`faber` model is mono at 22 050 Hz by
    // construction, so the channel count (mono) and the width
    // (`f32`, 32-bit) are type-level facts passed to the gate as
    // declared.
    let delivered_rate = audio.sample_rate();
    let rate_hz = match u32::try_from(delivered_rate) {
        Ok(rate) if rate > 0 => rate,
        // A non-positive rate is not a valid sample rate: the
        // delivered shape deviates from the declared 22 050 Hz.
        Ok(_) => {
            return Err(named_failure(
                FailureCondition::RateDeviation,
                "delivered_rate=0",
                segment,
            ));
        }
        Err(_) => {
            return Err(named_failure(
                FailureCondition::RateDeviation,
                &format!("delivered_rate={delivered_rate}"),
                segment,
            ));
        }
    };
    if rate_hz != DECLARED_RATE_HZ {
        return Err(named_failure(
            FailureCondition::RateDeviation,
            &format!("delivered_rate={rate_hz}"),
            segment,
        ));
    }
    // §6's duration derivation (data-model E2): the duration is
    // DERIVED from the buffer's own length, never supplied. `f64`
    // has no lossless `From<usize>`, so the cast is unavoidable —
    // the identical computation the delivery unit performs
    // (audio_chunk.rs) and the gate's own deviation comparison
    // (format_check.rs) — sanctioned by API_CONTRACT.md §0.1 entry 4.
    #[expect(
        clippy::cast_precision_loss,
        reason = "data-model E2 mandates deriving duration_ms as pcm_data.len() / rate_hz * 1000; f64 has no lossless From<usize>, so the usize -> f64 cast is unavoidable — the identical computation is frozen by the delivery-unit tests (tests/audio_chunk.rs:231-232, :516-518) and is measured against the half-sample band that format_check.rs:130-133 sanctions the same cast for; a >2^53-sample buffer (over 2.8 years of audio at 22 050 Hz) is unreachable in memory; sanctioned by API_CONTRACT.md §0.1 entry 4"
    )]
    let duration_ms = sample_count as f64 / f64::from(rate_hz) * 1000.0;
    // The authoritative conformance gate (FR-024b / SC-035): shape
    // first, then the derived duration. With the rate, channel and
    // width already verified this is the final fail-closed check —
    // any refusal names a closed-set member, never a silent pass.
    if let Err(deviation) = check_delivery(
        rate_hz,
        DECLARED_CHANNELS,
        DECLARED_WIDTH_BITS,
        sample_count,
        duration_ms,
    ) {
        let condition = condition_for_format_error(deviation);
        return Err(named_failure(condition, condition.detail(), segment));
    }
    tracing::info!(
        target: "adapter_synthesis::synth",
        segment_index = segment.segment_index,
        samples = sample_count,
        duration_ms = %format!("{:.1}", duration_ms),
        synth_ms = synth_start.elapsed().as_millis(),
        "segment synthesized"
    );
    Ok(samples.to_vec())
}

/// Synthesize one bounded segment using the currently selected
/// voice ([`selected_voice`]).
///
/// Compatibility wrapper: callers that do not need explicit voice
/// selection use this single-argument form; it resolves to the
/// voice set via [`set_voice`] (default [`Voice::Faber`]).
///
/// # Errors
///
/// Returns [`EngineFailure`] if synthesis fails (voice init, process death, rate mismatch).
pub fn synthesize_for_selected(segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure> {
    synthesize(segment, selected_voice())
}

/// The [`FailureCondition`] whose [`FailureCondition::variant`] is
/// `error` — the inverse of the condition→variant table
/// (`spec.md:86`), used to name the condition behind a
/// [`check_delivery`] refusal.
///
/// [`check_delivery`] refuses with [`EngineError::RateMismatch`]
/// for a rate/shape deviation — the width is a type-level `f32`
/// fact and the channel count is mono by the model's construction,
/// so [`EngineError::ChannelMismatch`] cannot arise from this call
/// site — but the match is exhaustive over all seven members with
/// no "unmapped" arm. `RateMismatch` inverts to
/// [`FailureCondition::RateDeviation`], not `WidthDeviation` (the
/// width is fixed by the `f32` type) or `PitchToleranceViolation`
/// (a synthesis-parameter concern, not a delivered-format one).
#[must_use]
const fn condition_for_format_error(error: EngineError) -> FailureCondition {
    match error {
        EngineError::RateMismatch => FailureCondition::RateDeviation,
        EngineError::ChannelMismatch => FailureCondition::ChannelDeviation,
        EngineError::InitFailed => FailureCondition::VoiceInitFailure,
        EngineError::Unreachable => FailureCondition::TransportUnreachable,
        EngineError::DeviceRefused => FailureCondition::DeviceRefusedAtOpen,
        EngineError::DeviceLost => FailureCondition::DeviceRemoved,
        EngineError::StartRefused => FailureCondition::StartRefusedAtStart,
    }
}

/// Build the named failure for `condition`, failing closed through
/// [`EngineFailure::report`] so its bounds cannot be bypassed.
///
/// The `session_position` echoes the segment's `resume_point`
/// (falling back to its `source_range`) — never adapter-computed
/// (segment-resume.md CHK030; the `player` is the sole playback
/// authority, `constitution.md:486-490`).
///
/// The refusal arm is unreachable at runtime: `observed` is a
/// fixed identifier or a short `key=value` pattern (well under the
/// 200-char bound) and the offer is exactly [`MAX_OFFER_MS`]
/// against the same bound (the refusal check is a strict `>`). It
/// is still handled — the same named failure is reported with the
/// same constants — so the named-failure-over-silence rule holds
/// even in the impossible refusal world.
#[must_use]
fn named_failure(
    condition: FailureCondition,
    observed: &str,
    segment: &DocumentSegment,
) -> EngineFailure {
    let session_position = echoed_position(segment);
    match EngineFailure::report(condition, observed, session_position, MAX_OFFER_MS) {
        Ok(failure) => failure,
        Err(FailureReportError::DetailTooLong { .. } | FailureReportError::OfferTooLong { .. }) => {
            EngineFailure {
                variant: condition.variant(),
                condition,
                observed: observed.to_owned(),
                session_position,
                offer_expires_in_ms: MAX_OFFER_MS,
                tier_at_failure: crate::REGISTERED_TIER,
            }
        }
    }
}

/// The session position a failure echoes for `segment`: its
/// `resume_point` when the segment declares one, else its own
/// `source_range` — never adapter-computed (segment-resume.md
/// CHK030; `constitution.md:486-490`).
#[must_use]
fn echoed_position(segment: &DocumentSegment) -> SourceRange {
    segment.resume_point.unwrap_or(segment.source_range)
}
