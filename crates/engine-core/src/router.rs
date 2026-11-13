//! Provider router (constitution: zero direct provider implementation).
//!
//! Providers are reached through the [`Engine`] trait (Principle VI):
//! no vendor SDK type, no provider-specific identifier crosses into
//! the core. Every adapter declares what it can deliver through
//! [`Engine::declared_capabilities`], and the router selects the
//! best provider for each operation based on declared capabilities
//! and current conditions.
//!
//! The router selects from the closed [`EngineTier`] set using the
//! closed [`ProviderCapability`], [`PaymentMethod`], [`TransactionScope`],
//! [`VolumeTier`], and [`RecurrenceType`] sets — exhaustive matches,
//! no `_ =>` arms, no `Other`/`Unknown` catch-all.
//!
//! The router selects the provider that is most advantageous for each
//! operation based on the declared capabilities and the current
/// conditions (value, volume, recurrence, payment method, national/
/// international, etc.).

use crate::closed::EngineTier;
use serde::{Deserialize, Serialize};

/// The closed payment-method set (constitution: router selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// Pix instant payment (Brazil).
    Pix,
    /// Boleto bancário payment.
    Boleto,
    /// Debit card payment.
    CreditCardDebit,
    /// Credit card payment.
    CreditCardCredit,
}

/// The closed transaction-scope set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionScope {
    /// National transaction.
    National,
    /// International transaction.
    International,
}

/// The closed volume-tier set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeTier {
    /// Low volume tier.
    Low,
    /// Medium volume tier.
    Medium,
    /// High volume tier.
    High,
}

/// The closed recurrence-type set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceType {
    /// One-time transaction.
    OneTime,
    /// Monthly recurrence.
    Monthly,
    /// Recurring transaction.
    Recurring,
}

/// The closed set of provider capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCapability {
    /// No special capability (plain text/PCM delivery).
    None,
    /// Can handle international transactions.
    International,
    /// Can handle high-volume transactions.
    HighVolume,
    /// Can handle recurring billing.
    Recurring,
    /// Can handle payment-method specific processing.
    PaymentMethodSpecific,
}

/// The routing decision: which provider was selected and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDecision {
    /// The selected provider tier.
    pub tier: EngineTier,
    /// The reason this tier was selected.
    pub reason: RouteReason,
}

/// The reason for the routing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteReason {
    /// Default tier was available and acceptable.
    DefaultAvailable,
    /// Declared tier was unavailable, fell back to next best.
    FallbackToNext,
    /// No suitable provider was found.
    NoProviderAvailable,
    /// Consent was not granted for the preferred tier.
    ConsentDenied,
}

/// A registered provider with its capabilities.
#[derive(Debug, Clone)]
pub struct ProviderRegistration {
    /// The engine tier this provider represents.
    pub tier: EngineTier,
    /// The capabilities this provider declares.
    pub capabilities: Vec<ProviderCapability>,
    /// Whether this provider is currently consented (FR-023c).
    pub consented: bool,
    /// Whether this provider is currently available.
    pub available: bool,
}

/// The router: selects from the closed [`EngineTier`] set based on
#[derive(Debug, Clone)]
pub struct Router {
    /// The registered providers and their capabilities.
    providers: Vec<ProviderRegistration>,
}

impl Router {
    /// Create a new router with the given providers.
    pub fn new(providers: Vec<ProviderRegistration>) -> Self {
        Self { providers }
    }

    /// Select the best provider for the given conditions.
    ///
    /// Selection criteria (in priority order):
    /// 1. Consent must be granted for the tier
    /// 2. Provider must be available
    /// 3. Provider must have the required capabilities
    /// 4. Prefer higher tiers in the closed order
    /// 5. Prefer providers whose capabilities match the conditions
    pub fn select(
        &self,
        payment_method: PaymentMethod,
        scope: TransactionScope,
        volume: VolumeTier,
        recurrence: RecurrenceType,
    ) -> Option<RouteDecision> {
        // Filter to consented, available providers — collect into owned
        // values to avoid nested reference issues.
        let available_providers: Vec<ProviderRegistration> = self
            .providers
            .iter()
            .filter(|p| p.consented && p.available)
            .cloned()
            .collect();

        if available_providers.is_empty() {
            return None;
        }

        // Filter by capabilities based on conditions.
        let required_caps = self.required_capabilities(payment_method, scope, volume, recurrence);
        let qualified: Vec<ProviderRegistration> = available_providers
            .into_iter()
            .filter(|p| required_caps.iter().all(|c| p.capabilities.contains(c)))
            .collect();

        if qualified.is_empty() {
            return None;
        }

        // Select the highest-tier provider among qualified ones.
        let selected = qualified
            .into_iter()
            .max_by_key(|p| tier_order_index(p.tier))
            .unwrap();

        Some(RouteDecision {
            tier: selected.tier,
            reason: RouteReason::DefaultAvailable,
        })
    }

