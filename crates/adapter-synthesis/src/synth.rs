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
//! time from `CARGO_MANIFEST_DIR`; nothing is fetched from the
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

/// The Piper `faber` VITS model — mono, 22 050 Hz, CC0
/// (`MODEL_CARD`; blake3 recorded in `PROVENANCE.md`).
const MODEL_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/voices/pt_BR-faber-medium/pt_BR-faber-medium.onnx",
);

/// The token map the `faber` model was trained with.
const TOKENS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/voices/pt_BR-faber-medium/tokens.txt",
);

/// The espeak-ng data tree the Piper phonemizer resolves pt-BR
/// phonemes against at synthesis time.
const ESPEAK_DATA_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/voices/pt_BR-faber-medium/espeak-ng-data",
);

/// Build the `sherpa-onnx` VITS configuration for the embedded
/// `faber` voice. The inference parameters mirror the model's own
/// `pt_BR-faber-medium.onnx.json` (`noise_scale` 0.667, `noise_w`
/// 0.8, `length_scale` 1.0); `provider` stays `None` for the
/// default ONNX Runtime execution provider.
///
/// `num_threads` is pinned to `1`: a single-threaded ONNX Runtime
/// session is resource-predictable for an embedded offline engine —
/// it spawns no uncontrolled per-session thread pool that would
/// contend with the playback, focus and notification threads of the
/// driving shell — and the pre-roll architecture (FR-015a) means
/// synthesis runs ahead of real-time, so single-threaded inference
/// keeps up comfortably within the 60 s pre-roll bound.
fn voice_config() -> OfflineTtsConfig {
    OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            num_threads: 1,
            vits: OfflineTtsVitsModelConfig {
                model: Some(MODEL_PATH.to_owned()),
                tokens: Some(TOKENS_PATH.to_owned()),
                data_dir: Some(ESPEAK_DATA_DIR.to_owned()),
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

/// The lazily-constructed engine. The model loads exactly once —
/// not per call, so the property tests (which synthesize hundreds
/// of segments) pay one load, not hundreds (T044). A failed load
/// is cached as `None`: the cause is deterministic (missing or
/// corrupt voice assets), so every later call re-reports it — with
/// the *current* segment's echoed position, not the first call's.
static ENGINE: OnceLock<Option<OfflineTts>> = OnceLock::new();

/// The shared engine, loading the voice model on first use.
/// `None` means the model failed to load.
fn engine() -> Option<&'static OfflineTts> {
    ENGINE
        .get_or_init(|| OfflineTts::create(&voice_config()))
        .as_ref()
}

/// Synthesize one bounded segment to mono `f32` at the declared
/// 22 050 Hz, through the embedded Piper pt-BR `faber` voice.
///
/// The engine is loaded once (see [`engine`]) and `segment.content`
/// is synthesized with the VITS model. On success the delivered
/// buffer is non-empty, mono, `f32`, at 22 050 Hz, and is verified
/// against the declared handoff shape with [`check_delivery`]
/// before it is returned — any deviation names a member of the
/// closed [`EngineError`] set instead of being silently resampled
/// or downmixed (FR-024b / SC-035).
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
pub fn synthesize(segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure> {
    let Some(tts) = engine() else {
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
    if segment.content.contains('\0') {
        return Err(named_failure(
            FailureCondition::TransportUnreachable,
            FailureCondition::TransportUnreachable.detail(),
            segment,
        ));
    }
    let Some(audio) = tts.generate_with_config(
        &segment.content,
        &GenerationConfig::default(),
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
    // The Piper `faber` model is mono at 22 050 Hz by construction,
    // so the channel count (mono) and the width (`f32`, 32-bit) are
    // type-level facts passed to the gate as declared.
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
    Ok(samples.to_vec())
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
