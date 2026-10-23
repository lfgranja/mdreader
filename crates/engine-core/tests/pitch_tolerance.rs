//! T116 — convergence: the single contract pitch tolerance (Phase 12).
//!
//! The engine-handoff contract pins "±3 cents at 2.0×" as the
//! `RateMismatch` tolerance (`crates/engine-core/src/trait_.rs`,
//! `Engine::set_rate`), while SC-015
//! (`specs/001-android-media-shell/spec.md`) and the T055 property
//! (`crates/engine-core/tests/prop_pitch.rs`) enforce "±50 cents F0
//! + spectral-peak alignment" — so a 10-cent drift is simultaneously
//!   a contract `RateMismatch` and an SC-015 pass. T117 resolves the
//!   divergence by creating ONE shared constant,
//!   `engine_core::format::PITCH_TOLERANCE_CENTS: u32 = 50`, and
//!   unifying all three sites to it.
//!
//! This suite is the BLOCKING TEST: it imports that single constant
//! and asserts it equals the measurable SC-015 bound (50 cents).
//! RED STATE (T116): the constant does not exist yet — T117 creates
//! it — so this suite fails to compile with an unresolved import.
//! That compilation failure is the required pre-implementation red
//! state; do not weaken the bound to manufacture a green build.

use engine_core::format::PITCH_TOLERANCE_CENTS;

// The single contract tolerance is SC-015's measurable bound.
#[test]
fn contract_tolerance_is_the_measurable_sc015_bound() {
    assert_eq!(PITCH_TOLERANCE_CENTS, 50u32);
}
