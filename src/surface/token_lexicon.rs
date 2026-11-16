//! DESIGN-RESSOA token lexicon — closed 10-member set (RessoaToken + 2 signal; data-model.md §RessoaToken).
//! No extension permitted (SC-007 / gate 7); no Unknown / Other arm; negative deserialization fails.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RessoaToken {
    RessoaVoid,    // #0B0F1A
    RessoaStage,   // #141B2D
    RessoaVoice,   // #F2F0E6 (accent-only notification; primary text + progress)
    RessoaFocus,   // #8FD8E8 (focus ring only)
    RessoaHush,    // #A9B8CE (current-section text)
    RessoaHarp,    // #E8C46A (rate-dot, value-blind)
    RessoaCeleste, // #B9A6F5 (reserved, unpainted — vacancy, not removal)
    RessoaBoundary, // #55688F (progress track)
    SignalCaution, // #FFD400 (warning banner ONLY)
    SignalFailure, // #FF4D63 (failure banner ONLY)
}

impl RessoaToken {
    pub const COUNT: usize = 10;
    pub fn all() -> &'static [Self] {
        &[Self::RessoaVoid, Self::RessoaStage, Self::RessoaVoice, Self::RessoaFocus,
          Self::RessoaHush, Self::RessoaHarp, Self::RessoaCeleste, Self::RessoaBoundary,
          Self::SignalCaution, Self::SignalFailure]
    }
}
