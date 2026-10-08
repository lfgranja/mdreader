//! EngineError — closed 7-member set (Principle VII; spec.md:85 / C1–C4).
//! No Other / Unknown / catch-all; never expand without coordinated edit + semver.

/// Closed 7-member set — never expand without coordinated edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    /// Transport unreachable.
    Unreachable,
    /// Init failed.
    InitFailed,
    /// Rate deviation.
    RateMismatch,
    /// Format not mono f32 @ 22050.
    FormatMismatch,
    /// Chunk >5000 bytes.
    ChunkTooLarge,
    /// Pipeline failure.
    SynthesisFailed,
    /// Reservation failure.
    ReservationFailed,
}
