use std::sync::{Arc, Mutex};

use crate::{
    define_service,
    framework::runner_manager::RunnerManager,
    manifest::{ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{
        allocation_batch::AllocationBatch,
        identity::Id,
        market_data::PriceUpdate,
        portfolio::{Actual, Target},
    },
};
use trading::Manager;

define_service!(
    name: portfolio_manager,
    service_type: "PortfolioManager",
    inputs: {
        allocation => fn on_allocation(AllocationBatch) [ required: true, variadic: false ]
        portfolio => fn on_portfolio(Actual) [ required: true, variadic: false ]
        market_data => fn on_market_data(PriceUpdate) [ required: false, variadic: false ]
    },
    outputs: {
        target => Target
    }
);

pub struct PortfolioManager<State> {
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> portfolio_manager::Handler for State
where
    State: Manager + Send + 'static,
{
    fn on_allocation(
        &mut self,
        _id: Id,
        data: AllocationBatch,
        outputs: &mut portfolio_manager::Outputs,
    ) {
        if let Some(target) = self.on_allocation(data) {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = outputs.target.send(target).await;
                });
            });
        }
    }

    fn on_portfolio(&mut self, _id: Id, data: Actual, outputs: &mut portfolio_manager::Outputs) {
        if let Some(target) = self.on_portfolio(data) {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = outputs.target.send(target).await;
                });
            });
        }
    }

    fn on_market_data(
        &mut self,
        _id: Id,
        data: PriceUpdate,
        outputs: &mut portfolio_manager::Outputs,
    ) {
        if let Some(target) = self.on_market_data(data) {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = outputs.target.send(target).await;
                });
            });
        }
    }
}

impl<State> PortfolioManager<State> {
    pub fn new() -> Self {
        Self {
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> Configurable for PortfolioManager<State>
where
    State: Manager + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        id: Id,
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        portfolio_manager::create_runner_manager(id, config, state)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &portfolio_manager::MANIFEST
    }
}
