use anyhow::Result;
use execution_engine::engine::Engine;
use trading_core::microservice::{
    configuration::{execution_engine::ExecutionEngine as ServiceWrapper, Configuration},
    Microservice,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Initial State Factory
    let initial_state = || Engine::new();

    // 2. Wrap Config
    let config = Configuration::new(ServiceWrapper::new());

    // 3. Run Microservice
    let service = Microservice::new(initial_state, config);
    service.run().await;

    Ok(())
}
