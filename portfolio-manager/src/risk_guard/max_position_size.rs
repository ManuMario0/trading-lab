use super::{Policy, RiskContext, RiskDecision};
use trading::model::allocation::Allocation;

/// Enforces that no single position exceeds a specific percentage of Total Equity.
pub struct MaxPositionSizePolicy {
    pub max_percent: f64,
}

impl Policy for MaxPositionSizePolicy {
    fn name(&self) -> &str {
        "MaxPositionSize"
    }

    fn check(&self, target: &Allocation, ctx: &RiskContext) -> RiskDecision {
        let equity = ctx.portfolio.total_equity;
        if equity <= 0.0 {
            // Can't optimize if no equity.
            // But if we are opening positions, we need equity.
            // If checking existing positions, maybe okay?
            // Let's assume positive equity required for new allocations.
            return RiskDecision::Rejected("Zero or Negative Equity".to_string());
        }

        let limit = equity * self.max_percent;

        for (instrument_id, pos) in target.get_positions() {
            if let Some(price) = ctx.prices.get(instrument_id) {
                let position_value = pos.get_quantity().abs() * price;
                if position_value > limit {
                    return RiskDecision::Rejected(format!(
                        "Position {} Value {:.2} exceeds limit {:.2} ({:.1}% of Equity {:.2})",
                        instrument_id,
                        position_value,
                        limit,
                        self.max_percent * 100.0,
                        equity
                    ));
                }
            } else {
                return RiskDecision::Rejected(format!(
                    "No price for Instrument {}",
                    instrument_id
                ));
            }
        }

        RiskDecision::Approved
    }
}
