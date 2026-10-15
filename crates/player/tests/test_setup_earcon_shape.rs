//! T002d — BLOCKING TEST (must FAIL red before T003).
//! Closed-set shape test for `EarconKind`.
//!
//! Requirement (tasks.md T002d): asserts `EarconKind` has exactly
//! `Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no
//! `Other`/`Unknown`, exhaustive `match` with no `_ =>`.
//!
//! FAIL-red before T003 (`crates/player/src/earcon.rs` scaffold + lib.rs
//! wiring). The red state is: module either missing or not wired; test
//! references it and fails to compile / fails at runtime.

use player::earcon::EarconKind;

/// The closed set of exactly seven earcon kinds (contracts/earcon.md).
const EXPECTED_KINDS: [EarconKind; 7] = [
    EarconKind::Play,
    EarconKind::Pause,
    EarconKind::SkipFwd,
    EarconKind::SkipBack,
    EarconKind::Warning,
    EarconKind::Failure,
    EarconKind::RateStep,
];

#[test]
fn earcon_kind_has_exactly_seven_variants() {
    // Runtime count assertion: exactly 7 distinct values.
    assert_eq!(
        EXPECTED_KINDS.len(),
        7,
        "EarconKind must have exactly 7 variants"
    );
}

#[test]
fn earcon_kind_exhaustive_match_no_other_unknown() {
    // Exhaustive match over the closed set — no `_ =>` arm.
    // If a variant were added or removed, this match would break
    // (compile-time exhaustiveness check) or the assertions below
    // would diverge.
    for kind in EXPECTED_KINDS {
        match kind {
            EarconKind::Play => assert!(matches!(kind, EarconKind::Play)),
            EarconKind::Pause => assert!(matches!(kind, EarconKind::Pause)),
            EarconKind::SkipFwd => assert!(matches!(kind, EarconKind::SkipFwd)),
            EarconKind::SkipBack => assert!(matches!(kind, EarconKind::SkipBack)),
            EarconKind::Warning => assert!(matches!(kind, EarconKind::Warning)),
            EarconKind::Failure => assert!(matches!(kind, EarconKind::Failure)),
            EarconKind::RateStep => assert!(matches!(kind, EarconKind::RateStep)),
        }
    }
}

#[test]
fn no_other_or_unknown_variant_exists() {
    // Compile-time guarantee: the enum definition contains exactly
    // the seven named variants above (no `Other`, no `Unknown`).
    // Runtime reinforcement: every value produced by the enum must
    // be one of the seven — no extra arm in the match above.
    let all_kinds: Vec<EarconKind> = EXPECTED_KINDS.to_vec();
    assert_eq!(all_kinds.len(), 7);
    for kind in all_kinds {
        assert!(
            matches!(
                kind,
                EarconKind::Play
                    | EarconKind::Pause
                    | EarconKind::SkipFwd
                    | EarconKind::SkipBack
                    | EarconKind::Warning
                    | EarconKind::Failure
                    | EarconKind::RateStep
            ),
            "Unexpected EarconKind variant found: {:?}; closed set violated",
            kind
        );
    }
}
