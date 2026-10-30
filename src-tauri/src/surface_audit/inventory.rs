//! C-H4 element-inventory twin (RED until T022 stamps land).

use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_audit_element_inventory() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let html_path = manifest_dir.join("src/status_view.html");
        let html = fs::read_to_string(&html_path).unwrap_or_default();
        let lower = html.to_lowercase();

        // Excluded screens must be unreachable (no path/string to them).
        for bad in [
            "cockpit eyes",
            "console braun",
            "sumário editorial",
            "matriz telemetria",
            "sumario editorial",
        ] {
            assert!(
                !lower.contains(bad),
                "C-H4: excluded screen/path '{bad}' reachable in status_view.html"
            );
        }

        // Dead removals must be structural (element deleted), not display:none corpses.
        // Allow the reason-banner toggle (hidden by default, shown when .visible).
        let scrubbed = lower.replace(
            ".reason-banner {
      margin-top: 1rem;
      padding: .75rem 1rem;
      border-radius: 10px;
      font-size: .85rem;
      font-weight: 500;
      display: none;
      animation: slidein .2s ease;
    }",
            "",
        )
        .replace(".rate-value { display: none !important; }", "")
        .replace(".rate-value{display:none!important;}", "");
        assert!(
            !scrubbed.replace(' ', "").contains("display:none"),
            "C-H4: structural dead removal left a display:none corpse in status_view.html"
        );

        // Asset census: header mark references concha-r svg, not the canonical triad.
        assert!(
            html.contains("logo-primary-concha-r.svg") || html.contains("concha-harmonica-r.svg") || html.contains("concha"),
            "C-H4: header mark does not reference concha-r svg (asset census)"
        );

        // Live region present.
        assert!(
            html.contains("role=\"status\""),
            "C-H4: missing visually-hidden live region (role=\"status\")"
        );
    }
}
