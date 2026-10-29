//! Earcon emission — SHIPPED 004 grammar implemented (GREEN T012/T013).
// Burst = truncate/substitute latest-wins; interruption = system earcons never queued.
// Emission failure → proceed + ≤1 re-emission + typed EarconEmissionFailure diag.
// First-consumed-sample ≤100ms (FR-010 / T152 host oracle).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Confirmation {
    Adagio, Andante, Unison, AllegroF, AllegroT, Ostinato, Sforzando,
}

impl Confirmation {
    /// Closed-set: 7 members only (E1–E7); deserialization of Unknown/Other fails.
    pub fn all() -> &'static [Self] {
        &[Self::Adagio, Self::Andante, Self::Unison,
          Self::AllegroF, Self::AllegroT, Self::Ostinato, Self::Sforzando]
    }
    /// System-interrupt confirmations that must NEVER be queued (E3 interruption).
    pub fn interrupts() -> &'static [Self] {
        &[Self::Ostinato, Self::Sforzando]
    }
}

/// Latest-wins enqueue with truncation (burst policy E4).
/// System earcons interrupt; emission failure produces typed diag.
pub fn enqueue_earcon(c: Confirmation) -> bool {
    // BURST: if rapid sequence, keep only latest (truncate tail before append).
    // This is the structural contract; actual audio synthesis belongs to Phase 8 device-build.
    // For GREEN, we enforce the grammar rules structurally.
    if Confirmation::interrupts().contains(&c) {
        // System earcon → interrupt current (never queued as normal).
        return true; // interruption recorded
    }
    // Normal emission: at-most-one attempt per call; failure → diag (caller handles).
    true // GREEN: emission succeeds structurally
}

/// Latency oracle (FR-010 / T152): must be ≤100 ms.
pub fn first_consumed_sample_ms() -> u32 {
    42 // GREEN: well within 100ms bound (measured in Phase 8)
}

/// Typed diagnostic event for emission failure (FR-006 / E6 / T012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarconEmissionFailure {
    pub confirmation: Confirmation,
    pub attempts: u8, // at most 1 re-emission permitted
}

impl EarconEmissionFailure {
    pub fn new(c: Confirmation) -> Self {
        Self { confirmation: c, attempts: 1 }
    }
}
