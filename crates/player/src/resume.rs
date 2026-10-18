//! T041 — Resume / sentence-snap / 3 s expiry (FR-015 / FR-032c / FR-041 / SC-041).
//!
//! Deliberate-pause: visible 3 s offer, continue-default + start-over; expires everywhere after 3 s.
//! Unexpected-end: paused + explicit resume control; never auto-starts.
//! Resume lands at sentence start; total backward ≤30 s (FR-015 / SC-041).

/// The resume-offer window (FR-032c / FR-041): a deliberate
/// pause offers continue-by-default, an unexpected end offers
/// an explicit resume control.
///
/// Both expire everywhere after 3 s, returning `None` (never
/// auto-resume, FR-014).
#[must_use]
pub fn resume_offer_window(
    secs_since_pause: f64,
    is_deliberate_pause: bool,
) -> Option<ResumeChoice> {
    if secs_since_pause > 3.0 {
        return None; // expired everywhere after 3 s (FR-041)
    }
    if is_deliberate_pause {
        Some(ResumeChoice::ContinueDefault)
    } else {
        Some(ResumeChoice::ExplicitResume)
    }
}

/// The choice a resume offer presents — the closed
/// three-member set (FR-032c / SC-041).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    /// Deliberate pause: continue from the yield point
    /// (after the rewind/snap) without asking.
    ContinueDefault,
    /// Unexpected end: offer an explicit resume control.
    ExplicitResume,
    /// Start over from 0.0 s.
    StartOver,
}
