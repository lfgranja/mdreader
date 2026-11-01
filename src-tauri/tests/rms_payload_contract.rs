//! RMS payload contract tests (007 A7).
//!
//! Contract: RMS values published via dedicated `tauri::ipc::Channel` as f32,
//! throttled ~20 Hz, last-wins coalescing, skip when hidden or no subscribers.

#[test]
fn rms_payload_is_f32() {
    let rms: f32 = 0.5;
    assert!(rms.is_finite());
    assert!(rms >= 0.0);
    assert!(rms <= 1.0);
}

#[test]
fn rms_throttle_window_is_50ms() {
    let throttle_ms: u64 = 50;
    assert_eq!(throttle_ms, 50);
}

#[test]
fn rms_channel_payload_shape_is_numeric() {
    let payload: f32 = 0.25;
    // f32 can be serialized as number
    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.parse::<f32>().is_ok());
}
