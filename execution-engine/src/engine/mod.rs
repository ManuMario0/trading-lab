use log::info;
use std::collections::HashSet;
use trading::model::{
    execution::ExecutionResult,
    instrument::InstrumentId,
    order::{Order, OrderSide, OrderType},
    portfolio::{Actual, Target},
};
use trading::traits::executor::Executor;
use uuid::Uuid;

pub struct Engine {
    /// The current "Actual" portfolio state (what we own).
    actual: Actual,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            actual: Actual(trading::model::portfolio::Portfolio::new()),
        }
    }

    /// Helper to generate orders to move from Actual -> Target
    fn reconcile(&self, target: &Target) -> Vec<Order> {
        let mut orders = Vec::new();

        // We only look at Positions in the Target (Quantity-based).

        let target_portfolio = &target.0;
        let actual_portfolio = &self.actual.0;

        let mut all_ids: HashSet<InstrumentId> = HashSet::new();
        all_ids.extend(target_portfolio.positions.keys());
        all_ids.extend(actual_portfolio.positions.keys());

        for inst_id in all_ids {
            let target_qty = target_portfolio
                .positions
                .get(&inst_id)
                .map(|p| p.get_quantity())
                .unwrap_or(0.0);
            let actual_qty = actual_portfolio
                .positions
                .get(&inst_id)
                .map(|p| p.get_quantity())
                .unwrap_or(0.0);

            let diff = target_qty - actual_qty;

            // Simple threshold to avoid dust orders
            if diff.abs() > 1e-6 {
                let side = if diff > 0.0 {
                    OrderSide::buy()
                } else {
                    OrderSide::sell()
                };

                let order = Order::new(
                    Uuid::new_v4().to_string(), // id
                    inst_id.to_string(),        // instrument_id (as string)
                    side,
                    OrderType::market(),                   // order_type
                    0.0,                                   // price
                    diff.abs(),                            // quantity
                    chrono::Utc::now().timestamp_millis(), // timestamp
                );

                orders.push(order);
            }
        }

        orders
    }
}

impl Executor for Engine {
    fn on_target(&mut self, target: Target) -> (Vec<Order>, Option<Actual>) {
        info!("Received Target Portfolio");

        // Reconcile Target vs Actual
        let orders = self.reconcile(&target);

        // We don't update Actual here; we wait for Fills.

        (orders, None)
    }

    fn on_execution(&mut self, execution: ExecutionResult) -> (Vec<Order>, Option<Actual>) {
        // info!("Received Execution for Order: {}", execution.order_id);

        // Update Local Actual from Fill
        let inst_id_str = execution.instrument_id;
        let inst_id = inst_id_str.parse::<usize>().unwrap_or(0);

        if inst_id > 0 {
            let _current_qty = self
                .actual
                .0
                .positions
                .get(&inst_id)
                .map(|p| p.get_quantity())
                .unwrap_or(0.0);
            // Logic to update position would go here.
            // For "Dumb Clean Engine", we trust the next Target/Actual loop or
            // Broker updates.
        }

        (vec![], None)
    }
}
