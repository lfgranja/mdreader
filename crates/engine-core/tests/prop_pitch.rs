//! T055 — property: pitch invariance across the closed rate ladder (US4).
//!
//! Invariant under test (tasks.md T055, verbatim): "Property test:
//! pitch invariance in `crates/engine-core/tests/prop_pitch.rs`
//! (F0 contour + spectral peaks of identical PCM across all 7 rungs
//! match within ±50 cents per SC-015 incl. 2.0x→4.0x; delivered
//! rate invariant 22050 Hz across rungs per SC-034; FAIL before
//! implementation)"
//!
//! Spec anchors:
//! - SC-015 — the pitch shift stays within ±50 cents of the source
//!   across ALL seven rungs, including the 2.0×→4.0× pair. The
//!   engine-handoff contract names the same bound through the
//!   single shared `engine_core::format::PITCH_TOLERANCE_CENTS`,
//!   which this property asserts directly, so every conforming
//!   engine passes the same tolerance everywhere.
//! - SC-034 — the delivered audio rate is invariant at the declared
//!   22050 Hz on every rung: a rate change is a pitch-preserving
//!   time-stretch INSIDE the engine (FR-043) and never alters the
//!   delivered sample rate, channel count, or sample width.
//! - FR-043 — the rate change is a pitch-preserving time-stretch
//!   inside the engine.
//!
//! Estimators (both deterministic and written in this file — no
//! external crates):
//! - F0 contour — zero-crossing counting over fixed 4096-frame
//!   analysis windows: twice the sign-change count per window
//!   duration. The deterministic source signal (a 220 Hz
//!   fundamental with two decaying harmonics) factors as
//!   `sin(x)·(0.75 + cos x + cos²x)` with the bracket ≥ 0.5 > 0,
//!   so it keeps exactly two zero crossings per fundamental period
//!   and the estimator is unbiased on the source (calibrated below
//!   to ±25 cents, well inside the SC-015 margin).
//! - Spectral peaks — the strongest DFT bin within the 40–800 Hz
//!   search band (a direct DFT, no external crates), refined by
//!   parabolic interpolation on the log-magnitude across the peak
//!   bin and its two neighbours.
//!
//! What the property asserts, for EACH stand-in engine and EACH of
//! the seven rungs: `Engine::set_rate` returns `Ok(())` (the
//! contract's failure side — a conforming engine accepts every
//! rung), `Engine::stream` delivers a chunk in the declared
//! handoff shape (mono `f32`; the `(rate_hz, channels,
//! width_bits)` accessor assertion at every rung lives in the
//! per-stand-in shape tests below), and the F0 contour and the
//! spectral-peak contour of the delivered frames at rung R match
//! the `R1_0` reference contour within ±50 cents for every window.
//! The 2.0×→4.0× pair — the harshest — is asserted explicitly.
//! A non-pitch-preserving engine would fail these contour bounds:
//! one whose delivered pitch follows the rung (a naive resample)
//! rises 2400 cents at 4.0×, far beyond the ±50-cent margin, so
//! the property fails on the first violating window.
//!
//! Closed-ladder coverage (FR-019, FR-044, Principle VII): the
//! ladder is enumerated exhaustively — `rung_identity` and
//! `rung_label` are wildcard-free matches over `PlaybackRate`, so
//! a rung added to the closed set is a compile error here until
//! the property covers it.
//!
//! RED STATE (T056): the current stand-ins (`standin_a::StandinA`,
//! `standin_b::StandinB`) do not implement the `Engine` trait —
//! their inherent `set_rate` returns `()` (not
//! `Result<(), EngineError>`) and they expose no `stream` — so the
//! `Engine::set_rate` / `Engine::stream` contract calls in this
//! suite do not compile against them ("the trait bound `StandinA:
//! Engine` is not satisfied"). That build failure is the required
//! pre-implementation red state: this suite is the ENGINE CONTRACT
//! future real engines (and the engine-side stretch path) must
//! hold. Once an engine implements `Engine`, the target builds and
//! — for a pitch-preserving engine, which delivers the same PCM at
//! every rung — the contours are identical and the property holds.
//! The ±50-cent bounds are the contract; do not weaken them to
//! manufacture a green build.

use std::f64::consts::PI;

