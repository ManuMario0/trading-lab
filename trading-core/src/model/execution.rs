use serde::{Deserialize, Serialize};

/// Status of an order execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Order has been accepted by the system but not yet sent to broker.
    New,
    /// Order has been acknowledged by the broker.
    Pending,
    /// Order has been partially filled.
    PartiallyFilled,
    /// Order has been fully filled.
    Filled,
    /// Order has been cancelled.
    Cancelled,
    /// Order has been rejected by the broker or exchange.
    Rejected,
    /// Order has expired.
    Expired,
}

/// Represents the result of an execution report from the broker/exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The ID of the order this report corresponds to.
    pub order_id: String,
    /// The instrument ID.
    pub instrument_id: String,
    /// The current status of the order.
    pub status: ExecutionStatus,
    /// The quantity filled in this specific report (delta).
    pub last_filled_quantity: f64,
    /// The price at which the last fill occurred.
    pub last_filled_price: f64,
    /// The total quantity filled so far for this order.
    pub cumulative_fill_quantity: f64,
    /// The average price of fills so far.
    pub average_price: f64,
    /// Timestamp of the execution report (unix millis).
    pub timestamp: u128,
    /// Optional rejection reason or message.
    pub message: Option<String>,
}

impl ExecutionResult {
    pub fn new(
        order_id: impl Into<String>,
        instrument_id: impl Into<String>,
        status: ExecutionStatus,
        timestamp: u128,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            instrument_id: instrument_id.into(),
            status,
            last_filled_quantity: 0.0,
            last_filled_price: 0.0,
            cumulative_fill_quantity: 0.0,
            average_price: 0.0,
            timestamp,
            message: None,
        }
    }

    pub fn with_fill(
        mut self,
        last_qty: f64,
        last_price: f64,
        cum_qty: f64,
        avg_price: f64,
    ) -> Self {
        self.last_filled_quantity = last_qty;
        self.last_filled_price = last_price;
        self.cumulative_fill_quantity = cum_qty;
        self.average_price = avg_price;
        self
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}
