//! EarconKind — procedural audio confirmation grammar (FR-016 / SC-004 / SC-008).
//! Mono f32 @ 22050 Hz; no `.wav`/`SoundPool`/`ToneGenerator` assets.
//!
//! Closed set: exactly the seven named variants below. No `Other`/`Unknown`
//! — every surface match must be exhaustive with no `_ =>` catch-all.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EarconKind {
    Play,
    Pause,
    SkipFwd,
    SkipBack,
    Warning,
    Failure,
    RateStep,
}