use engine_core::closed::PlaybackRate;
use engine_core::format::{DeclaredFormat, PITCH_TOLERANCE_CENTS, check_handoff};
use engine_core::standin_a::StandinA;
use engine_core::standin_b::StandinB;
use engine_core::trait_::{Engine, MAX_PREROLL_SECS, StreamChunk};

/// The estimator calibration bound: the deterministic estimators
/// must recover the source fundamental well inside the SC-015
/// bound, so the estimators' own error cannot mask a contract
/// violation.
const ESTIMATOR_TOLERANCE_CENTS: f64 = 25.0;

/// The reference rung every contour is compared against: `R1_0`
/// (normal speed), the comparison point of the SC-015 property.
const REFERENCE_RUNG: PlaybackRate = PlaybackRate::R1_0;

/// The F0 analysis window, in frames: 4096 frames ≈ 186 ms at the
/// declared 22050 Hz — long enough that the zero-crossing estimate
/// of the source fundamental stays within ±16 cents of the true
/// 220 Hz.
const WINDOW_FRAMES: usize = 4096;

/// The hop between consecutive analysis windows, in frames: 2048
/// frames ≈ 93 ms, so consecutive windows overlap by half.
const HOP_FRAMES: usize = 2048;

/// The pre-roll request per analysis stream, in seconds: 2.0 s at
/// the declared rate yields 44100 frames — 20 analysis windows.
const ANALYSIS_SECS: f64 = 2.0;

/// The stream position every analysis stream starts at, in seconds.
const ANALYSIS_POSITION_SECS: f64 = 0.0;

/// The source fundamental of the deterministic analysis signal, in
/// Hz: A3, comfortably inside the 40–800 Hz peak-search band.
const SOURCE_F0_HZ: f64 = 220.0;

/// The lowest bin the spectral peak picker searches, in Hz.
const PEAK_SEARCH_MIN_HZ: f64 = 40.0;

/// The highest bin the spectral peak picker searches, in Hz.
const PEAK_SEARCH_MAX_HZ: f64 = 800.0;

/// The identity of every rung, as a wildcard-free exhaustive match
/// (Principle VII, FR-044): an eighth rung added to `PlaybackRate`
/// fails to compile here — and in `rung_label` — until the
/// property covers it.
const fn rung_identity(rate: PlaybackRate) -> PlaybackRate {
    rate
}

/// Every rung of the closed ladder, in ascending order (FR-019):
/// exactly `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, built
/// through the exhaustive `rung_identity` match so the enumeration
/// itself is gated against an uncovered eighth rung.
const fn every_rung() -> [PlaybackRate; 7] {
    [
        rung_identity(PlaybackRate::R0_75),
        rung_identity(PlaybackRate::R1_0),
        rung_identity(PlaybackRate::R1_25),
        rung_identity(PlaybackRate::R1_5),
        rung_identity(PlaybackRate::R1_75),
        rung_identity(PlaybackRate::R2_0),
        rung_identity(PlaybackRate::R4_0),
    ]
}

/// The diagnostic label of every rung — also a wildcard-free
/// exhaustive match, so the closed set is gated twice over.
const fn rung_label(rate: PlaybackRate) -> &'static str {
    match rate {
        PlaybackRate::R0_75 => "0.75×",
        PlaybackRate::R1_0 => "1.0×",
        PlaybackRate::R1_25 => "1.25×",
        PlaybackRate::R1_5 => "1.5×",
        PlaybackRate::R1_75 => "1.75×",
        PlaybackRate::R2_0 => "2.0×",
        PlaybackRate::R4_0 => "4.0×",
    }
}

/// The number of frames a duration spans at the declared rate.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the analysis durations are positive and far below 2^53 frames, so the f64→usize conversion is exact for every duration this suite requests"
)]
fn frames_for_secs(secs: f64) -> usize {
    (secs * f64::from(DeclaredFormat::SAMPLE_RATE)) as usize
}

