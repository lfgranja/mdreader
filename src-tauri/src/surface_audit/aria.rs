//! C-H5 aria-inventory twin (RED until T020 aria semantics land).

use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    fn html(manifest_dir: &Path) -> String {
        let p = manifest_dir.join("src/status_view.html");
        fs::read_to_string(&p).unwrap_or_default()
    }

    /// Extract the opening tag that contains `needle` (an attribute fragment).
    fn tag_containing<'a>(doc: &'a str, needle: &str) -> Option<&'a str> {
        let idx = doc.find(needle)?;
        // walk back to '<'
        let start = doc[..idx].rfind('<')?;
        let end = doc[idx..].find('>')? + idx;
        Some(&doc[start..=end])
    }

    fn aria_label_nonempty(tag: &str) -> bool {
        if let Some(i) = tag.find("aria-label=\"") {
            let rest = &tag[i + "aria-label=\"".len()..];
            if let Some(j) = rest.find('"') {
                return !rest[..j].trim().is_empty();
            }
        }
        false
    }

    #[test]
    fn surface_audit_aria_inventory() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let doc = html(manifest_dir);

        // Every enumerated interactive control carries a non-empty aria-label.
        for id in [
            "#btnSkipBack",
            "#btnPlayPause",
            "#btnStop",
            "#btnSkipForward",
            "#btnRateDown",
            "#btnRateUp",
        ] {
            let needle = format!("id=\"{}\"", id.trim_start_matches('#'));
            let tag = tag_containing(&doc, &needle);
            assert!(
                tag.is_some(),
                "surface_audit_aria_inventory: missing control {id}"
            );
            let tag = tag.unwrap_or_default();
            assert!(
                aria_label_nonempty(tag),
                "surface_audit_aria_inventory: {id} missing non-empty aria-label"
            );
        }

        // #progress carries role="progressbar" + aria-valuenow + aria-valuetext.
        let progress_tag = tag_containing(&doc, "id=\"progress\"").unwrap_or_default();
        assert!(
            progress_tag.contains("role=\"progressbar\""),
            "surface_audit_aria_inventory: #progress missing role=\"progressbar\""
        );
        assert!(
            progress_tag.contains("aria-valuenow"),
            "surface_audit_aria_inventory: #progress missing aria-valuenow"
        );
        assert!(
            progress_tag.contains("aria-valuetext"),
            "surface_audit_aria_inventory: #progress missing aria-valuetext"
        );

        // A visually-hidden role="status" live region exists (ladder value channel).
        // Today there is none -> RED.
        let status_count = doc.matches("role=\"status\"").count();
        assert!(
            status_count >= 2,
            "surface_audit_aria_inventory: visually-hidden role=\"status\" live region missing (found {status_count} role=status, need >=3)"
        );

        // #speedDots stays aria-hidden.
        let dots_tag = tag_containing(&doc, "id=\"speedDots\"").unwrap_or_default();
        assert!(
            dots_tag.contains("aria-hidden=\"true\"") || dots_tag.contains("aria-hidden='true'"),
            "surface_audit_aria_inventory: #speedDots not aria-hidden"
        );

        // #reasonBanner typed role="status".
        let banner_tag = tag_containing(&doc, "id=\"reasonBanner\"").unwrap_or_default();
        assert!(
            banner_tag.contains("role=\"status\""),
            "surface_audit_aria_inventory: #reasonBanner not typed role=\"status\""
        );

        // Legacy role="alert"+aria-live="polite" pair absent.
        assert!(
            !doc.contains("role=\"alert\""),
            "surface_audit_aria_inventory: legacy role=\"alert\"+aria-live=\"polite\" pair present"
        );

        // NO painted numeric rate (value-blind invariant).
        assert!(
            !doc.contains("Taxa:"),
            "surface_audit_aria_inventory: painted numeric rate 'Taxa:' found"
        );
        // Painted numeric rate is sanctioned ONLY in the
        // speed-control area (#rateValue) per FR-003 (007).
        let rate_value_tag = tag_containing(&doc, "id=\"rateValue\"").unwrap_or_default();
        assert!(
            !rate_value_tag.is_empty(),
            "surface_audit_aria_inventory: #rateValue element missing"
        );
        // The sanctioned rateValue span carries the numeric rate text.
        assert!(
            doc.contains("id=\"rateValue\"") && doc.contains("1.25×"),
            "surface_audit_aria_inventory: sanctioned #rateValue must display the numeric rate"
        );
        // Only one painted '×' (the sanctioned one) is allowed.
        assert!(
            doc.matches('×').count() <= 2,
            "surface_audit_aria_inventory: painted numeric rate '×' found outside #rateValue"
        );
    }
}
