use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::{
    args, comms,
    framework::runner_manager::RunnerManager,
    manifest::{Binding, PortDefinition, ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{
        allocation_batch::AllocationBatch, identity::Id, market_data::PriceUpdate,
        portfolio::Actual, portfolio::Target,
    },
};
use trading::Manager;

lazy_static! {
    pub(crate) static ref PORTFOLIO_MANAGER_MANIFEST: ServiceBlueprint = ServiceBlueprint {
        service_type: "PortfolioManager".to_string(),
        inputs: vec![
            PortDefinition {
                name: "allocation".to_string(),
                data_type: "AllocationBatch".to_string(),
                required: true,
                is_variadic: false,
            },
            PortDefinition {
                name: "portfolio".to_string(),
                data_type: "Actual".to_string(),
                required: true,
                is_variadic: false,
            },
            PortDefinition {
                name: "market_data".to_string(),
                data_type: "PriceUpdate".to_string(),
                required: false,
                is_variadic: false,
            },
        ],
        outputs: vec![PortDefinition {
            name: "target".to_string(),
            data_type: "Target".to_string(),
            required: true,
            is_variadic: false,
        }],
    };
}

pub struct PortfolioManager<State> {
    manifest: ServiceBlueprint,
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> PortfolioManager<State> {
    pub fn new() -> Self {
        Self {
            manifest: PORTFOLIO_MANAGER_MANIFEST.clone(),
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
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        create_portfolio_manager_runner_manager(config, state)
    }
}

fn create_portfolio_manager_runner_manager<State>(
    mut config: ServiceBindings,
    state: Arc<Mutex<State>>,
) -> Result<RunnerManager, String>
where
    State: Manager + Send + 'static,
{
    let mut runner_manager = RunnerManager::new();

    // Create publisher for Target Portfolio
    let publisher = if let Some(Binding::Single(source)) = config.outputs.get("target") {
        Arc::new(
            comms::build_publisher::<Target>(
                &source.address,
                args::CommonArgs::new().get_service_id(),
            )
            .map_err(|e| format!("Failed to create publisher: {}", e))?,
        )
    } else {
        return Err("Missing 'target' binding".to_owned());
    };

    // Runner for Allocations
    let publisher_alloc = publisher.clone();
    runner_manager.add_runner(
        "allocation",
        state.clone(),
        Box::new(
            move |state: &mut State, _config_id: Id, batch: AllocationBatch| {
                if let Some(target) = state.on_allocation(batch) {
                    let pub_clone = publisher_alloc.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = pub_clone.send(target).await;
                        });
                    });
                }
            },
        ),
        None,
    );

    // Runner for Portfolio Updates
    let publisher_portfolio = publisher.clone();
    runner_manager.add_runner(
        "portfolio",
        state.clone(),
        Box::new(
            move |state: &mut State, _config_id: Id, portfolio: Actual| {
                if let Some(target) = state.on_portfolio(portfolio) {
                    let pub_clone = publisher_portfolio.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = pub_clone.send(target).await;
                        });
                    });
                }
            },
        ),
        None,
    );

    // Runner for Market Data
    let publisher_market_data = publisher.clone();
    runner_manager.add_runner(
        "market_data",
        state.clone(),
        Box::new(
            move |state: &mut State, _config_id: Id, quote: PriceUpdate| {
                if let Some(target) = state.on_market_data(quote) {
                    let pub_clone = publisher_market_data.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = pub_clone.send(target).await;
                        });
                    });
                }
            },
        ),
        None,
    );

    // Apply bindings
    if let Some(binding) = config.inputs.remove("allocation") {
        runner_manager.update_from_binding("allocation", binding);
    }
    if let Some(binding) = config.inputs.remove("portfolio") {
        runner_manager.update_from_binding("portfolio", binding);
    }
    if let Some(binding) = config.inputs.remove("market_data") {
        runner_manager.update_from_binding("market_data", binding);
    }

    Ok(runner_manager)
}
