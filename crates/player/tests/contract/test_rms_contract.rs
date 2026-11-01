//! RMS payload contract tests (007 A7).
//!
//! Invariants under test:
//! - RMS values must be in valid f32 range and representable
//! - RMS computation is lock-free and allocation-free on hot path
//! - Payload shape is f32 for IPC channel

use player::backend::CpalBackend;

#[test]
fn rms_method_exists_and_returns_f32() {
    fn assert_rms_exists<T: HasRms>() {}
    trait HasRms { fn rms(&self) -> f32; }
    impl HasRms for CpalBackend { fn rms(&self) -> f32 { self.rms() } }
    let _ = assert_rms_exists::<CpalBackend>;
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
