//! Save all the default network configurations for microservices here.
//!
//! This includes:
//! - Strategy
//! - Multiplexer
//! - Portfolio Manager
//! - Execution Engine
//! - Broker Gateway

pub mod broker_gateway;
pub mod execution_engine;
pub mod multiplexer;
pub mod portfolio_manager;
pub mod strategy;

use std::sync::{Arc, Mutex};

use trading::Id;

use crate::{
    framework::runner_manager::RunnerManager,
    manifest::{ServiceBindings, ServiceBlueprint},
};

pub struct Configuration<Config> {
    config: Config,
    runners: Option<RunnerManager>,
}

impl<Config> Configuration<Config>
where
    Config: Configurable,
{
    pub fn new(config: Config) -> Self {
        Self {
            config,
            runners: None,
        }
    }

    pub fn launch(&mut self, id: Id, bindings: ServiceBindings, state: Arc<Mutex<Config::State>>) {
        self.runners = Some(
            self.config
                .create_runners(id, bindings, state)
                .expect("Failed to create runners"),
        );
    }

    pub fn update_from_service_config(&mut self, config: ServiceBindings) {
        for (name, binding) in config.inputs {
            self.runners
                .as_mut()
                .map(|runners| runners.update_from_binding(&name, binding));
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(runners) = self.runners.take() {
            runners.shutdown();
        }
    }

    pub fn manifest(&self) -> &ServiceBlueprint {
        self.config.manifest()
    }
}

pub trait Configurable {
    type State;

    fn create_runners(
        &self,
        id: Id,
        bindings: ServiceBindings,
        state: Arc<Mutex<Self::State>>,
    ) -> Result<RunnerManager, String>;

    fn manifest(&self) -> &ServiceBlueprint;
}
