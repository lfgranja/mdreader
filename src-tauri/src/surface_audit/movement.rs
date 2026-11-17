//! C-H6 movement census twin (RED until T016 wiring lands).

use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_audit_movement_census() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let html_path = manifest_dir.join("src/status_view.html");
        let html = fs::read_to_string(&html_path).unwrap_or_default();

        // Reduced-motion guard must exist.
        assert!(
            html.contains("prefers-reduced-motion"),
            "missing reduced-motion guard"
        );
        // Freeze must be present.
        assert!(
            html.contains("animation-play-state: paused"),
            "missing freeze"
        );

        // At most one continuous (infinite) loop, and it must be the pill.
        let infinite_count = html.matches("infinite").count();
        assert!(
            infinite_count <= 1,
            "too many continuous loops: {infinite_count}"
        );

        // Count @keyframes declarations (informational bound: pill + banner).
        let keyframes_count = html.matches("@keyframes").count();
        assert!(
            keyframes_count <= 2,
            "too many @keyframes declarations: {keyframes_count}"
        );

        // Extract the prefers-reduced-motion guard block.
        let guard = html.find("prefers-reduced-motion").map_or("", |idx| {
            let start = html[..idx]
                .rfind("@media")
                .unwrap_or_else(|| idx.saturating_sub(64));
            let end = (idx + 256).min(html.len());
            &html[start..end]
        });

        // Freeze must land the pill at a VISIBLE on-track position:
        // paused + a static offset (transform/left/margin/inset). An
        // invisible/opacity-zero freeze FAILS.
        let has_static_offset = guard.contains("transform")
            || guard.contains("left:")
            || guard.contains("left :")
            || guard.contains("margin")
            || guard.contains("inset")
            || guard.contains("translate");
        assert!(
            has_static_offset,
            "reduced-motion guard missing static on-track offset"
        );
        assert!(
            !guard.contains("opacity: 0")
                && !guard.contains("opacity:0")
                && !guard.contains("opacity:0%"),
            "invisible/opacity-zero freeze (FAILS)"
        );

        // Banner entry must be one-shot (<=200ms) and not looping.
        assert!(
            !html.contains("slideIn") || !html.contains("slideIn infinite"),
            "banner slideIn must be one-shot"
        );
    }
}
