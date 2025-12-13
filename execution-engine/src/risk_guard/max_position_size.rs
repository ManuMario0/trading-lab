use super::{Policy, RiskContext, RiskDecision};
use crate::models::{Order, Side};

/// Enforces that no single position exceeds a specific percentage of Total
/// Equity.
pub struct MaxPositionSizePolicy {
    pub max_percent: f64, // e.g. 0.10 for 10%
}

impl Policy for MaxPositionSizePolicy {
    fn name(&self) -> &str {
        "MaxPositionSize"
    }

    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        // Unified lookup!
        let price = match ctx.prices.get(order.instrument()) {
            Some(p) => p.last(),
            None => {
                return RiskDecision::Rejected(format!(
                    "No price found for instrument {:?}",
                    order.instrument()
                ))
            }
        };

        // Calculate estimated new position size
        let current_qty = ctx.portfolio.positions().get_quantity(order.instrument());

        let change_qty = match order.side() {
            Side::Buy => order.quantity(),
            Side::Sell => -order.quantity(),
        };
        let new_qty = current_qty + change_qty;

        let new_exposure = new_qty.abs() * price;
        let limit = ctx.total_equity * self.max_percent;

        if new_exposure > limit {
            return RiskDecision::Rejected(format!(
                "New exposure {} exceeds limit {} ({:.1}%)",
                new_exposure,
                limit,
                self.max_percent * 100.0
            ));
        }

        RiskDecision::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AllocationConfig, ConsolidatedPortfolio, Instrument, MultiplexerId, Order, OrderType,
        Portfolio, Price, Prices, Side, SymbolId,
    };
    use crate::risk_guard::RiskGuard;
    use uuid::Uuid;

    fn mock_instrument(s: &str) -> Instrument {
        Instrument::Stock(SymbolId::new(s, "US"))
    }

    #[test]
    fn test_max_position_size_rejection() {
        let mut guard = RiskGuard::new();
        guard.add_policy(Box::new(MaxPositionSizePolicy { max_percent: 0.10 }));

        let aapl = mock_instrument("AAPL");
        let mut prices = Prices::default();
        prices.insert(
            aapl.clone(),
            Price::new(aapl.clone(), 100.0, 100.0, 100.0, 0),
        );

        let portfolio = Portfolio::new();
        let alloc_config = AllocationConfig::default();
        let consolidated = ConsolidatedPortfolio::default();

        let ctx = RiskContext {
            portfolio: &portfolio,
            prices: &prices,
            total_equity: 10000.0,
            allocation_config: &alloc_config,
            consolidated: &consolidated,
        };

        // Order for 11 shares @ 100 = 1100 (11%) -> Should fail
        let order = Order::new(
            Uuid::new_v4(),
            MultiplexerId::new("Test"),
            aapl.clone(),
            Side::Buy,
            11.0,
            OrderType::Market,
            0,
        );

        let decision = guard.check_order(&order, &ctx);
        assert!(matches!(decision, RiskDecision::Rejected(_)));
    }
}
