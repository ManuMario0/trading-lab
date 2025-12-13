use crate::models::{Instrument, Order, Side};
use uuid::Uuid;

pub mod mock;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub order_id: Uuid,
    pub instrument: Instrument,
    pub side: Side,
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub timestamp: i64,
}

/// Interface for executing orders against a market (Real or Mock).
pub trait Exchange: Send + Sync {
    /// Submit an order to the exchange.
    /// Returns an ExecutionResult synchronously (for now, simplistic fill).
    /// In a real async system, this might return a Submission ID, and fills
    /// come later.
    fn submit_order(&mut self, order: &Order, market_price: f64) -> ExecutionResult;
}
