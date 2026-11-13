//! The real `offline_default` synthesis engine — `sherpa-onnx`
//! 1.13.8 driving bundled Piper pt-BR VITS voices from the closed
//! catalog (FR-024; spec 018; Principle IX).
//!
//! [`synthesize`] synthesizes each bounded [`DocumentSegment`] to
//! mono `f32` at the declared 22 050 Hz using the active/default
//! catalog voice. [`synthesize_with_voice`] synthesizes with an
//! explicit [`VoiceId`].
//!
//! Voice assets are bundled under `assets/voices/<voice-id>/` with
//! shared phonemizer data in `assets/voices/shared/espeak-ng-data/`.
//! Paths resolve from the compile-time `CARGO_MANIFEST_DIR` tree on
//! hosts, or from the runtime root set via [`set_voices_dir`] where
//! assets ship packaged (Android). Nothing is fetched from the
//! network at synthesis time (Principle IX).
//!
//! Every failure path fails closed through [`EngineFailure::report`]
//! naming a member of the closed 7-member [`EngineError`] set
//! (R2/R7; FR-035; SC-035).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

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
use crate::voice::{VoiceId, inference_defaults, spec as voice_spec};

/// Active playback rate multiplier for synthesis, stored as `rate * 100` (e.g. 100 = 1.0x).
/// Default is 100 (1.0x).
static ACTIVE_RATE_MULTIPLIER_CENTIS: AtomicU32 = AtomicU32::new(100);

/// Set the playback rate multiplier for synthesis (e.g. 1.0, 1.25, 1.5, 2.0).
pub fn set_playback_rate(rate: f32) {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rate is clamped between 0.5 and 4.0 so multiplying by 100.0 is strictly within [50.0, 400.0]"
    )]
    let centis = (rate.clamp(0.5, 4.0) * 100.0).round() as u32;
    ACTIVE_RATE_MULTIPLIER_CENTIS.store(centis, Ordering::Release);
}

/// The currently configured playback rate multiplier for synthesis.
#[must_use]
pub fn playback_rate() -> f32 {
    let centis = ACTIVE_RATE_MULTIPLIER_CENTIS.load(Ordering::Acquire);
    #[expect(
        clippy::cast_precision_loss,
        reason = "centis is strictly within [50, 400], fitting losslessly in f32 mantissa"
    )]
    let centis_f32 = centis as f32;
    centis_f32 / 100.0
}

/// Runtime override for the bundled voices root directory.
///
/// Hosts that cannot use the compile-time manifest path set this once
/// at startup: on Android the voice files ship inside the APK and are
/// extracted to the app data dir, so the bridge calls this during setup
/// before the first synthesis. The first call wins; later calls are
/// ignored. Unset means the embedded `assets/voices/` tree beside
/// `CARGO_MANIFEST_DIR` (host CLI, tests).
pub fn set_voices_dir(dir: PathBuf) {
    let _ignored = VOICES_DIR_OVERRIDE.set(dir);
}

/// Backward-compatible alias for [`set_voices_dir`].
///
/// Accepts either a voices-root directory (`.../voices`) or a legacy
/// single-voice directory (`.../voices/pt_BR-faber-medium`) and
/// normalizes to the voices root.
pub fn set_asset_dir(dir: PathBuf) {
    let root = normalize_voices_root(dir);
    set_voices_dir(root);
}

/// Override storage for [`set_voices_dir`].
static VOICES_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Optional model-path override for the active/default synthesis path.
///
/// Compatibility shim for out-of-tree voice bundles that provide a
/// standalone `.onnx` with sibling `tokens.txt` and `espeak-ng-data/`.
/// When set, the active voice engine is created from this path instead
/// of the catalog voice directory. The first call wins.
static MODEL_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Set a custom voice model ONNX path for synthesis.
///
/// Compatibility API: when a complete bundle (sibling `tokens.txt` and
/// `espeak-ng-data/`) is supplied, this path is used for the active
/// synthesis engine. Catalog voice selection APIs remain available.
///
/// The first call wins; later calls are ignored.
pub fn set_model_path(path: PathBuf) {
    let _ignored = MODEL_PATH_OVERRIDE.set(path);
}