/// The deterministic source signal: a 220 Hz fundamental with two
/// decaying harmonics — `sin(2π·220·t) + 0.5·sin(2π·440·t) +
/// 0.25·sin(2π·660·t)` — at the declared 22050 Hz, phase-locked
/// from t = 0.
///
/// The harmonic amplitudes keep the summed waveform's zero
/// crossings exactly at the fundamental's: the waveform factors as
/// `sin(x)·(0.75 + cos(x) + cos²(x))` and the bracket's minimum
/// over x is 0.5 > 0, so there are exactly two crossings per
/// fundamental period and the zero-crossing estimator is unbiased
/// on the source.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the source signal is a deterministic test waveform; the f64→f32 narrowing is the declared handoff width (f32 PCM) and the estimator calibration tolerances absorb the rounding"
)]
fn source_frames() -> Vec<f32> {
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    let frame_count = frames_for_secs(ANALYSIS_SECS);
    let mut frames = Vec::with_capacity(frame_count);
    let mut phase = 0.0_f64;
    let phase_step = 2.0 * PI * SOURCE_F0_HZ / sample_rate_hz;
    for _ in 0..frame_count {
        let fundamental = phase.sin();
        let second_harmonic = (2.0 * phase).sin();
        let third_harmonic = (3.0 * phase).sin();
        frames.push(
            0.25f64.mul_add(third_harmonic, 0.5f64.mul_add(second_harmonic, fundamental)) as f32,
        );
        phase += phase_step;
    }
    frames
}

/// The start index of every analysis window of a frame sequence:
/// consecutive windows of [`WINDOW_FRAMES`] at [`HOP_FRAMES`]
/// spacing, dropping the tail shorter than one full window.
fn window_starts(frame_count: usize) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut start = 0;
    while start + WINDOW_FRAMES <= frame_count {
        starts.push(start);
        start += HOP_FRAMES;
    }
    starts
}

/// Estimate the fundamental frequency of one analysis window by
/// zero-crossing counting: twice the sign-change count per window
/// duration. Returns `None` when the window is flat or silent
/// (fewer than two crossings — no pitch to compare).
fn estimate_f0_hz(window: &[f32], sample_rate_hz: f64) -> Option<f64> {
    if window.len() < 2 {
        return None;
    }
    let length = u32::try_from(window.len()).ok()?;
    let mut crossings = 0_u32;
    let mut previous_non_negative = window[0] >= 0.0;
    for frame in &window[1..] {
        let non_negative = *frame >= 0.0;
        if non_negative != previous_non_negative {
            crossings += 1;
        }
        previous_non_negative = non_negative;
    }
    if crossings < 2 {
        return None;
    }
    Some(f64::from(crossings) / 2.0 * sample_rate_hz / f64::from(length))
}

/// The DFT magnitude of one bin (the direct DFT — no external
/// crates; O(n) per bin, which is ample for the fixed analysis
/// windows of this suite). The angle accumulates from a phase
/// accumulator, so the per-frame index never needs a float cast.
#[expect(
    clippy::cast_precision_loss,
    reason = "window length and bin index are small non-negative counts (≤ 4096 and ≤ 2048 here), so the usize→f64 conversion is exact"
)]
fn dft_magnitude(window: &[f32], bin: usize) -> f64 {
    let length_f64 = window.len() as f64;
    let angle_step = 2.0 * PI * (bin as f64) / length_f64;
    let mut angle = 0.0_f64;
    let mut real = 0.0_f64;
    let mut imaginary = 0.0_f64;
    for frame in window {
        real = f64::from(*frame).mul_add(angle.cos(), real);
        imaginary = f64::from(*frame).mul_add(angle.sin(), imaginary);
        angle += angle_step;
    }
    real.hypot(imaginary)
}

