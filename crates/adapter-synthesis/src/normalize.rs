//! TTS symbol normalization.
//!
//! Turns markdown structural markers into spoken text at the
//! synthesis boundary (`synth.rs::synthesize`), preserving the
//! `DocumentSegment` metadata and the document fingerprint. The
//! rule set follows the convergent industry pattern (Azure
//! `mstts:markdown`, Pipecat `StripMarkdown`, `NousResearch`
//! `tts_text_normalize`): strip markers, keep visible text,
//! protect code/URL/math verbatim, reduce links to their label,
//! and turn thematic breaks into a pause.
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Normalization mode for spoken text.
pub enum NormalizationMode {
    /// Fluid: strip structural markers.
    Fluid,
    /// Literal: no normalization.
    Literal,
}

static NORMALIZATION_MODE: OnceLock<NormalizationMode> = OnceLock::new();

/// Set the active normalization mode (first call wins).
pub fn set_normalization_mode(mode: NormalizationMode) {
    let _ = NORMALIZATION_MODE.set(mode);
}

/// Get the active normalization mode (default Fluid).
pub fn normalization_mode() -> NormalizationMode {
    *NORMALIZATION_MODE
        .get()
        .unwrap_or(&NormalizationMode::Fluid)
}

/// Normalize markdown text for TTS speech.
#[must_use]
pub fn normalize_for_speech(text: &str, mode: NormalizationMode) -> String {
    if mode == NormalizationMode::Literal {
        return text.to_string();
    }
    // Fenced code blocks are protected verbatim: the line scan
    // tracks the fence state and leaves the interior untouched.
    let lines: Vec<&str> = text.split('\n').collect();
    let trailing_nl = text.ends_with('\n');
    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut in_fence = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            result_lines.push((*line).to_string());
            continue;
        }
        if in_fence {
            result_lines.push((*line).to_string());
            continue;
        }
        result_lines.push(normalize_line(line));
    }
    let mut result = result_lines.join("\n");
    if trailing_nl && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Replace every match of `pattern` with the literal replacement.
fn replace_with(s: &str, pattern: &str, replacement: &str) -> String {
    let Ok(re) = Regex::new(pattern) else {
        return s.to_string();
    };
    re.replace_all(s, replacement).to_string()
}

/// Protect every match of `pattern` behind a private-use placeholder.
fn protect_zone(placeholders: &mut Vec<String>, s: &str, pattern: &str) -> String {
    let Ok(re) = Regex::new(pattern) else {
        return s.to_string();
    };
    re.replace_all(s, |m: &regex::Captures| {
        let idx = placeholders.len();
        placeholders.push(m.get(0).map_or("", |x| x.as_str()).to_string());
        format!("\u{E000}MDREADER{idx}\u{E001}")
    })
    .to_string()
}

/// Restore the protected zones behind their placeholders.
fn restore_zones(placeholders: &[String], s: &str) -> String {
    let Ok(re) = Regex::new("\u{E000}MDREADER(\\d+)\u{E001}") else {
        return s.to_string();
    };
    re.replace_all(s, |m: &regex::Captures| {
        let idx: usize = m.get(1).and_then(|g| g.as_str().parse().ok()).unwrap_or(0);
        placeholders.get(idx).cloned().unwrap_or_default()
    })
    .to_string()
}

fn normalize_line(line: &str) -> String {
    let trimmed = line.trim();
    // Thematic break → pause.
    if is_thematic_break(trimmed) {
        return ".".to_string();
    }
    // Table row: delimiter rows become a pause; data rows keep
    // their cells separated by a spoken comma.
    if trimmed.starts_with('|') {
        return normalize_table_row(trimmed);
    }

    let mut placeholders: Vec<String> = Vec::new();
    let mut s = line.to_string();

    // Links and images reduce to their visible label / alt text
    // BEFORE the destination can be mistaken for a bare URL;
    // the destination is never spoken.
    s = replace_with(&s, r"!?\[([^\]]*)\]\([^)]*\)", "$1");

    // Protect verbatim zones before any marker stripping.
    s = protect_zone(&mut placeholders, &s, r"`[^`]*`");
    s = protect_zone(&mut placeholders, &s, r"https?://\S+");
    s = protect_zone(&mut placeholders, &s, r"\$[^$]*\$");
    s = protect_zone(&mut placeholders, &s, r"\b\d+\s*\*\s*\d+\b");
    // Non-numeric multiplication / intraword asterisks (`a*b`).
    s = protect_zone(&mut placeholders, &s, r"\b\w+\*\w+\b");

    // Raw HTML tags are stripped.
    s = replace_with(&s, r"</?[a-zA-Z][^>]*>", "");

    // Blockquote markers (including nested `>>`).
    s = replace_with(&s, r"^>+\s?", "");

    // Heading markers.
    s = replace_with(&s, r"^#{1,6}\s*", "");

    // List bullets.
    s = replace_with(&s, r"^(\s*)[*+-]\s+", "$1");

    // Strikethrough.
    s = replace_with(&s, r"~~(.+?)~~", "$1");

    // Emphasis markers.
    s = replace_with(&s, r"\*\*(.+?)\*\*", "$1");
    s = replace_with(&s, r"\*(.+?)\*", "$1");
    s = replace_with(&s, r"__(.+?)__", "$1");
    s = replace_with(&s, r"_(.+?)_", "$1");

    // Remove orphan asterisks left outside the protected zones.
    s = s.replace('*', "");

    restore_zones(&placeholders, &s)
}

