// RED — reduced-motion freeze: animation/transform zero when prefers-reduced-motion: reduce
// Contract: spec FR-008 (reduced Lente) / contracts/surface.md
fn animation_transform_for_reduced() -> &'static str {
    "transform: scale(1.05); animation: pulse 2s infinite" // RED: violation present
}

#[test]
fn reduced_motion_freeze_zero_transform() {
    let css = animation_transform_for_reduced();
    assert!(!css.contains("transform"),
        "reduced-motion freeze FAIL: transform present under prefers-reduced-motion: reduce");
    assert!(!css.contains("animation"),
        "reduced-motion freeze FAIL: animation present under prefers-reduced-motion: reduce");
}
