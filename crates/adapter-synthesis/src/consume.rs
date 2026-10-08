//! Player declaration consumption — the typed player-plan surface
//! (`API_CONTRACT.md` §5; `FR-024c` / `spec.md:97`;
//! `contracts/adapter-capabilities.md` §4.1).
//!
//! The player branches on what the adapter *declares*, never on what it
//! infers: `plan_for` turns an `AdapterCapabilities` declaration into a
//! `PlayerPlan` — how the player must treat SSML-bearing source
//! (`SsmlDelivery`) and which boundary resume may snap to
//! (`ResumeAnchor`). The mapping is the identity over the 3×3
//! declaration cross product: every `SsmlLevel` maps to the same-strength
//! `SsmlDelivery` and every `AlignmentGranularity` to the same-strength
//! `ResumeAnchor`, so a declared `None` consumes as `PlainText` /
//! `ParagraphHeading` and never upgrades to `FullSsml` / `Sentence`
//! (Principle VI — degradation is observable, never silent).
//!
//! `plan_for` is total over all nine `(SsmlLevel,
//! AlignmentGranularity)` pairs: both component matches enumerate every
//! member of their closed set explicitly — no wildcard arm (constitution
//! gate 7 / Principle VII), so a future variant fails to compile instead
//! of being silently consumed.

use crate::{AdapterCapabilities, AlignmentGranularity, SsmlLevel};

/// What the player must do with SSML-bearing source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsmlDelivery {
    /// Deliver plain text only — no SSML rendering path
    /// (`SsmlLevel::None`, the v1 `offline_default` declaration).
    PlainText,
    /// Interpret partial SSML (`SsmlLevel::Partial`; reserved for a
    /// future engine).
    PartialSsml,
    /// Interpret full SSML (`SsmlLevel::Full`; reserved for a future
    /// engine).
    FullSsml,
}

/// The boundary resume may snap to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeAnchor {
    /// Resume at the nearest completed paragraph/heading boundary
    /// (`AlignmentGranularity::None`, the v1 declaration).
    ParagraphHeading,
    /// Resume at the nearest word boundary (`AlignmentGranularity::Word`;
    /// reserved).
    Word,
    /// Resume at the nearest sentence boundary
    /// (`AlignmentGranularity::Sentence`; deferred to v2 — Piper exposes
    /// no sentence-boundary events).
    Sentence,
}

/// The player's plan, derived only from the adapter's declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerPlan {
    /// How the player must handle SSML-bearing source.
    pub ssml: SsmlDelivery,
    /// The boundary resume may snap to.
    pub resume: ResumeAnchor,
}

/// The delivery path for a declared SSML level (identity mapping:
/// `SsmlLevel::None` → `SsmlDelivery::PlainText`, `Partial` →
/// `PartialSsml`, `Full` → `FullSsml`).
///
/// Exhaustive three-arm match over the closed `SsmlLevel` set — every
/// arm explicit, no catch-all (Principle VII; constitution gate 7).
#[must_use]
pub const fn ssml_delivery(level: SsmlLevel) -> SsmlDelivery {
    match level {
        SsmlLevel::None => SsmlDelivery::PlainText,
        SsmlLevel::Partial => SsmlDelivery::PartialSsml,
        SsmlLevel::Full => SsmlDelivery::FullSsml,
    }
}

/// The resume anchor for a declared alignment granularity (identity
/// mapping: `AlignmentGranularity::None` →
/// `ResumeAnchor::ParagraphHeading`, `Word` → `Word`, `Sentence` →
/// `Sentence`).
///
/// Exhaustive three-arm match over the closed `AlignmentGranularity`
/// set — every arm explicit, no catch-all (Principle VII; constitution
/// gate 7).
#[must_use]
pub const fn resume_anchor(granularity: AlignmentGranularity) -> ResumeAnchor {
    match granularity {
        AlignmentGranularity::None => ResumeAnchor::ParagraphHeading,
        AlignmentGranularity::Word => ResumeAnchor::Word,
        AlignmentGranularity::Sentence => ResumeAnchor::Sentence,
    }
}

/// The player's plan for a declaration — the composite of
/// `ssml_delivery` and `resume_anchor`, so each half of the identity
/// mapping is written exactly once and the plan is total over all nine
/// `(SsmlLevel, AlignmentGranularity)` pairs.
///
/// A declared `None` never upgrades: the v1 declaration
/// (`declared_capabilities()`, `(None, None)`) plans
/// `(PlainText, ParagraphHeading)`, never `FullSsml` / `Sentence`
/// (Principle VI — degradation is observable, never silent).
#[must_use]
pub const fn plan_for(capabilities: AdapterCapabilities) -> PlayerPlan {
    PlayerPlan {
        ssml: ssml_delivery(capabilities.ssml_level),
        resume: resume_anchor(capabilities.alignment_granularity),
    }
}
