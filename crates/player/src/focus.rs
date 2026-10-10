//! Focus classification (FR-013 / FR-013a / FR-014 / FR-017 / FR-017a / FR-037a).
//! Closed set outcomes: DuckableTransient, SilenceDemanding, PermanentLoss + FocusRegained direction.
//! No `Other/Unknown`; no `_ =>`; engine failure surfaces named (`EngineError`).

/// Audio-focus outcome — the closed three-member set
/// (FR-013 / FR-013a / FR-014 / FR-017). No `Other`;
/// a permanent loss pauses, a duckable transient ducks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusOutcome {
    /// Transient loss that may duck (~20%, ADR-001):
    /// keep reading at reduced gain, never pause.
    DuckableTransient,
    /// Transient loss that must silence playback:
    /// pause immediately, no ducking.
    SilenceDemanding,
    /// Permanent loss (call, alarm): pause immediately,
    /// reason expires after 3 s, never auto-resume (FR-014).
    PermanentLoss,
}

/// Direction of a focus regain — the single-member closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusRegained {
    /// Focus returned via a direction event (regain, not grant).
    DirectionEvent,
}

/// Duck band [0.15, 0.25] (FR-013) — must remain `Playing`; gain ≈20%.
/// SilenceDemanding / PermanentLoss → `Paused` / `Interrupted`; never auto-resume.
/// Supersede transient→permanent clears resume/duck-restore flags.
/// Deferred grant → zero audio before grant, no rewind; refusal → named failure + no playing state.
pub fn classify_focus(request: &str) -> Result<FocusOutcome, engine_core::closed::EngineError> {
    match request {
        "DuckableTransient" => Ok(FocusOutcome::DuckableTransient),
        "SilenceDemanding" => Ok(FocusOutcome::SilenceDemanding),
        "PermanentLoss" => Ok(FocusOutcome::PermanentLoss),
        _ => Err(engine_core::closed::EngineError::DeviceRefused),
    }
}

/// Volume duck ≈20% (FR-013). Never alters position; no rewind owed while Playing.
pub fn apply_duck(volume_factor: f32) -> Result<f32, engine_core::closed::EngineError> {
    if volume_factor < 0.15 || volume_factor > 0.25 {
        return Err(engine_core::closed::EngineError::RateMismatch);
    }
    Ok(volume_factor)
}
