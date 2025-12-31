use std::sync::{Arc, Mutex};

use crate::define_service;
use crate::manifest::{ServiceBindings, ServiceBlueprint};
use crate::microservice::configuration::Configurable;
use crate::model::identity::Id;
use crate::{
    framework::runner_manager::RunnerManager,
    model::{allocation_batch::AllocationBatch, market_data::MarketDataBatch},
};
use trading::Strategist; // External trait

define_service!(
    name: strategy,
    service_type: "Strategy",
    inputs: {
        market_data => fn on_market_data(MarketDataBatch) [ required: true, variadic: false ]
    },
    outputs: {
        allocation => AllocationBatch
    }
);

pub struct Strategy<State> {
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> Strategy<State> {
    pub fn new() -> Self {
        Self {
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> strategy::Handler for State
where
    State: Strategist + Send + 'static,
{
    fn on_market_data(&mut self, _id: Id, data: MarketDataBatch, outputs: &mut strategy::Outputs) {
        let allocation_batch = self.on_market_data(data);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = outputs.allocation.send(allocation_batch).await;
            });
        });
    }
}

impl<State> Configurable for Strategy<State>
where
    State: Strategist + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        id: Id,
        bindings: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        strategy::create_runner_manager(id, bindings, state)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &strategy::MANIFEST
    }
}
