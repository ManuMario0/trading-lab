use crate::microservice::configuration::{
    broker_gateway::BrokerGateway, execution_engine::ExecutionEngine, feeder::Feeder,
    multiplexer::Multiplexer, portfolio_manager::PortfolioManager, strategy::Strategy,
    Configuration,
};
use crate::microservice::Microservice;
use trading::prelude::*;
use trading::traits::{
    broker::Broker, data_feed::DataFeed, executor::Executor, manager::Manager,
    multiplexist::Multiplexist,
};

fn run_service<State, Config>(
    state: State,
    config: Configuration<Config>,
    version: String,
    description: String,
) where
    State: Send + 'static,
    Config: crate::microservice::configuration::Configurable<State = State> + 'static,
{
    let _ = env_logger::try_init();
    let microservice = Microservice::new(move |_| state, config, version, description);
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        microservice.run().await;
    });
}

pub fn boot_strategy(state: Box<dyn Strategist>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(Strategy::new()),
        version.to_string(),
        description.to_string(),
    );
}

pub fn boot_multiplexer(state: Box<dyn Multiplexist>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(Multiplexer::new()),
        version.to_string(),
        description.to_string(),
    );
}

pub fn boot_execution_engine(state: Box<dyn Executor>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(ExecutionEngine::new()),
        version.to_string(),
        description.to_string(),
    );
}

pub fn boot_portfolio_manager(state: Box<dyn Manager>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(PortfolioManager::new()),
        version.to_string(),
        description.to_string(),
    );
}

pub fn boot_broker_gateway(state: Box<dyn Broker>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(BrokerGateway::new()),
        version.to_string(),
        description.to_string(),
    );
}

pub fn boot_feeder(state: Box<dyn DataFeed + Send>, version: &str, description: &str) {
    run_service(
        state,
        Configuration::new(Feeder::new()),
        version.to_string(),
        description.to_string(),
    );
}

// Deprecated alias for backward compatibility during refactor
pub fn boot(state: Box<dyn Strategist>) {
    boot_strategy(state, "0.1.0", "Auto-booted Strategy");
}