    /// The required capabilities based on the given conditions.
    fn required_capabilities(
        &self,
        payment_method: PaymentMethod,
        scope: TransactionScope,
        volume: VolumeTier,
        recurrence: RecurrenceType,
    ) -> Vec<ProviderCapability> {
        let mut caps = Vec::new();

        match scope {
            TransactionScope::International => caps.push(ProviderCapability::International),
            TransactionScope::National => {}
        }

        match volume {
            VolumeTier::High => caps.push(ProviderCapability::HighVolume),
            VolumeTier::Medium | VolumeTier::Low => {}
        }

        match recurrence {
            RecurrenceType::Recurring | RecurrenceType::Monthly => {
                caps.push(ProviderCapability::Recurring);
            }
            RecurrenceType::OneTime => {}
        }

        match payment_method {
            PaymentMethod::Pix
            | PaymentMethod::Boleto
            | PaymentMethod::CreditCardDebit
            | PaymentMethod::CreditCardCredit => {
                caps.push(ProviderCapability::PaymentMethodSpecific);
            }
        }

        caps
    }
}

/// The index of a tier in the single authoritative [`EngineTier`] order
/// (FR-023c).
fn tier_order_index(tier: EngineTier) -> usize {
    match tier {
        EngineTier::OfflineDefault => 0,
        EngineTier::CloudDefault => 1,
        EngineTier::CloudEscalation => 2,
        EngineTier::TestDouble => 3,
        EngineTier::EvaluatedFallback => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_selects_highest_available_tier() {
        let router = Router::new(vec![
            ProviderRegistration {
                tier: EngineTier::OfflineDefault,
                capabilities: vec![ProviderCapability::None],
                consented: true,
                available: true,
            },
            ProviderRegistration {
                tier: EngineTier::CloudDefault,
                capabilities: vec![ProviderCapability::International],
                consented: true,
                available: true,
            },
        ]);

        let decision = router.select(
            PaymentMethod::Pix,
            TransactionScope::National,
            VolumeTier::Low,
            RecurrenceType::OneTime,
        );

        assert!(decision.is_some());
        assert_eq!(decision.unwrap().tier, EngineTier::CloudDefault);
    }

    #[test]
    fn test_router_respects_consent() {
        let router = Router::new(vec![
            ProviderRegistration {
                tier: EngineTier::CloudDefault,
                capabilities: vec![ProviderCapability::International],
                consented: false, // not consented
                available: true,
            },
            ProviderRegistration {
                tier: EngineTier::OfflineDefault,
                capabilities: vec![ProviderCapability::None],
                consented: true,
                available: true,
            },
        ]);

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
    fn test_router_no_available_providers() {
        let router = Router::new(vec![
            ProviderRegistration {
                tier: EngineTier::OfflineDefault,
                capabilities: vec![ProviderCapability::None],
                consented: true,
                available: false, // not available
            },
        ]);

        let decision = router.select(
            PaymentMethod::Pix,
            TransactionScope::National,
            VolumeTier::Low,
            RecurrenceType::OneTime,
        );

        assert!(decision.is_none());
    }

    #[test]
    fn test_router_international_requires_international_capability() {
        let router = Router::new(vec![
            ProviderRegistration {
                tier: EngineTier::OfflineDefault,
                capabilities: vec![ProviderCapability::None],
                consented: true,
                available: true,
            },
            ProviderRegistration {
                tier: EngineTier::CloudDefault,
                capabilities: vec![ProviderCapability::International],
                consented: true,
                available: true,
            },
        ]);

        let decision = router.select(
            PaymentMethod::Pix,
            TransactionScope::International,
            VolumeTier::Low,
            RecurrenceType::OneTime,
        );

        assert!(decision.is_some());
        assert_eq!(decision.unwrap().tier, EngineTier::CloudDefault);
    }
}
