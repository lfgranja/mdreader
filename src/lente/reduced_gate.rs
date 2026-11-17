//! Reduced Synesthetic Lens gate — US4 P3 (T022).
// Pre-defined token set only; no extra dimensions; single-column preserved.

use super::token_lexicon::RessoaToken;

pub fn allowed_in_reduced() -> &'static [RessoaToken] {
    // Subset of full lexicon permitted in reduced mode (structural — exact subset TBD in Phase 8).
    &[RessoaToken::RessoaVoice, RessoaToken::RessoaStage, RessoaToken::SignalCaution, RessoaToken::SignalFailure]
}
