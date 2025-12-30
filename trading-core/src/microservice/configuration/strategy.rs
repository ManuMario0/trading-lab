use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::manifest::{Binding, PortDefinition, ServiceBindings, ServiceBlueprint};
use crate::microservice::configuration::Configurable;
use crate::model::identity::Id;
use crate::{args, comms};
use crate::{
    framework::runner_manager::RunnerManager,
    model::{allocation_batch::AllocationBatch, market_data::MarketDataBatch},
};

lazy_static! {
    pub(crate) static ref STRATEGY_MANIFEST: ServiceBlueprint = ServiceBlueprint {
        service_type: "Strategy".to_string(),
        inputs: vec![PortDefinition {
            name: "market_data".to_string(),
            data_type: "MarketDataBatch".to_string(),
            required: true,
            is_variadic: false,
        }],
        outputs: vec![PortDefinition {
            name: "allocation".to_string(),
            data_type: "AllocationBatch".to_string(),
            required: true,
            is_variadic: false,
        }],
    };
}

pub struct Strategy<State> {
    manifest: ServiceBlueprint,
    _state_phantom: std::marker::PhantomData<State>,
}

pub trait Strategist<State> {
    fn on_market_data(&mut self, market_data: MarketDataBatch) -> AllocationBatch;

    /// Creates a new Strategy.
    ///
    /// The returned Strategy can be used to create a new Configuration instance.
    fn make_strategy() -> Strategy<State> {
        Strategy::new()
    }
}

impl<State> Strategy<State> {
    pub fn new() -> Self {
        Self {
            manifest: STRATEGY_MANIFEST.clone(),
            _state_phantom: std::marker::PhantomData,
        }
    }
}

impl<State> Configurable for Strategy<State>
where
    State: Strategist<State> + Send + 'static,
{
    type State = State;

    fn create_runners(
        &self,
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        create_strategy_runner_manager(config, state)
    }
}

pub(super) fn create_strategy_runner_manager<State>(
    config: ServiceBindings,
    state: Arc<Mutex<State>>,
) -> Result<RunnerManager, String>
where
    State: Strategist<State> + Send + 'static,
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

    // Create the market data runner
    if let Some(Binding::Single(source)) = config.inputs.get("market_data") {
        runner_manager.add_runner(
            "market_data",
            state,
            Box::new(
                move |state: &mut State, _config_id: Id, data: MarketDataBatch| {
                    let allocation_batch = state.on_market_data(data);
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = publisher.send(allocation_batch).await;
                        });
                    });
                },
            ),
            Some(source.address.clone()),
        );
    } else {
        return Err("Missing 'market_data' binding".to_owned());
    }
    Ok(runner_manager)
}
