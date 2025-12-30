use crate::model::{execution::ExecutionResult, order::Order, portfolio::Actual};

pub trait Broker: Send {
    /// Called when the Broker Gateway receives an order.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to process.
    ///
    /// # Returns
    ///
    /// * `(Vec<ExecutionResult>, Option<Actual>)` - A list of execution results and an optional portfolio update to publish.
    fn on_order(&mut self, order: Order) -> (Vec<ExecutionResult>, Option<Actual>);
}

impl Broker for Box<dyn Broker> {
    fn on_order(&mut self, order: Order) -> (Vec<ExecutionResult>, Option<Actual>) {
        (**self).on_order(order)
    }
}
