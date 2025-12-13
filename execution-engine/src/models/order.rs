use super::ids::MultiplexerId;
use super::instrument::Instrument;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit(f64),
}

/// An instruction to buy or sell an instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    id: Uuid,
    multiplexer_id: MultiplexerId,
    instrument: Instrument,
    side: Side,
    quantity: f64,
    order_type: OrderType,
    timestamp: i64,
}

impl Order {
    pub fn new(
        id: Uuid,
        multiplexer_id: MultiplexerId,
        instrument: Instrument,
        side: Side,
        quantity: f64,
        order_type: OrderType,
        timestamp: i64,
    ) -> Self {
        Self {
            id,
            multiplexer_id,
            instrument,
            side,
            quantity,
            order_type,
            timestamp,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn multiplexer_id(&self) -> &MultiplexerId {
        &self.multiplexer_id
    }

    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    pub fn order_type(&self) -> &OrderType {
        &self.order_type
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
}
