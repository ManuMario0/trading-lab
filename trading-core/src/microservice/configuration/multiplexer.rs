use std::sync::{Arc, Mutex};

use crate::{
    define_service,
    framework::runner_manager::RunnerManager,
    manifest::{ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{allocation_batch::AllocationBatch, identity::Id},
};
use trading::Multiplexist;

define_service!(
    name: multiplexer,
    service_type: "Multiplexer",
    inputs: {
        strategies => fn on_allocation_batch(AllocationBatch) [ required: true, variadic: true ]
    },
    outputs: {
        allocation => AllocationBatch
    }
);

pub struct Multiplexer<State> {
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> multiplexer::Handler for State
where
    State: Multiplexist + Send + 'static,
{
    fn on_allocation_batch(
        &mut self,
        id: Id,
        data: AllocationBatch,
        outputs: &mut multiplexer::Outputs,
    ) {
        let output_batch = self.on_allocation_batch(id, data);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = outputs.allocation.send(output_batch).await;
            });
        });
    }
}

impl<State> Multiplexer<State> {
    pub fn new() -> Self {
        Self {
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> Configurable for Multiplexer<State>
where
    State: Multiplexist + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        id: Id,
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        multiplexer::create_runner_manager(id, config, state)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &multiplexer::MANIFEST
    }
}
