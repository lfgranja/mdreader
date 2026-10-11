//! Registration and tier selectability (R11 / SC-051).
//!
//! This adapter registers in the `offline_default` tier
//! only, and the selector invariant — which tiers may be
//! selected while it is registered — is implemented here
//! over the closed `EngineTier` set, never asserted as a
//! second `&str` copy of a tier name (`API_CONTRACT.md`
//! §4; `FR-044`: the registry in `engine_core::closed`
//! is the single list).

use engine_core::closed::EngineTier;

/// The only tier this adapter registers at v1.
///
/// `offline_default` (sherpa-onnx + Piper pt-BR) is the
/// sole registered tier: no second real engine exists at
/// v1 (`SC-044` deferred; R11).
pub const REGISTERED_TIERS: [EngineTier; 1] = [EngineTier::OfflineDefault];

/// The registered tiers (`&[EngineTier::OfflineDefault]`).
///
/// Returns [`REGISTERED_TIERS`] itself, so the constant
/// and the accessor are one list and cannot disagree
/// (`FR-044`).
#[must_use]
pub const fn registered_tiers() -> &'static [EngineTier] {
    &REGISTERED_TIERS
}

/// Whether `tier` may be selected while this adapter is
/// registered.
///
/// Exhaustive 5-arm match — never a grouped wildcard,
/// never an underscore arm (Principle VII; constitution
/// gate 7): an
/// added or removed `EngineTier` member fails to compile
/// here instead of silently skipping a tier. Only
/// [`EngineTier::OfflineDefault`] is selectable:
/// [`EngineTier::CloudDefault`],
/// [`EngineTier::CloudEscalation`] and
/// [`EngineTier::EvaluatedFallback`] are unregistered at
/// v1 (`SC-044` deferred), and [`EngineTier::TestDouble`]
/// is `false` because the stand-ins are fixtures and are
/// never selectable while a non-`test_double` adapter is
/// registered (`001/spec.md:89`; `closed.rs`:398-406).
// §4 requires five distinct arms — never a grouped
// wildcard, never an underscore arm. `CloudDefault`,
// `CloudEscalation` and `EvaluatedFallback`
// legitimately all return `false`: all three are
// unregistered at v1 with distinct deferral reasons
// (`SC-044` deferred, consent deferred,
// evaluated-fallback deferred), and the reasons belong
// in the doc comments above, not in duplicated arm
// bodies — sanctioned by API_CONTRACT.md §0.1 entry 2.
#[expect(
    clippy::match_same_arms,
    reason = "API_CONTRACT.md §4 requires five distinct arms; CloudDefault, CloudEscalation and EvaluatedFallback legitimately all return false (unregistered at v1 with distinct deferral reasons: SC-044 deferred, consent deferred, evaluated-fallback deferred) — the reasons live in the doc comments, not in duplicated arm bodies; sanctioned by API_CONTRACT.md §0.1 entry 2"
)]
#[must_use]
pub const fn is_selectable(tier: EngineTier) -> bool {
    match tier {
        EngineTier::OfflineDefault => true,
        EngineTier::CloudDefault => false,
        EngineTier::CloudEscalation => false,
        EngineTier::TestDouble => false,
        EngineTier::EvaluatedFallback => false,
    }
}
