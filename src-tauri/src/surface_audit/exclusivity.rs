//! C-H2 role-exclusivity twin (`status_view.html`).

use std::fs;
use std::path::Path;

#[cfg(test)]
fn read_status_view(manifest_dir: &Path) -> String {
    let p = manifest_dir.join("src/status_view.html");
    fs::read_to_string(&p).unwrap_or_default()
}

/// Byte ranges (open brace .. matching close brace) of every `:focus-visible { ... }` block.
#[cfg(test)]
fn focus_visible_blocks(text: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let needle = ":focus-visible";
    let mut search = 0;
    while let Some(rel) = text[search..].find(needle) {
        let pos = search + rel;
        if let Some(brace_rel) = text[pos..].find('{') {
            let open = pos + brace_rel;
            let bytes = text.as_bytes();
            let mut depth: i32 = 0;
            let mut i = open;
            while i < bytes.len() {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            blocks.push((open, i + 1));
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
        search = pos + needle.len();
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_audit_role_exclusivity() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let text = read_status_view(manifest_dir);
        let blocks = focus_visible_blocks(&text);

        // #8FD8E8 (sona-etwahl) occurs ONLY inside :focus-visible rules.
        let hex = "#8FD8E8";
        let mut outside: Vec<String> = Vec::new();
        let mut offset = 0usize;
        for line in text.lines() {
            let line_start = offset;
            let line_end = offset + line.len();
            if line.contains(hex) {
                // Token declaration in :root (and the scaffold mirror) is allowed;
                // only painted occurrences outside :focus-visible are violations.
                let is_decl = line.contains("--sona-etwahl")
                    || line.contains(":root")
                    || line.contains("sona-scaffold-tokens");
                if is_decl { /* allowed */
                } else {
                    let inside = blocks
                        .iter()
                        .any(|(s, e)| line_start >= *s && line_end <= *e);
                    if !inside {
                        outside.push(line.trim().to_string());
                    }
                }
            }
            offset = line_end + 1;
        }
        assert!(
            outside.is_empty(),
            "C-H2: #8FD8E8 (sona-etwahl) outside :focus-visible rules: {outside:?}"
        );
    }
}
