use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::{
    args, comms,
    framework::runner_manager::RunnerManager,
    manifest::{Binding, PortDefinition, ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{
        execution::ExecutionResult, identity::Id, order::Order, portfolio::Actual,
        portfolio::Target,
    },
};
use trading::Executor;

lazy_static! {
    pub static ref EXECUTION_ENGINE_MANIFEST: ServiceBlueprint = ServiceBlueprint {
        service_type: "ExecutionEngine".to_string(),
        inputs: vec![
            PortDefinition {
                name: "target".to_string(),
                data_type: "Target".to_string(),
                required: true,
                is_variadic: false,
            },
            PortDefinition {
                name: "execution_result".to_string(),
                data_type: "ExecutionResult".to_string(),
                required: true,
                is_variadic: false,
            },
        ],
        outputs: vec![
            PortDefinition {
                name: "orders".to_string(),
                data_type: "Order".to_string(),
                required: true,
                is_variadic: false,
            },
            PortDefinition {
                name: "portfolio".to_string(),
                data_type: "Actual".to_string(),
                required: true,
                is_variadic: false,
            },
        ],
    };
}

pub struct ExecutionEngine<State> {
    manifest: ServiceBlueprint,
    _state_phantom: std::marker::PhantomData<State>,
}

impl<State> ExecutionEngine<State> {
    pub fn new() -> Self {
        Self {
            manifest: EXECUTION_ENGINE_MANIFEST.clone(),
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
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        create_execution_engine_runner_manager(config, state)
    }
}

fn create_execution_engine_runner_manager<State>(
    mut config: ServiceBindings,
    state: Arc<Mutex<State>>,
) -> Result<RunnerManager, String>
where
    State: Executor + Send + 'static,
{
    let mut runner_manager = RunnerManager::new();

    // Create publisher for Orders
    let order_publisher = if let Some(Binding::Single(source)) = config.outputs.get("orders") {
        Arc::new(
            comms::build_publisher::<Order>(
                &source.address,
                args::CommonArgs::new().get_service_id(),
            )
            .map_err(|e| format!("Failed to create order publisher: {}", e))?,
        )
    } else {
        return Err("Missing 'orders' binding".to_owned());
    };

    // Create publisher for Portfolio updates
    let portfolio_publisher = if let Some(Binding::Single(source)) = config.outputs.get("portfolio")
    {
        Arc::new(
            comms::build_publisher::<Actual>(
                &source.address,
                args::CommonArgs::new().get_service_id(),
            )
            .map_err(|e| format!("Failed to create portfolio publisher: {}", e))?,
        )
    } else {
        return Err("Missing 'portfolio' binding".to_owned());
    };

    // Runner for Target
    let op_clone = order_publisher.clone();
    let pp_clone = portfolio_publisher.clone();

    runner_manager.add_runner(
        "target",
        state.clone(),
        Box::new(move |state: &mut State, _config_id: Id, target: Target| {
            let (orders, portfolio_update) = state.on_target(target);

            // Publish Orders
            if !orders.is_empty() {
                let pub_clone = op_clone.clone();
                // We publish orders individually or as a batch?
                // The publisher is typed <Order>. So individually.
                // Ideally we should use OrderBatch but for now let's loop.
                // Or check if Order publisher expects OrderBatch.
                // Plan said "Output: orders". Usually better to batch.
                // But let's assume single Order for now as existing models don't have OrderBatch yet visible.
                // Actually `AllocationBatch` exists. `OrderBatch`? I didn't verify.
                // Let's assume loop for now.
                for order in orders {
                    let p = pub_clone.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = p.send(order).await;
                        });
                    });
                }
            }

            // Publish Portfolio
            if let Some(actual) = portfolio_update {
                let p = pp_clone.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = p.send(actual).await;
                    });
                });
            }
        }),
        None,
    );

    // Runner for ExecutionResult
    let op_clone_2 = order_publisher.clone();
    let pp_clone_2 = portfolio_publisher.clone();

    runner_manager.add_runner(
        "execution_result",
        state.clone(),
        Box::new(
            move |state: &mut State, _config_id: Id, execution: ExecutionResult| {
                let (orders, portfolio_update) = state.on_execution(execution);

                if !orders.is_empty() {
                    let pub_clone = op_clone_2.clone();
                    for order in orders {
                        let p = pub_clone.clone();
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                let _ = p.send(order).await;
                            });
                        });
                    }
                }

                if let Some(actual) = portfolio_update {
                    let p = pp_clone_2.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = p.send(actual).await;
                        });
                    });
                }
            },
        ),
        None,
    );

    // Apply bindings
    if let Some(binding) = config.inputs.remove("target") {
        runner_manager.update_from_binding("target", binding);
    }
    if let Some(binding) = config.inputs.remove("execution_result") {
        runner_manager.update_from_binding("execution_result", binding);
    }

    Ok(runner_manager)
}
