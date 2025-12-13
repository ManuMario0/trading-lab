use super::{Policy, RiskContext, RiskDecision};
use crate::models::{Order, Side};

/// Safeguard enforcing that a strategy's Total Equity does not exceed
/// its allocated fraction of the Firm's Global Equity (plus a tolerance).
///
/// If a strategy is overweight, it blocks any order that increases absolute
/// exposure.
pub struct MaxStrategyAllocation {
    /// Tolerance (e.g., 0.05 for 5% drift).
    /// Max Allowed = GlobalEquity * (AllocationTarget + Tolerance)
    tolerance: f64,
}

impl MaxStrategyAllocation {
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }
}

impl Policy for MaxStrategyAllocation {
    fn name(&self) -> &str {
        "MaxStrategyAllocation"
    }

    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        // 1. Get Strategy Config
        let config = match ctx.allocation_config.get(order.multiplexer_id()) {
            Some(c) => c,
            None => {
                // Should not happen in a valid system, but if no config, maybe strictly reject?
                // Or assume 0 allocation? Let's conservative reject.
                return RiskDecision::Rejected(format!(
                    "No allocation config found for {}",
                    order.multiplexer_id()
                ));
            }
        };

        // 2. Check Equity Limit
        let target_pct = config.allocation_fraction();
        let max_pct = target_pct + self.tolerance;

        // Use Consolidated Total Equity for the denominator
        let global_equity = ctx.consolidated.total_equity;
        let limit_equity = global_equity * max_pct;

        // Use Strategy's Current Equity for the numerator
        let strategy_equity = ctx.portfolio.metrics().cur_equity;

        // If we are within limits, we are good.
        if strategy_equity <= limit_equity {
            return RiskDecision::Approved;
        }

        // 3. We are Overweight. Only allow Risk Reduction.
        let current_qty = ctx.portfolio.positions().get_quantity(order.instrument());
        let change = match order.side() {
            Side::Buy => order.quantity(),
            Side::Sell => -order.quantity(),
        };
        let new_qty = current_qty + change;

        if new_qty.abs() > current_qty.abs() {
            return RiskDecision::Rejected(format!(
                "Strategy Overweight: Equity {:.2} > Limit {:.2} ({:.1}%). Increasing exposure rejected.",
                strategy_equity,
                limit_equity,
                max_pct * 100.0
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
        Prices, StrategyConfig, SymbolId,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_allocation_safeguard() {
        let id = MultiplexerId::new("WinnerStrat");
        let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));

        // Config: 10% Target. 5% Tolerance. Max = 15%.
        let mut alloc_config = AllocationConfig::default();
        alloc_config.insert(id.clone(), StrategyConfig::new(0.10, 0.20, 0.0));

        let policy = MaxStrategyAllocation::new(0.05);

        // Scenario: Global Equity 100k. Limit = 15k.
        let mut consolidated = ConsolidatedPortfolio::default();
        consolidated.total_equity = 100000.0;

        let mut portfolio = Portfolio::new();
        // Strategy has 16k Equity (Overweight!)
        portfolio.metrics_mut().cur_equity = 16000.0;
        // Holds 100 AAPL (Value doesn't matter for this check, just qty direction)
        portfolio.positions_mut().set_quantity(aapl.clone(), 100.0);

        let prices = Prices::default(); // Not needed for this specific logic check

        let ctx = RiskContext {
            portfolio: &portfolio,
            prices: &prices,
            total_equity: 0.0, // Unused
            allocation_config: &alloc_config,
            consolidated: &consolidated,
        };

        // 1. Try to BUY more (Increase Risk) -> Fail
        let buy_order = Order::new(
            Uuid::new_v4(),
            id.clone(),
            aapl.clone(),
            Side::Buy,
            10.0,
            OrderType::Market,
            Utc::now().timestamp_millis(),
        );

        match policy.check(&buy_order, &ctx) {
            RiskDecision::Rejected(msg) => {
                assert!(msg.contains("Strategy Overweight"));
            }
            _ => panic!("Should reject risk increase"),
        }

        // 2. Try to SELL (Reduce Risk) -> Approve
        let sell_order = Order::new(
            Uuid::new_v4(),
            id.clone(),
            aapl.clone(),
            Side::Sell,
            10.0, // Reduces 100 -> 90
            OrderType::Market,
            Utc::now().timestamp_millis(),
        );

        match policy.check(&sell_order, &ctx) {
            RiskDecision::Approved => {}
            _ => panic!("Should approve risk reduction"),
        }
    }
}
