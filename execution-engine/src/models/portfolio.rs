use super::instrument::{CurrencyPair, Instrument};
use super::market::Prices;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub cur_equity: f64,
    pub high_water_mark: f64,
    pub drawdown: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CashAccount {
    currency: String,
    amount: f64,
}

impl CashAccount {
    pub fn new(currency: impl Into<String>, amount: f64) -> Self {
        Self {
            currency: currency.into(),
            amount,
        }
    }
    pub fn amount(&self) -> f64 {
        self.amount
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CashAccounts {
    accounts: HashMap<String, CashAccount>,
}

impl CashAccounts {
    pub fn get_balance(&self, currency: &str) -> f64 {
        self.accounts.get(currency).map(|a| a.amount).unwrap_or(0.0)
    }

    pub fn deposit(&mut self, currency: &str, amount: f64) {
        let entry = self.accounts.entry(currency.to_string()).or_default();
        entry.currency = currency.to_string(); // Ensure currency code is set
        entry.amount += amount;
    }

    pub fn withdraw(&mut self, currency: &str, amount: f64) {
        let entry = self.accounts.entry(currency.to_string()).or_default();
        entry.currency = currency.to_string();
        entry.amount -= amount;
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, CashAccount> {
        self.accounts.iter()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Positions {
    holdings: HashMap<Instrument, f64>,
}

impl Positions {
    pub fn get_quantity(&self, instrument: &Instrument) -> f64 {
        *self.holdings.get(instrument).unwrap_or(&0.0)
    }

    pub fn update_quantity(&mut self, instrument: Instrument, quantity: f64) {
        let entry = self.holdings.entry(instrument).or_insert(0.0);
        *entry += quantity;
    }

    // Set absolute quantity (used for testing or syncing)
    pub fn set_quantity(&mut self, instrument: Instrument, quantity: f64) {
        self.holdings.insert(instrument, quantity);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Instrument, f64> {
        self.holdings.iter()
    }

    pub fn calculate_equity(&self, prices: &Prices) -> f64 {
        let mut equity = 0.0;
        for (instrument, quantity) in &self.holdings {
            if let Some(price) = prices.get(instrument) {
                equity += quantity * price.last();
            }
        }
        equity
    }

    pub fn calculate_total_position_value(&self, prices: &Prices) -> f64 {
        let mut total_value = 0.0;
        for (instrument, quantity) in &self.holdings {
            if let Some(price) = prices.get(instrument) {
                total_value += quantity * price.last();
            }
        }
        total_value
    }
}

/// Represents the current state of holdings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Portfolio {
    positions: Positions,
    cash: CashAccounts,
    pub metrics: PerformanceMetrics,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: Positions::default(),
            cash: CashAccounts::default(),
            metrics: PerformanceMetrics::default(),
        }
    }

    pub fn positions(&self) -> &Positions {
        &self.positions
    }

    // Mutable access needed for complex logic or specific setters?
    // Better to expose domain methods on Portfolio?
    pub fn positions_mut(&mut self) -> &mut Positions {
        &mut self.positions
    }

    pub fn cash(&self) -> &CashAccounts {
        &self.cash
    }

    pub fn cash_mut(&mut self) -> &mut CashAccounts {
        &mut self.cash
    }

    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn metrics_mut(&mut self) -> &mut PerformanceMetrics {
        &mut self.metrics
    }

    /// Calculates the current equity based on the provided prices.
    pub fn calculate_equity(&self, base_currency: &str, prices: &Prices) -> f64 {
        let mut equity = 0.0;

        // 1. Value of Positions
        for (instrument, quantity) in self.positions.iter() {
            if let Some(price) = prices.get(instrument) {
                equity += quantity * price.last();
            }
        }

        // 2. Value of Cash (converted to base currency)
        for (currency_code, account) in self.cash.iter() {
            if currency_code == base_currency {
                equity += account.amount;
            } else {
                let pair = CurrencyPair::new(currency_code.clone(), base_currency.to_string());
                let instrument = Instrument::Forex(pair);

                if let Some(price) = prices.get(&instrument) {
                    equity += account.amount * price.last();
                } else {
                    let inv_pair =
                        CurrencyPair::new(base_currency.to_string(), currency_code.clone());
                    let inv_instrument = Instrument::Forex(inv_pair);

                    if let Some(price) = prices.get(&inv_instrument) {
                        if price.last() != 0.0 {
                            equity += account.amount / price.last();
                        }
                    }
                }
            }
        }

        equity
    }

    pub fn calculate_gross_exposure(&self, prices: &Prices) -> f64 {
        let mut exposure = 0.0;
        for (instrument, quantity) in self.positions.iter() {
            if let Some(price) = prices.get(instrument) {
                exposure += quantity.abs() * price.last();
            }
        }
        exposure
    }

    pub fn calculate_liquidation_value(
        &self,
        base_currency: &str,
        prices: &Prices,
        safety_margin: f64,
    ) -> f64 {
        let mut value = 0.0;

        let calc_pos_val = |qty: f64, bid: f64, ask: f64| -> f64 {
            if qty > 0.0 {
                // Long: Sell at Bid * Safety
                qty * bid * safety_margin
            } else {
                // Short: Buy at Ask * (1/Safety) or Ask * (1 + (1-Safety))?
                // Plan said: "Buy at Ask * safety_margin" (implicit conservative valuation)
                // Actually, buying back to cover costs MORE. So we should increase the cost.
                // If safety_margin is 0.99 (1% buffer).
                // Cost = Ask / 0.99 OR Ask * (1 + 0.01).
                // Let's use Ask / safety_margin to be symmetric with "value reduction".
                // Negative quantity * Positive Price = Negative Value (Liability).
                // Increasing Liability means making it MORE negative.
                // qty (-10) * Ask (100) / 0.99 = -1010. Correct (more liability).
                qty * ask / safety_margin
            }
        };

        for (instrument, quantity) in self.positions.iter() {
            if let Some(price) = prices.get(instrument) {
                value += calc_pos_val(*quantity, price.bid(), price.ask());
            }
        }

        for (currency_code, account) in self.cash.iter() {
            if currency_code == base_currency {
                value += account.amount;
            } else {
                let pair = CurrencyPair::new(currency_code.clone(), base_currency.to_string());
                let instrument = Instrument::Forex(pair);

                if let Some(price) = prices.get(&instrument) {
                    // Forex is just another asset
                    value += calc_pos_val(account.amount, price.bid(), price.ask());
                }
            }
        }

        value
    }
}