/// Active voice id for [`synthesize`] (delegation target).
/// Initialized once by the bridge/CLI; defaults to [`VoiceId::DEFAULT`].
static ACTIVE_VOICE: OnceLock<VoiceId> = OnceLock::new();

/// Set the active voice id used by [`synthesize`].
///
/// First call wins (startup seeding). Returns `true` if this call set
/// the value.
#[must_use]
pub fn set_active_voice(voice: VoiceId) -> bool {
    ACTIVE_VOICE.set(voice).is_ok()
}

/// Replace the active voice id for subsequent [`synthesize`] calls.
///
/// Returns the previous active voice (or default if unset).
pub fn replace_active_voice(voice: VoiceId) -> VoiceId {
    // ACTIVE_VOICE is OnceLock for startup seeding; runtime switches are
    // tracked via atomic index so synthesis delegation follows the
    // current selection without unsafe global mutation.
    let prev = active_voice();
    let idx = u32::try_from(voice.index()).unwrap_or_default();
    ACTIVE_VOICE_INDEX.store(idx, Ordering::Release);
    // Ensure startup-seeded state is initialized for callers that read
    // ACTIVE_VOICE directly in future extensions.
    let _ = ACTIVE_VOICE.set(prev);
    prev
}

/// Runtime-active voice index (mirrors catalog index). Defaults to 0.
static ACTIVE_VOICE_INDEX: AtomicU32 = AtomicU32::new(0);

/// Returns the active voice id, defaulting to [`VoiceId::DEFAULT`].
#[must_use]
pub fn active_voice() -> VoiceId {
    match ACTIVE_VOICE_INDEX.load(Ordering::Acquire) {
        1 => VoiceId::PtBrCaduMedium,
        _ => VoiceId::PtBrFaberMedium,
    }
}

/// Normalize a caller-supplied path to the voices root.
///
/// Accepts `.../voices` or legacy `.../voices/<voice-id>` and returns
/// the directory containing per-voice subdirectories.
fn normalize_voices_root(dir: PathBuf) -> PathBuf {
    if dir.file_name().and_then(|n| n.to_str()) == Some(VoiceId::PtBrFaberMedium.id())
        || dir.file_name().and_then(|n| n.to_str()) == Some(VoiceId::PtBrCaduMedium.id())
    {
        return dir.parent().map(Path::to_path_buf).unwrap_or(dir);
    }
    dir
}

/// The bundled voices root: runtime override when set, else the
/// compile-time manifest tree.
#[must_use]
pub fn voices_root() -> PathBuf {
    VOICES_DIR_OVERRIDE
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/voices"))
}

/// The directory for `voice` under the voices root.
#[must_use]
pub fn voice_dir(voice: VoiceId) -> PathBuf {
    voices_root().join(voice_spec(voice).dir_name)
}

/// The ONNX model path for `voice`.
#[must_use]
pub fn model_path(voice: VoiceId) -> PathBuf {
    let s = voice_spec(voice);
    voice_dir(voice).join(s.onnx)
}

/// The token map path for `voice`.
#[must_use]
pub fn tokens_path(voice: VoiceId) -> PathBuf {
    voice_dir(voice).join(voice_spec(voice).tokens)
}

/// The espeak-ng data directory for `voice`.
///
/// Prefers shared `shared/espeak-ng-data` under the voices root; falls
/// back to a per-voice `espeak-ng-data` directory when present.
#[must_use]
pub fn espeak_data_dir(voice: VoiceId) -> PathBuf {
    let shared = voices_root().join("shared").join("espeak-ng-data");
    if shared.is_dir() {
        return shared;
    }
    voice_dir(voice).join("espeak-ng-data")
}

