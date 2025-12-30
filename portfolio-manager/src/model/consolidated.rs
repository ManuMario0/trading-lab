use super::ids::MultiplexerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use trading::model::{instrument::InstrumentId, portfolio::Portfolio};

/// Represents the aggregated state of the entire firm.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidatedPortfolio {
    /// Net position across all strategies.
    pub net_positions: HashMap<InstrumentId, f64>,

    /// Total Equity (Sum of all strategy equities).
    pub total_equity: f64,

    /// Total Cash by currency (Sum of all strategy cash).
    pub total_cash: HashMap<String, f64>,
}

impl ConsolidatedPortfolio {
    pub fn aggregate(portfolios: &HashMap<MultiplexerId, Portfolio>) -> Self {
        let mut net_positions = HashMap::new();
        let mut total_equity = 0.0;
        let mut total_cash = HashMap::new();

        for portfolio in portfolios.values() {
            // Aggregate Positions
            for (instrument_id, pos) in &portfolio.positions {
                *net_positions.entry(*instrument_id).or_insert(0.0) += pos.get_quantity();
            }

            // Aggregate Cash
            for (currency, account) in &portfolio.cash {
                *total_cash.entry(currency.clone()).or_insert(0.0) += account.amount;
            }

            // Aggregate Equity
            total_equity += portfolio.total_equity;
        }

        Self {
            net_positions,
            total_equity,
            total_cash,
        }
    }

    pub fn get_net_quantity(&self, instrument_id: &InstrumentId) -> f64 {
        *self.net_positions.get(instrument_id).unwrap_or(&0.0)
    }
}
