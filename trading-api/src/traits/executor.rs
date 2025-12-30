use crate::model::{
    execution::ExecutionResult,
    order::Order,
    portfolio::{Actual, Target},
};

pub trait Executor: Send {
    /// Called when the Execution Engine receives a target portfolio.
    ///
    /// # Arguments
    ///
    /// * `target` - The target portfolio.
    ///
    /// # Returns
    ///
    /// * `(Vec<Order>, Option<Actual>)` - A list of orders to execute and an optional portfolio update to publish.
    fn on_target(&mut self, target: Target) -> (Vec<Order>, Option<Actual>);

    /// Called when the Execution Engine receives an execution result from the broker.
    ///
    /// # Arguments
    ///
    /// * `execution` - The execution result.
    ///
    /// # Returns
    ///
    /// * `(Vec<Order>, Option<Actual>)` - A list of orders to execute and an optional portfolio update to publish.
    fn on_execution(&mut self, execution: ExecutionResult) -> (Vec<Order>, Option<Actual>);
}
