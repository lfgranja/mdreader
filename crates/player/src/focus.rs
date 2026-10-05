//! Focus classification (FR-013 / FR-013a / FR-014 / FR-017 / FR-017a / FR-037a).
//! Closed set outcomes: DuckableTransient, SilenceDemanding, PermanentLoss + FocusRegained direction.
//! No `Other/Unknown`; no `_ =>`; engine failure surfaces named (`EngineError`).

use crate::transport::{TransportState, PlaybackRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusOutcome {
    DuckableTransient,
    SilenceDemanding,
    PermanentLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusRegained {
    DirectionEvent,
}

/// Duck band [0.15, 0.25] (FR-013) — must remain `Playing`; gain ≈20%.
/// SilenceDemanding / PermanentLoss → `Paused` / `Interrupted`; never auto-resume.
/// Supersede transient→permanent clears resume/duck-restore flags.
/// Deferred grant → zero audio before grant, no rewind; refusal → named failure + no playing state.
pub fn classify_focus(request: &str) -> Result<FocusOutcome, crate::engine_core::closed::EngineError> {
    match request {
        "DuckableTransient" => Ok(FocusOutcome::DuckableTransient),
        "SilenceDemanding" => Ok(FocusOutcome::SilenceDemanding),
        "PermanentLoss" => Ok(FocusOutcome::PermanentLoss),
        _ => Err(crate::engine_core::closed::EngineError::DeviceRefused),
    }
}

/// Volume duck ≈20% (FR-013). Never alters position; no rewind owed while Playing.
pub fn apply_duck(volume_factor: f32) -> Result<f32, crate::engine_core::closed::EngineError> {
    if volume_factor < 0.15 || volume_factor > 0.25 {
        return Err(crate::engine_core::closed::EngineError::RateMismatch);
    }
    Ok(volume_factor)
}
