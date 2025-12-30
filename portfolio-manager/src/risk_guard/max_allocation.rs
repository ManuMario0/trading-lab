use super::{Policy, RiskContext, RiskDecision};
use trading::model::allocation::Allocation;

/// Enforces that the total value of the portfolio does not exceed limits.
pub struct MaxAllocationPolicy;

impl Policy for MaxAllocationPolicy {
    fn name(&self) -> &str {
        "MaxAllocation"
    }

    fn check(&self, target: &Allocation, ctx: &RiskContext) -> RiskDecision {
        // 1. Get Allowed Allocation
        let allowed_fraction = match ctx.allocation_config.get(ctx.multiplexer_id) {
            Some(config) => config.allocation_fraction(),
            None => {
                // If no config, assume 0 allocation allowed? Or default?
                // For safety, reject.
                return RiskDecision::Rejected(format!(
                    "No allocation config for {}",
                    ctx.multiplexer_id
                ));
            }
        };

        let allowed_equity = ctx.consolidated.total_equity * allowed_fraction;

        // 2. Calculate NAV of Target Allocation
        // Note: This only counts "Positions Value".
        // It does NOT count Cash because Allocation doesn't track cash.
        // We assume rebalancing trades will convert cash <-> assets.
        // So we check if (Asset Value) <= Allowed Equity.
        // Actually, Total Equity = Asset Value + Cash.
        // So we are checking if `Projected Asset Value` is too high?
        // Or if the `Total Projected Equity` is too high?
        // Since we don't know the exact cash impact without simulating execution prices (which may slip),
        // we can estimate: Current Equity.
        // Wait, Allocation sets the target *positions*.
        // If we want to limit Leverage, we check: Sum(Abs(Qty) * Price) <= Leverage * Equity.
        // If we want to limit "Allocation Size" (like in a multi-manager setup),
        // we usually limit the Capital Allocated to the strategy.
        // The strategy already has `current_equity`.
        // If `current_equity` > `allowed`, we should force reduce.
        // But here we are checking a specific Allocation request.

        // Impl: Calculate Gross Exposure of Target.
        let mut gross_exposure = 0.0;

        for (instrument_id, pos) in target.get_positions() {
            if let Some(price) = ctx.prices.get(instrument_id) {
                gross_exposure += pos.get_quantity().abs() * price;
            } else {
                return RiskDecision::Rejected(format!(
                    "No price for Instrument {}",
                    instrument_id
                ));
            }
        }

        // If we assume standard 1.0 leverage, Exposure <= Equity.
        // And Equity is limited by configuration.
        // But the Strategy's Equity tracks its PnL.
        // The `allowed_fraction` limits how much of the *Firm's* equity this strategy can *hold*.
        // This effectively limits the strategy's size.

        if gross_exposure > allowed_equity {
            return RiskDecision::Rejected(format!(
                "Target Exposure {:.2} exceeds Max Allocation {:.2} (Fraction: {:.2} of Global {:.2})",
                gross_exposure, allowed_equity, allowed_fraction, ctx.consolidated.total_equity
            ));
        }

        RiskDecision::Approved
    }
}
