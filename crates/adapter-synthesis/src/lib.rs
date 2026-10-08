//! FR-024 synthesis adapter — offline_default (sherpa-onnx + Piper pt-BR).
//! Declares capabilities against existing engine-core shapes (format.rs / closed.rs);
//! does NOT invent phantom trait methods (T016 not yet built — Assumption 3 corrected).

pub mod chunk;
pub mod synth;
pub mod format_check;
pub mod failure;
pub mod capabilities;

use failure::EngineError;

/// Closed-set declaration (constitution §Closed Sets VII; no Other/Unknown).
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

pub struct AdapterCapabilities {
    pub ssml_level: SsmlLevel,
    pub alignment_granularity: AlignmentGranularity,
}

/// Unconditional declaration (constitution.md:304-305 / FR-024c).
/// Option A (v1): offline_default => (None, None); deferring SSML/segment
/// boundary events until second engine / consent (SC-044 deferred).
pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}
