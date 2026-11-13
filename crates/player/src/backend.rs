//! CPAL playback backend.
//!
//! Rust owns synthesis and PCM playback. The backend requests
//! exactly mono `f32` at 22 050 Hz; when the device cannot serve
//! that shape the construction fails closed with a named
//! [`BackendError`] — never a silent resample.
//!
//! Synthesized audio enters through [`CpalBackend::enqueue_pcm`]
//! and is consumed by the output-stream callback in FIFO order;
//! the remainder of a short buffer is silence (`0.0`). The
//! position clock advances only by samples actually consumed
//! while the stream runs.
//!
//! The resident audio is BOUNDED (FR-018; `plan.md:25`): the queue
//! holds at most [`MAX_PREROLL_SECS`] (60 s) of not-yet-played
//! speech — the pre-roll window ahead of the play head — so the
//! whole document's PCM never stays resident for the whole session
//! (~620 MB for the SC-001 fixture: ~2 h of speech at 22 050 Hz
//! mono f32). The bound lives in the player crate, below the app
//! shell, and the diagnostic buffer-state entry carries it
//! ([`CpalBackend::buffer_state`]).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use engine_core::closed::EngineError;
use engine_core::format::{DeclaredFormat, check_handoff};
use thiserror::Error;
use tracing::{error, info, instrument};

use crate::dsp::DspProcessor;
use crate::earcon::EarconKind;
use crate::focus::FULL_GAIN;
use crate::position::PositionClock;

/// Errors from the CPAL backend.
///
/// Every variant is a named, surfaced failure — the backend never
/// fails silently.
#[derive(Debug, Error)]
pub enum BackendError {
    /// No default output device was available.
    #[error("no audio device available: {0}")]
    NoDevice(String),
    /// The output stream could not be built.
    #[error("failed to build stream: {0}")]
    BuildStream(#[from] cpal::Error),
    /// The device refused the declared handoff shape.
    #[error("device refused the declared shape: {0}")]
    DeviceRefused(String),
    /// The handoff shape check named a deviation.
    #[error("handoff format mismatch: {0:?}")]
    HandoffMismatch(EngineError),
    /// Playback start was refused.
    #[error("playback start refused: {0}")]
    StartRefused(String),
    /// The gain factor was outside `[0.0, 1.0]` — never applied;
    /// the previous gain stands.
    #[error("gain must be within [0.0, 1.0], got {0}")]
    InvalidGain(f32),
    /// The queue lock was poisoned when configuring DSP;
    /// the previous configuration stands.
    #[error("queue lock poisoned while configuring DSP")]
    DspLockPoisoned,
}

/// SCAFFOLD (ADR-002 §VII diagnostic extension).
///
/// Template source: `docs/adr/adr-002-diag-§VII.md` + existing `backend.rs`.
/// Typed event kinds `EarconTruncation` / `EarconEmissionFailure`;
/// closed 11-member set; zero catch-all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticEventKind {
    /// E1 earcon truncated: confirmation cut short.
    ///
    /// `samples_dropped` is how many unplayed confirmation samples
    /// were discarded when a newer trigger arrived.
    EarconTruncation {
        /// Number of unplayed confirmation samples dropped.
        samples_dropped: usize,
    },
    /// E4 emission failure: synthesis/short-channel unavailability.
    EarconEmissionFailure {
        /// The underlying error string describing the failure.
        error: String,
    },
}

/// E4 emission failure: synthesis/short-channel unavailability.
/// Transport proceeds; narration never interrupted; at most one
/// re-emission; typed diagnostic (`EarconEmissionFailure`) — never silence.
#[must_use]
pub const fn record_emission_failure(error: String) -> DiagnosticEventKind {
    DiagnosticEventKind::EarconEmissionFailure { error }
}

