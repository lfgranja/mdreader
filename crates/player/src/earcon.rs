//! EarconKind — procedural audio confirmation grammar (FR-016 / SC-004 / SC-008).
//! Mono f32 @ 22050 Hz; no `.wav`/`SoundPool`/`ToneGenerator` assets.
//!
//! Closed set: exactly the seven named variants below. No `Other`/`Unknown`
//! — every surface match must be exhaustive with no `_ =>` catch-all.

use serde::{Deserialize, Serialize};

/// The closed set of earcon kinds (contracts/earcon.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarconKind {
    /// Play confirmation.
    Play,
    /// Pause confirmation.
    Pause,
    /// Skip forward confirmation.
    SkipFwd,
    /// Skip backward confirmation.
    SkipBack,
    /// Warning earcon.
    Warning,
    /// Failure earcon.
    Failure,
    /// Rate step confirmation.
    RateStep,
}

/// Sample format for earcon PCM output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    /// 32-bit floating point mono.
    F32,
}

/// Frequency band in Hz (min, max).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrequencyBand(pub u32, pub u32);

/// Perceptual fingerprint for pairwise distinctness testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerceptualFingerprint {
    /// Timbre category (waveform / harmonic content).
    pub timbre: TimbreClass,
    /// Rhythm pattern class.
    pub rhythm: RhythmClass,
    /// Pitch contour class.
    pub contour: ContourClass,
}

/// Timbre classification for perceptual distinctness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimbreClass {
    /// Pure sine wave.
    Sine,
    /// Square wave (rich harmonics).
    Square,
    /// Dissonant double-tick (two close frequencies).
    DissonantPair,
}

/// Rhythm classification for perceptual distinctness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RhythmClass {
    /// Two notes, 120 ms each (play/pause).
    TwoNote120,
    /// Single short tick ~40 ms (skip).
    SingleTick40,
    /// Double tick 60+80 ms (warning).
    DoubleTick60_80,
    /// 200 ms + repeat (failure).
    LongRepeat200,
    /// Single unison 60 ms (rate step).
    Unison60,
}

/// Pitch contour classification for perceptual distinctness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContourClass {
    /// Rising C5→E5 (play).
    RisingMajorThird,
    /// Falling E5→C5 (pause).
    FallingMajorThird,
    /// Rising 880→1108 Hz (skip forward).
    RisingQuarterTone,
    /// Falling 880→660 Hz (skip back).
    FallingQuarterTone,
    /// Ostinato at A5 (warning).
    OstinatoA5,
    /// E4 with minor second (failure).
    MinorSecondCluster,
    /// Unison at 880 Hz (rate step).
    Unison880,
}

/// Complete descriptor for a synthesized earcon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EarconDescriptor {
    /// The earcon kind this descriptor represents.
    pub kind: EarconKind,
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Frequency band (min, max) in Hz.
    pub band_hz: FrequencyBand,
    /// Sample rate in Hz (always 22050).
    pub sample_rate_hz: u32,
    /// Number of channels (always 1 for mono).
    pub channels: u8,
    /// Sample format (always F32).
    pub format: SampleFormat,
    /// Perceptual fingerprint for distinctness verification.
    pub perceptual_fingerprint: PerceptualFingerprint,
    /// Generated PCM samples (mono f32).
    ///
    /// Skipped in the serialized descriptor: the wire form is
    /// the descriptor (kind, duration, band, format) — the
    /// PCM itself reaches the backend through
    /// [`EarconDescriptor::samples`], never through the JSON
    /// descriptor a `--format json` consumer reads (FR-045:
    /// no bulk payload in the machine-readable shape; the
    /// descriptor is the contract, the samples are the
    /// delivery).
    #[serde(skip)]
    pub samples: Vec<f32>,
}

impl EarconDescriptor {
    /// Create an earcon descriptor for the given kind.
    pub fn for_kind(kind: EarconKind) -> Self {
        match kind {
            EarconKind::Play => Self::play(),
            EarconKind::Pause => Self::pause(),
            EarconKind::SkipFwd => Self::skip_fwd(),
            EarconKind::SkipBack => Self::skip_back(),
            EarconKind::Warning => Self::warning(),
            EarconKind::Failure => Self::failure(),
            EarconKind::RateStep => Self::rate_step(),
        }
    }

