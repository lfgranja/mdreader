//! Pipeline wrapper — sherpa-onnx + Piper pt-BR (offline_default); design-phase.
//! Actual synthesis deferred to engine integration; adapter declares capability only.

/// Design-phase pipeline wrapper (sherpa-onnx + Piper).
pub fn synthesize_chunk(_chunk: &str) -> Result<Vec<f32>, crate::failure::EngineError> {
    // Design-phase stub: returns empty frame; real pipeline needs sherpa-onnx
    // + Piper model binding (faber CC0) at integration stage.
    Ok(Vec::new())
}
