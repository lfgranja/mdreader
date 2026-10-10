//! Player consumption of adapter declaration (FR-024c / T067).
//! Mirrors adapter's `consume::plan_for` identity mapping.
//! Explicit arms for all declared pairs — no `_ =>` (Principle VI / SC-051).

/// Delivery strength derived from declared SSML level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlDelivery {
    /// `SsmlLevel::None` (v1 `offline_default`).
    PlainText,
    /// Reserved (`SsmlLevel::Partial`).
    PartialSsml,
    /// Reserved (`SsmlLevel::Full`).
    FullSsml,
}

/// Resume boundary derived from declared alignment granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeAnchor {
    /// `AlignmentGranularity::None` (v1 declaration); snap to paragraph/heading.
    ParagraphHeading,
    /// Reserved (`AlignmentGranularity::Word`).
    WordBoundary,
    /// Reserved (`AlignmentGranularity::Sentence`; deferred v2).
    SentenceBoundary,
}

/// Declared adapter capabilities consumed by player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterCapabilities {
    /// `(None, None)` — the only selectable v1 declaration per SC-051 (`test_double` excluded).
    OfflineDefault,
}

/// Declaration input (matches adapter's declared shapes, not phantom trait).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlLevel {
    None,
    Partial,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentGranularity {
    None,
    Word,
    Sentence,
}

/// Total plan_for over the 3×3 declaration cross-product (identity mapping).
/// Every declared `(SsmlLevel, AlignmentGranularity)` maps to the same-strength
/// delivery and resume — therefore `(None, None)` yields `(PlainText, ParagraphHeading)`.
pub fn plan_for(
    ssml: SsmlLevel,
    align: AlignmentGranularity,
) -> (SsmlDelivery, ResumeAnchor) {
    match (ssml, align) {
        (SsmlLevel::None, AlignmentGranularity::None) => {
            (SsmlDelivery::PlainText, ResumeAnchor::ParagraphHeading)
        }
        (SsmlLevel::None, AlignmentGranularity::Word) => {
            (SsmlDelivery::PlainText, ResumeAnchor::WordBoundary)
        }
        (SsmlLevel::None, AlignmentGranularity::Sentence) => {
            (SsmlDelivery::PlainText, ResumeAnchor::SentenceBoundary)
        }
        (SsmlLevel::Partial, AlignmentGranularity::None) => {
            (SsmlDelivery::PartialSsml, ResumeAnchor::ParagraphHeading)
        }
        (SsmlLevel::Partial, AlignmentGranularity::Word) => {
            (SsmlDelivery::PartialSsml, ResumeAnchor::WordBoundary)
        }
        (SsmlLevel::Partial, AlignmentGranularity::Sentence) => {
            (SsmlDelivery::PartialSsml, ResumeAnchor::SentenceBoundary)
        }
        (SsmlLevel::Full, AlignmentGranularity::None) => {
            (SsmlDelivery::FullSsml, ResumeAnchor::ParagraphHeading)
        }
        (SsmlLevel::Full, AlignmentGranularity::Word) => {
            (SsmlDelivery::FullSsml, ResumeAnchor::WordBoundary)
        }
        (SsmlLevel::Full, AlignmentGranularity::Sentence) => {
            (SsmlDelivery::FullSsml, ResumeAnchor::SentenceBoundary)
        }
    }
}

/// Wired branch on adapter's declared capabilities (FR-024c / T067).
/// Only `OfflineDefault` is selectable (`SC-051`; `test_double` excluded by absence).
pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities::OfflineDefault
}
