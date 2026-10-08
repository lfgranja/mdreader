//! Property: valid = mono f32 @ 22050 Hz; invalid = named EngineError (7 members).
// Closed-set: no Other/Unknown; never silent pass.

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    Unreachable,
    InitFailed,
    RateMismatch,
    FormatMismatch,
    ChunkTooLarge,
    SynthesisFailed,
    ReservationFailed,
}

pub fn verify_format(sample_rate: u32, channels: u16, sample_format: &str) -> Result<(), EngineError> {
    if sample_rate != 22050 || channels != 1 || sample_format != "f32" {
        return Err(EngineError::FormatMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod prop_tests {
    use super::*;

    #[test]
    fn valid_format_accepted() {
        assert!(verify_format(22050, 1, "f32").is_ok());
    }
    #[test]
    fn wrong_rate_named_error() {
        assert_eq!(verify_format(44100, 1, "f32"), Err(EngineError::FormatMismatch));
    }
    #[test]
    fn stereo_named_error() {
        assert_eq!(verify_format(22050, 2, "f32"), Err(EngineError::FormatMismatch));
    }
}
