use crate::model::{
    allocation_batch::AllocationBatch,
    market_data::PriceUpdate,
    portfolio::{Actual, Target},
};

pub trait Manager: Send {
    /// Called when the Portfolio Manager receives a batch of allocations.
    ///
    /// # Arguments
    ///
    /// * `batch` - The batch of allocations.
    ///
    /// # Returns
    ///
    /// * `Option<Target>` - An optional target portfolio to publish.
    fn on_allocation(&mut self, batch: AllocationBatch) -> Option<Target>;

    /// Called when the Portfolio Manager receives an update on the actual portfolio state.
    ///
    /// # Arguments
    ///
    /// * `portfolio` - The actual portfolio state.
    ///
    /// # Returns
    ///
    /// * `Option<Target>` - An optional target portfolio to publish.
    fn on_portfolio(&mut self, portfolio: Actual) -> Option<Target>;

    /// Called when the Portfolio Manager receives a market data update.
    ///
    /// # Arguments
    ///
    /// * `market_data` - The price update.
    ///
    /// # Returns
    ///
    /// * `Option<Target>` - An optional target portfolio to publish.
    fn on_market_data(&mut self, market_data: PriceUpdate) -> Option<Target>;
}

impl Manager for Box<dyn Manager> {
    fn on_allocation(&mut self, batch: AllocationBatch) -> Option<Target> {
        (**self).on_allocation(batch)
    }

    fn on_portfolio(&mut self, portfolio: Actual) -> Option<Target> {
        (**self).on_portfolio(portfolio)
    }

    fn on_market_data(&mut self, market_data: PriceUpdate) -> Option<Target> {
        (**self).on_market_data(market_data)
    }
}