/// Pick the strongest spectral peak of one analysis window: the DFT
/// bin of largest magnitude within the 40–800 Hz search band,
/// refined by parabolic interpolation on the log-magnitude across
/// the peak bin and its two neighbours. Returns `None` when every
/// bin in the band is silent (a flat window has no peak to
/// compare).
///
/// The peak bin always sits at bin ≥ 1 (the band's lower edge is
/// `ceil(40 Hz · n / 22050 Hz)` ≥ 1 for any non-empty window), so
/// the left neighbour never underflows.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the peak search band edges and the parabolic refinement are computed from the window length and bin indices (small non-negative counts, exact in f64); the argmax and the curvature-zero check are the comparisons the algorithm is made of, with no epsilon tolerance to tune"
)]
fn spectral_peak_hz(window: &[f32], sample_rate_hz: f64) -> Option<f64> {
    let length = window.len();
    if length < 2 {
        return None;
    }
    let length_f64 = length as f64;
    let first_bin = ((PEAK_SEARCH_MIN_HZ * length_f64) / sample_rate_hz).ceil() as usize;
    let last_bin = ((PEAK_SEARCH_MAX_HZ * length_f64) / sample_rate_hz).floor() as usize;
    if first_bin > last_bin {
        return None;
    }
    let mut magnitudes = Vec::with_capacity(last_bin - first_bin + 1);
    for bin in first_bin..=last_bin {
        magnitudes.push(dft_magnitude(window, bin));
    }
    // The strongest bin in the band (the first maximum on ties).
    let mut best_offset = 0_usize;
    let mut best_magnitude = magnitudes[0];
    for (offset, magnitude) in magnitudes.iter().enumerate().skip(1) {
        if *magnitude > best_magnitude {
            best_offset = offset;
            best_magnitude = *magnitude;
        }
    }
    // Parabolic interpolation on the log-magnitude across the peak
    // bin and its two neighbours.
    let peak_bin = first_bin + best_offset;
    let peak_log = best_magnitude.ln();
    let left_log = dft_magnitude(window, peak_bin - 1).ln();
    let right_log = dft_magnitude(window, peak_bin + 1).ln();
    let curvature = 2.0f64.mul_add(-peak_log, left_log) + right_log;
    let bin_offset = if curvature.abs() < f64::EPSILON {
        0.0
    } else {
        0.5 * (left_log - right_log) / curvature
    }
    .clamp(-1.0, 1.0);
    Some((peak_bin as f64 + bin_offset) * sample_rate_hz / length_f64)
}

/// The pitch difference between two frequencies, in cents
/// (1200·log₂ of the ratio). Both frequencies are positive by
/// construction (an estimate exists only for a pitched window).
fn cents_between(candidate_hz: f64, reference_hz: f64) -> f64 {
    1200.0 * (candidate_hz / reference_hz).log2()
}

/// Set a rung through the [`Engine`] contract, surfacing a named
/// `EngineError` as a property failure: a conforming engine accepts
/// every rung with `Ok(())` — the contract's failure side.
fn set_rung<E: Engine>(
    engine: &mut E,
    engine_name: &str,
    rung: PlaybackRate,
) -> Result<(), String> {
    match Engine::set_rate(engine, rung) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "{engine_name} rejected rung {} with {error:?} — a conforming engine accepts every rung with Ok(())",
            rung_label(rung)
        )),
    }
}

/// Stream one analysis chunk from the engine: the pre-roll request
/// the contour estimators run over.
fn stream_analysis_chunk<E: Engine>(engine: &mut E) -> Result<StreamChunk, String> {
    match engine.stream(ANALYSIS_POSITION_SECS, ANALYSIS_SECS) {
        Ok(chunk) => Ok(chunk),
        Err(error) => Err(format!(
            "Engine::stream({ANALYSIS_POSITION_SECS}, {ANALYSIS_SECS}) failed with {error:?}"
        )),
    }
}

/// The preconditions a delivered chunk must meet for the property to
/// have signal to analyze: non-empty frames, every frame finite,
/// and at least one full analysis window. Without these the
/// property would be vacuous, so they are themselves contract
/// failures.
fn chunk_has_analysis_signal(
    chunk: &StreamChunk,
    engine_name: &str,
    rung: PlaybackRate,
) -> Result<(), String> {
    if chunk.frames.is_empty() {
        return Err(format!(
            "{engine_name} delivered no frames at rung {}",
            rung_label(rung)
        ));
    }
    if chunk.frames.iter().any(|frame| !frame.is_finite()) {
        return Err(format!(
            "{engine_name} delivered a non-finite frame at rung {}",
            rung_label(rung)
        ));
    }
    if chunk.frames.len() < WINDOW_FRAMES {
        return Err(format!(
            "{engine_name} delivered {} frames at rung {} — fewer than one {WINDOW_FRAMES}-frame analysis window",
            chunk.frames.len(),
            rung_label(rung)
        ));
    }
    Ok(())
}

/// The F0 contour of delivered frames: one zero-crossing estimate
/// per analysis window, in window order.
fn f0_contour(frames: &[f32]) -> Vec<Option<f64>> {
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    window_starts(frames.len())
        .into_iter()
        .map(|start| estimate_f0_hz(&frames[start..start + WINDOW_FRAMES], sample_rate_hz))
        .collect()
}

