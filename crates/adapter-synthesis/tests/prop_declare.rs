//! Property: declared_capabilities always (None, None); immutable.
// Implementation reserved for T6 (different agent from this test writer).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlLevel { None, Partial, Full }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentGranularity { None, Word, Sentence }

pub struct AdapterCapabilities {
    pub ssml_level: SsmlLevel,
    pub alignment_granularity: AlignmentGranularity,
}

pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;

    #[test]
    fn declare_invariant_none_none() {
        let cap = declared_capabilities();
        assert_eq!(cap.ssml_level, SsmlLevel::None);
        assert_eq!(cap.alignment_granularity, AlignmentGranularity::None);
    }
}
