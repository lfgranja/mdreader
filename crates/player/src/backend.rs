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
}
/// SCAFFOLD (ADR-002 §VII diagnostic extension).
///
/// Template source: `docs/adr/adr-002-diag-§VII.md` + existing `backend.rs`.
/// Typed event kinds `EarconTruncation` / `EarconEmissionFailure`;
/// closed 11-member set; zero catch-all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticEventKind {
    /// Truncation of an in-flight user earcon's tail before the
    /// latest trigger (E2 "latest wins"); counted once per
    /// truncation in the diag record (`EarconTruncation`).
    EarconTruncation,
    /// Typed emission-failure event (E4): synthesis error or
    /// short-channel unavailability; `error` carries the
    /// typed reason; the transport action still proceeds.
    EarconEmissionFailure {
        /// Typed reason for the emission failure.
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
    reason = "MAX_PREROLL_SECS × 22 050 = 1 323 000 is an exact integer, far below usize range"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "MAX_PREROLL_SECS and the declared rate are positive constants, so their product is positive — no sign bit is lost in the cast"
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
    /// The resident, not-yet-played audio in seconds (the pre-roll
    /// depth).
    pub buffered_secs: f64,
    /// The bound the resident audio never exceeds, in seconds.
    pub cap_secs: f64,
    /// The samples dropped because they fell beyond the bound.
    pub dropped_samples: u64,
}

impl BufferState {
    /// The state of a queue with nothing resident.
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
}

