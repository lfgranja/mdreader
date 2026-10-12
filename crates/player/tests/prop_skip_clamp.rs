//! T107 — Phase 12 Convergence skip-clamp property test (BLOCKING).
//!
//! Invariant under test: skip displaces the position by exactly ±5.0 s,
//! clamped into `[0.0, duration]`, and never panics and never errors
//! (FR-018 / SC-016).
//!
//! Spec anchors: FR-018 ("Skip MUST move the playback position by
//! exactly 5 seconds — forward for skip-forward, backward for
//! skip-backward — measured against the position at the moment the
//! control was pressed", clamped to `[0.0, duration]`), SC-016
//! (clamp-without-error: a skip that would leave the document bounds
//! clamps to the bound instead of reporting an error), and the
//! Constitution Rust Code Rigor rules (no `panic!`/`assert!` in shipped
//! library code).
//!
//! Why this test exists: the two pre-convergence skip paths both
//! violate the invariant — `player::skip::apply_skip_exact`
//! (`crates/player/src/skip.rs`) takes a raw `delta: f64` and enforces
//! the ±5.0 s shape with `assert!` (a panic in shipped library code),
//! and floors at `0.0` only (no duration ceiling); the dead
//! `player::transport::apply_skip` (`crates/player/src/transport.rs`)
//! returns `Err(OutOfBounds)` on `target < 0.0`, contradicting the
//! SC-016 clamp-without-error rule. The canonical behavior today lives
//! only as the session-level composition in
//! `player::cli::PlaybackSession::skip_landing` (`apply_skip_exact`
//! plus `.min(duration)`).
//!
//! Expected API contract for T108 (the production code this test gates
//! — it does not exist yet, which is the required red state):
//! - `player::skip::skip_clamped(position: f64, direction:
//!   SkipDirection, duration: f64) -> f64` — exactly `+5.0` for
//!   `SkipDirection::Forward`, exactly `-5.0` for
//!   `SkipDirection::Backward`, clamped into `[0.0, duration]`.
//!   `SkipDirection::{Forward, Backward}` is the closed two-member set
//!   re-exported from its home in `player::transport` (FR-018); the
//!   direction-typed signature makes an off-±5.0 displacement
//!   unrepresentable, which is the fix for the `assert!`.
//!
//! Deliberately absent: a second proptest over an arbitrary `delta: f64`
//! is NOT needed — the direction-typed signature admits only the two
//! closed displacements, so there is no third input to fuzz (that
//! unrepresentability is what retires the `assert!` and the
//! `SkipDisplacement` error arm).
//!
//! Exactness note: the oracle is computed with the same expression the
//! contract states — `(position ± 5.0).max(0.0).min(duration)` — so the
//! comparison is bit-exact. A tolerance would wrongly admit an off-step
//! skip (FR-018 demands *exactly* ±5.0 s) or a clamp violation (SC-016
//! bounds are hard ceilings, not targets).
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures (`{var}`) —
//! proptest builds the message through `concat!`, and rustc rejects
//! implicit captures in macro-expanded format strings.

use proptest::prelude::*;

use player::skip::skip_clamped;
use player::transport::SkipDirection;

/// The closed skip-direction set (FR-018): exactly the two displacements
/// the signature admits — no third input exists to fuzz.
fn direction_strategy() -> impl Strategy<Value = SkipDirection> {
    prop_oneof![Just(SkipDirection::Forward), Just(SkipDirection::Backward),]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn skip_clamped_displaces_exactly_five_seconds_clamped_to_zero_and_duration(
        position in 0.0f64..=7200.0f64,
        direction in direction_strategy(),
        duration in 0.0f64..=7200.0f64,
    ) {
        // FR-018: exactly ±5.0 s against the press-time position;
        // SC-016: clamped into [0.0, duration], never an error.
        let delta = match direction {
            SkipDirection::Forward => 5.0,
            SkipDirection::Backward => -5.0,
        };
        let expected = (position + delta).max(0.0).min(duration);
        // The call must never panic (no assert!-on-±5.0, no
        // OutOfBounds path): a panic fails the case, so reaching the
        // assertion below over the whole input space is the no-panic
        // proof.
        let landed = skip_clamped(position, direction, duration);
        prop_assert_eq!(
            landed,
            expected,
            "a {:?} skip from {} with duration {} landed at {}, expected the clamped ±5.0 s point {}",
            direction,
            position,
            duration,
            landed,
            expected,
        );
    }
}
