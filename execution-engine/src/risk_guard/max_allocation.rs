use super::{Policy, RiskContext, RiskDecision};
use crate::models::{Order, Side};

/// Enforces that the total value of the portfolio for a specific Multiplexer
/// does not exceed its allocated fraction of the Global Equity.
pub struct MaxAllocationPolicy;

impl Policy for MaxAllocationPolicy {
    fn name(&self) -> &str {
        "MaxAllocation"
    }

    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        let multiplexer_id = order.multiplexer_id();

        // 1. Get Allowed Allocation
        let allowed_fraction = match ctx.allocation_config.get(multiplexer_id) {
            Some(config) => config.allocation_fraction(),
            None => {
                // If no config exists for this ID, do we block or allow?
                // Safe default: Block unless explicitly allowed.
                return RiskDecision::Rejected(format!(
                    "No allocation config for {}",
                    multiplexer_id
                ));
            }
        };

        let allowed_equity = ctx.total_equity * allowed_fraction;
        debug_assert!(
            allowed_fraction < 1.0,
            "Allowed allocation fraction must be strictly less than 1.0"
        );

        // 2. Calculate Current NAV & Simulate Order Impact
        // We calculate the NAV *after* the order would be executed to ensure
        // compliance. Theoretically, swapping Cash for Asset at market price
        // doesn't change NAV (Net Asset Value), but this check ensures rigour.

        let mut post_trade_cash = ctx.portfolio.cash().clone();
        let mut post_trade_positions = ctx.portfolio.positions().clone();

        let execution_price = match ctx.prices.get(order.instrument()) {
            Some(p) => p.last(),
            None => {
                return RiskDecision::Rejected(format!("No price for {:?}", order.instrument()))
            }
        };

        // Update Cash (Assuming USD)
        match order.side() {
            Side::Buy => {
                post_trade_cash.withdraw("USD", order.quantity() * execution_price);
                post_trade_positions.update_quantity(order.instrument().clone(), order.quantity());
            }
            Side::Sell => {
                post_trade_cash.deposit("USD", order.quantity() * execution_price);
                post_trade_positions.update_quantity(order.instrument().clone(), -order.quantity());
            }
        }

        let mut post_trade_nav = 0.0;

        // Sum Instrument Positions
        for (instrument, qty) in post_trade_positions.iter() {
            if let Some(price) = ctx.prices.get(instrument) {
                post_trade_nav += qty * price.last();
            } else {
                return RiskDecision::Rejected(format!("No price for {:?}", instrument));
            }
        }

        // Sum Cash
        for (currency, acc) in post_trade_cash.iter() {
            if currency == "USD" {
                post_trade_nav += acc.amount();
            } else {
                // Ignore other currencies or implement FX lookup if needed for
                // this policy
                // For now, assuming only USD cash or ignoring others for this
                // specific check
            }
        }

        // 3. Enforce Limit
        if post_trade_nav > allowed_equity && order.side() == Side::Buy {
            return RiskDecision::Rejected(format!(
                "Position Limit: Allocating {:.2} > Max Alloc {:.2} (Total Eq {:.2})",
                post_trade_nav, allowed_equity, ctx.total_equity
            ));
        }

        RiskDecision::Approved
    }
}
