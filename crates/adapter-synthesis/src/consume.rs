//! T014 property: player branches on adapter declaration (None/None).
use crate::{SsmlLevel, AlignmentGranularity};

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
        assert_eq!(consume_declaration(SsmlLevel::None, AlignmentGranularity::None),
                   "direct_text_no_ssml_paragraph_resume");
    }
}
