use super::ids::MultiplexerId;
use super::instrument::Instrument;
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

    pub fn with_cash_buffer(mut self, buffer: f64) -> Self {
        self.cash_buffer = buffer;
        self
    }

    pub fn allocation_fraction(&self) -> f64 {
        self.allocation_fraction
    }

    pub fn max_drawdown(&self) -> f64 {
        self.max_drawdown
    }

    pub fn min_global_equity(&self) -> f64 {
        self.min_global_equity
    }

    pub fn cash_buffer(&self) -> f64 {
        self.cash_buffer
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

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, MultiplexerId, StrategyConfig> {
        self.allocations.iter()
    }

    pub fn remove(&mut self, id: &MultiplexerId) -> Option<StrategyConfig> {
        self.allocations.remove(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPortfolio {
    multiplexer_id: MultiplexerId,

    /// Target weights for each instrument relative to total equity.
    #[serde(default)]
    target_weights: HashMap<Instrument, f64>,

    /// Target quantity for each instrument.
    target_positions: Option<HashMap<Instrument, f64>>,
}

impl TargetPortfolio {
    pub fn new(
        multiplexer_id: MultiplexerId,
        target_weights: HashMap<Instrument, f64>,
        target_positions: Option<HashMap<Instrument, f64>>,
    ) -> Self {
        Self {
            multiplexer_id,
            target_weights,
            target_positions,
        }
    }

    pub fn multiplexer_id(&self) -> &MultiplexerId {
        &self.multiplexer_id
    }

    pub fn target_weights(&self) -> &HashMap<Instrument, f64> {
        &self.target_weights
    }

    pub fn target_positions(&self) -> Option<&HashMap<Instrument, f64>> {
        self.target_positions.as_ref()
    }
}
