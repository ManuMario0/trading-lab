use crate::models::{AllocationConfig, ConsolidatedPortfolio, Order, Portfolio, Prices};
use log::warn;

pub mod global_max_position_size;
pub mod margin_requirement;
pub mod max_allocation;
pub mod max_position_size;
pub mod max_strategy_allocation;

#[derive(Debug, PartialEq)]
pub enum RiskDecision {
    Approved,
    Rejected(String),
}

/// Context passed to policies to make decisions.
/// Requires prices to estimate NAV and position values.
pub struct RiskContext<'a> {
    pub portfolio: &'a Portfolio,
    pub prices: &'a Prices,
    pub total_equity: f64,
    pub allocation_config: &'a AllocationConfig,
    pub consolidated: &'a ConsolidatedPortfolio,
}

pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision;

    /// Checks a batch of orders atomically.
    /// Default implementation simply checks each order individually.
    /// Policies that need to verify the net result (e.g. margin) should
    /// override this.
    fn check_batch(&self, orders: &[Order], ctx: &RiskContext) -> RiskDecision {
        for order in orders {
            if let RiskDecision::Rejected(reason) = self.check(order, ctx) {
                return RiskDecision::Rejected(reason);
            }
        }
        RiskDecision::Approved
    }
}

pub struct RiskGuard {
    policies: Vec<Box<dyn Policy>>,
}

impl Default for RiskGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskGuard {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: Box<dyn Policy>) {
        self.policies.push(policy);
    }

    pub fn check_order(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        for policy in &self.policies {
            match policy.check(order, ctx) {
                RiskDecision::Rejected(reason) => {
                    warn!(
                        "Order {} rejected by policy {}: {}",
                        order.id(),
                        policy.name(),
                        reason
                    );
                    return RiskDecision::Rejected(format!("{}: {}", policy.name(), reason));
                }
                RiskDecision::Approved => continue,
            }
        }
        RiskDecision::Approved
    }

    pub fn check_batch(&self, orders: &[Order], ctx: &RiskContext) -> RiskDecision {
        for policy in &self.policies {
            match policy.check_batch(orders, ctx) {
                RiskDecision::Rejected(reason) => {
                    warn!("Batch rejected by policy {}: {}", policy.name(), reason);
                    return RiskDecision::Rejected(format!("{}: {}", policy.name(), reason));
                }
                RiskDecision::Approved => continue,
            }
        }
        RiskDecision::Approved
    }
}
