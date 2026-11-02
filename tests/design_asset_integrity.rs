//! Design asset integrity tests to prevent corrupted/replaced design assets.

use std::fs;
use std::path::Path;

#[test]
fn proposta2_not_corrupted_by_unrelated_content() {
    let path = Path::new("DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png");
    assert!(path.exists(), "proposta-2 asset must exist");
    
    let data = fs::read(path).expect("read proposta-2 asset");
    // PNG magic number check
    assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "must be a valid PNG");
    
    // Check file size is reasonable (not tiny/corrupt)
    assert!(data.len() > 10000, "proposta-2 asset must have reasonable size");
}

#[test]
fn design_ui_assets_are_valid_pngs() {
    let assets = [
        "DESIGN-SONA/assets/ui/ui-proposta-1-console-braun.png",
        "DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png",
        "DESIGN-SONA/assets/ui/ui-proposta-3-terminal-editorial.png",
        "DESIGN-SONA/assets/ui/ui-proposta-4-cockpit-eyes-free.png",
        "DESIGN-SONA/assets/ui/ui-proposta-5-matriz-telemetria.png",
    ];
    
    for asset in &assets {
        let path = Path::new(asset);
        assert!(path.exists(), "asset {} must exist", asset);
        let data = fs::read(path).expect(&format!("read {}", asset));
        assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "{} must be a valid PNG", asset);
    }
}
