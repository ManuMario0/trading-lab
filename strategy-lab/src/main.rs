use anyhow::Result;
use clap::Parser;
use log::info;
use trading_core::{
    args::CommonArgs,
    microservice::{configuration::Configuration, Microservice},
    model::{allocation::Allocation, market_data::MarketDataBatch},
};

struct DummyStrategy {
    allocation: Allocation,
    allocation_amount: f64,
}

impl DummyStrategy {
    pub fn new(allocation_amount: f64) -> Self {
        Self {
            allocation: Allocation::new(trading_core::model::identity::Identity::new(
                "dummy_strategy",
                "1.0",
            )),
            allocation_amount,
        }
    }

    pub fn on_market_data(&mut self, batch: &MarketDataBatch) -> Allocation {
        // info!("Received batch with {} updates", batch.get_count());

        // Simple dummy logic: always allocate fixed amount to instrument 1 if present
        let mut allocation = self.allocation.clone();

        // Just a toy example: if we see an update, buy 1 unit of instrument 1
        if batch.get_count() > 0 {
            allocation.update_position(1, self.allocation_amount);
        }

        allocation
    }
}

fn main() -> Result<()> {
    env_logger::init();

    // 1. Parse Args
    // We use CommonArgs derive parser.
    // Note: In a real app we might combine this with local args,
    // but for now we just use the common ones.
    let args = CommonArgs::parse();

    info!("Starting Strategy Lab (Rust): {}", args.get_service_name());

    // 2. Define State Closure
    let initial_state = || DummyStrategy::new(10.0);

    // 3. Define Configuration (Strategy)
    // The callback receives &mut State and &MarketDataBatch, returns Allocation
    let config = Configuration::new_strategy(Box::new(
        |strategy: &mut DummyStrategy, batch: &MarketDataBatch| strategy.on_market_data(batch),
    ));

    // 4. Create and Run Microservice
    let service = Microservice::new(initial_state, config);

    service.run();

    Ok(())
}
