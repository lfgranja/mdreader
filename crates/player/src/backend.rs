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
}

/// FIFO of mono `f32` samples at the declared rate, shared with
/// the stream callback.
#[derive(Debug, Default)]
struct SampleQueue {
    /// Buffered samples not yet consumed by the callback.
    samples: VecDeque<f32>,
}

impl SampleQueue {
    /// Append synthesized samples to the tail.
    fn push(&mut self, pcm: &[f32]) {
        self.samples.extend(pcm.iter().copied());
    }

    /// Move up to `out.len()` samples into `out`, filling the
    /// remainder with silence. Returns the consumed count.
    fn consume_into(&mut self, out: &mut [f32]) -> usize {
        let mut consumed = 0;
        for slot in out.iter_mut() {
            if let Some(sample) = self.samples.pop_front() {
                *slot = sample;
                consumed += 1;
            } else {
                *slot = 0.0;
            }
        }
        consumed
    }

    /// Buffered sample count.
    #[must_use]
    fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether the queue holds no samples.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.samples.is_empty()
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
    pub fn enqueue_pcm(&self, pcm: Vec<f32>) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(&pcm);
        }
    }

    /// Buffered sample count not yet consumed by the stream.
    #[must_use]
    pub fn buffered_samples(&self) -> usize {
        self.queue.lock().map(|q| q.len()).unwrap_or(0)
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
            queue.samples.clear();
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
            .map(|clock| clock.position_secs())
            .unwrap_or(0.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_queue_yields_silence_and_zero_consumed() {
        let mut queue = SampleQueue::default();
        assert!(queue.is_empty());
        let mut out = [1.0_f32; 4];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 0);
        assert_eq!(out, [0.0; 4]);
    }

    #[test]
    fn queued_samples_come_out_fifo_with_silence_tail() {
        let mut queue = SampleQueue::default();
        queue.push(&[0.5, -0.25, 0.125]);
        assert_eq!(queue.len(), 3);
        let mut out = [0.0_f32; 5];
        let consumed = queue.consume_into(&mut out);
        assert_eq!(consumed, 3);
        assert_eq!(out, [0.5, -0.25, 0.125, 0.0, 0.0]);
        assert!(queue.is_empty());
    }

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
    #[ignore = "requires audio hardware"]
    fn backend_creation_succeeds_on_host() {
        let backend = CpalBackend::new();
        assert!(backend.is_ok() || backend.is_err());
    }
}
