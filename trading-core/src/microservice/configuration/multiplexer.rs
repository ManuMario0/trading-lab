use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::{
    args, comms,
    framework::runner_manager::RunnerManager,
    manifest::{Binding, PortDefinition, ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{allocation_batch::AllocationBatch, identity::Id},
};
use trading::Multiplexist;

lazy_static! {
    pub(crate) static ref MULTIPLEXER_MANIFEST: ServiceBlueprint = ServiceBlueprint {
        service_type: "Strategy".to_string(),
        inputs: vec![PortDefinition {
            name: "strategies".to_string(),
            data_type: "AllocationBatch".to_string(),
            required: true,
            is_variadic: true,
        }],
        outputs: vec![PortDefinition {
            name: "allocation".to_string(),
            data_type: "AllocationBatch".to_string(),
            required: true,
            is_variadic: false,
        }],
    };
}

pub struct Multiplexer<State> {
    manifest: ServiceBlueprint,
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> Multiplexer<State> {
    pub fn new() -> Self {
        Self {
            manifest: MULTIPLEXER_MANIFEST.clone(),
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
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        create_multiplexer_runner_manager(config, state)
    }
}

fn create_multiplexer_runner_manager<State>(
    mut config: ServiceBindings,
    state: Arc<Mutex<State>>,
) -> Result<RunnerManager, String>
where
    State: Multiplexist + Send + 'static,
{
    let mut runner_manager = RunnerManager::new();

    // Create publisher
    let publisher = if let Some(Binding::Single(source)) = config.outputs.get("allocation") {
        comms::build_publisher::<AllocationBatch>(
            &source.address,
            args::CommonArgs::new().get_service_id(),
        )
        .map_err(|e| format!("Failed to create publisher: {}", e))?
    } else {
        return Err("Missing 'allocation' binding".to_owned());
    };

    runner_manager.add_runner(
        "strategies",
        state.clone(),
        Box::new(
            move |state: &mut State, config_id: Id, batch: AllocationBatch| {
                let output_batch = state.on_allocation_batch(config_id, batch);
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = publisher.send(output_batch).await;
                    });
                });
            },
        ),
        None,
    );

    if let Some(binding) = config.inputs.remove("strategies") {
        runner_manager.update_from_binding("strategies", binding);
    }

    Ok(runner_manager)
}
