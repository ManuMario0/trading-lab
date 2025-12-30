use crate::microservice::configuration::{
    broker_gateway::BrokerGateway, execution_engine::ExecutionEngine, multiplexer::Multiplexer,
    portfolio_manager::PortfolioManager, strategy::Strategy, Configuration,
};
use crate::microservice::Microservice;
use trading::prelude::*;
use trading::traits::{
    broker::Broker, executor::Executor, manager::Manager, multiplexist::Multiplexist,
};

fn run_service<State, Config>(state: State, config: Configuration<Config>)
where
    State: Send + 'static,
    Config: crate::microservice::configuration::Configurable<State = State> + 'static,
{
    let _ = env_logger::try_init();
    let microservice = Microservice::new(move || state, config);
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        microservice.run().await;
    });
}

pub fn boot_strategy(state: Box<dyn Strategist>) {
    run_service(state, Configuration::new(Strategy::new()));
}

pub fn boot_multiplexer(state: Box<dyn Multiplexist>) {
    run_service(state, Configuration::new(Multiplexer::new()));
}

pub fn boot_execution_engine(state: Box<dyn Executor>) {
    run_service(state, Configuration::new(ExecutionEngine::new()));
}

pub fn boot_portfolio_manager(state: Box<dyn Manager>) {
    run_service(state, Configuration::new(PortfolioManager::new()));
}

pub fn boot_broker_gateway(state: Box<dyn Broker>) {
    run_service(state, Configuration::new(BrokerGateway::new()));
}

// Deprecated alias for backward compatibility during refactor
pub fn boot(state: Box<dyn Strategist>) {
    boot_strategy(state);
}