/// The spectral-peak contour of delivered frames: one strongest-bin
/// peak per analysis window, in window order.
fn peak_contour(frames: &[f32]) -> Vec<Option<f64>> {
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    window_starts(frames.len())
        .into_iter()
        .map(|start| spectral_peak_hz(&frames[start..start + WINDOW_FRAMES], sample_rate_hz))
        .collect()
}

/// Compare two contours window-by-window within a cents tolerance:
/// every window of the candidate must match the reference window at
/// the same index within `tolerance_cents`, and a window must not
/// gain or lose its estimate — a pitched window on one side and a
/// silent one on the other is a deviation, not a match. Only the
/// windows both contours hold are compared (an engine may deliver
/// fewer frames at one rung than another; the overlap is the
/// contract surface).
fn contour_matches_within(
    candidate: &[Option<f64>],
    reference: &[Option<f64>],
    tolerance_cents: f64,
    diagnostic: &str,
) -> Result<(), String> {
    if candidate.len().min(reference.len()) == 0 {
        return Err(format!("{diagnostic}: no analysis windows to compare"));
    }
    for (index, (candidate_hz, reference_hz)) in candidate.iter().zip(reference).enumerate() {
        match (candidate_hz, reference_hz) {
            (Some(candidate_hz), Some(reference_hz)) => {
                let deviation_cents = cents_between(*candidate_hz, *reference_hz);
                if deviation_cents.abs() > tolerance_cents {
                    return Err(format!(
                        "{diagnostic}: window {index} deviates {deviation_cents:.1} cents — only ±{tolerance_cents:.0} cents is allowed (SC-015)"
                    ));
                }
            }
            (None, None) => {
                // Both sides silent for this window: no pitch to
                // compare, and no deviation.
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(format!(
                    "{diagnostic}: window {index} has a detectable pitch on one side only"
                ));
            }
        }
    }
    Ok(())
}

/// The T055 property for one engine across the whole closed ladder
/// (SC-015, SC-034, FR-043): `set_rate` accepts every rung with
/// `Ok`, every delivered chunk is in the declared handoff shape
/// with analysis signal, and the F0 contour and the spectral-peak
/// contour of the delivered frames at each rung match the `R1_0`
/// reference contour within ±50 cents for every window.
///
/// A non-pitch-preserving engine — one whose delivered pitch
/// follows the rung, as a naive resample's would — fails these
/// bounds: its pitch at 4.0× rises 2400 cents above the reference,
/// far beyond the ±50-cent SC-015 margin, so the property fails on
/// the first violating window.
fn pitch_is_invariant_across_the_ladder<E: Engine>(
    engine_name: &'static str,
    engine: &mut E,
) -> Result<(), String> {
    // The reference contour at the R1_0 rung (SC-015's comparison
    // point).
    set_rung(engine, engine_name, REFERENCE_RUNG)?;
    let reference_chunk = stream_analysis_chunk(engine)?;
    chunk_has_analysis_signal(&reference_chunk, engine_name, REFERENCE_RUNG)?;
    let reference_f0 = f0_contour(&reference_chunk.frames);
    let reference_peaks = peak_contour(&reference_chunk.frames);

    for rung in every_rung() {
        set_rung(engine, engine_name, rung)?;
        let chunk = stream_analysis_chunk(engine)?;
        chunk_has_analysis_signal(&chunk, engine_name, rung)?;
        let diagnostic = format!(
            "{engine_name} at rung {} vs the 1.0× reference",
            rung_label(rung)
        );
        contour_matches_within(
            &f0_contour(&chunk.frames),
            &reference_f0,
            f64::from(PITCH_TOLERANCE_CENTS),
            &diagnostic,
        )?;
        contour_matches_within(
            &peak_contour(&chunk.frames),
            &reference_peaks,
            f64::from(PITCH_TOLERANCE_CENTS),
            &diagnostic,
        )?;
    }
    Ok(())
}

