//! T014 property: player branches on adapter declaration (None/None).

use crate::{AlignmentGranularity, SsmlLevel};

/// How the player consumes a declared capability pair.
///
/// Replaced by the typed `PlayerPlan` surface in T026; this string
/// form exists only so the crate compiles under the workspace lint
/// set (`missing_docs = deny`) before that task runs.
pub fn consume_declaration(ssml: SsmlLevel, granular: AlignmentGranularity) -> &'static str {
    match (ssml, granular) {
        (SsmlLevel::None, AlignmentGranularity::None) => "direct_text_no_ssml_paragraph_resume",
        _ => "unsupported",
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;

    #[test]
    fn declare_none_consumed_direct() {
        assert_eq!(
            consume_declaration(SsmlLevel::None, AlignmentGranularity::None),
            "direct_text_no_ssml_paragraph_resume"
        );
    }
}
