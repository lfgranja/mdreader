//! EngineError — closed 7-member set (Principle VII; spec.md:85 / C1–C4).
//! No Other / Unknown / catch-all; never expand without coordinated edit + semver.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    Unreachable,
    InitFailed,
    RateMismatch,
    FormatMismatch,
    ChunkTooLarge,
    SynthesisFailed,
    ReservationFailed,
}
