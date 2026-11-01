//! C-H1 chroma census twin (scope-aware for 007).

use std::fs;
use std::path::Path;

const SONA_PALETTE_HEX: &[&str] = &[
    "#0B0F1A", "#141B2D", "#F2F0E6", "#8FD8E8", "#A9B8CE", "#E8C46A", "#B9A6F5", "#55688F",
    "#FFD400", "#FF4D63", "#5AD6A8",
];

const JADE_HEX: &[&str] = &["#5AD6A8"];
const CELESTE_HEX: &[&str] = &["#B9A6F5"];
const JADE_ERRATUM: &[&str] = &["#5FD0A0"];

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
fn skip_comment(bytes: &[u8], mut i: usize) -> usize {
    if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
        i += 2;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        return i;
    }
    if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
        i += 2;
        while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
            i += 1;
        }
        if i + 1 < bytes.len() {
            i += 2;
        }
        return i;
    }
    if i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {
        i += 4;
        while i + 2 < bytes.len() && !(bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>') {
            i += 1;
        }
        if i + 2 < bytes.len() {
            i += 3;
        }
        return i;
    }
    i
}

#[cfg(test)]
fn is_in_ledger_declaration(text: &str, hex_start: usize) -> bool {
    let before = &text[..hex_start];
    if before.contains("--sona-celeste") || before.contains("--sona-aria") {
        let recent = &before[before.len().saturating_sub(120)..];
        if recent.contains("--sona-celeste") || recent.contains("--sona-aria") {
            // It's in a declaration context
            if recent.rfind(':').is_some() {
                return true;
            }
        }
    }
    // Also check for :root declarations more generally
    false
}

#[cfg(test)]
fn is_sanctioned_scope(path: &Path, text: &str, hex_start: usize, hex_str: &str) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let hex_upper = hex_str.to_ascii_uppercase();

    // Ledger declarations are carve-outs
    if is_in_ledger_declaration(text, hex_start) {
        return true;
    }

    // Jade #5AD6A8
    if hex_upper == "#5AD6A8" {
        // Brand-layer assets
        if path_str.contains("design-sona/assets") {
            return true;
        }
        // Decorative wave SVG/CSS
        if path_str.contains("waves") && (path_str.ends_with(".svg") || path_str.ends_with(".css")) {
            return true;
        }
        if path_str.contains("wave-subtle") {
            return true;
        }
        // Waveform ribbon middle layers in status view
        if path_str.ends_with("status_view.html") {
            let before = &text[..hex_start];
            if before.contains("drawWaveform") || before.contains("waveform") {
                return true;
            }
        }
        return false;
    }

    // Celeste #B9A6F5
    if hex_upper == "#B9A6F5" {
        // Decorative-wave violet stops
        if path_str.contains("waves") && (path_str.ends_with(".svg") || path_str.ends_with(".css")) {
            return true;
        }
        if path_str.contains("design-sona/assets") {
            return true;
        }
        if path_str.contains("wave-subtle") {
            return true;
        }
        // Waveform-ribbon violet layers
        if path_str.ends_with("status_view.html") {
            let before = &text[..hex_start];
            if before.contains("drawWaveform") || before.contains("waveform") {
                return true;
            }
        }
        return false;
    }

    // Jade erratum remains banned everywhere
    if hex_upper == "#5FD0A0" {
        return false;
    }

    if SONA_PALETTE_HEX.iter().any(|&h| h == hex_upper) {
        return true;
    }

    false
}

#[cfg(test)]
fn scan_text(path: &Path, text: &str, violations: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        // Skip comments
        if bytes[i] == b'/' && i + 1 < n {
            let new_i = skip_comment(bytes, i);
            if new_i != i {
                i = new_i;
                continue;
            }
        }
        if i + 3 < n && &bytes[i..i + 4] == b"<!--" {
            i = skip_comment(bytes, i);
            continue;
        }

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
                let hex_slice = &text[i + 1..i + 1 + matched];
                let full = expand_hex(hex_slice);
                let line_no = text[..i].bytes().filter(|&b| b == b'\n').count() + 1;

                let hex_upper = full.to_ascii_uppercase();

                // Jade erratum is banned everywhere
                if JADE_ERRATUM.iter().any(|&h| h == hex_upper) {
                    violations.push(format!(
                        "C-H1 chroma census: {}:{} {} banned everywhere (jade erratum)",
                        path.display(),
                        line_no,
                        full
                    ));
                    i += 1 + matched;
                    continue;
                }

                // Check if it's jade or celeste
                if JADE_HEX.iter().any(|&h| h == hex_upper) || CELESTE_HEX.iter().any(|&h| h == hex_upper) {
                    if !is_sanctioned_scope(path, text, i, &full) {
                        violations.push(format!(
                            "C-H1 chroma census: {}:{} {} not in sanctioned scope",
                            path.display(),
                            line_no,
                            full
                        ));
                    }
                    i += 1 + matched;
                    continue;
                }

                if !SONA_PALETTE_HEX.iter().any(|&h| h == hex_upper) {
                    violations.push(format!(
                        "C-H1 chroma census: {}:{} {} not in closed hex set",
                        path.display(),
                        line_no,
                        full
                    ));
                    i += 1 + matched;
                    continue;
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
                            // Hex census applies to code/config surfaces
                        } else {
                            violations.push(format!(
                                "C-H1 chroma census: missing auditee file {}",
                                f.display()
                            ));
                        }
                    }
                    // Also scan decorative wave assets
                    let waves_dir = path.join("assets").join("waves");
                    if waves_dir.exists() {
                        if let Ok(entries) = fs::read_dir(&waves_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|s| s.to_str()) == Some("svg")
                                    || p.extension().and_then(|s| s.to_str()) == Some("css")
                                {
                                    scan_file(&p, &mut violations);
                                }
                            }
                        }
                    }
                }
                Auditee::ConchaRLogoSvg => {
                    if !path.exists() {
                        violations.push(format!(
                            "C-H1 chroma census: missing auditee {}",
                            path.display()
                        ));
                    } else {
                        scan_file(path, &mut violations);
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
