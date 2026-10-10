//! T032 — US2 property: rewind/snap/cap (FR-015, FR-018, FR-038, FR-019a pitch).
//! Invariant: cursor = min(0.05 * paused_duration_secs, 30.0); resume sentence start;
//! total backward ≤30.0 s even when sentence > cap (land exactly cap point);
//! cap-point mid-word snaps forward to next inter-word gap unless gap beyond cap.
//! FAILS before T037 exists.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    #[test]
    fn prop_rewind_cap_30s() {
        assert!(
            false,
            "T032 RED: rewind cap 30 s invariant — implementation T037 missing"
        );
    }
}