impl Default for SampleQueue {
    fn default() -> Self {
        Self {
            speech: ResidentQueue::new(),
            gain: FULL_GAIN,
            short_mix: VecDeque::new(),
            short_mix_kind: None,
            rms: AtomicU32::new(0.0f32.to_bits()),
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

    /// The currently applied output gain.
    const fn gain(&self) -> f32 {
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
            let speech = self.speech.pop_front();
            let earcon = self.short_mix.pop_front();
            match (speech, earcon) {
                (Some(sample), Some(tone)) => {
                    *slot = sample.mul_add(gain, tone);
                    let s = f64::from(sample);
                    sum_sq += s * s;
                    count += 1;
                    speech_consumed += 1;
                }
                (Some(sample), None) => {
                    *slot = sample * gain;
                    let s = f64::from(sample);
                    sum_sq += s * s;
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
    ///
    /// The caller (the T029 trigger path) drops and re-fires
    /// the confirmation when a pause stands; a pause or stop
    /// through [`pause`](Self::pause) / [`stop`](Self::stop)
    /// stands the short mix down with the speech queue.
    /// E2 burst/substitute: in-flight USER earcon is cut (tail
    /// dropped) BEFORE the new append so the latest triggered
    /// action keeps audible confirmation (≤100 ms per FR-010).
    /// SYSTEM earcons (Ostinato/Sforzando) interrupt any
    /// in-flight user confirmation immediately.
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
    use crate::focus::DUCK_GAIN;

    // Exact-value contract: the queue copies samples verbatim along the empty and unity paths and
    // every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance
    // would admit a corrupted sample
    #[test]
    fn test_earcon_truncation_diag() {
        // Truncation of an in-flight user earcon registers the typed
        // `EarconTruncation` diagnostic kind.
        let truncation = DiagnosticEventKind::EarconTruncation;
        assert_eq!(truncation, DiagnosticEventKind::EarconTruncation);

        // An emission failure registers `EarconEmissionFailure` carrying
        // the typed `error` field.
        let failure = record_emission_failure(String::from("synthesis_error"));
        let is_failure = matches!(failure, DiagnosticEventKind::EarconEmissionFailure { .. });
        assert!(is_failure, "expected EarconEmissionFailure");
        if let DiagnosticEventKind::EarconEmissionFailure { error } = failure {
            assert_eq!(error, "synthesis_error");
        }
    }

    #[test]
    fn an_empty_queue_yields_silence_and_zero_consumed() {
        let mut queue = SampleQueue::default();
        assert_eq!(queue.len(), 0);
        let mut out = [1.0_f32; 4];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 0);
        assert_eq!(out, [0.0; 4]);
    }

    // Exact-value contract: the queue copies samples verbatim along the empty and unity paths and
    // every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance
    // would admit a corrupted sample
    #[test]
    fn queued_samples_come_out_fifo_with_silence_tail() {
        let mut queue = SampleQueue::default();
        queue.push(&[0.5, -0.25, 0.125]);
        assert_eq!(queue.len(), 3);
        let mut out = [0.0_f32; 5];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 3);
        assert_eq!(out, [0.5, -0.25, 0.125, 0.0, 0.0]);
        assert_eq!(queue.len(), 0);
    }

    // Exact-value contract: the queue copies samples verbatim along the empty and unity paths and
    // every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance
    // would admit a corrupted sample
    #[test]
    fn consumption_shorter_than_queued_leaves_the_rest() {
        let mut queue = SampleQueue::default();
        queue.push(&[1.0, 2.0, 3.0, 4.0]);
        let mut out = [0.0_f32; 2];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 2);
        assert_eq!(out, [1.0, 2.0]);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn an_empty_queue_without_progress_is_not_drained() {
        // The fresh backend and the stopped one both read
        // `(empty, 0.0)`: nothing consumed yet, and a stop must
        // never look like an end-of-document drain.
        assert!(
            !drained(0, 0.0),
            "no audible progress yet — not an end-of-document drain"
        );
    }

    #[test]
    fn buffered_samples_hold_the_drain_back() {
        assert!(
            !drained(5, 3.0),
            "unconsumed speech keeps the drain signal false"
        );
    }

    #[test]
    fn an_empty_queue_after_progress_is_drained() {
        assert!(
            drained(0, 10.36),
            "an emptied speech queue after audible progress is the drain"
        );
    }

    // Exact-value contract: the default gain must be exactly FULL_GAIN; == pins the 1.0 default and
    // a tolerance would admit drift
    #[test]
    fn a_fresh_queue_reads_full_gain() {
        let queue = SampleQueue::default();
        assert_eq!(queue.gain(), FULL_GAIN);
    }

    // Exact-value contract: the queue copies samples verbatim along the empty and unity paths and
    // every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance
    // would admit a corrupted sample
    #[test]
    fn unity_gain_leaves_samples_bit_identical() {
        let mut queue = SampleQueue::default();
        queue.push(&[0.5, -0.25, 0.125]);
        let mut out = [0.0_f32; 5];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 3);
        assert_eq!(out, [0.5, -0.25, 0.125, 0.0, 0.0]);
    }

    #[test]
    fn duck_gain_scales_audible_samples_within_epsilon() {
        let mut queue = SampleQueue::default();
        assert!(queue.set_gain(DUCK_GAIN).is_ok());
        queue.push(&[1.0, -0.5, 0.25]);
        let mut out = [0.0_f32; 5];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 3);
        let expected = [0.2_f32, -0.1, 0.05, 0.0, 0.0];
        for (index, (got, want)) in out.iter().zip(expected.iter()).enumerate() {
            let drift = (*got - *want).abs();
            assert!(
                drift <= 1e-6,
                "sample {index} drifted by {drift}: got {got}, want {want}"
            );
        }
    }

    // Exact-value contract: the queue copies samples verbatim along the empty and unity paths and
    // every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance
    // would admit a corrupted sample
    #[test]
    fn duck_gain_leaves_silence_silent() {
        let mut queue = SampleQueue::default();
        assert!(queue.set_gain(DUCK_GAIN).is_ok());
        let mut out = [1.0_f32; 4];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 0);
        assert_eq!(out, [0.0; 4]);
    }

    // Exact-value contract: boundary gains must echo bit-for-bit and rejections must leave the
    // stored gain bit-for-bit intact; == pins the echo/unchanged laws and a tolerance would admit
    // drift
    #[test]
    fn boundary_gains_are_accepted_and_echoed() {
        let mut queue = SampleQueue::default();
        for factor in [0.0_f32, DUCK_GAIN, FULL_GAIN] {
            assert!(
                queue.set_gain(factor).is_ok(),
                "gain factor {factor} inside [0.0, 1.0] was rejected"
            );
            assert_eq!(
                queue.gain(),
                factor,
                "gain did not echo the accepted factor {factor}"
            );
        }
    }

