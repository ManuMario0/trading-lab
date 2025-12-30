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

use crate::{framework::runner_manager::RunnerManager, manifest::ServiceBindings};

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

    pub(crate) fn launch(
        &mut self,
        state: Arc<Mutex<Config::State>>,
        initial_bindings: ServiceBindings,
    ) {
        self.runners = Some(
            self.config
                .create_runners(initial_bindings, state)
                .expect("Failed to create runners"),
        );
    }

    pub(crate) fn update_from_service_config(&mut self, config: ServiceBindings) {
        for (name, binding) in config.inputs {
            self.runners
                .as_mut()
                .map(|runners| runners.update_from_binding(&name, binding));
        }
    }

    pub(crate) fn shutdown(&mut self) {
        if let Some(runners) = self.runners.take() {
            runners.shutdown();
        }
    }
}

pub trait Configurable {
    type State;

    fn create_runners(
        &self,
        config: ServiceBindings,
        state: Arc<Mutex<Self::State>>,
    ) -> Result<RunnerManager, String>;
}
