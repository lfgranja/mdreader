//! FR-024 synthesis adapter — offline_default (sherpa-onnx + Piper pt-BR).
//! Declares capabilities against existing engine-core shapes (format.rs / closed.rs);
//! does NOT invent phantom trait methods (T016 not yet built — Assumption 3 corrected).

pub mod capabilities;
pub mod chunk;
pub mod failure;
pub mod format_check;
pub mod synth;

/// Closed-set declaration (constitution §Closed Sets VII; no Other/Unknown).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Closed SSML capability level (no catch-all).
pub enum SsmlLevel {
    /// No SSML rendering.
    None,
    /// Partial SSML (reserved v2).
    Partial,
    /// Full SSML (reserved cloud).
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Closed resume-granularity level (deferred to v2 for Sentence).
pub enum AlignmentGranularity {
    /// Paragraph/heading only (v1).
    None,
    /// Word boundary (reserved).
    Word,
    /// Sentence boundary (deferred v2).
    Sentence,
}

/// Declared adapter capabilities (FR-024c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilities {
    /// SSML level (None for v1 offline_default).
    pub ssml_level: SsmlLevel,
    /// Resume granularity (None for v1 paragraph/heading).
    pub alignment_granularity: AlignmentGranularity,
}

/// Unconditional declaration (constitution.md:304-305 / FR-024c).
/// Option A (v1): offline_default => (None, None); deferring SSML/segment
/// boundary events until second engine / consent (SC-044 deferred).
/// Unconditional declaration — Option A v1 (offline_default => None/None).
pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}