    // Exact-value contract: a rejection must leave the stored gain bit-for-bit intact; == pins the
    // unchanged law and a tolerance would admit drift
    #[test]
    fn invalid_gains_are_rejected_with_gain_unchanged() {
        let mut queue = SampleQueue::default();
        assert!(queue.set_gain(0.5).is_ok());
        for factor in [
            -1.0_f32,
            -10.0,
            1.000_001,
            10.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ] {
            assert!(
                matches!(queue.set_gain(factor), Err(BackendError::InvalidGain(_))),
                "gain factor {factor} outside [0.0, 1.0] was not rejected with the named InvalidGain error"
            );
            assert_eq!(
                queue.gain(),
                0.5,
                "a rejected gain factor {factor} moved the stored gain"
            );
        }
    }

    #[test]
    fn rejection_carries_the_offending_factor() {
        let mut queue = SampleQueue::default();
        assert!(
            matches!(
                queue.set_gain(-0.5),
                Err(BackendError::InvalidGain(got)) if (got + 0.5).abs() <= f32::EPSILON
            ),
            "rejection did not carry the offending factor"
        );
        assert!(
            matches!(
                queue.set_gain(f32::NAN),
                Err(BackendError::InvalidGain(got)) if got.is_nan()
            ),
            "NaN rejection did not carry the offending factor"
        );
    }

    // Exact-value contract: the bound is the plan's 60 s and the single
    // PREROLL_CAP_SECS the pre-roll ring and the pipeline-side buffer already
    // share; == pins the exact value and a tolerance would admit drift
    #[test]
    fn the_bound_is_the_single_preroll_cap() {
        assert_eq!(MAX_PREROLL_SECS, 60.0);
        assert_eq!(MAX_PREROLL_SECS, crate::position::PREROLL_CAP_SECS);
    }

    // Exact-value contract: the engine rate is the declared 22 050 Hz; == pins
    // the exact value and a tolerance would admit drift
    #[test]
    fn the_engine_rate_is_the_declared_rate() {
        assert_eq!(ENGINE_RATE_HZ, f64::from(DeclaredFormat::SAMPLE_RATE));
    }

    // Exact-value contract: the sample bound is the seconds bound at the
    // declared rate — 60 s x 22 050 Hz = 1 323 000 samples; == pins the exact
    // integer and a tolerance would admit drift
    #[test]
    fn the_sample_bound_is_the_seconds_bound_at_the_declared_rate() {
        assert_eq!(MAX_PREROLL_SAMPLES, 1_323_000);
    }

    #[test]
    fn the_resident_window_holds_at_most_the_bound() {
        let mut queue = ResidentQueue::new();
        let chunk = vec![0.5_f32; MAX_PREROLL_SAMPLES + 1_000];
        queue.push(&chunk);
        assert_eq!(queue.len(), MAX_PREROLL_SAMPLES);
        assert_eq!(queue.buffered_secs(), 60.0);
        assert_eq!(queue.dropped_samples(), 1_000);
    }

    #[test]
    fn the_drop_keeps_the_head_of_the_window() {
        let mut queue = ResidentQueue::new();
        // The audio at the head: the next audible sample.
        queue.push(&[1.0, 2.0, 3.0]);
        // A chunk that overruns the window: its tail falls beyond the
        // bound and is dropped; its head joins the window.
        let overrun = vec![0.0_f32; MAX_PREROLL_SAMPLES + 10];
        queue.push(&overrun);

        assert_eq!(queue.len(), MAX_PREROLL_SAMPLES);
        // The head survived: the reading stays audibly contiguous from
        // the play head.
        let head: Vec<f32> = queue.samples().iter().copied().take(3).collect();
        assert_eq!(head, vec![1.0, 2.0, 3.0]);
        // Only the audio beyond the window was dropped: the chunk
        // carried the bound plus ten, and the window had three free
        // places for it.
        assert_eq!(queue.dropped_samples(), 13);
    }

    #[test]
    fn an_empty_window_reports_the_empty_buffer_state() {
        let state = BufferState::EMPTY;
        assert_eq!(state.buffered_secs, 0.0);
        assert_eq!(state.cap_secs, MAX_PREROLL_SECS);
        assert_eq!(state.dropped_samples, 0);
    }

    #[test]
    #[ignore = "requires audio hardware"]
    fn backend_creation_succeeds_on_host() {
        let backend = CpalBackend::new();
        assert!(backend.is_ok() || backend.is_err());
    }
}

