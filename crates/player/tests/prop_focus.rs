//! T033 — US2 property: focus classification + duck band (FR-013 / FR-013a / FR-014).
//! Invariant: DuckableTransient → gain [0.15,0.25] AND Playing; SilenceDemanding|PermanentLoss → Paused/Interrupted.
//! FAILS before T037.

#[cfg(test)]
mod tests {
    #[test]
    fn prop_focus_classification() {
        assert!(false, "T033 RED: focus classification — T037 missing");
    }
}
