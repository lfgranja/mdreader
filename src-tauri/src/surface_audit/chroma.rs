//! C-H1 chroma census twin (RED until violations are cleared).

use std::fs;
use std::path::Path;

const ALLOWED_HEX: &[&str] = &[
    "#0B0F1A", "#141B2D", "#F2F0E6", "#8FD8E8", "#A9B8CE", "#E8C46A", "#B9A6F5", "#55688F",
    "#FFD400", "#FF4D63",
];

const DESIGN_SONA_FILES: &[&str] = &[
    "tokens.md",
    "components.md",
    "DESIGN-SONA.md",
    "preview.html",
    "CHANGELOG.md",
    "VERSION",
];

#[cfg(test)]
fn is_hex_byte(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

#[cfg(test)]
fn is_hex_bytes(bs: &[u8]) -> bool {
    !bs.is_empty() && bs.iter().all(|&b| b.is_ascii_hexdigit())
}

#[cfg(test)]
fn expand_hex(slice: &str) -> String {
    let u = slice.to_ascii_uppercase();
    if u.len() == 3 {
        format!(
            "#{}{}{}{}{}{}",
            &u[0..1],
            &u[0..1],
            &u[1..2],
            &u[1..2],
            &u[2..3],
            &u[2..3]
        )
    } else {
        format!("#{u}")
    }
}

#[cfg(test)]
fn scan_text(path: &Path, text: &str, violations: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        if bytes[i] == b'#' {
            let mut matched = 0usize;
            // Prefer 6 hex digits.
            if i + 7 <= n && is_hex_bytes(&bytes[i + 1..i + 7]) {
                let after = i + 7;
                if after == n || !is_hex_byte(bytes[after]) {
                    matched = 6;
                }
            } else if i + 4 <= n && is_hex_bytes(&bytes[i + 1..i + 4]) {
                let after = i + 4;
                if after == n || !is_hex_byte(bytes[after]) {
                    matched = 3;
                }
            }
            if matched > 0 {
                let full = expand_hex(&text[i + 1..i + 1 + matched]);
                let line_no = text[..i].bytes().filter(|&b| b == b'\n').count() + 1;
                if !ALLOWED_HEX.contains(&full.as_str()) {
                    violations.push(format!(
                        "C-H1 chroma census: {}:{} {} not in closed hex set",
                        path.display(),
                        line_no,
                        full
                    ));
                }
                i += 1 + matched;
                continue;
            }
        }
        i += 1;
    }
}

#[cfg(test)]
fn scan_file(path: &Path, violations: &mut Vec<String>) {
    match fs::read_to_string(path) {
        Ok(text) => scan_text(path, &text, violations),
        Err(e) => violations.push(format!(
            "C-H1 chroma census: cannot read {} ({})",
            path.display(),
            e
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface_audit::{Auditee, resolve_all};

    #[test]
    fn surface_audit_chroma_census() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let auditees = resolve_all(manifest_dir).unwrap_or_default();
        let mut violations: Vec<String> = Vec::new();

        for (auditee, path) in &auditees {
            match auditee {
                Auditee::DesignSonaDir => {
                    for name in DESIGN_SONA_FILES {
                        let f = path.join(name);
                        if f.exists() {
                            // Hex census applies to code/config surfaces; the
                            // design-dir markdown + SVG asset are existence-checked
                            // here (missing = fail-closed) and reconciled by T023.
                        } else {
                            violations.push(format!(
                                "C-H1 chroma census: missing auditee file {}",
                                f.display()
                            ));
                        }
                    }
                }
                Auditee::ConchaRLogoSvg => {
                    if !path.exists() {
                        violations.push(format!(
                            "C-H1 chroma census: missing auditee {}",
                            path.display()
                        ));
                    }
                }
                Auditee::MediaMirrorKt => {
                    // Auditee A1 #3: constants block :728-743 only.
                    if let Ok(text) = fs::read_to_string(path) {
                        let lines: Vec<&str> = text.lines().collect();
                        let start = 727usize.min(lines.len());
                        let end = 743usize.min(lines.len());
                        let slice = lines[start..end].join("\n");
                        scan_text(path, &slice, &mut violations);
                    } else {
                        violations.push(format!(
                            "C-H1 chroma census: cannot read {}",
                            path.display()
                        ));
                    }
                }
                _ => {
                    scan_file(path, &mut violations);
                }
            }
        }

        assert!(
            violations.is_empty(),
            "C-H1 chroma census violations:\n{}",
            violations.join("\n")
        );
    }
}
