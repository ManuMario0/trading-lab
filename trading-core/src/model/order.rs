use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub instrument_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: i64,
}

impl Order {
    pub fn new(
        id: impl Into<String>,
        instrument_id: impl Into<String>,
        side: OrderSide,
        order_type: OrderType,
        price: f64,
        quantity: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            id: id.into(),
            instrument_id: instrument_id.into(),
            side,
            order_type,
            price,
            quantity,
            timestamp,
        }
    }
}