    /// Play: C5→E5 sine, 120 ms/note, ≤500 ms total.
    fn play() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const NOTE_DURATION_MS: u32 = 120;
        const TOTAL_DURATION_MS: u32 = 240; // 2 notes × 120 ms

        let c5_hz = 523.25; // C5
        let e5_hz = 659.25; // E5

        let samples =
            generate_two_note_sequence(SAMPLE_RATE, NOTE_DURATION_MS, c5_hz, e5_hz, Waveform::Sine);

        Self {
            kind: EarconKind::Play,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(523, 660),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Sine,
                rhythm: RhythmClass::TwoNote120,
                contour: ContourClass::RisingMajorThird,
            },
            samples,
        }
    }

    /// Pause: E5→C5 sine, 120 ms/note, ≤500 ms total.
    fn pause() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const NOTE_DURATION_MS: u32 = 120;
        const TOTAL_DURATION_MS: u32 = 240;

        let e5_hz = 659.25; // E5
        let c5_hz = 523.25; // C5

        let samples =
            generate_two_note_sequence(SAMPLE_RATE, NOTE_DURATION_MS, e5_hz, c5_hz, Waveform::Sine);

        Self {
            kind: EarconKind::Pause,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(523, 660),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Sine,
                rhythm: RhythmClass::TwoNote120,
                contour: ContourClass::FallingMajorThird,
            },
            samples,
        }
    }

    /// SkipFwd: 880→1108 Hz rising, 40 ms tick.
    fn skip_fwd() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const TICK_DURATION_MS: u32 = 40;
        const TOTAL_DURATION_MS: u32 = 40;

        let start_hz = 880.0;
        let end_hz = 1108.0; // ~880 * 2^(1/4) ≈ quarter tone up

        let samples = generate_frequency_sweep(
            SAMPLE_RATE,
            TICK_DURATION_MS,
            start_hz,
            end_hz,
            Waveform::Sine,
        );

        Self {
            kind: EarconKind::SkipFwd,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(880, 1108),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Sine,
                rhythm: RhythmClass::SingleTick40,
                contour: ContourClass::RisingQuarterTone,
            },
            samples,
        }
    }

    /// SkipBack: 880→660 Hz falling, 40 ms tick.
    fn skip_back() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const TICK_DURATION_MS: u32 = 40;
        const TOTAL_DURATION_MS: u32 = 40;

        let start_hz = 880.0;
        let end_hz = 660.0; // ~880 * 2^(-1/4) ≈ quarter tone down

        let samples = generate_frequency_sweep(
            SAMPLE_RATE,
            TICK_DURATION_MS,
            start_hz,
            end_hz,
            Waveform::Sine,
        );

        Self {
            kind: EarconKind::SkipBack,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(660, 880),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Sine,
                rhythm: RhythmClass::SingleTick40,
                contour: ContourClass::FallingQuarterTone,
            },
            samples,
        }
    }

    /// Warning: double-tick A5 (880 Hz), 60+80 ms, dissonant.
    fn warning() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const TICK1_DURATION_MS: u32 = 60;
        const TICK2_DURATION_MS: u32 = 80;
        const GAP_MS: u32 = 20;
        const TOTAL_DURATION_MS: u32 = TICK1_DURATION_MS + GAP_MS + TICK2_DURATION_MS; // 160 ms

        // A5 = 880 Hz, dissonant pair: 880 Hz + 932 Hz (minor second ≈ 100 cents)
        let freq1 = 880.0;
        let freq2 = 932.33; // 880 * 2^(1/12) ≈ minor second up

        let mut samples = Vec::new();

        // First tick: dissonant pair
        let tick1 = generate_dissonant_pair(SAMPLE_RATE, TICK1_DURATION_MS, freq1, freq2);
        samples.extend(tick1);

        // Gap (silence)
        let gap_samples = (SAMPLE_RATE as f32 * GAP_MS as f32 / 1000.0) as usize;
        samples.extend(vec![0.0; gap_samples]);

        // Second tick: same dissonant pair
        let tick2 = generate_dissonant_pair(SAMPLE_RATE, TICK2_DURATION_MS, freq1, freq2);
        samples.extend(tick2);

        Self {
            kind: EarconKind::Warning,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(880, 932),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::DissonantPair,
                rhythm: RhythmClass::DoubleTick60_80,
                contour: ContourClass::OstinatoA5,
            },
            samples,
        }
    }

    /// Failure: E4 ~330 Hz square + minor-2nd, 200 ms + repeat.
    fn failure() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const NOTE_DURATION_MS: u32 = 200;
        const GAP_MS: u32 = 50;
        const TOTAL_DURATION_MS: u32 = NOTE_DURATION_MS + GAP_MS + NOTE_DURATION_MS; // 450 ms

        // E4 = 329.63 Hz, minor second = ~349.23 Hz
        let freq1 = 329.63;
        let freq2 = 349.23; // E4 + minor second

        let mut samples = Vec::new();

        // First note: square wave with minor second cluster
        let note1 = generate_square_with_minor_second(SAMPLE_RATE, NOTE_DURATION_MS, freq1, freq2);
        samples.extend(note1);

        // Gap (silence)
        let gap_samples = (SAMPLE_RATE as f32 * GAP_MS as f32 / 1000.0) as usize;
        samples.extend(vec![0.0; gap_samples]);

        // Repeat
        let note2 = generate_square_with_minor_second(SAMPLE_RATE, NOTE_DURATION_MS, freq1, freq2);
        samples.extend(note2);

        Self {
            kind: EarconKind::Failure,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(330, 349),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Square,
                rhythm: RhythmClass::LongRepeat200,
                contour: ContourClass::MinorSecondCluster,
            },
            samples,
        }
    }

    /// RateStep: Unison 880 Hz / 60 ms.
    fn rate_step() -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const DURATION_MS: u32 = 60;
        const TOTAL_DURATION_MS: u32 = 60;
        const FREQ_HZ: f32 = 880.0;

        let samples = generate_tone(SAMPLE_RATE, DURATION_MS, FREQ_HZ, Waveform::Sine);

        Self {
            kind: EarconKind::RateStep,
            duration_ms: TOTAL_DURATION_MS,
            band_hz: FrequencyBand(880, 880),
            sample_rate_hz: SAMPLE_RATE,
            channels: 1,
            format: SampleFormat::F32,
            perceptual_fingerprint: PerceptualFingerprint {
                timbre: TimbreClass::Sine,
                rhythm: RhythmClass::Unison60,
                contour: ContourClass::Unison880,
            },
            samples,
        }
    }
}