/// The resident pre-roll bound, in seconds (FR-018; `plan.md:25`).
///
/// The bound on the audio the player keeps RESIDENT — the synthesized
/// PCM that is queued for the device but not yet played. Without it
/// the whole document's PCM stays resident for the whole session
/// (~620 MB for the SC-001 fixture: ~2 h of speech at 22 050 Hz mono
/// f32) and the app runs out of memory on a mid-range phone; with it
/// the resident audio is the pre-roll window ahead of the play head
/// and nothing more.
///
/// The value is the single [`crate::position::PREROLL_CAP_SECS`]
/// (60 s) — an alias, never a second cap that can drift (Principle
/// VII / FR-044). The diagnostic contract names the same bound
/// `MAX_PREROLL_SECS=60` (`contracts/diagnostic-trace.md`).
pub const MAX_PREROLL_SECS: f64 = crate::position::PREROLL_CAP_SECS;

/// The declared engine rate (FR-019): 22 050 Hz — the same value as
/// [`DeclaredFormat::SAMPLE_RATE`] (a unit test pins the two, so the
/// rate never drifts).
const ENGINE_RATE_HZ: f64 = 22_050.0;

/// The resident bound in samples at the declared rate (FR-019:
/// 22 050 Hz mono f32): 60 s × 22 050 = 1 323 000 samples.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "60.0 * 22_050.0 = 1_323_000.0 is exact and fits in usize"
)]
const MAX_PREROLL_SAMPLES: usize = (MAX_PREROLL_SECS * ENGINE_RATE_HZ) as usize;

/// The resident playback queue: the synthesized speech the device has
/// not played yet — the pre-roll window ahead of the play head.
///
/// The queue is BOUNDED (FR-018; `plan.md:25`): it never holds more
/// than [`MAX_PREROLL_SECS`] (60 s) of resident audio, so the whole
/// document's PCM never stays resident for the whole session.
///
/// The drop policy keeps the reading AUDIBLY CONTIGUOUS from the play
/// head: when a chunk would push the resident audio beyond the bound,
/// the audio BEYOND the window is dropped and the audio AT the head
/// is kept — the head is the next audible sample, and dropping it
/// would put a hole in the reading. The dropped audio is the
/// producer's overrun: the part of the document prepared past the
/// window.
///
/// The queue holds NO position: dropping never moves the reported
/// position. The audible clock is the only advance path (FR-002) — a
/// dropped sample is not an audible one.
#[derive(Debug, Default)]
pub struct ResidentQueue {
    /// The resident, not-yet-played samples, oldest first.
    samples: VecDeque<f32>,
    /// The samples dropped because they fell beyond the bound.
    dropped_samples: u64,
}

