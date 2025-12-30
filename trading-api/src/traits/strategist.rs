use crate::model::{allocation::Allocation, identity::Id, market_data::MarketDataBatch};

pub trait Strategist: Send {
    /// Called when the Strategy receives a batch of market data updates.
    ///
    /// # Arguments
    ///
    /// * `md` - The batch of updates.
    ///
    /// # Returns
    ///
    /// * `Option<Allocation>` - An optional target allocation to publish.
    fn on_market_data(&mut self, md: MarketDataBatch) -> Option<Allocation>;

    /// Called periodically or on events to allow the strategy to update its state.
    fn update(&mut self) -> Option<Allocation> {
        None
    }
}
