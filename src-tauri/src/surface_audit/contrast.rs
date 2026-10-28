//! C-H4 contrast twin (WCAG 2.1 AA contrast solver).

use std::fs;
use std::path::Path;

/// Relative luminance of a (possibly `#`-prefixed) hex color.
#[cfg(test)]
#[must_use]
pub fn rel_luminance(hex: &str) -> f64 {
    fn chan(v: u8) -> f64 {
        let x = f64::from(v) / 255.0;
        if x <= 0.03928 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    }
    let h = hex.trim_start_matches('#');
    let (r, g, b) = if h.len() >= 6 {
        (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };
    0.0722f64.mul_add(chan(b), 0.7152f64.mul_add(chan(g), 0.2126 * chan(r)))
}

/// WCAG 2.1 contrast ratio between two hex colors.
#[cfg(test)]
#[must_use]
pub fn contrast_ratio(fg: &str, bg: &str) -> f64 {
    let l1 = rel_luminance(fg);
    let l2 = rel_luminance(bg);
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_audit_contrast_solver() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let html_path = manifest_dir.join("src/status_view.html");
        let _html = fs::read_to_string(&html_path).unwrap_or_default();

        // Sanity margins (tolerance 0.05).
        assert!((contrast_ratio("#F2F0E6", "#0B0F1A") - 16.75).abs() < 0.05);
        assert!((contrast_ratio("#F2F0E6", "#141B2D") - 15.01).abs() < 0.05);
        assert!((contrast_ratio("#F2F0E6", "#55688F") - 4.88).abs() < 0.05);
        assert!((contrast_ratio("#55688F", "#0B0F1A") - 3.43).abs() < 0.05);
        assert!((contrast_ratio("#55688F", "#141B2D") - 3.07).abs() < 0.05);

        // Painted text pairs parsed (coarse) from status_view.html.
        let text_pairs: &[(&str, &str)] = &[
            ("#F2F0E6", "#0B0F1A"),
            ("#F2F0E6", "#141B2D"),
            ("#A9B8CE", "#0B0F1A"),
            ("#A9B8CE", "#141B2D"),
        ];
        for (fg, bg) in text_pairs {
            let r = contrast_ratio(fg, bg);
            assert!(
                r >= 4.5,
                "surface_audit_contrast_solver: text pair {fg} on {bg} = {r:.2} < 4.5"
            );
        }

        // Non-text (boundary) pairs.
        let nontext_pairs: &[(&str, &str)] = &[
            ("#55688F", "#0B0F1A"),
            ("#55688F", "#141B2D"),
            ("#FF4D63", "#0B0F1A"),
            ("#FFD400", "#0B0F1A"),
        ];
        for (fg, bg) in nontext_pairs {
            let r = contrast_ratio(fg, bg);
            assert!(
                r >= 3.0,
                "surface_audit_contrast_solver: non-text pair {fg} on {bg} = {r:.2} < 3.0"
            );
        }
    }
}
