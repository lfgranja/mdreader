//! Real-time DSP post-processing layer for the player crate.
//!
//! Implements the "Sona ethereal" vocal texture: optical compressor →
//! plate/shimmer reverb → chorus, all in mono `f32` at the declared
//! handoff rate (22 050 Hz). Zero external dependencies; every
//! algorithm is hand-written so the workspace supply-chain /
//! deny-list gates are untouched.
//!
//! The processor works sample-by-sample, holds state between calls,
//! and never allocates after construction. With [`DspPreset::Off`] it
//! is a pure pass-through (one branch, no gain change).

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// One-pole low-pass filter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// One-pole IIR low-pass filter.
///
/// `y[n] = alpha * x[n] + (1 - alpha) * y[n-1]` where `alpha` is derived
/// from the cutoff frequency and the sample rate. Used in the reverb
/// feedback path to cut frequencies above 8 kHz, keeping the texture
/// velvety without harsh sibilance.
#[derive(Debug, Clone, Copy)]
pub struct OnePoleLowPass {
    /// Filter coefficient in `[0.0, 1.0]`. Higher = slower cutoff.
    alpha: f32,
    /// Previous output sample (filter state).
    prev: f32,
}

impl OnePoleLowPass {
    /// Create a low-pass filter with the given cutoff frequency (Hz).
    ///
    /// `alpha` is `1 - exp(-2 * pi * fc / sr)`, clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn new(cutoff_hz: f32, sample_rate: u32) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "sample rates are far below 2^24, so u32->f32 is exact"
        )]
        let sr = sample_rate as f32;
        let x = (2.0 * std::f32::consts::PI * cutoff_hz / sr).clamp(0.0, 20.0);
        let alpha = (1.0 - (-x).exp()).clamp(0.0, 1.0);
        Self { alpha, prev: 0.0 }
    }

    /// Process one sample and return the filtered output.
    #[must_use]
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.prev = self.alpha.mul_add(x, (1.0 - self.alpha) * self.prev);
        self.prev
    }

    /// Reset the filter state to silence.
    pub const fn reset(&mut self) {
        self.prev = 0.0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Circular delay line
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Circular delay line supporting integer and fractional (linearly
/// interpolated) reads.
///
/// The buffer is fixed after construction; no allocation happens
/// during processing.
#[derive(Debug, Clone)]
pub struct DelayLine {
    /// The ring buffer, oldest-to-newest by write position.
    buffer: Vec<f32>,
    /// Index where the next sample will be written.
    write_pos: usize,
}

impl DelayLine {
    /// Create a delay line able to hold `max_delay_samples` samples.
    ///
    /// At least one sample of storage is always allocated.
    #[must_use]
    pub fn new(max_delay_samples: usize) -> Self {
        let len = max_delay_samples.max(1);
        Self {
            buffer: vec![0.0; len],
            write_pos: 0,
        }
    }

    /// Write a sample and advance the write pointer by one.
    #[inline]
    pub fn push(&mut self, x: f32) {
        let len = self.buffer.len();
        self.buffer[self.write_pos] = x;
        self.write_pos = (self.write_pos + 1) % len;
    }

    /// Read the sample `delay_samples` back (integer delay, no
    /// interpolation). `delay_samples` must be `< buffer.len()`.
    #[must_use]
    #[inline]
    pub fn read(&self, delay_samples: usize) -> f32 {
        let len = self.buffer.len();
        let offset = (1 + delay_samples) % len;
        let read_pos = (self.write_pos + len - offset) % len;
        self.buffer[read_pos]
    }

    /// Read with fractional delay using linear interpolation.
    ///
    /// `delay_samples` may be fractional; the two adjacent samples are
    /// interpolated. Useful for the chorus modulated delay.
    #[must_use]
    #[inline]
    pub fn read_frac(&self, delay_samples: f32) -> f32 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "buffer length is far below 2^24, so usize->f32 is exact"
        )]
        let len = self.buffer.len() as f32;
        #[expect(
            clippy::cast_precision_loss,
            reason = "write_pos is far below 2^24, so usize->f32 is exact"
        )]
        let read_pos = (self.write_pos as f32 - 1.0 - delay_samples).rem_euclid(len);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "read_pos is within [0, len), so f32->usize is exact"
        )]
        let i = read_pos as usize;
        #[expect(
            clippy::cast_precision_loss,
            reason = "i is far below 2^24, so usize->f32 is exact"
        )]
        let frac = read_pos - i as f32;
        let j = (i + 1) % self.buffer.len();
        self.buffer[j].mul_add(frac, self.buffer[i] * (1.0 - frac))
    }

    /// Return whether the buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Return the buffer length in samples.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Fill the buffer with silence and reset the write pointer.
    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Convert `sample_rate * seconds` to a sample count (usize).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "sample rates are far below 2^24, so f64->usize is exact"
)]
#[must_use]
fn delay_samples(sample_rate: u32, seconds: f64) -> usize {
    (f64::from(sample_rate) * seconds) as usize
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Plate / shimmer reverb
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Simplified plate-reverb with a short pre-delay, four feedback delay
/// lines, and a one-pole low-pass (8 kHz) in each feedback path.
///
/// The decay target is 1.4 s RT60 at the declared sample rate. The wet
/// mix is about 25 %, keeping the voice intimate and velvety.
#[derive(Debug, Clone)]
pub struct PlateReverb {
    /// Pre-delay line (fixed 12 ms = 264 samples @ 22 050 Hz).
    predelay: DelayLine,
    /// Pre-delay length in samples.
    predelay_len: usize,
    /// Four feedback delay lines of different lengths.
    delays: [DelayLine; 4],
    /// One-pole low-pass in each feedback path (8 kHz cutoff).
    lowpasses: [OnePoleLowPass; 4],
    /// Feedback gain per delay line, derived from the RT60 target.
    gains: [f32; 4],
    /// Delay length (samples) for each of the four lines.
    delay_lengths: [usize; 4],
    /// Wet/dry mix: `0.0` = dry only, `1.0` = wet only.
    mix: f32,
    /// Additional gain applied to the wet sum before mixing.
    output_gain: f32,
}

impl PlateReverb {
    /// Create a plate reverb at `sample_rate` (expected to be the
    /// declared sample rate).
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        let predelay_len = delay_samples(sample_rate, 0.012).max(1); // 12 ms
        let delay_lengths = [
            delay_samples(sample_rate, 0.0116), // ~11.6 ms
            delay_samples(sample_rate, 0.0145), // ~14.5 ms
            delay_samples(sample_rate, 0.0181), // ~18.1 ms
            delay_samples(sample_rate, 0.0232), // ~23.2 ms
        ];
        let mut delays = [
            DelayLine::new(delay_lengths[0]),
            DelayLine::new(delay_lengths[1]),
            DelayLine::new(delay_lengths[2]),
            DelayLine::new(delay_lengths[3]),
        ];
        for d in &mut delays {
            d.clear();
        }

        let rt60 = 1.4f32;
        #[expect(
            clippy::cast_precision_loss,
            reason = "sample rates are far below 2^24, so u32->f32 is exact"
        )]
        let sr = sample_rate as f32;
        let mut gains = [0.0f32; 4];
        for (i, &len) in delay_lengths.iter().enumerate() {
            #[expect(
                clippy::cast_precision_loss,
                reason = "delay length is far below 2^24, so usize->f32 is exact"
            )]
            let x = (-3.0 * len as f32 / (rt60 * sr)).exp();
            gains[i] = x.clamp(0.0, 0.999);
        }

        let cutoff = 8000.0f32;
        let lowpasses = [
            OnePoleLowPass::new(cutoff, sample_rate),
            OnePoleLowPass::new(cutoff, sample_rate),
            OnePoleLowPass::new(cutoff, sample_rate),
            OnePoleLowPass::new(cutoff, sample_rate),
        ];

        Self {
            predelay: DelayLine::new(predelay_len),
            predelay_len,
            delays,
            lowpasses,
            gains,
            delay_lengths,
            mix: 0.25,
            output_gain: 0.5,
        }
    }

    /// Process one sample and return the reverberated output.
    #[must_use]
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        // Pre-delay: write current input, read what was written predelay_len ago.
        let predelayed = self.predelay.read(self.prelay_len());
        self.predelay.push(x);

        let mut wet = 0.0f32;
        for i in 0..4 {
            let output = self.delays[i].read(self.delay_lengths[i]);
            let feedback = self.lowpasses[i].process(output * self.gains[i]);
            let to_write = predelayed + feedback;
            self.delays[i].push(to_write);
            wet += output;
        }

        wet = wet * self.output_gain * self.mix;
        let dry = x * (1.0 - self.mix);
        dry + wet
    }

    #[must_use]
    #[inline]
    const fn prelay_len(&self) -> usize {
        self.predelay_len
    }

    /// Reset all delay lines and filters to silence.
    pub fn reset(&mut self) {
        self.predelay.clear();
        for d in &mut self.delays {
            d.clear();
        }
        for lp in &mut self.lowpasses {
            lp.reset();
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Chorus
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Chorus effect: a short delay modulated by a low-frequency sine LFO.
///
/// The LFO depth is chosen so the perceived pitch shift stays within
/// +/- 4 cents, giving the ethereal doubled-voice illusion. Wet mix is
/// kept subtle (~15 %).
#[derive(Debug, Clone)]
pub struct Chorus {
    /// Delay line holding the modulated delay.
    delay: DelayLine,
    /// Current LFO phase in radians.
    phase: f32,
    /// LFO frequency in Hz (~0.6 Hz).
    lfo_freq: f32,
    /// Peak modulation depth in samples (~30 samples).
    lfo_depth: f32,
    /// Base delay in samples (~150 samples ≈ 6.8 ms @ 22 050 Hz).
    base_delay: f32,
    /// Wet/dry mix: `0.0` = dry only, `1.0` = wet only.
    mix: f32,
    sample_rate: u32,
}

impl Chorus {
    /// Create a chorus at `sample_rate`.
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "sample rates are far below 2^24, so u32->f32 is exact"
        )]
        let sr = sample_rate as f32;
        let max_delay = {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "sample_rate * 0.020 is far below usize::MAX and positive"
            )]
            ((sr * 0.020) as usize).max(4)
        }; // 20 ms max
        Self {
            delay: DelayLine::new(max_delay),
            phase: 0.0,
            lfo_freq: 0.6,
            lfo_depth: 30.0,
            base_delay: 150.0,
            mix: 0.15,
            sample_rate,
        }
    }

    /// Process one sample and return the chorus-affected output.
    #[must_use]
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let sr = sample_rate_to_f32(self.sample_rate);
        // Advance LFO phase.
        self.phase += 2.0 * std::f32::consts::PI * self.lfo_freq / sr;
        if self.phase >= 2.0 * std::f32::consts::PI {
            self.phase = 2.0f32.mul_add(-std::f32::consts::PI, self.phase);
        }
        let mod_depth = self.lfo_depth * self.phase.sin();
        let delay_samples = self.base_delay + mod_depth;

        let delayed = self.delay.read_frac(delay_samples);
        self.delay.push(x);

        delayed.mul_add(self.mix, x * (1.0 - self.mix))
    }

    /// Reset the delay line and LFO phase.
    pub fn reset(&mut self) {
        self.delay.clear();
        self.phase = 0.0;
    }
}

