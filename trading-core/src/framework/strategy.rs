use crate::framework::context::Context;
use crate::model::allocation::Allocation;

/// The core trait that all user strategies must implement.
///
/// This trait isolates the business logic from the infrastructure.
/// The strategy simply receives a `Context` (inputs) and produces an `Allocation` (output).
pub trait Strategy {
    /// Called whenever a relevant event occurs (e.g., new market data).
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context containing the latest data updates.
    ///
    /// # Returns
    ///
    /// * `Option<Allocation>` - The target allocation to publish. Returns `None` if no action is needed.
    fn on_event(&mut self, ctx: &Context) -> Option<Allocation>;
}
