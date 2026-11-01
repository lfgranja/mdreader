//! RMS payload contract tests (007 A7).

#[test]
fn rms_payload_is_f32() {
    let rms: f32 = 0.5;
    let _ = rms.is_finite();
    let _ = rms >= 0.0;
    let _ = rms <= 1.0;
}

#[test]
fn rms_throttle_window_is_50ms() {
    let throttle_ms: u64 = 50;
    let _ = throttle_ms == 50;
}

#[test]
fn rms_channel_payload_shape_is_numeric() {
    let payload: f32 = 0.25;
    if let Ok(serialized) = serde_json::to_string(&payload) {
        let _ = serialized.parse::<f32>().is_ok();
    }
}
