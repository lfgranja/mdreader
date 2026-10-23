//! T122 — Phase 13 BLOCKING TEST (RED): backend gain-setter law
//! (FR-013 duck gain applied through the CPAL backend).
//!
//! Pins EXACTLY this public API (created by T123; missing today, so
//! this suite is RED — compilation failure on the unresolved
//! `set_gain` / `gain` / `InvalidGain` items):
//!
//! - `CpalBackend::set_gain(&self, factor: f32) -> Result<(),
//!   BackendError>` where `BackendError` is
//!   `player::backend::BackendError` (declared in
//!   `crates/player/src/backend.rs`) extended with a documented
//!   `InvalidGain(f32)` variant (backend-domain error — the gain
//!   actuator lives in the backend, not the session CLI);
//! - `pub fn gain(&self) -> f32` introspection: default `1.0` (the
//!   `FULL_GAIN` constant in `crates/player/src/focus.rs`); reflects
//!   the last accepted set.
//!
//! Laws under test:
//!
//! - any factor in `[0.0, 1.0]` — including `0.0`, the nominal duck
//!   factor `0.20` (`DUCK_GAIN`) and full gain `1.0` (`FULL_GAIN`) —
//!   is accepted and `gain()` echoes it back exactly;
//! - any factor outside `[0.0, 1.0]` (strictly negative, strictly
//!   above `1.0`, both infinities, and `NaN`) is rejected with the
//!   named `BackendError::InvalidGain` error and `gain()` is unchanged.
//!
//! Payload-shape note: rejection is asserted as
//! `Err(BackendError::InvalidGain(_))` — a single-field tuple holding
//! the offending factor, mirroring the existing
//! `NegativeProgress(f64)` precedent in `cli::PlayerError`.
//! T123 must conform to this shape.
//!
//! Out of scope: PCM-sample scaling itself is pinned white-box by
//! T123's in-file unit tests — this suite pins the public setter law
//! only, without forcing further API surface.
//!
//! Hardware note: `CpalBackend::new` needs an output device. Cases
//! that find none pass vacuously — the same convention as the
//! `#[ignore = "requires audio hardware"]` backend test in
//! `crates/player/src/backend.rs`.
//!
//! Format note: every `prop_assert*!` message uses explicit positional
//! arguments (`{}`, `arg`), never implicit named captures
//! (`{var}`). proptest 1.11 builds the message through
//! `concat!`, and rustc rejects implicit captures in
//! macro-expanded format strings.

use proptest::prelude::*;

use player::backend::BackendError;
use player::backend::CpalBackend;
use player::focus::{DUCK_GAIN, FULL_GAIN};

/// Baseline gain for the rejection cases: a mid-interval factor whose
/// set must succeed, so "unchanged" is measured against a known state
/// rather than the default.
const BASELINE_GAIN: f32 = 0.5;

