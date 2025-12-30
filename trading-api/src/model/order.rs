//! Order models.
//!
//! Defines the structure of orders and their associated types (Side, Type).

use serde::{Deserialize, Serialize};

/// Internal representation of the side of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum OrderSideInternal {
    Buy,
    Sell,
}

/// Represents the side of an order (Buy or Sell).
/// Wrapped in a struct for FFI compatibility (opaque pointer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderSide {
    inner: OrderSideInternal,
}

impl OrderSide {
    pub fn buy() -> Self {
        Self {
            inner: OrderSideInternal::Buy,
        }
    }

    pub fn sell() -> Self {
        Self {
            inner: OrderSideInternal::Sell,
        }
    }

    /// Helper for FFI: 0 = Buy, 1 = Sell
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => Self::buy(),
            _ => Self::sell(),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self.inner {
            OrderSideInternal::Buy => 0,
            OrderSideInternal::Sell => 1,
        }
    }
}

/// Internal representation of the type of order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum OrderTypeInternal {
    Limit,
    Market,
    Stop,
}

/// Represents the type of order.
/// Wrapped in a struct for FFI compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderType {
    inner: OrderTypeInternal,
}

impl OrderType {
    pub fn limit() -> Self {
        Self {
            inner: OrderTypeInternal::Limit,
        }
    }

    pub fn market() -> Self {
        Self {
            inner: OrderTypeInternal::Market,
        }
    }

    pub fn stop() -> Self {
        Self {
            inner: OrderTypeInternal::Stop,
        }
    }

    /// Helper for FFI: 0 = Limit, 1 = Market, 2 = Stop
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => Self::limit(),
            1 => Self::market(),
            _ => Self::stop(),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self.inner {
            OrderTypeInternal::Limit => 0,
            OrderTypeInternal::Market => 1,
            OrderTypeInternal::Stop => 2,
        }
    }
}

/// Represents a trading order to buy or sell an instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Unique identifier for the order.
    id: String,
    /// Identifier for the instrument being traded.
    instrument_id: String,
    /// Side of the order (Buy/Sell).
    side: OrderSide,
    /// Type of the order (Limit/Market/Stop).
    order_type: OrderType,
    /// Price at which to execute (relevant for Limit orders).
    price: f64,
    /// Quantity to trade.
    quantity: f64,
    /// Timestamp when the order was created (Unix timestamp).
    timestamp: i64,
}

impl Order {
    /// Creates a new Order.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique order ID.
    /// * `instrument_id` - ID of the instrument to trade.
    /// * `side` - `OrderSide::Buy` or `OrderSide::Sell`.
    /// * `order_type` - Type of order (`Limit`, `Market`, etc.).
    /// * `price` - execution price (ignored for Market orders).
    /// * `quantity` - Quantity to trade.
    /// * `timestamp` - Creation timestamp.
    ///
    /// # Returns
    ///
    /// A new `Order` instance.
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

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_instrument_id(&self) -> &str {
        &self.instrument_id
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_type(&self) -> OrderType {
        self.order_type
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_side_i32(&self) -> i32 {
        self.side.as_i32()
    }

    pub fn get_type_i32(&self) -> i32 {
        self.order_type.as_i32()
    }
}
