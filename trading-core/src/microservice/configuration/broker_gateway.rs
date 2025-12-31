use std::sync::{Arc, Mutex};

use crate::{
    define_service,
    framework::runner_manager::RunnerManager,
    manifest::{ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{execution::ExecutionResult, identity::Id, order::Order, portfolio::Actual},
};
use trading::Broker;

define_service!(
    name: broker_gateway,
    service_type: "BrokerGateway",
    inputs: {
        orders => fn on_order(Order) [ required: true, variadic: false ]
    },
    outputs: {
        execution_result => ExecutionResult,
        portfolio => Actual
    }
);

pub struct BrokerGateway<State> {
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> broker_gateway::Handler for State
where
    State: Broker + Send + 'static,
{
    fn on_order(&mut self, _id: Id, data: Order, outputs: &mut broker_gateway::Outputs) {
        let (executions, portfolio_update) = self.on_order(data);

        // Publish Execution Results
        if !executions.is_empty() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    for exec in executions {
                        let _ = outputs.execution_result.send(exec).await;
                    }
                });
            });
        }

        // Publish Portfolio
        if let Some(actual) = portfolio_update {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = outputs.portfolio.send(actual).await;
                });
            });
        }
    }
}

impl<State> BrokerGateway<State> {
    pub fn new() -> Self {
        Self {
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> Configurable for BrokerGateway<State>
where
    State: Broker + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        id: Id,
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        broker_gateway::create_runner_manager(id, config, state)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &broker_gateway::MANIFEST
    }
}