impl ResidentQueue {
    /// A new, empty queue: nothing resident, nothing dropped.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            dropped_samples: 0,
        }
    }

    /// The resident bound in samples at the declared rate.
    #[must_use]
    pub const fn cap_samples(&self) -> usize {
        MAX_PREROLL_SAMPLES
    }

    /// The resident bound in seconds.
    #[must_use]
    pub const fn cap_secs(&self) -> f64 {
        MAX_PREROLL_SECS
    }

    /// Append synthesized samples to the tail of the resident window.
    ///
    /// The window never grows past the bound: what does not fit is
    /// DROPPED — the audio BEYOND the window, never the audio at the
    /// head. The head is the next audible sample, so dropping it
    /// would put a hole in the reading; the dropped audio is the
    /// producer's overrun — the part of the document prepared past
    /// the window.
    ///
    /// The drop moves no position: a dropped sample is not an audible
    /// one, and the audible clock is the only advance path (FR-002).
    pub fn push(&mut self, pcm: &[f32]) {
        let room = MAX_PREROLL_SAMPLES.saturating_sub(self.samples.len());
        let kept = room.min(pcm.len());
        self.samples.extend(pcm[..kept].iter().copied());
        let dropped = pcm.len() - kept;
        if dropped > 0 {
            self.dropped_samples = self
                .dropped_samples
                .saturating_add(u64::try_from(dropped).unwrap_or(u64::MAX));
        }
    }

    /// The resident sample count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether no audio is resident.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The resident audio in seconds — the pre-roll depth the
    /// diagnostic buffer-state entry carries.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the resident sample count is a count of samples, far below f64's 52-bit mantissa"
    )]
    #[must_use]
    pub fn buffered_secs(&self) -> f64 {
        self.samples.len() as f64 / ENGINE_RATE_HZ
    }

    /// The samples dropped because they fell beyond the bound.
    #[must_use]
    pub const fn dropped_samples(&self) -> u64 {
        self.dropped_samples
    }

    /// The resident samples, oldest first — the introspection view of
    /// the window (the diagnostic buffer state and the property tests
    /// read it; the device's consumption path takes the same samples
    /// from the head).
    #[must_use]
    pub const fn samples(&self) -> &VecDeque<f32> {
        &self.samples
    }

    /// Take the next resident sample — the device's consumption path
    /// (the head of the window: the next audible sample).
    pub fn pop_front(&mut self) -> Option<f32> {
        self.samples.pop_front()
    }

    /// Drop every resident sample; the dropped counter is untouched
    /// (it is the session's cumulative count, not the queue's depth).
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// The resident buffer state (FR-017; `contracts/diagnostic-trace.md`).
///
/// The diagnostic buffer-state entry carries the pre-roll depth — the
/// resident, not-yet-played audio — against the bound it never
/// exceeds, so the bound is observable in the record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferState {
    /// Resident, not-yet-played audio in seconds.
    pub buffered_secs: f64,
    /// The bound it never exceeds, in seconds.
    pub cap_secs: f64,
    /// Cumulative samples dropped beyond the bound.
    pub dropped_samples: u64,
}

impl BufferState {
    /// Empty buffer state.
    pub const EMPTY: Self = Self {
        buffered_secs: 0.0,
        cap_secs: MAX_PREROLL_SECS,
        dropped_samples: 0,
    };
}

/// FIFO of mono `f32` samples at the declared rate, shared with
/// the stream callback.
///
/// Carries the output gain factor (the FR-013 duck actuator):
/// every audible sample is scaled by it on the way out, so a
/// duck or restore applies to already-queued audio too. The
/// silence written for an empty queue stays `0.0` regardless
/// of the gain.
#[derive(Debug)]
struct SampleQueue {
    /// The resident, not-yet-played speech — bounded by
    /// [`MAX_PREROLL_SECS`] (FR-018; `plan.md:25`).
    speech: ResidentQueue,
    /// Output gain in `[0.0, 1.0]`; full gain by default, so a
    /// fresh queue is bit-identical to the unscaled path.
    gain: f32,
    /// Duck-exempt short-mix buffer (contracts/earcon.md): the
    /// procedural earcon PCM rides the SAME stream and the
    /// SAME clock as speech, but is never scaled by the gain —
    /// a nav-duck never quiets a confirmation. Consumed
    /// independently of the speech queue and never counted as
    /// audible speech, so it can never advance the position
    /// bookmark.
    short_mix: VecDeque<f32>,
    /// The kind of the earcon currently in the short-mix buffer
    /// (if any), so a SYSTEM earcon can interrupt a USER one
    /// but a USER trigger never interrupts an in-flight SYSTEM
    /// confirmation (E2.3).
    short_mix_kind: Option<EarconKind>,
    /// RMS of the most recently consumed speech block (0.0–1.0).
    /// Updated lock-free in `consume_into`; read by the frontend
    /// via `CpalBackend::rms()`.
    rms: AtomicU32,
    /// DSP post-processor applied to speech samples before gain.
    dsp: DspProcessor,
}

impl Default for SampleQueue {
    fn default() -> Self {
        Self {
            speech: ResidentQueue::new(),
            gain: FULL_GAIN,
            short_mix: VecDeque::new(),
            short_mix_kind: None,
            rms: AtomicU32::new(0.0f32.to_bits()),
            dsp: DspProcessor::new(DeclaredFormat::SAMPLE_RATE),
        }
    }
}

