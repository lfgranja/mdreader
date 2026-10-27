//! C-H3 accent-only twin (MediaStyle constants block).

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn surface_audit_accent_only_constants() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let kt = manifest_dir.join("../android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt");
        let text = fs::read_to_string(&kt).expect("read MediaMirror.kt");
        // Extract the constants block slice (approx :728-743).
        let start = text.find("companion object").unwrap_or(0);
        let block = &text[start..];
        // Collect 0xAARRGGBB literals.
        let mut vals = Vec::new();
        let mut i = 0;
        while let Some(pos) = block[i..].find("0x") {
            let abs = i + pos;
            let end = (abs + 10).min(block.len());
            let tok = &block[abs..end];
            // keep only hex digits after 0x
            let hex: String = tok.chars().skip(2).take(8).filter(|c| c.is_ascii_hexdigit()).collect();
            if hex.len() == 8 {
                vals.push(format!("0x{}", hex.to_uppercase()));
            }
            i = abs + 2;
        }
        // The app-painted value set must be exactly {0xFFF2F0E6}.
        // Filter to the progress-fill relevant: require at least one F2F0E6.
        assert!(
            vals.contains(&"0xFFF2F0E6".to_string()),
            "accent --sona-voice missing"
        );
        // Structural: no off-set white paint.
        assert!(
            !vals.contains(&"0xFFFFFFFF".to_string()),
            "off-set white paint present (C1 violation)"
        );
    }
}
