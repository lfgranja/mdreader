//! The declared engine→shell handoff format (T015).
//!
//! The numeric values are fixed by the specification, not
//! by implementation (contracts/engine-handoff.md, FR-033):
//! planning owns stream negotiation against them and never
//! substitutes (FR-033, FR-035, SC-034). Rust owns the
//! synthesis and PCM playback; an engine delivers audio in
//! exactly this shape, already resampled — the shell only
//! converts to the device format.

use crate::closed::EngineError;

/// The declared engine→shell handoff format: mono `f32`
/// PCM at exactly `22050` Hz (FR-033,
/// contracts/engine-handoff.md).
///
/// The associated constants are the single declaration of
/// the shape; the handoff check ([`check_handoff`]) and
/// every stand-in engine derive their conformance from
/// these constants, so the shape cannot drift between
/// declaration and enforcement (FR-044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeclaredFormat;

impl DeclaredFormat {
    /// The declared sample rate, in Hz: `22050`.
    pub const SAMPLE_RATE: u32 = 22_050;
    /// The declared channel count: `1` (mono).
    pub const CHANNELS: u16 = 1;
    /// The declared sample width, in bits: `32` (one
    /// `f32` sample).
    pub const SAMPLE_WIDTH_BITS: u16 = 32;
}

/// Check an engine-delivered audio format against the
/// declared handoff shape (FR-033, FR-035).
///
/// The check fails closed: any deviation in rate, channel
/// count, or sample width is surfaced as a named
/// [`EngineError`] — never a silent resample, downmix, or
/// buffer reinterpretation (contracts/engine-handoff.md:
/// "Any deviation (rate, channel count, sample width) ⇒
/// named `EngineError` (`RateMismatch | ChannelMismatch`),
/// never silent resample/downmix"). There is no adapting
/// path through this function: a format either conforms to
/// the declaration or the check names the deviation.
///
/// # Errors
///
/// Returns [`EngineError::RateMismatch`] when `rate_hz`
/// is not the declared [`DeclaredFormat::SAMPLE_RATE`];
/// [`EngineError::ChannelMismatch`] when `channels` is
/// not the declared mono count; and a named [`EngineError`]
/// when `width_bits` is not the declared
/// [`DeclaredFormat::SAMPLE_WIDTH_BITS`] — the closed
/// error set names no width-specific variant (Principle
/// VII), so a non-`f32` width surfaces as
/// [`EngineError::RateMismatch`], one of the two errors
/// the contract groups all three deviation kinds under.
pub const fn check_handoff(
    rate_hz: u32,
    channels: u16,
    width_bits: u16,
) -> Result<(), EngineError> {
    if rate_hz != DeclaredFormat::SAMPLE_RATE {
        return Err(EngineError::RateMismatch);
    }
    if channels != DeclaredFormat::CHANNELS {
        return Err(EngineError::ChannelMismatch);
    }
    if width_bits != DeclaredFormat::SAMPLE_WIDTH_BITS {
        return Err(EngineError::RateMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_declared_shape_is_mono_f32_at_22050_hz() {
        assert_eq!(DeclaredFormat::SAMPLE_RATE, 22_050);
        assert_eq!(DeclaredFormat::CHANNELS, 1);
        assert_eq!(DeclaredFormat::SAMPLE_WIDTH_BITS, 32);
    }

    #[test]
    fn the_declared_shape_passes_its_own_check() {
        assert_eq!(
            check_handoff(
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS
            ),
            Ok(())
        );
    }

    #[test]
    fn a_rate_deviation_names_rate_mismatch() {
        for rate in [0, 1, 8_000, 22_049, 22_051, 44_100, u32::MAX] {
            assert_eq!(
                check_handoff(
                    rate,
                    DeclaredFormat::CHANNELS,
                    DeclaredFormat::SAMPLE_WIDTH_BITS
                ),
                Err(EngineError::RateMismatch),
                "rate {rate} Hz"
            );
        }
    }

    #[test]
    fn a_channel_deviation_names_channel_mismatch() {
        for channels in [0, 2, 6, u16::MAX] {
            assert_eq!(
                check_handoff(
                    DeclaredFormat::SAMPLE_RATE,
                    channels,
                    DeclaredFormat::SAMPLE_WIDTH_BITS
                ),
                Err(EngineError::ChannelMismatch),
                "channel count {channels}"
            );
        }
    }

    #[test]
    fn a_width_deviation_names_an_error_never_ok() {
        for width in [0, 8, 16, 24, 64, u16::MAX] {
            let result =
                check_handoff(DeclaredFormat::SAMPLE_RATE, DeclaredFormat::CHANNELS, width);
            assert!(
                matches!(
                    result,
                    Err(EngineError::RateMismatch) | Err(EngineError::ChannelMismatch)
                ),
                "width {width} bits was not surfaced as a named error"
            );
        }
    }
}