/// Normalize a pipe-table row: delimiter rows become a pause,
/// data rows keep their cells separated by a spoken comma.
fn normalize_table_row(trimmed: &str) -> String {
    let inner = trimmed.trim_start_matches('|').trim_end_matches('|');
    let cells: Vec<&str> = inner.split('|').map(str::trim).collect();
    let is_delimiter = !cells.is_empty()
        && cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':'));
    if is_delimiter {
        return ".".to_string();
    }
    cells.join(", ")
}

fn is_thematic_break(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let chars: Vec<char> = t.chars().collect();
    if chars.len() < 3 {
        return false;
    }
    let first = chars[0];
    if first != '*' && first != '-' && first != '_' {
        return false;
    }
    chars.iter().all(|&c| c == first || c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strips_emphasis() {
        assert_eq!(
            normalize_for_speech("*itálico* e **negrito**", NormalizationMode::Fluid),
            "itálico e negrito"
        );
    }
    #[test]
    fn strips_heading() {
        assert_eq!(
            normalize_for_speech("# Título", NormalizationMode::Fluid),
            "Título"
        );
    }
    #[test]
    fn thematic_break() {
        assert_eq!(normalize_for_speech("***", NormalizationMode::Fluid), ".");
    }
    #[test]
    fn protected_inline_code() {
        assert_eq!(
            normalize_for_speech("use `a*b` here", NormalizationMode::Fluid),
            "use `a*b` here"
        );
    }
    #[test]
    fn protected_url() {
        assert_eq!(
            normalize_for_speech("see https://x.com/a*b", NormalizationMode::Fluid),
            "see https://x.com/a*b"
        );
    }
    #[test]
    fn literal_mode_identity() {
        let t = "*a* # b";
        assert_eq!(normalize_for_speech(t, NormalizationMode::Literal), t);
    }
    #[test]
    fn idempotent() {
        let t = "*a* **b** # c\n***\n* item";
        let once = normalize_for_speech(t, NormalizationMode::Fluid);
        let twice = normalize_for_speech(&once, NormalizationMode::Fluid);
        assert_eq!(once, twice);
    }

    #[test]
    fn reduces_links_to_label() {
        assert_eq!(
            normalize_for_speech(
                "veja [o docs](https://exemplo.com/a) aqui",
                NormalizationMode::Fluid
            ),
            "veja o docs aqui"
        );
    }
    #[test]
    fn reduces_images_to_alt_text() {
        assert_eq!(
            normalize_for_speech("![foto do gato](img/gato.png)", NormalizationMode::Fluid),
            "foto do gato"
        );
    }
    #[test]
    fn strips_blockquote_marker() {
        assert_eq!(
            normalize_for_speech("> citação", NormalizationMode::Fluid),
            "citação"
        );
    }
    #[test]
    fn strips_strikethrough() {
        assert_eq!(
            normalize_for_speech("~~antigo~~ novo", NormalizationMode::Fluid),
            "antigo novo"
        );
    }
    #[test]
    fn strips_html_tags() {
        assert_eq!(
            normalize_for_speech("texto <b>negrito</b> aqui", NormalizationMode::Fluid),
            "texto negrito aqui"
        );
    }
    #[test]
    fn protects_intraword_asterisk() {
        assert_eq!(
            normalize_for_speech("a*b fica", NormalizationMode::Fluid),
            "a*b fica"
        );
    }
    #[test]
    fn normalizes_table_rows() {
        assert_eq!(
            normalize_for_speech(
                "| Nome | Idade |\n|---|---|\n| Ana | 30 |",
                NormalizationMode::Fluid
            ),
            "Nome, Idade\n.\nAna, 30"
        );
    }
    #[test]
    fn fenced_code_is_verbatim() {
        let src = "```\n# não é título\n*a*\n```";
        assert_eq!(normalize_for_speech(src, NormalizationMode::Fluid), src);
    }
}