/// Waveform types for synthesis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Waveform {
    Sine,
    Square,
}

/// Generate a simple tone.
fn generate_tone(sample_rate: u32, duration_ms: u32, freq_hz: f32, waveform: Waveform) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let phase = 2.0 * std::f32::consts::PI * freq_hz * t;
        let sample = match waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Square => phase.sin().signum(),
        };
        // Apply envelope to avoid clicks
        let envelope = envelope(t, duration_ms as f32 / 1000.0);
        samples.push(sample * envelope * 0.3); // 0.3 gain to avoid clipping
    }

    samples
}

/// Generate a frequency sweep (chirp).
fn generate_frequency_sweep(
    sample_rate: u32,
    duration_ms: u32,
    start_hz: f32,
    end_hz: f32,
    waveform: Waveform,
) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let duration_s = duration_ms as f32 / 1000.0;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Linear frequency sweep
        let freq = start_hz + (end_hz - start_hz) * (t / duration_s);
        let phase = 2.0 * std::f32::consts::PI * freq * t;
        let sample = match waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Square => phase.sin().signum(),
        };
        let envelope = envelope(t, duration_s);
        samples.push(sample * envelope * 0.3);
    }

    samples
}

/// Generate two-note sequence (for play/pause).
fn generate_two_note_sequence(
    sample_rate: u32,
    note_duration_ms: u32,
    freq1_hz: f32,
    freq2_hz: f32,
    waveform: Waveform,
) -> Vec<f32> {
    let mut samples = Vec::new();

    // First note
    let note1 = generate_tone(sample_rate, note_duration_ms, freq1_hz, waveform);
    samples.extend(note1);

    // Second note
    let note2 = generate_tone(sample_rate, note_duration_ms, freq2_hz, waveform);
    samples.extend(note2);

    samples
}

/// Generate a dissonant pair (two close frequencies summed).
fn generate_dissonant_pair(
    sample_rate: u32,
    duration_ms: u32,
    freq1_hz: f32,
    freq2_hz: f32,
) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let duration_s = duration_ms as f32 / 1000.0;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let phase1 = 2.0 * std::f32::consts::PI * freq1_hz * t;
        let phase2 = 2.0 * std::f32::consts::PI * freq2_hz * t;
        let sample = (phase1.sin() + phase2.sin()) * 0.5; // Mix at equal amplitude
        let envelope = envelope(t, duration_s);
        samples.push(sample * envelope * 0.3);
    }

    samples
}

