use crate::model::{allocation_batch::AllocationBatch, market_data::MarketDataBatch};

pub trait Strategist: Send {
    /// Called when the Strategy receives a batch of market data updates.
    ///
    /// # Arguments
    ///
    /// * `md` - The batch of updates.
    ///
    /// # Returns
    ///
    /// * `AllocationBatch` - The batch of allocations.
    fn on_market_data(&mut self, md: MarketDataBatch) -> AllocationBatch;
}

impl Strategist for Box<dyn Strategist> {
    fn on_market_data(&mut self, md: MarketDataBatch) -> AllocationBatch {
        (**self).on_market_data(md)
    }
}
