use super::ids::MultiplexerId;
use super::instrument::Instrument;
use super::portfolio::Portfolio;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the aggregated state of the entire firm (all virtual portfolios
/// combined). This is computed on-the-fly to ensure consistency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidatedPortfolio {
    /// Net position across all strategies.
    /// Long 10 AAPL + Short 5 AAPL = Long 5 AAPL.
    pub net_positions: HashMap<Instrument, f64>,

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
            for (instrument, qty) in portfolio.positions().iter() {
                *net_positions.entry(instrument.clone()).or_insert(0.0) += qty;
            }

            // Aggregate Cash
            for (currency, account) in portfolio.cash().iter() {
                *total_cash.entry(currency.clone()).or_insert(0.0) += account.amount();
            }

            // Aggregate Equity (Just sum the current metric)
            total_equity += portfolio.metrics().cur_equity;
        }

        Self {
            net_positions,
            total_equity,
            total_cash,
        }
    }

    pub fn get_net_quantity(&self, instrument: &Instrument) -> f64 {
        *self.net_positions.get(instrument).unwrap_or(&0.0)
    }

    pub fn get_total_cash(&self, currency: &str) -> f64 {
        *self.total_cash.get(currency).unwrap_or(&0.0)
    }
}
