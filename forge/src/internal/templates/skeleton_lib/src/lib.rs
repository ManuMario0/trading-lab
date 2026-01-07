use trading::prelude::*;

#[derive(Default)]
pub struct MyStrategy;

impl Strategist for MyStrategy {
    fn on_market_data(&mut self, _md: MarketDataBatch) -> AllocationBatch {
        // Simple example: do nothing
        AllocationBatch::new(vec![])
    }
}

impl Initiable for MyStrategy {
    fn init() -> Self {
        Self::default()
    }
}

trading::export_strategy!(MyStrategy);