/// Convert u32 `sample_rate` to f32 for DSP calculations.
#[expect(
    clippy::cast_precision_loss,
    reason = "sample rates are far below 2^24"
)]
#[must_use]
pub const fn sample_rate_to_f32(rate: u32) -> f32 {
    rate as f32
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Optical compressor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Optical-style compressor with slow attack, smooth release, and a
/// soft knee.
///
/// Smoothes the dynamics of continuous text reading without audible
/// pumping. Makeup gain compensates for the average reduction.
#[derive(Debug, Clone)]
pub struct OptoCompressor {
    /// Attack smoothing coefficient in `[0.0, 1.0]`.
    attack_coef: f32,
    /// Release smoothing coefficient in `[0.0, 1.0]`.
    release_coef: f32,
    /// Current envelope estimate (0.0–1.0).
    envelope: f32,
    /// Compression threshold (linear amplitude).
    threshold: f32,
    /// Compression ratio (e.g. `2.5` = 2.5:1).
    ratio: f32,
    /// Soft-knee width around the threshold (linear amplitude).
    knee_width: f32,
    /// Makeup gain applied after compression.
    makeup_gain: f32,
}

impl OptoCompressor {
    /// Create a compressor at `sample_rate`.
    ///
    /// Defaults: attack 150 ms, release 400 ms, threshold 0.3, ratio
    /// 2.5:1, knee width 0.1, makeup gain 1.2.
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "sample rates are far below 2^24, so u32->f32 is exact"
        )]
        let sr = sample_rate as f32;
        let attack_s = 0.150f32;
        let release_s = 0.400f32;
        // One-pole smoother coefficients: 1 - exp(-1 / (time * sr)).
        let attack_coef = (1.0 - (-1.0 / (attack_s * sr)).exp()).clamp(0.0, 1.0);
        let release_coef = (1.0 - (-1.0 / (release_s * sr)).exp()).clamp(0.0, 1.0);
        Self {
            attack_coef,
            release_coef,
            envelope: 0.0,
            threshold: 0.3,
            ratio: 2.5,
            knee_width: 0.1,
            makeup_gain: 1.2,
        }
    }

    /// Process one sample and return the compressed output.
    #[must_use]
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let abs_x = x.abs();

        // Envelope follower: attack when rising, release when falling.
        if abs_x > self.envelope {
            self.envelope = (abs_x - self.envelope).mul_add(self.attack_coef, self.envelope);
        } else {
            self.envelope = (abs_x - self.envelope).mul_add(self.release_coef, self.envelope);
        }

        let gr = self.gain_reduction(self.envelope);
        x * gr * self.makeup_gain
    }

    /// Compute the gain-reduction multiplier for a given envelope level.
    #[must_use]
    fn gain_reduction(&self, envelope: f32) -> f32 {
        let lo = self.threshold - self.knee_width;
        let hi = self.threshold + self.knee_width;

        if envelope <= lo {
            return 1.0;
        }

        if envelope >= hi {
            let over = envelope / self.threshold;
            let compressed = over.powf(1.0 / self.ratio);
            return compressed / envelope;
        }

        // Soft-knee transition: quadratic curve.
        let t = (envelope - lo) / (2.0 * self.knee_width);
        let curve = t * t;
        let over = envelope / self.threshold;
        let compressed = over.powf(1.0 / self.ratio);
        let hard_gr = compressed / envelope;
        (hard_gr - 1.0).mul_add(curve, 1.0)
    }

    /// Reset the envelope to silence.
    pub const fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DSP processor (public API)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Which DSP preset is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DspPreset {
    /// No processing: pass-through, bit-identical to the input.
    #[default]
    Off,
    /// Sona ethereal vocal texture: compressor → reverb → chorus.
    SonaEthereal,
}

