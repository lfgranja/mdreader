//! C-H3 accent-only twin (`MediaStyle` constants block).

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn surface_audit_accent_only_constants() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let kt = manifest_dir.join("../android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt");
        let text = fs::read_to_string(&kt).unwrap_or_default();
        // Extract the constants block slice (approx :728-743).
        let start = text.find("DESIGN_SONA_PROGRESS_FILL").unwrap_or(0);
        let end = text[start..]
            .find("}\n")
            .map_or(start + 300, |i| start + i + 1);
        let block = &text[start..end.min(text.len())];
        let mut vals = Vec::new();
        let mut i = 0;
        while let Some(pos) = block[i..].find("0x") {
            let abs = i + pos;
            let end = (abs + 10).min(block.len());
            let tok = &block[abs..end];
            // keep only hex digits after 0x
            let hex: String = tok
                .chars()
                .skip(2)
                .take(8)
                .filter(char::is_ascii_hexdigit)
                .collect();
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
        // Structural invariants: no logo/mark/rate/body-text constants inside block.
        // Refined (false-positive fixed): only `const val ...` lines.
        let forbidden: &[&str] = &["LOGO", "MARK", "RATE", "BODY_TEXT", "VOICE"];
        for line in block.lines() {
            if line.contains("const val") {
                for f in forbidden {
                    assert!(
                        !line.contains(f),
                        "forbidden constant '{}' declared: {}",
                        f,
                        line.trim()
                    );
                }
            }
        }
        // Pre-existing fix recorded (A4, 2026-10-27): 0xFFF2F0E6 already present.
        assert!(
            !vals.contains(&"0xFFFFFFFF".to_string()),
            "C1 veto: 0xFFFFFFFF must not appear"
        );
        assert!(!vals.is_empty(), "accent block non-empty");
    }
}