#[cfg(test)]
mod earcon_burst {
    use super::*;
    use crate::earcon::EarconDescriptor;
    use proptest::prelude::*;

    const ALL_KINDS: [EarconKind; 7] = [
        EarconKind::Play,
        EarconKind::Pause,
        EarconKind::SkipFwd,
        EarconKind::SkipBack,
        EarconKind::Warning,
        EarconKind::Failure,
        EarconKind::RateStep,
    ];

    fn is_system(k: EarconKind) -> bool {
        matches!(k, EarconKind::Warning | EarconKind::Failure)
    }

    fn enqueue(q: &mut SampleQueue, kind: EarconKind, pcm: &[f32]) {
        let incoming_system = is_system(kind);
        let in_flight_system = q.short_mix_kind.is_some_and(is_system);
        if incoming_system || !in_flight_system {
            q.truncate_short_mix_before_append();
            q.push_short_mix(kind, pcm);
        }
    }

    proptest! {
        #[test]
        fn prop_earcon_burst_latest_wins(
            bursts in prop::collection::vec((0usize..7usize, 1usize..50usize), 0..64usize)
        ) {
            let mut q = SampleQueue::default();
            for (ki, len) in &bursts {
                let kind = ALL_KINDS[*ki % 7];
                let pcm = vec![0.25_f32; *len];
                let in_flight = q.short_mix_kind;
                let in_flight_system = in_flight.is_some_and(is_system);
                let incoming_system = is_system(kind);
                let accepted = incoming_system || !in_flight_system;

                let before_len = q.short_mix.len();
                let before_kind = q.short_mix_kind;

                enqueue(&mut q, kind, &pcm);

                if accepted {
                    // in-flight USER (or any) truncated before append: latest wins,
                    // so the channel holds exactly the new PCM, never a widened sum.
                    prop_assert_eq!(q.short_mix.len(), pcm.len());
                    prop_assert_eq!(q.short_mix_kind, Some(kind));
                    // single channel never widened: never before_len + pcm.len()
                    prop_assert!(q.short_mix.len() <= pcm.len());
                } else {
                    // SYSTEM in flight: USER trigger does not interrupt; unchanged.
                    prop_assert_eq!(q.short_mix.len(), before_len);
                    prop_assert_eq!(q.short_mix_kind, before_kind);
                }
                // The short-mix channel is never wider than one trigger.
                prop_assert!(q.short_mix.len() <= 50);
            }
        }
    }

    #[test]
    fn test_earcon_latency_oracle() {
        const ONE_HUNDRED_MS_SAMPLES: usize = 2205; // 100 ms @ 22 050 Hz

        // Isolation: every trigger's first short-mix sample is consumed
        // within the first 100 ms block.
        for kind in ALL_KINDS {
            let desc = EarconDescriptor::for_kind(kind);
            let mut q = SampleQueue::default();
            enqueue(&mut q, kind, &desc.samples);
            let mut out = vec![0.0_f32; ONE_HUNDRED_MS_SAMPLES];
            let _ = q.consume_into(&mut out);
            assert_eq!(
                out[0], desc.samples[0],
                "{kind:?}: first short-mix sample not consumed in first 100 ms block"
            );
        }

        // Bursts: after interleaving all seven triggers, the last accepted
        // earcon's first sample is the first consumed short-mix sample.
        let mut q = SampleQueue::default();
        let mut last_accepted: Option<Vec<f32>> = None;
        for kind in ALL_KINDS {
            let desc = EarconDescriptor::for_kind(kind);
            let pcm = desc.samples.clone();
            let in_flight_system = q.short_mix_kind.is_some_and(is_system);
            let incoming_system = is_system(kind);
            let accepted = incoming_system || !in_flight_system;
            enqueue(&mut q, kind, &pcm);
            if accepted {
                last_accepted = Some(pcm);
            }
        }
        let mut out = vec![0.0_f32; ONE_HUNDRED_MS_SAMPLES];
        let _ = q.consume_into(&mut out);
        if let Some(pcm) = last_accepted {
            assert_eq!(
                out[0], pcm[0],
                "burst: last accepted earcon's first sample not consumed within 100 ms"
            );
        }
    }
}
