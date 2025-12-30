use super::ids::MultiplexerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_cash_buffer() -> f64 {
    0.01
}

/// Configuration for a specific strategy (Multiplexer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Maximum relative allocation of the Global Equity (0.0 to 1.0).
    allocation_fraction: f64,
    /// Maximum allowed drawdown before the Kill Switch is triggered (0.0 to
    /// 1.0).
    max_drawdown: f64,
    /// Minimum Global Equity required to enter NEW positions.
    min_global_equity: f64,
    /// Ratio of capital reserved (0.01 = 1%)
    #[serde(default = "default_cash_buffer")]
    cash_buffer: f64,
}

impl StrategyConfig {
    pub fn new(allocation_fraction: f64, max_drawdown: f64, min_global_equity: f64) -> Self {
        Self {
            allocation_fraction,
            max_drawdown,
            min_global_equity,
            cash_buffer: 0.01,
        }
    }

    pub fn allocation_fraction(&self) -> f64 {
        self.allocation_fraction
    }
}

/// Dynamic map defining max capital access per MultiplexerId.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationConfig {
    allocations: HashMap<MultiplexerId, StrategyConfig>,
}

impl AllocationConfig {
    pub fn get(&self, id: &MultiplexerId) -> Option<&StrategyConfig> {
        self.allocations.get(id)
    }

    pub fn insert(&mut self, id: MultiplexerId, config: StrategyConfig) {
        self.allocations.insert(id, config);
    }
}