impl SampleQueue {
    /// Append synthesized samples to the tail.
    fn push(&mut self, pcm: &[f32]) {
        self.speech.push(pcm);
    }

    /// Append duck-exempt earcon PCM to the short-mix buffer
    /// (contracts/earcon.md): mixed at full gain on the same
    /// stream, never scaled by the duck actuator.
    fn push_short_mix(&mut self, kind: EarconKind, pcm: &[f32]) {
        self.short_mix.extend(pcm.iter().copied());
        self.short_mix_kind = Some(kind);
    }

    /// Truncate the tail of any in-flight earcon (short-mix
    /// buffer) BEFORE appending the latest trigger — E2
    /// "latest wins": the remaining unplayed samples of the
    /// previous confirmation are dropped so the new action's
    /// first sample is the next consumed one (≤100 ms,
    /// FR-010). The short-mix buffer holds only earcon PCM,
    /// so clearing it never touches speech or the position
    /// clock.
    fn truncate_short_mix_before_append(&mut self) {
        self.short_mix.clear();
        self.short_mix_kind = None;
    }

    /// The currently applied output gain factor.
    #[must_use]
    pub(crate) const fn gain(&self) -> f32 {
        self.gain
    }

    /// Set the output gain, accepting `[0.0, 1.0]` and rejecting
    /// negatives, factors above `1.0`, infinities, and `NaN`
    /// with the offending factor carried in the error. A
    /// rejection leaves the previous gain untouched.
    fn set_gain(&mut self, factor: f32) -> Result<(), BackendError> {
        if factor.is_finite() && (0.0..=1.0).contains(&factor) {
            self.gain = factor;
            Ok(())
        } else {
            Err(BackendError::InvalidGain(factor))
        }
    }

    /// Move up to `out.len()` samples into `out`, scaling each
    /// audible speech sample by the current gain, mixing in the
    /// duck-exempt short-mix (earcon) samples at full gain, and
    /// filling the remainder with silence. Returns the count of
    /// SPEECH samples consumed — the only count that may advance
    /// the position clock (contracts/earcon.md: "Must not
    /// advance the position bookmark — clock counts speech
    /// samples only").
    fn consume_into(&mut self, out: &mut [f32]) -> usize {
        let gain = self.gain;
        let mut speech_consumed = 0usize;
        let mut sum_sq: f64 = 0.0;
        let mut count: usize = 0;
        for slot in out.iter_mut() {
            let speech = self.speech.pop_front().map(|s| self.dsp.process_sample(s));
            let earcon = self.short_mix.pop_front();
            match (speech, earcon) {
                (Some(sample), Some(tone)) => {
                    *slot = sample.mul_add(gain, tone);
                    let s = f64::from(sample);
                    sum_sq = f64::mul_add(s, s, sum_sq);
                    count += 1;
                    speech_consumed += 1;
                }
                (Some(sample), None) => {
                    *slot = sample * gain;
                    let s = f64::from(sample);
                    sum_sq = f64::mul_add(s, s, sum_sq);
                    count += 1;
                    speech_consumed += 1;
                }
                (None, Some(tone)) => {
                    // Duck-exempt: the confirmation plays at full
                    // gain even while the speech stream is ducked.
                    *slot = tone;
                }
                (None, None) => *slot = 0.0,
            }
        }
        if self.short_mix.is_empty() {
            self.short_mix_kind = None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            reason = "rms is a f32 magnitude, precision loss acceptable"
        )]
        let rms = if count > 0 {
            ((sum_sq / count as f64).sqrt()) as f32
        } else {
            0.0
        };
        self.rms.store(rms.to_bits(), Ordering::Relaxed);
        speech_consumed
    }

    /// Return the RMS of the most recently consumed speech block.
    #[must_use]
    fn rms(&self) -> f32 {
        f32::from_bits(self.rms.load(Ordering::Relaxed))
    }
    /// Drop every buffered sample, speech and short-mix alike:
    /// a pause or stop stands the confirmation down too (the
    /// trigger path re-fires it once Playing is restored, per
    /// contracts/earcon.md).
    fn clear(&mut self) {
        self.speech.clear();
        self.short_mix.clear();
        self.short_mix_kind = None;
    }

    /// Buffered speech sample count.
    #[must_use]
    fn len(&self) -> usize {
        self.speech.len()
    }

    /// Set the DSP configuration on the speech path.
    ///
    /// Switching presets resets internal state to avoid clicks.
    pub(crate) fn set_dsp_config(&mut self, config: crate::dsp::DspConfig) {
        self.dsp.set_config(config);
    }

    /// Return the current DSP configuration.
    #[must_use]
    pub(crate) const fn dsp_config(&self) -> crate::dsp::DspConfig {
        crate::dsp::DspConfig {
            preset: self.dsp.preset(),
        }
    }
}

