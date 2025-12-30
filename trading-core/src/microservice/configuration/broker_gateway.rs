use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::{
    args, comms,
    framework::runner_manager::RunnerManager,
    manifest::{Binding, PortDefinition, ServiceBindings, ServiceBlueprint},
    microservice::configuration::Configurable,
    model::{execution::ExecutionResult, identity::Id, order::Order, portfolio::Actual},
};

lazy_static! {
    pub(crate) static ref BROKER_GATEWAY_MANIFEST: ServiceBlueprint = ServiceBlueprint {
        service_type: "BrokerGateway".to_string(),
        inputs: vec![PortDefinition {
            name: "orders".to_string(),
            data_type: "Order".to_string(),
            required: true,
            is_variadic: false,
        },],
        outputs: vec![
            PortDefinition {
                name: "execution_result".to_string(),
                data_type: "ExecutionResult".to_string(),
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

pub struct BrokerGateway<State> {
    manifest: ServiceBlueprint,
    _state_phantom: std::marker::PhantomData<State>,
}

/// The trait that users must implement to define their Broker Gateway logic.
/// This adapts the internal order format to the external broker API.
pub trait Broker {
    /// Called when an order is received from the Execution Engine.
    /// Should process the order (send to broker) and return any immediate updates (e.g. Pending/Rejected status)
    /// or portfolio updates (e.g. margin usage).
    fn on_order(&mut self, order: Order) -> (Vec<ExecutionResult>, Option<Actual>);
}

impl<State> BrokerGateway<State> {
    pub fn new() -> Self {
        Self {
            manifest: BROKER_GATEWAY_MANIFEST.clone(),
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
        config: ServiceBindings,
        state: Arc<Mutex<State>>,
    ) -> Result<RunnerManager, String> {
        create_broker_gateway_runner_manager(config, state)
    }
}

fn create_broker_gateway_runner_manager<State>(
    mut config: ServiceBindings,
    state: Arc<Mutex<State>>,
) -> Result<RunnerManager, String>
where
    State: Broker + Send + 'static,
{
    let mut runner_manager = RunnerManager::new();

    // Create publisher for Execution Results
    let exec_publisher =
        if let Some(Binding::Single(source)) = config.outputs.get("execution_result") {
            Arc::new(
                comms::build_publisher::<ExecutionResult>(
                    &source.address,
                    args::CommonArgs::new().get_service_id(),
                )
                .map_err(|e| format!("Failed to create execution_result publisher: {}", e))?,
            )
        } else {
            return Err("Missing 'execution_result' binding".to_owned());
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

    // Runner for Orders
    let ep_clone = exec_publisher.clone();
    let pp_clone = portfolio_publisher.clone();

    runner_manager.add_runner(
        "orders",
        state.clone(),
        Box::new(move |state: &mut State, _config_id: Id, order: Order| {
            let (executions, portfolio_update) = state.on_order(order);

            // Publish Execution Results
            if !executions.is_empty() {
                let pub_clone = ep_clone.clone();
                for exec in executions {
                    let p = pub_clone.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let _ = p.send(exec).await;
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

    // Apply bindings
    if let Some(binding) = config.inputs.remove("orders") {
        runner_manager.update_from_binding("orders", binding);
    }

    Ok(runner_manager)
}
