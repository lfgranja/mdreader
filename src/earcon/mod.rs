//! Earcon emission stub — SHIPPED 004 grammar not yet implemented (RED phase).

pub enum Confirmation {
    Adagio, Andante, Unison, AllegroF, AllegroT, Ostinato, Sforzando,
}

/// Placeholder: does NOT yet enforce burst/truncate/latest-wins/interruption.
pub fn enqueue_earcon(_c: Confirmation) -> bool {
    false // RED: emission not implemented
}

/// Placeholder: latency oracle always fails (RED).
pub fn first_consumed_sample_ms() -> u32 {
    9999 // RED: >100ms; must be ≤100
}