/// Build the `sherpa-onnx` VITS configuration for `voice`.
///
/// Inference defaults come from the bundled `.onnx.json` contract via
/// [`crate::voice::inference_defaults`]. `num_threads` is pinned to `1`
/// for resource-predictable embedded synthesis.
#[must_use]
pub fn voice_config(voice: VoiceId) -> OfflineTtsConfig {
    let inf = inference_defaults(voice);
    OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            num_threads: 1,
            vits: OfflineTtsVitsModelConfig {
                model: Some(model_path(voice).to_string_lossy().into_owned()),
                tokens: Some(tokens_path(voice).to_string_lossy().into_owned()),
                data_dir: Some(espeak_data_dir(voice).to_string_lossy().into_owned()),
                noise_scale: inf.noise_scale,
                noise_scale_w: inf.noise_scale_w,
                length_scale: inf.length_scale,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Per-voice lazily-constructed engines.
///
/// A failed load is cached as `None` for that voice slot: the cause is
/// deterministic (missing/corrupt assets), so later calls re-report it
/// with the *current* segment's echoed position.
///
/// Safety/compatibility note: if host validation shows `OfflineTts`
/// cannot be safely held this way or causes resource issues, switch to
/// recreate-on-switch single active engine (contingency A2). This
/// implementation attempts per-voice cache first per plan.
static ENGINE_FABER: OnceLock<Option<OfflineTts>> = OnceLock::new();
static ENGINE_CADU: OnceLock<Option<OfflineTts>> = OnceLock::new();
/// Custom/override engine (compatibility path for `set_model_path`).
static ENGINE_CUSTOM: OnceLock<Option<OfflineTts>> = OnceLock::new();

fn engine_slot(voice: VoiceId) -> &'static OnceLock<Option<OfflineTts>> {
    match voice {
        VoiceId::PtBrFaberMedium => &ENGINE_FABER,
        VoiceId::PtBrCaduMedium => &ENGINE_CADU,
    }
}

fn custom_voice_config(model: &Path) -> OfflineTtsConfig {
    let tokens = model.with_file_name("tokens.txt");
    let data_dir = model
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("espeak-ng-data");
    OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            num_threads: 1,
            vits: OfflineTtsVitsModelConfig {
                model: Some(model.to_string_lossy().into_owned()),
                tokens: Some(tokens.to_string_lossy().into_owned()),
                data_dir: Some(data_dir.to_string_lossy().into_owned()),
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

/// The shared engine for `voice`, loading on first use.
/// `None` means the model failed to load (cached failure).
fn engine_for(voice: VoiceId) -> Option<&'static OfflineTts> {
    // Compatibility: if a custom model path was set, prefer it for the
    // active/default synthesis path (out-of-tree voice bundle).
    if let Some(model) = MODEL_PATH_OVERRIDE.get()
        && (voice == active_voice() || voice == VoiceId::DEFAULT)
    {
        return ENGINE_CUSTOM
            .get_or_init(|| {
                tracing::info!(
                    target: "adapter_synthesis::synth",
                    model = %model.display(),
                    "loading custom voice engine override"
                );
                OfflineTts::create(&custom_voice_config(model))
            })
            .as_ref();
    }
    engine_slot(voice)
        .get_or_init(|| {
            tracing::info!(target: "adapter_synthesis::synth", voice = voice.id(), "loading voice engine");
            OfflineTts::create(&voice_config(voice))
        })
        .as_ref()
}

/// Synthesize one bounded segment using the active/default voice.
///
/// Delegates to [`synthesize_with_voice`] with [`active_voice`].
///
/// # Errors
///
/// See [`synthesize_with_voice`].
pub fn synthesize(segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure> {
    synthesize_with_voice(active_voice(), segment)
}

/// Synthesize one bounded segment to mono `f32` at the declared
/// 22 050 Hz, through the requested bundled catalog voice.
///
/// # Errors
///
/// Always a named [`EngineFailure`] — never an empty or silent
/// success (R2/R7):
///
/// * [`FailureCondition::VoiceInitFailure`] → [`EngineError::InitFailed`]
///   when the voice model fails to load, or when the engine yields an
///   empty (corrupt) result for a non-empty segment;
/// * [`FailureCondition::TransportUnreachable`] → [`EngineError::Unreachable`]
///   when the segment text cannot be marshalled across the C FFI boundary;
/// * [`FailureCondition::ProcessDeath`] → [`EngineError::Unreachable`]
///   when the engine process dies (killed/OOM) mid-generation;
/// * [`FailureCondition::RateDeviation`] → [`EngineError::RateMismatch`]
///   when the delivered rate is not 22 050 Hz.
///
/// The `session_position` echoes the segment's `resume_point` (falling
/// back to its `source_range`) — never adapter-computed.
pub fn synthesize_with_voice(
    voice: VoiceId,
    segment: &DocumentSegment,
) -> Result<Vec<f32>, EngineFailure> {
    let Some(tts) = engine_for(voice) else {
        return Err(named_failure(
            FailureCondition::VoiceInitFailure,
            FailureCondition::VoiceInitFailure.detail(),
            segment,
        ));
    };
    let synth_start = Instant::now();
    // The C FFI boundary is the engine's transport: a `&str` with an
    // interior NUL cannot become a `CString`.
    if segment.content.contains('\0') {
        return Err(named_failure(
            FailureCondition::TransportUnreachable,
            FailureCondition::TransportUnreachable.detail(),
            segment,
        ));
    }
    let spoken = normalize_for_speech(&segment.content, normalization_mode());
    let rate_multiplier = playback_rate();
    let prosody_speed = plan_for_segment(segment).speed();
    let effective_speed = (prosody_speed * rate_multiplier).clamp(0.5, 4.0);
    let Some(audio) = tts.generate_with_config(
        &spoken,
        &GenerationConfig {
            speed: effective_speed,
            ..GenerationConfig::default()
        },
        // No progress callback: the whole buffer is returned at once.
        // The concrete `fn`-pointer type satisfies the `FnMut(&[f32], f32) -> bool`
        // bound so the generic `F` resolves (a bare `None` cannot).
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
    // An empty result for a non-empty segment is a corrupt synthesis result.
    if samples.is_empty() {
        return Err(named_failure(
            FailureCondition::VoiceInitFailure,
            FailureCondition::VoiceInitFailure.detail(),
            segment,
        ));
    }
    let sample_count = samples.len();
    let delivered_rate = audio.sample_rate();
    let rate_hz = match u32::try_from(delivered_rate) {
        Ok(rate) if rate > 0 => rate,
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
    // §6's duration derivation (data-model E2): derived from buffer length, never supplied.
    #[expect(
        clippy::cast_precision_loss,
        reason = "data-model E2 mandates deriving duration_ms as pcm_data.len() / rate_hz * 1000; f64 has no lossless From<usize>; unreachable buffer sizes (>2^53 samples) in practice; sanctioned by API_CONTRACT.md §0.1 entry 4"
    )]
    let duration_ms = sample_count as f64 / f64::from(rate_hz) * 1000.0;
    // The authoritative conformance gate (FR-024b / SC-035).
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
        voice = voice.id(),
        segment_index = segment.segment_index,
        samples = sample_count,
        duration_ms = %format!("{:.1}", duration_ms),
        synth_ms = synth_start.elapsed().as_millis(),
        "segment synthesized"
    );
    Ok(samples.to_vec())
}

/// The [`FailureCondition`] whose [`FailureCondition::variant`] is
/// `error` — inverse of the condition→variant table (`spec.md:86`).
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
/// (falling back to its `source_range`) — never adapter-computed.
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
/// `source_range` — never adapter-computed.
#[must_use]
fn echoed_position(segment: &DocumentSegment) -> SourceRange {
    segment.resume_point.unwrap_or(segment.source_range)
}
