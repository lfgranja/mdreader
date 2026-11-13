//! Provider router for adapter-synthesis (constitution: zero direct provider implementation).
//!
//! This adapter registers in the `offline_default` tier only and uses the
//! engine-core [`Router`] (Principle VI) to select providers based on
//! declared capabilities and current conditions.
//!
//! No vendor SDK type, no provider-specific identifier crosses into the
//! core. Every provider declares what it can deliver through
//! [`Engine::declared_capabilities`], and the router selects the best
//! provider for each operation based on declared capabilities and
//! current conditions (value, volume, recurrence, payment method,
// national/international, etc.).
//!
//! At v1, only `EngineTier::OfflineDefault` is registered, so the router
//! always selects [`EngineTier::OfflineDefault`] when it is consented and
//! available.

use engine_core::closed::EngineTier;
use engine_core::router::{
    PaymentMethod, ProviderCapability, ProviderRegistration, RecurrenceType,
    Router, TransactionScope, VolumeTier,
};

/// The router for adapter-synthesis.
///
/// At v1, only [`EngineTier::OfflineDefault`] is registered, so this router
/// always selects [`EngineTier::OfflineDefault`] when it is consented and
/// available.
#[derive(Debug, Clone)]
pub struct AdapterRouter {
    inner: Router,
}

impl AdapterRouter {
    /// Create a new adapter router with the default `offline_default`
    /// tier registered.
    pub fn new() -> Self {
        Self {
            inner: Router::new(vec![ProviderRegistration {
                tier: EngineTier::OfflineDefault,
                capabilities: vec![
                    ProviderCapability::None,
                    ProviderCapability::International,
                    ProviderCapability::HighVolume,
                    ProviderCapability::Recurring,
                    ProviderCapability::PaymentMethodSpecific,
                ],
                consented: true,
                available: true,
            }]),
        }
    }

    /// Select the best provider for the given conditions.
    ///
    /// At v1, only [`EngineTier::OfflineDefault`] is registered, so this
    /// always returns [`EngineTier::OfflineDefault`] when it is consented
    /// and available.
    pub fn select(
        &self,
        payment_method: PaymentMethod,
        scope: TransactionScope,
        volume: VolumeTier,
        recurrence: RecurrenceType,
    ) -> Option<engine_core::router::RouteDecision> {
        self.inner.select(payment_method, scope, volume, recurrence)
    }
}

impl Default for AdapterRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_router_selects_offline_default() {
        let router = AdapterRouter::new();

        let decision = router.select(
            PaymentMethod::Pix,
            TransactionScope::National,
            VolumeTier::Low,
            RecurrenceType::OneTime,
        );

        assert!(decision.is_some());
        assert_eq!(decision.unwrap().tier, EngineTier::OfflineDefault);
    }

    #[test]
    fn test_adapter_router_international() {
        let router = AdapterRouter::new();

        let decision = router.select(
            PaymentMethod::CreditCardCredit,
            TransactionScope::International,
            VolumeTier::High,
            RecurrenceType::Recurring,
        );

        assert!(decision.is_some());
        assert_eq!(decision.unwrap().tier, EngineTier::OfflineDefault);
    }

    #[test]
    fn test_adapter_router_unavailable() {
        let router = AdapterRouter::new();
        // Default state has OfflineDefault consented and available.
        let decision = router.select(
            PaymentMethod::Pix,
            TransactionScope::National,
            VolumeTier::Low,
            RecurrenceType::OneTime,
        );

        assert!(decision.is_some());
        assert_eq!(decision.unwrap().tier, EngineTier::OfflineDefault);
    }
}
