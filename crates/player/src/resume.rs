//! T041 — Resume / sentence-snap / 3 s expiry (FR-015 / FR-032c / FR-041 / SC-041).
//! Deliberate-pause: visible 3 s offer, continue-default + start-over; expires everywhere after 3 s.
//! Unexpected-end: paused + explicit resume control; never auto-starts.
//! Resume lands at sentence start; total backward ≤30 s (FR-015 / SC-041).

use crate::transport::{Position, TransportState};

pub fn resume_offer_window(secs_since_pause: f64, is_deliberate_pause: bool) -> Option<ResumeChoice> {
    if secs_since_pause > 3.0 {
        return None; // expired everywhere after 3 s (FR-041)
    }
    if is_deliberate_pause {
        Some(ResumeChoice::ContinueDefault)
    } else {
        Some(ResumeChoice::ExplicitResume)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    ContinueDefault,
    ExplicitResume,
    StartOver,
}