/// The CPAL backend driver.
///
/// Owns the output stream, the sample queue, and the position
/// clock. The declared rate is a constant of the contract, so the
/// elapsed-time divisor never drifts from it.
pub struct CpalBackend {
    /// The running output stream (`None` is unrepresentable after
    /// [`CpalBackend::new`] succeeds; the option exists so
    /// [`Drop`] can pause exactly once).
    stream: Option<cpal::Stream>,
    /// Samples waiting for the callback.
    queue: Arc<Mutex<SampleQueue>>,
    /// Audible-progress clock.
    position_clock: Arc<Mutex<PositionClock>>,
    /// The stream sample rate (always the declared rate).
    sample_rate: u32,
}

impl std::fmt::Debug for CpalBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalBackend")
            .field("queue", &self.queue)
            .field("position_clock", &self.position_clock)
            .field("sample_rate", &self.sample_rate)
            .finish_non_exhaustive()
    }
}

impl CpalBackend {
    /// Open the default output device at exactly the declared
    /// handoff shape and build (not yet start) the stream.
    ///
    /// # Errors
    ///
    /// * [`BackendError::NoDevice`] when no default output device
    ///   exists or its configs cannot be listed.
    /// * [`BackendError::DeviceRefused`] when no supported config
    ///   offers mono `f32` covering 22 050 Hz.
    /// * [`BackendError::HandoffMismatch`] when the negotiated
    ///   shape deviates from the declaration.
    /// * [`BackendError::BuildStream`] when the stream cannot be
    ///   built.
    #[instrument]
    pub fn new() -> Result<Self, BackendError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| BackendError::NoDevice("no default output device".to_string()))?;

        let mut negotiated: Option<StreamConfig> = None;
        let supported = device
            .supported_output_configs()
            .map_err(|e| BackendError::NoDevice(format!("failed to query configs: {e}")))?;
        for config in supported {
            if config.channels() == DeclaredFormat::CHANNELS
                && config.sample_format() == SampleFormat::F32
                && config.min_sample_rate() <= DeclaredFormat::SAMPLE_RATE
                && config.max_sample_rate() >= DeclaredFormat::SAMPLE_RATE
            {
                negotiated = Some(
                    config
                        .with_sample_rate(DeclaredFormat::SAMPLE_RATE)
                        .config(),
                );
                break;
            }
        }
        let config = negotiated.ok_or_else(|| {
            BackendError::DeviceRefused(
                "no supported config offers mono f32 at 22050 Hz".to_string(),
            )
        })?;

        // The width is 32 by construction: only `F32` configs
        // reach this point, and `Vec<f32>` is the only sample
        // carrier the queue accepts.
        check_handoff(
            config.sample_rate,
            config.channels,
            DeclaredFormat::SAMPLE_WIDTH_BITS,
        )
        .map_err(BackendError::HandoffMismatch)?;

        let sample_rate = config.sample_rate;
        let queue = Arc::new(Mutex::new(SampleQueue::default()));
        let position_clock = Arc::new(Mutex::new(PositionClock::new(0.0)));

        let queue_cb = Arc::clone(&queue);
        let clock_cb = Arc::clone(&position_clock);
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let consumed = if let Ok(mut q) = queue_cb.lock() {
                        q.consume_into(data)
                    } else {
                        for slot in data.iter_mut() {
                            *slot = 0.0;
                        }
                        0
                    };
                    if consumed > 0 {
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "elapsed time needs f64 division; a >2^53-sample callback buffer \
                                      (over 6 millennia of audio at 22 050 Hz) is unreachable in memory"
                        )]
                        let elapsed = consumed as f64 / f64::from(sample_rate);
                        if let Ok(mut clock) = clock_cb.lock() {
                            let _ignored = clock.on_audible_progress(elapsed);
                        }
                    }
                },
                |err| error!(error = %err, "audio stream error"),
                None,
            )
            .map_err(BackendError::BuildStream)?;

        info!(sample_rate, "stream built at declared shape");

        Ok(Self {
            stream: Some(stream),
            queue,
            position_clock,
            sample_rate,
        })
    }

    /// Append synthesized mono `f32` samples at the declared rate
    /// to the playback queue.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the app-shell caller hands over its owned synthesis buffer in an ownership transfer; the queue copies the samples immediately, but taking &[f32] would change the app-shell call signature for no behavioral difference"
    )]
    pub fn enqueue_pcm(&self, pcm: Vec<f32>) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(&pcm);
        }
    }

    /// Return the RMS of the most recently consumed speech block.
    ///
    /// Lock-free read of the atomic store updated by the output
    /// stream callback. Returns `0.0` when no speech has been
    /// consumed yet.
    #[must_use]
    pub fn rms(&self) -> f32 {
        self.queue.lock().map_or(0.0, |q| q.rms())
    }

    /// Enqueue duck-exempt earcon PCM (contracts/earcon.md):
    /// the procedural confirmation rides the same stream and
    /// the same clock as speech, is mixed at FULL gain even
    /// while the speech stream is ducked, files no
    /// `AudioFocusRequest`, and never advances the position
    /// bookmark (the clock counts speech samples only).
    pub fn enqueue_earcon(&self, kind: EarconKind, pcm: &[f32]) {
        if let Ok(mut queue) = self.queue.lock() {
            let incoming_system = matches!(kind, EarconKind::Warning | EarconKind::Failure);
            let in_flight_system = queue
                .short_mix_kind
                .is_some_and(|k| matches!(k, EarconKind::Warning | EarconKind::Failure));
            if incoming_system || !in_flight_system {
                queue.truncate_short_mix_before_append();
                queue.push_short_mix(kind, pcm);
            }
        }
    }

    /// Buffered sample count not yet consumed by the stream.
    #[must_use]
    pub fn buffered_samples(&self) -> usize {
        self.queue.lock().map_or(0, |q| q.len())
    }

    /// The resident buffer state (FR-017): the pre-roll depth against
    /// the bound it never exceeds, plus the samples dropped beyond it
    /// — the source of the diagnostic buffer-state entry
    /// (`contracts/diagnostic-trace.md`).
    #[must_use]
    pub fn buffer_state(&self) -> BufferState {
        self.queue
            .lock()
            .map_or(BufferState::EMPTY, |queue| BufferState {
                buffered_secs: queue.speech.buffered_secs(),
                cap_secs: MAX_PREROLL_SECS,
                dropped_samples: queue.speech.dropped_samples(),
            })
    }

    /// Set the output gain factor (the FR-013 duck actuator).
    ///
    /// Every factor in `[0.0, 1.0]` — including silence `0.0`,
    /// the nominal duck factor (`0.20`) and full gain (`1.0`)
    /// — is accepted and scales every subsequently consumed
    /// sample, including audio already queued. A rejected
    /// factor leaves the previous gain untouched.
    ///
    /// # Errors
    ///
    /// [`BackendError::InvalidGain`] carrying the offending
    /// factor when it is negative, above `1.0`, infinite, or
    /// `NaN` — or when the queue lock is poisoned (unreachable
    /// while panics are denied); the previous gain stands in
    /// every rejection case.
    pub fn set_gain(&self, factor: f32) -> Result<(), BackendError> {
        self.queue
            .lock()
            .map_or(Err(BackendError::InvalidGain(factor)), |mut queue| {
                queue.set_gain(factor)
            })
    }

    /// The currently applied output gain: full gain (`1.0`) on
    /// a fresh backend, otherwise the last accepted
    /// [`set_gain`](Self::set_gain) factor, echoed exactly.
    #[must_use]
    pub fn gain(&self) -> f32 {
        self.queue.lock().map_or(FULL_GAIN, |queue| queue.gain())
    }

    /// Set the DSP configuration applied to speech samples before gain.
    ///
    /// Switching presets resets internal state to avoid clicks.
    ///
    /// # Errors
    ///
    /// [`BackendError::DspLockPoisoned`] when the queue lock is
    /// poisoned (unreachable while panics are denied).
    pub fn set_dsp_config(&self, config: crate::dsp::DspConfig) -> Result<(), BackendError> {
        self.queue
            .lock()
            .map_or(Err(BackendError::DspLockPoisoned), |mut queue| {
                queue.set_dsp_config(config);
                Ok(())
            })
    }

    /// Return the current DSP configuration applied to speech.
    #[must_use]
    pub fn dsp_config(&self) -> crate::dsp::DspConfig {
        self.queue.lock().map_or_else(
            |_| crate::dsp::DspConfig::default(),
            |queue| queue.dsp_config(),
        )
    }

    /// Start (or resume) playback.
    ///
    /// # Errors
    ///
    /// [`BackendError::StartRefused`] when the stream refuses to
    /// start.
    #[instrument(skip(self))]
    pub fn play(&self) -> Result<(), BackendError> {
        if let Some(stream) = &self.stream {
            stream
                .play()
                .map_err(|e| BackendError::StartRefused(e.to_string()))?;
            info!("playback started");
            Ok(())
        } else {
            Err(BackendError::StartRefused("no stream".to_string()))
        }
    }

    /// Pause playback, keeping the queue and position.
    #[instrument(skip(self))]
    pub fn pause(&self) {
        if let Some(stream) = &self.stream {
            let _ignored = stream.pause();
            info!("playback paused");
        }
    }

    /// Stop playback, drop queued audio, and reset position.
    #[instrument(skip(self))]
    pub fn stop(&self) {
        if let Some(stream) = &self.stream {
            let _ignored = stream.pause();
        }
        if let Ok(mut queue) = self.queue.lock() {
            // The short mix stands down with the speech queue: a
            // stop or pause drops any standing confirmation.
            queue.clear();
        }
        if let Ok(mut clock) = self.position_clock.lock() {
            *clock = PositionClock::new(0.0);
        }
        info!("playback stopped, queue cleared, position reset");
    }

    /// Current playback position in seconds.
    #[must_use]
    pub fn position_secs(&self) -> f64 {
        self.position_clock
            .lock()
            .map_or(0.0, |clock| clock.position_secs())
    }

    /// Whether every enqueued speech sample has been consumed by
    /// the stream — the end-of-document drain signal (F1).
    ///
    /// The play path enqueues the whole document's PCM in one
    /// call before the stream starts, so an empty speech queue
    /// with a non-zero audible position means the stream consumed
    /// everything enqueued. A fresh backend (nothing played,
    /// position at zero) and a stopped one (queue dropped,
    /// position reset together) read `false`; a paused backend
    /// keeps its unconsumed samples and reads `false` too. The
    /// earcon short mix never counts: the drain is about the
    /// speech queue the position clock advances on.
    #[must_use]
    pub fn is_drained(&self) -> bool {
        drained(self.buffered_samples(), self.position_secs())
    }

    /// The stream sample rate (always the declared rate).
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Drop for CpalBackend {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ignored = stream.pause();
        }
    }
}

