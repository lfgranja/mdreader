//! Capability declaration — unconditional; Option A v1.
//! Matches adapter-capabilities.md §2.1 / spec.md:97.
use crate::{AdapterCapabilities, AlignmentGranularity, SsmlLevel};

/// Unconditional (None, None) declaration.
#[must_use]
pub const fn capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        ssml_level: SsmlLevel::None,
        alignment_granularity: AlignmentGranularity::None,
    }
}
