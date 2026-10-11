//! Player consumption of adapter declaration (FR-024c / T067).
//! Consumes `adapter_synthesis::declared_capabilities()` and `adapter_synthesis::consume::plan_for()`
//! — the identity mapping over the 3×3 declaration cross-product. No local duplicate types.
//! Explicit arms for all declared pairs — no `_ =>` (Principle VI / SC-051).

use adapter_synthesis::{
    consume::plan_for as adapter_plan_for, declared_capabilities as adapter_declared_capabilities,
};

/// Re-export the adapter's capability types so the player's public API is stable.
/// These are the single authoritative definitions (FR-044: no second list that can drift).
pub use adapter_synthesis::AdapterCapabilities;
pub use adapter_synthesis::AlignmentGranularity;
pub use adapter_synthesis::SsmlLevel;
pub use adapter_synthesis::consume::{PlayerPlan, ResumeAnchor, SsmlDelivery};

/// The player's plan for the v1 adapter declaration.
/// Calls the adapter's `declared_capabilities()` — the single
/// source of truth for what the adapter declares (FR-024c, Principle VI).
#[must_use]
pub fn declared_capabilities() -> AdapterCapabilities {
    adapter_declared_capabilities()
}

/// The player's plan for a declaration — delegates to the adapter's identity mapping.
/// Every declared `(SsmlLevel, AlignmentGranularity)` maps to the same-strength
/// delivery and resume — therefore `(None, None)` yields `(PlainText, ParagraphHeading)`.
#[must_use]
pub fn plan_for(caps: AdapterCapabilities) -> PlayerPlan {
    adapter_plan_for(caps)
}
