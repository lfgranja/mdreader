//! T200 C-H8 RED — served-identity twin (byte-identity not met).
//! Before fix: src/status_view.html = 27405 B, served bundle = 7689 B.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_t200_served_identity_binds_status_view_as_bundle() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bundle = manifest_dir.join("src-tauri/dist/index.html");
    let src = manifest_dir.join("src-tauri/src/status_view.html");

    assert!(
        bundle.exists(),
        "served bundle missing at {}",
        bundle.display()
    );
    assert!(
        src.exists(),
        "source of record missing at {}",
        src.display()
    );

    // Byte-identity: cmp exit 0 between source and served bundle.
    let cmp = Command::new("cmp")
        .arg("-s")
        .arg(&src)
        .arg(&bundle)
        .status()
        .expect("cmp failed to spawn");

    let src_bytes = fs::read(&src).unwrap();
    let bundle_bytes = fs::read(&bundle).unwrap();

    assert!(
        cmp.success() && src_bytes == bundle_bytes,
        "byte-identity: 27405 vs 7689 diverge (src={} B bundle={} B); cmp exit={:?}; S2 packaging invariant FAIL",
        src_bytes.len(),
        bundle_bytes.len(),
        cmp.code()
    );
}
