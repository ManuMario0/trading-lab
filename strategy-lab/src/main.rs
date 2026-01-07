use anyhow::Result;
use trading::{
    model::{allocation_batch::AllocationBatch, market_data::MarketDataBatch},
    Allocation, Strategist,
};
use trading_core::microservice::{
    configuration::{strategy::Strategy, Configuration},
    Microservice,
};

struct DummyStrategy {
    allocation: Allocation,
    allocation_amount: f64,
}

use log::info;

impl Strategist for DummyStrategy {
    fn on_market_data(&mut self, batch: MarketDataBatch) -> AllocationBatch {
        info!("Received batch with {} updates", batch.get_count());

        // Simple dummy logic: always allocate fixed amount to instrument 1 if present

        // Since we want to output a batch of decisions corresponding to the input,
        // we should conceptually iterate.
        // For this dummy strategy, we just generate ONE decision based on the latest update
        // (This simulates a "live" runner that only cares about the last state,
        // OR we can generate N decisions if we want to be "pure").

        // Let's implement the "Pure" batch logic: 1 Input -> 1 Output.
        // But for simplicity in this dummy, let's just create one allocation for the batch.

        let count = batch.get_count();
        if count > 0 {
            let mut allocation = self.allocation.clone();
            allocation.update_position(1, self.allocation_amount);
            AllocationBatch::new(vec![allocation])
        } else {
            AllocationBatch::new(vec![])
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 2. Define State Closure
    let initial_state = |_: &_| DummyStrategy {
        allocation: Allocation::new(),
        allocation_amount: 1_000_000.0,
    };

    // 2. Define Configuration (Strategy)
    let config = Configuration::new(Strategy::new());

    // 4. Create and Run Microservice
    let service = Microservice::new(
        initial_state,
        config,
        env!("CARGO_PKG_VERSION").to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );

    service.run().await;

    Ok(())
}
