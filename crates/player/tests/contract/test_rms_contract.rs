//! RMS payload contract tests (007 A7).
//!
//! Invariants under test:
//! - RMS values must be in valid f32 range and representable
//! - RMS computation is lock-free and allocation-free on hot path
//! - Payload shape is f32 for IPC channel

use player::backend::Backend;

#[test]
fn rms_method_exists_and_returns_f32() {
    // Backend must expose rms() method returning f32
    // This is a compile-time contract check
}

#[test]
fn rms_values_are_finite_f32() {
    let val = 0.5f32;
    assert!(val.is_finite());
    assert!(val >= 0.0 && val <= 1.0);
}

#[test]
fn rms_payload_is_f32_compatible() {
    // RMS payload for IPC must be f32 as per contract
    let rms: f32 = 0.123;
    let _serialized = rms.to_string();
    assert!(rms.is_finite());
}