/// The drain predicate [`CpalBackend::is_drained`] pins: an empty
/// speech queue after audible progress. Both needles are
/// load-bearing — the empty queue alone also matches a fresh or
/// stopped backend, and the non-zero position alone matches a
/// paused-at-progress one.
#[must_use]
fn drained(buffered_samples: usize, position_secs: f64) -> bool {
    buffered_samples == 0 && position_secs > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a queue, push N samples, consume them all.
    #[test]
    fn consume_into_returns_speech_count() {
        let mut q = SampleQueue::default();
        let samples: Vec<f32> = (0..100)
            .map(|i| {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "test sample index < 100, well within f32 exact integer range"
                )]
                let f = i as f32 * 0.01;
                f
            })
            .collect();
        q.push(&samples);

        let mut out = [0.0f32; 50];
        let n = q.consume_into(&mut out);
        assert_eq!(n, 50);
        // Off preset: pass-through unchanged.
        for i in 0..50 {
            assert_eq!(out[i], samples[i]);
        }
    }

    #[test]
    fn consume_into_applies_gain() {
        let mut q = SampleQueue::default();
        assert!(q.set_gain(0.5).is_ok());
        q.push(&[1.0, 0.0, -1.0, 0.0]);

        let mut out = [0.0f32; 4];
        let n = q.consume_into(&mut out);
        assert_eq!(n, 4);
        assert_eq!(out[0], 0.5);
        assert_eq!(out[1], 0.0);
        assert_eq!(out[2], -0.5);
        assert_eq!(out[3], 0.0);
    }

    #[test]
    fn dsp_off_is_bit_identical_through_queue() {
        let mut q = SampleQueue::default();
        q.push(&[0.3, -0.7, 1.0, -1.0, 0.123_456]);
        let mut out = [0.0f32; 5];
        let n = q.consume_into(&mut out);
        assert_eq!(n, 5);
        for i in 0..5 {
            assert_eq!(out[i], [0.3, -0.7, 1.0, -1.0, 0.123_456][i]);
        }
    }

    #[test]
    fn earcon_stays_full_gain_with_dsp_on() {
        let mut q = SampleQueue::default();
        // Speech alone: gain applied (default full gain).
        q.push(&[0.5]);
        let mut out = [0.0f32; 1];
        let _n = q.consume_into(&mut out);
        assert_eq!(out[0], 0.5);
        // Earcon alone: full gain, no DSP.
        q.push_short_mix(EarconKind::Play, &[1.0]);
        let mut out = [0.0f32; 1];
        let _n = q.consume_into(&mut out);
        assert_eq!(out[0], 1.0);
        // Both together: speech*gain + earcon (mixed).
        q.push(&[0.5]);
        q.push_short_mix(EarconKind::Play, &[1.0]);
        let mut out = [0.0f32; 1];
        let _n = q.consume_into(&mut out);
        assert_eq!(out[0], 1.5);
    }

    #[test]
    fn bounded_queue_drops_beyond_cap() {
        let mut q = SampleQueue::default();
        // Push way beyond the cap.
        let big: Vec<f32> = vec![1.0; MAX_PREROLL_SAMPLES * 2];
        q.push(&big);
        assert_eq!(q.len(), MAX_PREROLL_SAMPLES);
    }

    #[test]
    fn set_dsp_config_changes_preset() {
        let mut q = SampleQueue::default();
        q.set_dsp_config(crate::dsp::DspConfig {
            preset: crate::dsp::DspPreset::SonaEthereal,
        });
        assert_eq!(q.dsp_config().preset, crate::dsp::DspPreset::SonaEthereal);
        q.set_dsp_config(crate::dsp::DspConfig {
            preset: crate::dsp::DspPreset::Off,
        });
        assert_ne!(q.dsp_config().preset, crate::dsp::DspPreset::SonaEthereal);
    }
}
