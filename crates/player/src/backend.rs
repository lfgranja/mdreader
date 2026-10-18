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

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use engine_core::closed::EngineError;
use engine_core::format::{DeclaredFormat, check_handoff};
use thiserror::Error;
use tracing::{error, info, instrument};

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
    /// Buffered samples not yet consumed by the callback.
    samples: VecDeque<f32>,
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
}

impl Default for SampleQueue {
    fn default() -> Self {
        Self {
            samples: VecDeque::new(),
            gain: FULL_GAIN,
            short_mix: VecDeque::new(),
        }
    }
}

impl SampleQueue {
    /// Append synthesized samples to the tail.
    fn push(&mut self, pcm: &[f32]) {
        self.samples.extend(pcm.iter().copied());
    }

    /// Append duck-exempt earcon PCM to the short-mix buffer
    /// (contracts/earcon.md): mixed at full gain on the same
    /// stream, never scaled by the duck actuator.
    fn push_short_mix(&mut self, pcm: &[f32]) {
        self.short_mix.extend(pcm.iter().copied());
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
        let mut speech_consumed = 0;
        for slot in out.iter_mut() {
            let speech = self.samples.pop_front();
            let earcon = self.short_mix.pop_front();
            match (speech, earcon) {
                (Some(sample), Some(tone)) => {
                    *slot = sample.mul_add(gain, tone);
                    speech_consumed += 1;
                }
                (Some(sample), None) => {
                    *slot = sample * gain;
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
        speech_consumed
    }

    /// Drop every buffered sample, speech and short-mix alike:
    /// a pause or stop stands the confirmation down too (the
    /// trigger path re-fires it once Playing is restored, per
    /// contracts/earcon.md).
    fn clear(&mut self) {
        self.samples.clear();
        self.short_mix.clear();
    }

    /// Buffered speech sample count.
    #[must_use]
    fn len(&self) -> usize {
        self.samples.len()
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
    pub fn enqueue_earcon(&self, pcm: &[f32]) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_short_mix(pcm);
        }
    }

    /// Buffered sample count not yet consumed by the stream.
    #[must_use]
    pub fn buffered_samples(&self) -> usize {
        self.queue.lock().map_or(0, |q| q.len())
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

    #[expect(
        clippy::float_cmp,
        reason = "the queue copies samples verbatim along the empty and unity paths and every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance would admit a corrupted sample"
    )]
    #[test]
    fn an_empty_queue_yields_silence_and_zero_consumed() {
        let mut queue = SampleQueue::default();
        assert_eq!(queue.len(), 0);
        let mut out = [1.0_f32; 4];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 0);
        assert_eq!(out, [0.0; 4]);
    }

    #[expect(
        clippy::float_cmp,
        reason = "the queue copies samples verbatim along the empty and unity paths and every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance would admit a corrupted sample"
    )]
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

    #[expect(
        clippy::float_cmp,
        reason = "the queue copies samples verbatim along the empty and unity paths and every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance would admit a corrupted sample"
    )]
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

    #[expect(
        clippy::float_cmp,
        reason = "the default gain must be exactly FULL_GAIN; == pins the 1.0 default and a tolerance would admit drift"
    )]
    #[test]
    fn a_fresh_queue_reads_full_gain() {
        let queue = SampleQueue::default();
        assert_eq!(queue.gain(), FULL_GAIN);
    }

    #[expect(
        clippy::float_cmp,
        reason = "the queue copies samples verbatim along the empty and unity paths and every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance would admit a corrupted sample"
    )]
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

    #[expect(
        clippy::float_cmp,
        reason = "the queue copies samples verbatim along the empty and unity paths and every literal is an exact binary fraction, so the buffer must match bit-for-bit; a tolerance would admit a corrupted sample"
    )]
    #[test]
    fn duck_gain_leaves_silence_silent() {
        let mut queue = SampleQueue::default();
        assert!(queue.set_gain(DUCK_GAIN).is_ok());
        let mut out = [1.0_f32; 4];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 0);
        assert_eq!(out, [0.0; 4]);
    }

    #[expect(
        clippy::float_cmp,
        reason = "boundary gains must echo bit-for-bit and rejections must leave the stored gain bit-for-bit intact; == pins the echo/unchanged laws and a tolerance would admit drift"
    )]
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

    #[expect(
        clippy::float_cmp,
        reason = "a rejection must leave the stored gain bit-for-bit intact; == pins the unchanged law and a tolerance would admit drift"
    )]
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

    #[test]
    #[ignore = "requires audio hardware"]
    fn backend_creation_succeeds_on_host() {
        let backend = CpalBackend::new();
        assert!(backend.is_ok() || backend.is_err());
    }
}