/// Strategy for factors outside `[0.0, 1.0]`: strictly negative finite
/// values, strictly-above-`1.0` finite values, both infinities, and
/// quiet `NaN` — every arm must be rejected with the named error.
fn invalid_gain_strategy() -> impl Strategy<Value = f32> {
    prop_oneof![
        (-10.0f32..0.0f32).prop_filter("strictly negative", |v| *v < 0.0),
        (1.0f32..10.0f32).prop_filter("strictly above 1.0", |v| *v > 1.0),
        Just(f32::NEG_INFINITY),
        Just(f32::INFINITY),
        Just(f32::NAN),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[expect(
        clippy::float_cmp,
        reason = "the accepted factor is echoed back verbatim as the stored f32 and the rejected case pins the unchanged baseline; == is exact and a tolerance would admit drift"
    )]
    #[test]
    fn given_a_gain_inside_the_unit_interval_when_setting_then_it_is_accepted_and_echoed(
        factor in 0.0f32..=1.0f32
    ) {
        // Setter law: every factor in [0.0, 1.0] is accepted and
        // `gain()` echoes back exactly the accepted value.
        if let Ok(backend) = CpalBackend::new() {
            let accepted = backend.set_gain(factor);
            prop_assert!(
                accepted.is_ok(),
                "gain factor {} inside [0.0, 1.0] was rejected",
                factor
            );
            prop_assert_eq!(
                backend.gain(),
                factor,
                "gain() did not echo the accepted factor {}",
                factor
            );
        }
        // Without an output device the case passes vacuously — see
        // the hardware note above.
    }

    #[expect(
        clippy::float_cmp,
        reason = "the accepted factor is echoed back verbatim as the stored f32 and the rejected case pins the unchanged baseline; == is exact and a tolerance would admit drift"
    )]
    #[test]
    fn given_a_gain_outside_the_unit_interval_when_setting_then_it_is_rejected_with_gain_unchanged(
        factor in invalid_gain_strategy()
    ) {
        // Setter law: negatives, factors above 1.0 (including +inf),
        // -inf, and NaN are rejected with the named `InvalidGain`
        // error and leave the last accepted gain untouched.
        if let Ok(backend) = CpalBackend::new() {
            let baseline = backend.set_gain(BASELINE_GAIN);
            prop_assert!(
                baseline.is_ok(),
                "baseline gain {} was rejected before the invalid attempt",
                BASELINE_GAIN
            );
            let before = backend.gain();
            let rejected = backend.set_gain(factor);
            prop_assert!(
                matches!(rejected, Err(BackendError::InvalidGain(_))),
                "gain factor {} outside [0.0, 1.0] was not rejected with the named InvalidGain error",
                factor
            );
            prop_assert_eq!(
                backend.gain(),
                before,
                "a rejected gain factor {} moved gain() away from {}",
                factor,
                before
            );
        }
        // Without an output device the case passes vacuously — see
        // the hardware note above.
    }
}

/// Default gain is full (`1.0`): a fresh backend reads at unity.
// Exact-value contract: the default gain is the exact FULL_GAIN literal; == pins the 1.0 default
// and a tolerance would admit drift
#[test]
fn default_gain_is_full() {
    if let Ok(backend) = CpalBackend::new() {
        assert_eq!(
            backend.gain(),
            FULL_GAIN,
            "a fresh backend must read full gain, not a drifted default"
        );
    }
}

/// The spec-anchored factors — silence `0.0`, the nominal duck factor
/// `0.20`, and full gain `1.0` — are accepted and echoed exactly.
// Exact-value contract: gain() must echo the accepted f32 bit-for-bit; == pins the echo law and a
// tolerance would weaken it
#[test]
fn silence_duck_and_full_factors_are_accepted_and_echoed() {
    if let Ok(backend) = CpalBackend::new() {
        for factor in [0.0, DUCK_GAIN, FULL_GAIN] {
            let accepted = backend.set_gain(factor);
            assert!(
                accepted.is_ok(),
                "gain factor {factor} inside [0.0, 1.0] was rejected"
            );
            assert_eq!(
                backend.gain(),
                factor,
                "gain() did not echo the accepted factor {factor}"
            );
        }
    }
}

/// `NaN` is rejected with the named error and leaves gain unchanged.
// Exact-value contract: the unchanged-gain check compares the intact stored f32 against itself; ==
// is exact and a tolerance would admit drift
#[test]
fn nan_is_rejected_with_gain_unchanged() {
    if let Ok(backend) = CpalBackend::new() {
        let baseline = backend.set_gain(BASELINE_GAIN);
        assert!(
            baseline.is_ok(),
            "baseline gain {BASELINE_GAIN} was rejected before the NaN attempt"
        );
        let before = backend.gain();
        let rejected = backend.set_gain(f32::NAN);
        assert!(
            matches!(rejected, Err(BackendError::InvalidGain(_))),
            "NaN gain was not rejected with the named InvalidGain error"
        );
        assert_eq!(
            backend.gain(),
            before,
            "a rejected NaN moved gain() away from the last accepted value"
        );
    }
}
