use std::sync::{Arc, Mutex};

use crate::{
    define_service,
    framework::runner_manager::RunnerManager,
    manifest::{ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{
        execution::ExecutionResult, identity::Id, order::Order, portfolio::Actual,
        portfolio::Target,
    },
};
use trading::Executor;

define_service!(
    name: execution_engine,
    service_type: "ExecutionEngine",
    inputs: {
        target => fn on_target(Target) [ required: true, variadic: false ]
        execution_result => fn on_execution(ExecutionResult) [ required: true, variadic: false ]
    },
    outputs: {
        orders => Order,
        portfolio => Actual
    }
);

pub struct ExecutionEngine<State> {
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> execution_engine::Handler for State
where
    State: Executor + Send + 'static,
{
    fn on_target(&mut self, _id: Id, data: Target, outputs: &mut execution_engine::Outputs) {
        let (orders, portfolio_update) = self.on_target(data);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for order in orders {
                    let _ = outputs.orders.send(order).await;
                }
                if let Some(actual) = portfolio_update {
                    let _ = outputs.portfolio.send(actual).await;
                }
            });
        });
    }

    fn on_execution(
        &mut self,
        _id: Id,
        data: ExecutionResult,
        outputs: &mut execution_engine::Outputs,
    ) {
        let (orders, portfolio_update) = self.on_execution(data);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for order in orders {
                    let _ = outputs.orders.send(order).await;
                }
                if let Some(actual) = portfolio_update {
                    let _ = outputs.portfolio.send(actual).await;
                }
            });
        });
    }
}

impl<State> ExecutionEngine<State> {
    pub fn new() -> Self {
        Self {
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> Configurable for ExecutionEngine<State>
where
    State: Executor + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        id: Id,
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        execution_engine::create_runner_manager(id, config, state)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &execution_engine::MANIFEST
    }
}
