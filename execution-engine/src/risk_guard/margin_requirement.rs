use crate::models::{Order, Side};
use crate::risk_guard::{Policy, RiskContext, RiskDecision};

pub struct MarginRequirement {
    /// Margin required per unit of value (e.g. 0.5 = 2x leverage)
    pub initial_margin_rate: f64,
    /// Safety buffer (e.g. 1.05 = current equity must be 5% above margin req)
    pub buffer: f64,
}

impl MarginRequirement {
    pub fn new(initial_margin_rate: f64, buffer: f64) -> Self {
        Self {
            initial_margin_rate,
            buffer,
        }
    }
}

impl Policy for MarginRequirement {
    fn name(&self) -> &str {
        "MarginRequirement"
    }

    fn check(&self, order: &Order, ctx: &RiskContext) -> RiskDecision {
        // Single order check is just a batch of 1
        self.check_batch(std::slice::from_ref(order), ctx)
    }

    fn check_batch(&self, orders: &[Order], ctx: &RiskContext) -> RiskDecision {
        // 1. Simulate new positions
        let mut projected_positions = ctx.portfolio.positions().clone();

        for order in orders {
            match order.side() {
                Side::Buy => projected_positions
                    .update_quantity(order.instrument().clone(), order.quantity()),
                Side::Sell => projected_positions
                    .update_quantity(order.instrument().clone(), -order.quantity()),
            }
        }

        // 2. Calculate Required Margin
        let mut total_margin_required = 0.0;

        for (instrument, quantity) in projected_positions.iter() {
            if *quantity == 0.0 {
                continue;
            }

            let price_opt = ctx.prices.get(instrument);

            let price = match price_opt {
                Some(p) => p.last(),
                None => {
                    return RiskDecision::Rejected(format!(
                        "Missing price for instrument: {:?}",
                        instrument
                    ))
                }
            };

            let position_value = quantity.abs() * price;

            // Logic for Margin:
            // Usually different instruments have different margins.
            // For now, applying the global rate to EVERYTHING.

            total_margin_required += position_value * self.initial_margin_rate;
        }

        // 3. Check against Equity (with buffer)
        let required_equity_with_buffer = total_margin_required * self.buffer;

        if ctx.total_equity >= required_equity_with_buffer {
            RiskDecision::Approved
        } else {
            RiskDecision::Rejected(format!(
                "Insufficient Equity. Req: {:.2} (incl buffer), Avail: {:.2}",
                required_equity_with_buffer, ctx.total_equity
            ))
        }
    }
}
