use anyhow::Result;
use broker_gateway::paper::PaperBroker;
use trading_core::microservice::{
    configuration::{broker_gateway::BrokerGateway as ServiceWrapper, Configuration},
    Microservice,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Initial State Factory
    let initial_state = |_: &_| {
        // Initialize Paper Broker with $1M USD
        PaperBroker::new(1_000_000.0)
    };

    // 2. Configuration Wrapper
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