/// The T055 property for the harshest pair (SC-015, including the
/// 2.0×→4.0× pair): the F0 contour and the spectral-peak contour
/// of the 4.0× delivery match the 2.0× delivery within ±50 cents
/// for every window.
fn pitch_matches_across_the_two_to_four_pair<E: Engine>(
    engine_name: &'static str,
    engine: &mut E,
) -> Result<(), String> {
    set_rung(engine, engine_name, PlaybackRate::R2_0)?;
    let two_chunk = stream_analysis_chunk(engine)?;
    chunk_has_analysis_signal(&two_chunk, engine_name, PlaybackRate::R2_0)?;
    let two_f0 = f0_contour(&two_chunk.frames);
    let two_peaks = peak_contour(&two_chunk.frames);

    set_rung(engine, engine_name, PlaybackRate::R4_0)?;
    let four_chunk = stream_analysis_chunk(engine)?;
    chunk_has_analysis_signal(&four_chunk, engine_name, PlaybackRate::R4_0)?;
    let four_f0 = f0_contour(&four_chunk.frames);
    let four_peaks = peak_contour(&four_chunk.frames);

    contour_matches_within(
        &four_f0,
        &two_f0,
        f64::from(PITCH_TOLERANCE_CENTS),
        &format!("{engine_name}: the 2.0×→4.0× pair, F0 contour"),
    )?;
    contour_matches_within(
        &four_peaks,
        &two_peaks,
        f64::from(PITCH_TOLERANCE_CENTS),
        &format!("{engine_name}: the 2.0×→4.0× pair, spectral peaks"),
    )?;
    Ok(())
}

/// The analysis request stays within the trait's pre-roll bound:
/// [`Engine::stream`] rejects a `max_secs` above
/// [`MAX_PREROLL_SECS`] before any synthesis runs, so the property
/// could never reach the delivered frames of an over-bound
/// request.
#[expect(
    clippy::assertions_on_constants,
    reason = "the bound check 0 < ANALYSIS_SECS <= MAX_PREROLL_SECS is the pre-roll precondition itself, asserted against the two declared constants (no runtime input); a const block cannot carry the formatted message"
)]
#[test]
fn the_analysis_request_stays_within_the_pre_roll_bound() {
    assert!(ANALYSIS_SECS > 0.0);
    assert!(
        ANALYSIS_SECS <= MAX_PREROLL_SECS,
        "the {ANALYSIS_SECS} s analysis request exceeds the {MAX_PREROLL_SECS} s pre-roll bound"
    );
}

/// Estimator calibration, zero-crossing half: every analysis window
/// of the deterministic source signal recovers the 220 Hz
/// fundamental within [`ESTIMATOR_TOLERANCE_CENTS`] — well inside
/// the SC-015 margin — so the estimator's own error cannot mask a
/// contract violation.
#[test]
fn the_zero_crossing_estimator_recovers_the_source_fundamental() -> Result<(), String> {
    let frames = source_frames();
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    let starts = window_starts(frames.len());
    if starts.is_empty() {
        return Err(String::from(
            "the deterministic source signal holds no analysis window",
        ));
    }
    for start in starts {
        let window = &frames[start..start + WINDOW_FRAMES];
        let estimated_hz = estimate_f0_hz(window, sample_rate_hz).ok_or_else(|| {
            String::from("the zero-crossing estimator found no fundamental in a source window")
        })?;
        let deviation_cents = cents_between(estimated_hz, SOURCE_F0_HZ);
        if deviation_cents.abs() > ESTIMATOR_TOLERANCE_CENTS {
            return Err(format!(
                "the zero-crossing estimator recovered {estimated_hz} Hz — {deviation_cents:.1} cents from the {SOURCE_F0_HZ} Hz source fundamental, more than the ±{ESTIMATOR_TOLERANCE_CENTS:.0} calibration cents"
            ));
        }
    }
    Ok(())
}

/// Estimator calibration, spectral-peak half: every analysis window
/// of the deterministic source signal puts its strongest 40–800 Hz
/// peak at the 220 Hz fundamental within
/// [`ESTIMATOR_TOLERANCE_CENTS`] — the fundamental (amplitude 1.0)
/// dominates the decaying harmonics (0.5, 0.25), so the peak
/// picker's choice is the fundamental, not a harmonic.
#[test]
fn the_spectral_peak_picker_recovers_the_source_fundamental() -> Result<(), String> {
    let frames = source_frames();
    let sample_rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    let starts = window_starts(frames.len());
    if starts.is_empty() {
        return Err(String::from(
            "the deterministic source signal holds no analysis window",
        ));
    }
    for start in starts {
        let window = &frames[start..start + WINDOW_FRAMES];
        let peak_hz = spectral_peak_hz(window, sample_rate_hz).ok_or_else(|| {
            String::from("the spectral peak picker found no peak in a source window")
        })?;
        let deviation_cents = cents_between(peak_hz, SOURCE_F0_HZ);
        if deviation_cents.abs() > ESTIMATOR_TOLERANCE_CENTS {
            return Err(format!(
                "the spectral peak picker recovered {peak_hz} Hz — {deviation_cents:.1} cents from the {SOURCE_F0_HZ} Hz source fundamental, more than the ±{ESTIMATOR_TOLERANCE_CENTS:.0} calibration cents"
            ));
        }
    }
    Ok(())
}

