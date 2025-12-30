use crate::model::{
    config::AllocationConfig, consolidated::ConsolidatedPortfolio, ids::MultiplexerId,
};
use std::collections::HashMap;
use trading::model::{allocation::Allocation, instrument::InstrumentId, portfolio::Portfolio};

pub mod max_allocation;
pub mod max_position_size;

#[derive(Debug, PartialEq)]
pub enum RiskDecision {
    Approved,
    Rejected(String),
}

/// Context passed to policies to make decisions.
pub struct RiskContext<'a> {
    pub portfolio: &'a Portfolio,
    pub prices: &'a HashMap<InstrumentId, f64>,
    pub total_equity: f64,
    pub allocation_config: &'a AllocationConfig,
    pub consolidated: &'a ConsolidatedPortfolio,
    pub multiplexer_id: &'a MultiplexerId,
}

pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, target_allocation: &Allocation, ctx: &RiskContext) -> RiskDecision;
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

    pub fn check(&self, target: &Allocation, ctx: &RiskContext) -> RiskDecision {
        for policy in &self.policies {
            if let RiskDecision::Rejected(reason) = policy.check(target, ctx) {
                log::warn!("Allocation rejected by {}: {}", policy.name(), reason);
                return RiskDecision::Rejected(format!("{}: {}", policy.name(), reason));
            }
        }
        RiskDecision::Approved
    }
}
