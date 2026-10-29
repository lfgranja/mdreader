//! Reduced Synesthetic Lens gate — US4 P3 (T022).
// Pre-defined token set only; no extra dimensions; single-column preserved.

use super::token_lexicon::SonaToken;

pub fn allowed_in_reduced() -> &'static [SonaToken] {
    // Subset of full lexicon permitted in reduced mode (structural — exact subset TBD in Phase 8).
    &[SonaToken::SonaVoice, SonaToken::SonaStage, SonaToken::SignalCaution, SonaToken::SignalFailure]
}
