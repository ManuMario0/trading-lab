use anyhow::Result;
use clap::Parser;
use log::info;
use trading::{
    model::allocation::Allocation, model::allocation_batch::AllocationBatch,
    model::market_data::MarketDataBatch, Strategist,
};
use trading_core::{
    args::CommonArgs,
    microservice::{
        configuration::{strategy::Strategy, Configuration},
        Microservice,
    },
};

struct DummyStrategy {
    allocation: Allocation,
    allocation_amount: f64,
}

impl Strategist for DummyStrategy {
    fn on_market_data(&mut self, batch: MarketDataBatch) -> Option<Allocation> {
        // info!("Received batch with {} updates", batch.get_count());

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
            Some(allocation)
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Parse Args
    // We use CommonArgs derive parser.
    // Note: In a real app we might combine this with local args,
    // but for now we just use the common ones.
    let args = CommonArgs::parse();

    info!("Starting Strategy Lab (Rust): {}", args.get_service_name());

    // 2. Define State Closure
    let initial_state = || DummyStrategy {
        allocation: Allocation::new(),
        allocation_amount: 1_000_000.0,
    };

    // 2. Define Configuration (Strategy)
    let config = Configuration::new(Strategy::new());

    // 4. Create and Run Microservice
    let service = Microservice::new(initial_state, config);

    service.run().await;

    Ok(())
}