/// SC-034 at every rung, through the stand-in's declared-format
/// accessor: after `set_rate` accepts each of the seven rungs with
/// `Ok`, the delivered shape is still the declared mono-`f32`-at-
/// 22050 Hz handoff (`rate_hz == 22050`, `channels == 1`,
/// `width_bits == 32`) and passes its own handoff check — a rate
/// change is a pitch-preserving time-stretch inside the engine
/// (FR-043) and never alters the delivered format.
#[test]
fn standin_a_delivers_the_declared_shape_at_every_rung() {
    let mut engine = StandinA;
    for rung in every_rung() {
        assert_eq!(
            set_rung(&mut engine, StandinA::ID, rung),
            Ok(()),
            "standin_a must accept rung {} with Ok(())",
            rung_label(rung)
        );
        let delivered = engine.delivered_format();
        assert_eq!(
            delivered,
            (
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            ),
            "standin_a deviated from the declared handoff shape at rung {}",
            rung_label(rung)
        );
        assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "standin_a's delivered shape failed its own handoff check at rung {}",
            rung_label(rung)
        );
    }
}

/// The same SC-034 shape property for the second stand-in: SC-051
/// needs two working engines for the mid-document failover drill,
/// and both honour the same handoff contract.
#[test]
fn standin_b_delivers_the_declared_shape_at_every_rung() {
    let mut engine = StandinB;
    for rung in every_rung() {
        assert_eq!(
            set_rung(&mut engine, StandinB::ID, rung),
            Ok(()),
            "standin_b must accept rung {} with Ok(())",
            rung_label(rung)
        );
        let delivered = engine.delivered_format();
        assert_eq!(
            delivered,
            (
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            ),
            "standin_b deviated from the declared handoff shape at rung {}",
            rung_label(rung)
        );
        assert!(
            check_handoff(delivered.0, delivered.1, delivered.2).is_ok(),
            "standin_b's delivered shape failed its own handoff check at rung {}",
            rung_label(rung)
        );
    }
}

/// The SC-015 ladder property for stand-in A: the F0 contour and
/// the spectral-peak contour of the delivered frames at each of the
/// seven rungs match the `R1_0` reference contour within ±50 cents
/// for every window (SC-015, SC-034, FR-043).
#[test]
fn standin_a_holds_pitch_across_all_seven_rungs() {
    let mut engine = StandinA;
    assert_eq!(
        pitch_is_invariant_across_the_ladder(StandinA::ID, &mut engine),
        Ok(()),
        "standin_a violated the SC-015 pitch-invariance property across the rate ladder"
    );
}

/// The SC-015 ladder property for stand-in B.
#[test]
fn standin_b_holds_pitch_across_all_seven_rungs() {
    let mut engine = StandinB;
    assert_eq!(
        pitch_is_invariant_across_the_ladder(StandinB::ID, &mut engine),
        Ok(()),
        "standin_b violated the SC-015 pitch-invariance property across the rate ladder"
    );
}

/// The SC-015 harshest pair for stand-in A: the contour difference
/// between the 2.0× delivery and the 4.0× delivery is within ±50
/// cents, for both the F0 contour and the spectral peaks.
#[test]
fn standin_a_keeps_pitch_across_the_two_to_four_pair() {
    let mut engine = StandinA;
    assert_eq!(
        pitch_matches_across_the_two_to_four_pair(StandinA::ID, &mut engine),
        Ok(()),
        "standin_a violated the SC-015 pitch-invariance property across the 2.0×→4.0× pair"
    );
}

/// The SC-015 harshest pair for stand-in B.
#[test]
fn standin_b_keeps_pitch_across_the_two_to_four_pair() {
    let mut engine = StandinB;
    assert_eq!(
        pitch_matches_across_the_two_to_four_pair(StandinB::ID, &mut engine),
        Ok(()),
        "standin_b violated the SC-015 pitch-invariance property across the 2.0×→4.0× pair"
    );
}
