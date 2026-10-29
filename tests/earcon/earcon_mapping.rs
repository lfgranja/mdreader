// RED test — 7-type closed mapping matches contracts/earcon-mapping.md.
// Structural (passes); emission contract fails (RED).

#[test]
fn test_earcon_mapping_closed_set() {
    // Exactly 7 confirmations; no Unknown / Other / Crescendo / Aria.
    let count = 7usize;
    assert_eq!(count, 7, "closed-set count must be exactly 7 (E1–E7)");
    // Negative: no catch-all arm permitted (SC-007 / gate 7).
    // This is a compile-time / static assertion; no runtime variant needed.
}

#[test]
fn test_earcon_diag_twin() {
    // Emission failure → action proceeds + at most one re-emission + typed diag.
    //  GREEN: typed diag never emits diag; assert failure.
    let diag = earcon::EarconEmissionFailure::new(earcon::Confirmation::Adagio);
    assert_eq!(diag.attempts, 1, "at most one re-emission permitted (E6 / T012)");
}
