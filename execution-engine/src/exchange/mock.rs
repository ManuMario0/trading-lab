use super::{Exchange, ExecutionResult};
use crate::models::Order;
use chrono::Utc;

pub struct MockExchange {
    fee_rate: f64, // e.g. 0.0005 for 5 bps
}

impl MockExchange {
    pub fn new(fee_rate: f64) -> Self {
        Self { fee_rate }
    }
}

impl Exchange for MockExchange {
    fn submit_order(&mut self, order: &Order, market_price: f64) -> ExecutionResult {
        let val = order.quantity() * market_price;
        let fee = val * self.fee_rate;

        // In a real sim, we might add slippage here based on quantity vs liquidity.
        // For now, perfect fill at market price.

        ExecutionResult {
            order_id: order.id(),
            instrument: order.instrument().clone(),
            side: order.side(),
            quantity: order.quantity(),
            price: market_price,
            fee,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}
