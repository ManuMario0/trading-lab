use super::{Policy, RiskContext, RiskDecision};
use crate::models::{Order, Side};

/// Ensures that the total firm-wide exposure to a single instrument
/// does not exceed a specified percentage of Global Equity.
pub struct GlobalMaxPositionSize {
    max_pct: f64,
}

impl GlobalMaxPositionSize {
    pub fn new(max_pct: f64) -> Self {
        Self { max_pct }
    }
}

impl Policy for GlobalMaxPositionSize {
    fn name(&self) -> &str {
        "GlobalMaxPositionSize"
    }

    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        // 1. Calculate new projected global position
        let current_qty = ctx.consolidated.get_net_quantity(order.instrument());
        let change = match order.side() {
            Side::Buy => order.quantity(),
            Side::Sell => -order.quantity(),
        };
        let new_qty = current_qty + change;

        // 2. Calculate Value
        let price = ctx
            .prices
            .get(order.instrument())
            .map(|p| p.last())
            .unwrap_or(0.0);

        if price == 0.0 {
            // Can't evaluate value without price
            return RiskDecision::Rejected(format!(
                "No price for {:?} to check global limit",
                order.instrument()
            ));
        }

        let position_value = new_qty.abs() * price;
        let limit_value = ctx.consolidated.total_equity * self.max_pct;

        if position_value > limit_value {
            return RiskDecision::Rejected(format!(
                "Global Position Value {:.2} exceeds {:.2}% of Global Equity {:.2} (Limit: {:.2})",
                position_value,
                self.max_pct * 100.0,
                ctx.consolidated.total_equity,
                limit_value
            ));
        }

        RiskDecision::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AllocationConfig, ConsolidatedPortfolio, Instrument, MultiplexerId, OrderType, Portfolio,
        Price, Prices, SymbolId,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_global_limit_rejection() {
        let policy = GlobalMaxPositionSize::new(0.20); // 20% limit

        let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
        let mut prices = Prices::default();
        prices.insert(
            aapl.clone(),
            Price::new(
                aapl.clone(),
                100.0,
                100.0,
                100.0,
                Utc::now().timestamp_millis(),
            ),
        );

        // Setup: Strat A has 10 units ($1000). Total Equity $10,000.
        // Consolidated: 10 units. Equity 10k.
        let mut consolidated = ConsolidatedPortfolio::default();
        consolidated.net_positions.insert(aapl.clone(), 10.0);
        consolidated.total_equity = 10000.0;

        let portfolio = Portfolio::new(); // Local doesn't matter for this check
        let alloc = AllocationConfig::default();

        let ctx = RiskContext {
            portfolio: &portfolio,
            prices: &prices,
            total_equity: 10000.0, // Used by some, but logic uses ctx.consolidated.total_equity
            allocation_config: &alloc,
            consolidated: &consolidated,
        };

        // Order: Buy 20 units ($2000).
        // New Total: 30 units ($3000).
        // Limit: 20% of 10,000 = $2000.
        // 3000 > 2000 -> Reject.
        let order = Order::new(
            Uuid::new_v4(),
            MultiplexerId::new("StratA"),
            aapl.clone(),
            Side::Buy,
            20.0,
            OrderType::Market,
            Utc::now().timestamp_millis(),
        );

        let decision = policy.check(&order, &ctx);
        match decision {
            RiskDecision::Rejected(reason) => {
                assert!(reason.contains("exceeds 20.00%"));
            }
            _ => panic!("Should be rejected"),
        }
    }
}
