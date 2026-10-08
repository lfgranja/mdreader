//! Format verification (FR-024b / SC-035).
//!
//! The delivered audio format must be exactly the
//! declared mono-`f32`-at-22050 Hz shape, and a
//! delivered buffer must carry exactly the sample count
//! its duration implies — or the check names a
//! closed-set [`EngineError`]. Never a silent resample,
//! never a silent downmix, never a silent pass
//! (contracts/adapter-handoff.md §2.1; engine-core
//! `format.rs`; spec.md:42-45; data-model.md rules
//! R1/R2).

use std::cmp::Ordering;

use engine_core::format::DeclaredFormat;

use crate::failure::EngineError;

/// The declared sample rate, in Hz.
///
/// Re-exported so callers need not depend on
/// `engine-core` directly. Derived from
/// [`DeclaredFormat::SAMPLE_RATE`] — the single
/// declaration of the shape — so it cannot drift
/// (FR-044).
pub const DECLARED_RATE_HZ: u32 = DeclaredFormat::SAMPLE_RATE;

/// The declared channel count (mono).
///
/// Re-exported so callers need not depend on
/// `engine-core` directly. Derived from
/// [`DeclaredFormat::CHANNELS`] (FR-044).
pub const DECLARED_CHANNELS: u16 = DeclaredFormat::CHANNELS;

/// The declared sample width, in bits (one `f32`
/// sample).
///
/// Re-exported so callers need not depend on
/// `engine-core` directly. Derived from
/// [`DeclaredFormat::SAMPLE_WIDTH_BITS`] (FR-044).
pub const DECLARED_WIDTH_BITS: u16 = DeclaredFormat::SAMPLE_WIDTH_BITS;

/// Half a sample — the delivery length tolerance
/// (`API_CONTRACT` §6; data-model E2). The band is
/// inclusive: a deviation of exactly half a sample is
/// inside it.
const HALF_SAMPLE: f64 = 0.5;

/// Verify a delivered format against the declared handoff
/// shape ([`engine_core::format::DeclaredFormat`]).
///
/// Delegates to `engine_core::format::check_handoff` —
/// the single declaration of the shape — so the adapter's
/// check cannot drift from the contract (FR-044). Returns
/// `Ok(())` if and only if `(rate_hz, channels,
/// width_bits)` is exactly `(22050, 1, 32)`; any
/// deviation names a member of the closed error set:
/// `RateMismatch` for a rate deviation (a sample-width
/// deviation is grouped under the rate shape per
/// Principle VII — the closed set names no width-specific
/// variant) and `ChannelMismatch` for a channel-count
/// deviation.
///
/// # Errors
///
/// [`EngineError::RateMismatch`] for a rate or width
/// deviation; [`EngineError::ChannelMismatch`] for a
/// channel-count deviation.
#[must_use = "the verdict is the gate: a deviation must be observed, never silently passed"]
pub const fn check_format(rate_hz: u32, channels: u16, width_bits: u16) -> Result<(), EngineError> {
    engine_core::format::check_handoff(rate_hz, channels, width_bits)
}

/// Verify a delivered sample buffer: shape first, then
/// buffer length.
///
/// The shape is delegated to [`check_format`] — and
/// through it to `engine_core::format::check_handoff`,
/// the single declaration of the shape (FR-044) — so a
/// shape deviation is named before any length is
/// considered: a wrong rate or width names `RateMismatch`,
/// a wrong channel count names `ChannelMismatch`, whatever
/// the buffer length.
///
/// Only when the shape conforms is the length checked:
/// `pcm_sample_count` must agree with `(duration_ms /
/// 1000.0) * DECLARED_RATE_HZ` within half a sample,
/// because `duration_ms` is a derived `f64` (data-model
/// E2; CHK007). A buffer outside that band names
/// `EngineError::RateMismatch` — never a silent resample,
/// never a silent pass (FR-024b, SC-035).
///
/// A non-finite `duration_ms` (NaN, ±inf) is not a
/// derivable duration — `duration_ms` is computed from a
/// buffer length — so it fails closed as a `RateMismatch`
/// rather than being silently passed.
///
/// The length comparison avoids floating-point
/// comparison operators (denied via the pedantic
/// group): the absolute deviation is routed through
/// `f64::total_cmp` into the [`Ordering`] domain,
/// and the band is inclusive.
///
/// # Errors
///
/// Whatever [`check_format`] names for a shape deviation;
/// otherwise [`EngineError::RateMismatch`] when the buffer
/// length disagrees with the duration by more than half a
/// sample, or when the duration is not finite.
#[must_use = "the verdict is the gate: a deviation must be observed, never silently passed"]
pub fn check_delivery(
    rate_hz: u32,
    channels: u16,
    width_bits: u16,
    pcm_sample_count: usize,
    duration_ms: f64,
) -> Result<(), EngineError> {
    check_format(rate_hz, channels, width_bits)?;
    if !duration_ms.is_finite() {
        return Err(EngineError::RateMismatch);
    }
    let expected_samples = (duration_ms / 1000.0) * f64::from(DECLARED_RATE_HZ);
    let deviation = (pcm_sample_count as f64 - expected_samples).abs();
    match deviation.total_cmp(&HALF_SAMPLE) {
        Ordering::Greater => Err(EngineError::RateMismatch),
        Ordering::Equal | Ordering::Less => Ok(()),
    }
}
