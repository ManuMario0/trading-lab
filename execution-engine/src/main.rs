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
    let initial_state = |_: &_| Engine::new();

    // 2. Wrap Config
    let config = Configuration::new(ServiceWrapper::new());

    // 3. Run Microservice
    let service = Microservice::new(
        initial_state,
        config,
        env!("CARGO_PKG_VERSION").to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );
    service.run().await;

    Ok(())
}
