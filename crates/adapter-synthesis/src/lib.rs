//! FR-024 synthesis adapter — offline_default (sherpa-onnx + Piper pt-BR).
//! Declares capabilities against existing engine-core shapes (format.rs / closed.rs);
//! does NOT invent phantom trait methods (T016 not yet built — Assumption 3 corrected).

use engine_core::closed::EngineTier;

pub mod capabilities;
pub mod reg;
pub mod consume;
pub mod chunk;
pub mod failure;
pub mod format_check;
pub mod synth;

/// The tier this adapter registers at (contracts/adapter-handoff.md
/// §6.1; SC-051): `OfflineDefault` only — the sole selectable tier
/// at v1. The `test_double` stand-ins (`engine_core::standin_a` /
/// `standin_b`) are fixtures and are never selectable while this
/// non-`test_double` adapter is registered (`closed.rs`:305-312).
pub const REGISTERED_TIER: EngineTier = EngineTier::OfflineDefault;

/// Closed SSML capability level (no catch-all; Principle VII).
///
/// The closed set is exactly `None | Partial | Full` — no
/// `Other`/`Unknown` variant exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlLevel {
    /// No SSML rendering (the `offline_default` declaration:
    /// sherpa-onnx + Piper does not honor SSML).
    None,
    /// Partial SSML (reserved for a future cloud engine).
    Partial,
    /// Full SSML (reserved for a future engine with full SSML).
    Full,
}

/// Closed resume-granularity level (no catch-all; Principle VII).
///
/// The closed set is exactly `None | Word | Sentence` — no
/// `Other`/`Unknown` variant exists. `Sentence` is deferred to
/// v2 (Piper exposes no sentence-boundary events).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentGranularity {
    /// Paragraph/heading boundary only (v1 declaration).
    None,
    /// Word boundary (reserved).
    Word,
    /// Sentence boundary (deferred to v2).
    Sentence,
}

/// Declared adapter capabilities (FR-024c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilities {
    /// SSML level (`None` for the v1 offline_default adapter).
    pub ssml_level: SsmlLevel,
    /// Resume granularity (`None` for v1 paragraph/heading).
    pub alignment_granularity: AlignmentGranularity,
}

/// Unconditional capability declaration (constitution.md:304-305 /
/// FR-024c; contracts/adapter-capabilities.md §2).
///
/// Option A (v1): the adapter is `offline_default` only, so it
/// declares `(SsmlLevel::None, AlignmentGranularity::None)` —
/// plain-text delivery and paragraph/heading resume —
/// unconditionally, with no engine state and before any chunk is
/// split or synthesized. The declaration is a constant of the
/// adapter contract: no input, document, or operation can change
/// it, and callers branch on it explicitly (never silently
/// inferring SSML presence or sentence-level alignment;
/// Principle VI).
#[must_use]
pub fn declared_capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}