/// Generate square wave with minor second cluster.
fn generate_square_with_minor_second(
    sample_rate: u32,
    duration_ms: u32,
    freq1_hz: f32,
    freq2_hz: f32,
) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let duration_s = duration_ms as f32 / 1000.0;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let phase1 = 2.0 * std::f32::consts::PI * freq1_hz * t;
        let phase2 = 2.0 * std::f32::consts::PI * freq2_hz * t;
        // Square wave for both frequencies (using Waveform::Square)
        let sample = (square_wave(phase1) + square_wave(phase2)) * 0.5;
        let envelope = envelope(t, duration_s);
        samples.push(sample * envelope * 0.25); // Lower gain for square wave
    }

    samples
}

/// Square wave helper using Waveform enum.
fn square_wave(phase: f32) -> f32 {
    match Waveform::Square {
        Waveform::Square => phase.sin().signum(),
        Waveform::Sine => phase.sin(), // unreachable, but satisfies exhaustiveness
    }
}

/// Simple ADSR-like envelope (attack + release only for short sounds).
fn envelope(t: f32, duration_s: f32) -> f32 {
    const ATTACK_S: f32 = 0.005; // 5 ms attack
    const RELEASE_S: f32 = 0.010; // 10 ms release

    if t < ATTACK_S {
        t / ATTACK_S
    } else if t > duration_s - RELEASE_S {
        (duration_s - t) / RELEASE_S
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::Play);
        assert_eq!(desc.kind, EarconKind::Play);
        assert!(desc.duration_ms <= 500);
        assert_eq!(desc.sample_rate_hz, 22_050);
        assert_eq!(desc.channels, 1);
        assert_eq!(desc.format, SampleFormat::F32);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn pause_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::Pause);
        assert_eq!(desc.kind, EarconKind::Pause);
        assert!(desc.duration_ms <= 500);
        assert_eq!(desc.sample_rate_hz, 22_050);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn skip_fwd_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::SkipFwd);
        assert_eq!(desc.kind, EarconKind::SkipFwd);
        assert!(desc.duration_ms <= 500);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn skip_back_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::SkipBack);
        assert_eq!(desc.kind, EarconKind::SkipBack);
        assert!(desc.duration_ms <= 500);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn warning_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::Warning);
        assert_eq!(desc.kind, EarconKind::Warning);
        assert!(desc.duration_ms <= 500);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn failure_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::Failure);
        assert_eq!(desc.kind, EarconKind::Failure);
        assert!(desc.duration_ms <= 500);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn rate_step_descriptor_valid() {
        let desc = EarconDescriptor::for_kind(EarconKind::RateStep);
        assert_eq!(desc.kind, EarconKind::RateStep);
        assert!(desc.duration_ms <= 500);
        assert!(!desc.samples.is_empty());
    }

    #[test]
    fn all_kinds_have_distinct_fingerprints() {
        let mut fingerprints = std::collections::HashSet::new();
        for kind in [
            EarconKind::Play,
            EarconKind::Pause,
            EarconKind::SkipFwd,
            EarconKind::SkipBack,
            EarconKind::Warning,
            EarconKind::Failure,
            EarconKind::RateStep,
        ] {
            let desc = EarconDescriptor::for_kind(kind);
            let fp = desc.perceptual_fingerprint;
            assert!(
                fingerprints.insert((fp.timbre, fp.rhythm, fp.contour)),
                "Duplicate fingerprint for {:?}",
                kind
            );
        }
        assert_eq!(fingerprints.len(), 7);
    }

    #[test]
    fn frequency_bands_in_range() {
        for kind in [
            EarconKind::Play,
            EarconKind::Pause,
            EarconKind::SkipFwd,
            EarconKind::SkipBack,
            EarconKind::Warning,
            EarconKind::Failure,
            EarconKind::RateStep,
        ] {
            let desc = EarconDescriptor::for_kind(kind);
            assert!(
                desc.band_hz.0 >= 300,
                "{:?} min freq {} < 300",
                kind,
                desc.band_hz.0
            );
            assert!(
                desc.band_hz.1 <= 2500,
                "{:?} max freq {} > 2500",
                kind,
                desc.band_hz.1
            );
        }
    }
}
