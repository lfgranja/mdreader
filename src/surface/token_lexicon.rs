//! DESIGN-SONA token lexicon — closed 10-member set (SonaToken + 2 signal; data-model.md §SonaToken).
// No extension permitted (SC-007 / gate 7); no Unknown / Other arm; negative deserialization fails.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SonaToken {
    SonaVoid,    // #0B0F1A
    SonaStage,   // #141B2D
    SonaVoice,   // #F2F0E6 (accent-only notification; primary text + progress)
    SonaEtwahl,  // #8FD8E8 (focus ring only)
    SonaHush,    // #A9B8CE (current-section text)
    SonaHarp,    // #E8C46A (rate-dot, value-blind)
    SonaCeleste, // #B9A6F5 (reserved, unpainted — vacancy, not removal)
    SonaBoundary, // #55688F (progress track)
    SignalCaution, // #FFD400 (warning banner ONLY)
    SignalFailure, // #FF4D63 (failure banner ONLY)
}

impl SonaToken {
    pub const COUNT: usize = 10;
    pub fn all() -> &'static [Self] {
        &[Self::SonaVoid, Self::SonaStage, Self::SonaVoice, Self::SonaEtwahl,
          Self::SonaHush, Self::SonaHarp, Self::SonaCeleste, Self::SonaBoundary,
          Self::SignalCaution, Self::SignalFailure]
    }
}
