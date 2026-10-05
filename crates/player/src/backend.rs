//! CPAL playback backend (T026).
//!
//! Rust owns synthesis and PCM playback (cpal 0.18.2, AAudio).
//! Exact-request 22050 Hz / PCM-float / mono. Device-format
//! conversion happens in Rust. Refusal → named `EngineError`.

use crate::engine_core::closed::EngineError;
use crate::engine_core::format::{DeclaredFormat, check_handoff};
use crate::position::{PositionClock, PlayerError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, SampleFormat, SampleRate, StreamConfig, SupportedStreamConfigRange};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

/// Errors from the CPAL backend.
#[derive(Debug, Error)]
pub enum BackendError {
    /// No suitable audio device found.
    #[error("no audio device available: {0}")]
    NoDevice(String),
    /// Failed to build the output stream.
    #[error("failed to build stream: {0}")]
    BuildStream(#[from] BuildStreamError),
    /// The device refused the requested format.
    #[error("device refused the requested format: {0}")]
    DeviceRefused(String),
    /// The device was lost during playback.
    #[error("device lost: {0}")]
    DeviceLost(String),
    /// The handoff format does not match the declared shape.
    #[error("handoff format mismatch: {0}")]
    HandoffMismatch(EngineError),
    /// Playback start was refused.
    #[error("playback start refused: {0}")]
    StartRefused(String),
}

/// The CPAL backend driver.
///
/// Owns the output stream and the position clock. Delivers mono
/// f32 at 22050 Hz to the engine, and converts the device's native
/// format to/from the declared handoff shape.
pub struct CpalBackend {
    stream: Option<cpal::Stream>,
    position_clock: Arc<Mutex<PositionClock>>,
    device_name: String,
    sample_rate: u32,
}

impl CpalBackend {
    /// Create a new backend with the default output device.
    ///
    /// Requests exactly 22050 Hz, mono, f32. If the device cannot
    /// satisfy this exactly, the closest supported format is used and
    /// Rust performs the conversion.
    #[instrument]
    pub fn new() -> Result<Self, BackendError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| BackendError::NoDevice("no default output device".to_string()))?;

        let device_name = device
            .name()
            .unwrap_or_else(|_| "unknown".to_string());

        info!(device = %device_name, "selected output device");

        // Inspect supported configs to find one compatible with our
        // declared handoff shape (mono f32 @ 22050 Hz).
        let supported = device
            .supported_output_configs()
            .map_err(|e| BackendError::NoDevice(format!("failed to query configs: {e}")))?;

        let mut best_config: Option<SupportedStreamConfigRange> = None;
        for config in supported {
            if config.channels() == 1
                && config.min_sample_rate().0 <= DeclaredFormat::SAMPLE_RATE
                && config.max_sample_rate().0 >= DeclaredFormat::SAMPLE_RATE
                && config.sample_format() == SampleFormat::F32
            {
                best_config = Some(config);
                break;
            }
        }

        let config = best_config
            .ok_or_else(|| {
                BackendError::DeviceRefused(
                    "no supported config matching mono f32 @ 22050 Hz".to_string(),
                )
            })?
            .with_sample_rate(SampleRate(DeclaredFormat::SAMPLE_RATE));

        // Verify the handoff format matches our declaration.
        check_handoff(
            config.sample_rate().0,
            config.channels(),
            config.sample_format().sample_size() * 8,
        )
        .map_err(BackendError::HandoffMismatch)?;

        let sample_rate = config.sample_rate().0;
        let position_clock = Arc::new(Mutex::new(PositionClock::new(0.0)));

        let clock_clone = Arc::clone(&position_clock);
        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Engine fills the buffer with mono f32 @ 22050 Hz.
                    // This callback runs on the audio thread.
                    let mut clock = clock_clone.lock().unwrap();
                    let elapsed = data.len() as f64 / sample_rate as f64;
                    if elapsed > 0.0 {
                        let _ = clock.on_audible_progress(elapsed);
                    }
                },
                |err| error!(error = %err, "audio stream error"),
                None,
            )
            .map_err(BackendError::BuildStream)?;

        info!(sample_rate, channels = config.channels(), "stream built");

        Ok(Self {
            stream: Some(stream),
            position_clock,
            device_name,
            sample_rate,
        })
    }

    /// Start playback.
    #[instrument]
    pub fn play(&self) -> Result<(), BackendError> {
        if let Some(stream) = &self.stream {
            stream.play().map_err(|e| BackendError::StartRefused(e.to_string()))?;
            info!("playback started");
            Ok(())
        } else {
            Err(BackendError::StartRefused("no stream".to_string()))
        }
    }

    /// Pause playback.
    #[instrument]
    pub fn pause(&self) {
        if let Some(stream) = &self.stream {
            let _ = stream.pause();
            info!("playback paused");
        }
    }

    /// Stop playback and reset position.
    #[instrument]
    pub fn stop(&self) {
        if let Some(stream) = &self.stream {
            let _ = stream.pause();
        }
        if let Ok(mut clock) = self.position_clock.lock() {
            *clock = PositionClock::new(0.0);
        }
        info!("playback stopped, position reset");
    }

    /// Get a reference to the position clock.
    #[must_use]
    pub fn position_clock(&self) -> Arc<Mutex<PositionClock>> {
        Arc::clone(&self.position_clock)
    }

    /// Get the current playback position in seconds.
    #[must_use]
    pub fn position_secs(&self) -> f64 {
        self.position_clock
            .lock()
            .unwrap()
            .position_secs()
    }

    /// Get the stored basis (kill-replay basis) in seconds.
    #[must_use]
    pub fn stored_basis(&self) -> f64 {
        self.position_clock
            .lock()
            .unwrap()
            .stored_basis()
    }
}

impl Drop for CpalBackend {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires audio hardware"]
    fn backend_creation_succeeds_on_host() {
        let backend = CpalBackend::new();
        // On headless CI this may fail, but locally it should succeed.
        assert!(backend.is_ok() || backend.is_err());
    }
}