//! Format verification — mono f32 @ 22050 Hz; named failure (never silent).
use crate::failure::EngineError;

pub fn check_format(sample_rate: u32, channels: u16, sample_fmt: &str) -> Result<(), EngineError> {
    if sample_rate != 22050 || channels != 1 || sample_fmt != "f32" {
        return Err(EngineError::FormatMismatch);
    }
    Ok(())
}