/// DSP configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DspConfig {
    /// The active DSP preset.
    pub preset: DspPreset,
}

/// Real-time DSP post-processor.
///
/// Runs compressor → plate reverb → chorus on every sample, in mono
/// `f32` at the declared handoff rate. With [`DspPreset::Off`] it is
/// a pure pass-through. No allocation after construction.
#[derive(Debug, Clone)]
pub struct DspProcessor {
    compressor: OptoCompressor,
    reverb: PlateReverb,
    chorus: Chorus,
    /// The active preset.
    preset: DspPreset,
    enabled: bool,
}

impl DspProcessor {
    /// Create a new processor at `sample_rate` (expected to be the
    /// declared sample rate), preset `Off`.
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        Self {
            compressor: OptoCompressor::new(sample_rate),
            reverb: PlateReverb::new(sample_rate),
            chorus: Chorus::new(sample_rate),
            preset: DspPreset::Off,
            enabled: false,
        }
    }

    /// Set the DSP configuration. Switching presets resets all
    /// internal state to avoid clicks.
    pub fn set_config(&mut self, config: DspConfig) {
        let was_enabled = self.enabled;
        self.enabled = config.preset != DspPreset::Off;
        self.preset = config.preset;
        if was_enabled != self.enabled {
            self.reset();
        }
    }

    /// Process one sample and return the DSP-affected output.
    ///
    /// With `DspPreset::Off` the input is returned unchanged.
    #[must_use]
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        if !self.enabled {
            return x;
        }
        let x = self.compressor.process(x);
        let x = self.reverb.process(x);
        self.chorus.process(x)
    }

    /// Process an entire slice in place.
    #[inline]
    pub fn process_block(&mut self, buf: &mut [f32]) {
        for sample in buf.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    /// Return whether the DSP is currently enabled (preset != Off).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Return the current preset.
    #[must_use]
    pub const fn preset(&self) -> DspPreset {
        self.preset
    }

    /// Reset all internal state to silence.
    pub fn reset(&mut self) {
        self.compressor.reset();
        self.reverb.reset();
        self.chorus.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsp_off_is_bit_identical() {
        let mut dsp = DspProcessor::new(22_050);
        dsp.set_config(DspConfig {
            preset: DspPreset::Off,
        });
        assert!(!dsp.is_enabled());

        for x in &[0.0, 0.5, -0.25, 1.0, -1.0, 0.123_456] {
            assert_eq!(
                dsp.process_sample(*x),
                *x,
                "off preset must pass sample through unchanged"
            );
        }

        let mut block = [0.3f32, -0.7, 0.0, 1.0];
        let original = block;
        dsp.process_block(&mut block);
        assert_eq!(
            block, original,
            "off preset must pass block through unchanged"
        );
    }

    #[test]
    fn dsp_sona_ethereal_changes_signal_without_clipping() {
        let mut dsp = DspProcessor::new(22_050);
        dsp.set_config(DspConfig {
            preset: DspPreset::SonaEthereal,
        });
        assert!(dsp.is_enabled());
        assert_eq!(dsp.preset(), DspPreset::SonaEthereal);

        // An impulse: output must differ from input and never clip.
        let input = [1.0f32, 0.0, 0.0, 0.0, 0.0];
        let mut output = input;
        dsp.process_block(&mut output);
        assert_ne!(output, input, "on preset must alter the signal");
        for s in &output {
            assert!(
                s.is_finite() && s.abs() <= 1.0,
                "no clipping or NaN: got {s}"
            );
        }
    }

    #[test]
    fn dsp_stable_on_long_silence() {
        let mut dsp = DspProcessor::new(22_050);
        dsp.set_config(DspConfig {
            preset: DspPreset::SonaEthereal,
        });

        // 10 seconds of silence at 22 050 Hz = 220 500 samples.
        let mut buf = vec![0.0f32; 220_500];
        dsp.process_block(&mut buf);
        for s in &buf {
            assert!(
                s.is_finite() && s.abs() <= 1.0,
                "reverb must remain finite on silence: got {s}"
            );
        }
        assert!(buf[buf.len() - 1].abs() < 1e-3, "reverb tail should decay");
    }

    #[test]
    fn dsp_reset_clears_state() {
        let mut dsp = DspProcessor::new(22_050);
        dsp.set_config(DspConfig {
            preset: DspPreset::SonaEthereal,
        });
        let _ = dsp.process_sample(1.0);
        dsp.reset();
        let out = dsp.process_sample(0.0);
        assert!(
            out.abs() < 1e-3,
            "reset should clear reverb tail: got {out}"
        );
    }

    #[test]
    fn dsp_switch_preset_resets_state() {
        let mut dsp = DspProcessor::new(22_050);
        dsp.set_config(DspConfig {
            preset: DspPreset::SonaEthereal,
        });
        let _ = dsp.process_sample(1.0);
        assert!(dsp.is_enabled());

        dsp.set_config(DspConfig {
            preset: DspPreset::Off,
        });
        assert!(!dsp.is_enabled());
        assert_eq!(dsp.process_sample(0.5), 0.5);
    }
}
