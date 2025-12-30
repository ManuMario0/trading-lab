use trading::model::{
    execution::{ExecutionResult, ExecutionStatus},
    order::{Order, OrderSide},
    portfolio::{Actual, Portfolio},
};
use trading::traits::broker::Broker;

pub struct PaperBroker {
    portfolio: Portfolio,
}

impl PaperBroker {
    pub fn new(initial_cash: f64) -> Self {
        let mut p = Portfolio::new();
        p.set_cash("USD", initial_cash);
        Self {
            portfolio: p.with_equity(initial_cash),
        }
    }
}

impl Broker for PaperBroker {
    fn on_order(&mut self, order: Order) -> (Vec<ExecutionResult>, Option<Actual>) {
        let fill_price = order.get_price();
        let effective_price = if fill_price > 0.0 { fill_price } else { 100.0 };

        let qty = order.get_quantity();
        // ID handling? Assume parsable
        let instrument_id = order.get_instrument_id().parse::<usize>().unwrap_or(0);

        // --- Wallet Logic (Simplified) ---
        let cost = qty * effective_price;
        let currency = "USD";
        let mut cash_balance = self
            .portfolio
            .get_cash(currency)
            .cloned()
            .unwrap_or_else(|| trading::model::portfolio::CashBalance::new(currency, 0.0, 0.0));

        let side = order.get_side();
        match side {
            s if s == OrderSide::buy() => {
                cash_balance.amount -= cost;
                cash_balance.available -= cost;
                let current_pos_qty = self
                    .portfolio
                    .positions
                    .get(&instrument_id)
                    .map(|p| p.get_quantity())
                    .unwrap_or(0.0);
                self.portfolio
                    .update_position(instrument_id, current_pos_qty + qty);
            }
            s if s == OrderSide::sell() => {
                cash_balance.amount += cost;
                cash_balance.available += cost;
                let current_pos_qty = self
                    .portfolio
                    .positions
                    .get(&instrument_id)
                    .map(|p| p.get_quantity())
                    .unwrap_or(0.0);
                self.portfolio
                    .update_position(instrument_id, current_pos_qty - qty);
            }
            _ => {}
        }
        self.portfolio
            .cash
            .insert(currency.to_string(), cash_balance);

        let result = ExecutionResult {
            order_id: order.get_id().to_string(),
            instrument_id: order.get_instrument_id().to_string(),
            status: ExecutionStatus::Filled,

            // Correct fields for accumulated execution
            last_filled_quantity: qty,
            last_filled_price: effective_price,
            cumulative_fill_quantity: qty,
            average_price: effective_price,

            timestamp: order.get_timestamp() as u128,
            message: None,
        };

        (vec![result], Some(Actual(self.portfolio.clone())))
    }
}
