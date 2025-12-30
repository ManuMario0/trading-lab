use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use trading::{Broker, Executor, Manager, Multiplexist};
use trading_core::{
    args::CommonArgs,
    comms::address::Address,
    manifest::{Binding, PortDefinition, ServiceBindings},
    microservice::configuration::{
        broker_gateway::BrokerGateway, execution_engine::ExecutionEngine, multiplexer::Multiplexer,
        portfolio_manager::PortfolioManager, Configuration,
    },
    model::{
        allocation::{Allocation, Position},
        allocation_batch::AllocationBatch,
        execution::{ExecutionResult, ExecutionStatus},
        identity::Id,
        market_data::PriceUpdate,
        order::{Order, OrderSide, OrderType},
        portfolio::{Actual, Portfolio, Target},
    },
};

// --- 1. Mock Implementations ---

// A simple Multiplexist that just forwards the last allocation
struct SimpleMultiplexer;
impl Multiplexist for SimpleMultiplexer {
    fn on_allocation_batch(
        &mut self,
        _source_id: usize,
        batch: AllocationBatch,
    ) -> AllocationBatch {
        // Pass through
        batch
    }
}

// A simple Portfolio Manager that converts 10% weight -> 10 units
struct SimpleManager;
impl Manager for SimpleManager {
    fn on_allocation(&mut self, batch: AllocationBatch) -> Option<Target> {
        // Super simple logic: If we see Apple, buy 10 units.
        let mut p = Portfolio::new();
        for alloc in batch.iter() {
            for (inst_id, _) in alloc.get_positions() {
                p.update_position(*inst_id, 10.0);
            }
        }
        Some(Target(p))
    }
    fn on_portfolio(&mut self, _portfolio: Actual) -> Option<Target> {
        None
    }
    fn on_market_data(&mut self, _market_data: PriceUpdate) -> Option<Target> {
        None
    }
}

// A simple Execution Engine that diffs Target vs Actual (assumes 0 actual)
struct SimpleExecutor;
impl Executor for SimpleExecutor {
    fn on_target(&mut self, target: Target) -> (Vec<Order>, Option<Actual>) {
        let mut orders = Vec::new();
        // For every position in target, create a buy order (assuming we have 0)
        for (inst_id, pos) in &target.0.positions {
            orders.push(Order::new(
                format!("ord-{}", inst_id),
                format!("{}", inst_id),
                OrderSide::buy(),
                OrderType::market(),
                0.0,
                pos.get_quantity(),
                0, // timestamp
            ));
        }
        (orders, None)
    }
    fn on_execution(&mut self, _execution: ExecutionResult) -> (Vec<Order>, Option<Actual>) {
        (vec![], None)
    }
}

// A simple Broker that fills everything immediately
struct SimpleBroker;
impl Broker for SimpleBroker {
    fn on_order(&mut self, order: Order) -> (Vec<ExecutionResult>, Option<Actual>) {
        let mut results = Vec::new();
        let exec = ExecutionResult::new(
            order.get_id(),
            order.get_instrument_id(),
            ExecutionStatus::Filled,
            0,
        )
        .with_fill(order.get_quantity(), 100.0, order.get_quantity(), 100.0);

        results.push(exec);
        (results, None)
    }
}

// --- 2. Helper to create Bindings ---
fn make_binding(port: u16) -> Binding {
    Binding::Single(trading_core::manifest::Source {
        address: Address::Zmq(format!("tcp://127.0.0.1:{}", port)),
        id: 0,
    })
}

#[test]
fn test_pipeline_integration() {
    // Ports
    let port_alloc_input = 7001; // Strat -> Mux
    let port_alloc_output = 7002; // Mux -> PM
    let port_target_output = 7003; // PM -> Exec
    let port_orders_output = 7004; // Exec -> Broker
    let port_exec_result_output = 7005; // Broker -> Exec
    let port_portfolio_output_exec = 7006; // Exec -> PM
    let port_portfolio_output_broker = 7007; // Broker -> PM/Exec
                                             // For simplicity, we just verify the flow up to Broker receiving the order.
                                             // Full round trip requires more wiring.

    // 1. Multiplexer
    let mut mux_bindings = ServiceBindings::new();
    mux_bindings
        .inputs
        .insert("strategies".to_string(), make_binding(port_alloc_input));
    mux_bindings
        .outputs
        .insert("allocation".to_string(), make_binding(port_alloc_output));

    let mut mux_config = Configuration::new(Multiplexer::<SimpleMultiplexer>::new());
    mux_config.launch(Arc::new(Mutex::new(SimpleMultiplexer)), mux_bindings);

    // 2. Portfolio Manager
    let mut pm_bindings = ServiceBindings::new();
    pm_bindings
        .inputs
        .insert("allocation".to_string(), make_binding(port_alloc_output));
    // pm needs portfolio input, let's bind it to something dummy or the exec output
    pm_bindings.inputs.insert(
        "portfolio".to_string(),
        make_binding(port_portfolio_output_exec),
    );
    pm_bindings
        .inputs
        .insert("market_data".to_string(), make_binding(8008)); // Dummy MD
    pm_bindings
        .outputs
        .insert("target".to_string(), make_binding(port_target_output));

    let mut pm_config = Configuration::new(PortfolioManager::<SimpleManager>::new());
    pm_config.launch(Arc::new(Mutex::new(SimpleManager)), pm_bindings);

    // 3. Execution Engine
    let mut exec_bindings = ServiceBindings::new();
    exec_bindings
        .inputs
        .insert("target".to_string(), make_binding(port_target_output));
    exec_bindings.inputs.insert(
        "execution_result".to_string(),
        make_binding(port_exec_result_output),
    );
    exec_bindings
        .outputs
        .insert("orders".to_string(), make_binding(port_orders_output));
    exec_bindings.outputs.insert(
        "portfolio".to_string(),
        make_binding(port_portfolio_output_exec),
    );

    let mut exec_config = Configuration::new(ExecutionEngine::<SimpleExecutor>::new());
    exec_config.launch(Arc::new(Mutex::new(SimpleExecutor)), exec_bindings);

    // 4. Broker Gateway
    let mut broker_bindings = ServiceBindings::new();
    broker_bindings
        .inputs
        .insert("orders".to_string(), make_binding(port_orders_output));
    broker_bindings.outputs.insert(
        "execution_result".to_string(),
        make_binding(port_exec_result_output),
    );
    broker_bindings.outputs.insert(
        "portfolio".to_string(),
        make_binding(port_portfolio_output_broker),
    );

    let mut broker_config = Configuration::new(BrokerGateway::<SimpleBroker>::new());
    broker_config.launch(Arc::new(Mutex::new(SimpleBroker)), broker_bindings);

    // --- TEST INJECTION ---
    // Inject Allocation into Mux Input (simulate Strategy)
    // We use a raw publisher for this
    let strategy_pub = trading_core::comms::build_publisher::<AllocationBatch>(
        &Address::Zmq(format!("tcp://127.0.0.1:{}", port_alloc_input)),
        Id::from(999usize),
    )
    .unwrap();

    // Allow ZMQ to connect/bind
    thread::sleep(Duration::from_millis(2000));

    let mut alloc = Allocation::new();
    alloc.update_position(1, 0.5); // 50%
    let batch = AllocationBatch::new(vec![alloc]);

    println!("Sending Allocation...");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            strategy_pub.send(batch).await.unwrap();
        });

    // Verification: We need to see if Broker outputs an Execution Result
    // (Implies: Strat -> Mux -> PM -> Exec -> Broker -> Exec Result)
    // We can listen to `port_exec_result_output`.

    let mut result_sub = trading_core::comms::build_subscriber::<ExecutionResult>(&Address::Zmq(
        format!("tcp://127.0.0.1:{}", port_exec_result_output),
    ))
    .unwrap();

    println!("Waiting for Execution Result...");
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Give it some time
            tokio::select! {
                res = result_sub.recv() => Some(res),
                _ = tokio::time::sleep(Duration::from_secs(5)) => None,
            }
        });

    if let Some(Ok(packet)) = result {
        let exec_res = packet.data();
        println!("Received Execution: {:?}", exec_res);
        assert_eq!(exec_res.status, ExecutionStatus::Filled);
        assert_eq!(exec_res.last_filled_quantity, 10.0); // SimpleManager converted 0.5 -> 10.0
    } else {
        panic!("Timed out waiting for execution result!");
    }

    // Cleanup
    println!("Shutting down services...");
    mux_config.shutdown();
    pm_config.shutdown();
    exec_config.shutdown();
    broker_config.shutdown();
    println!("Services shut down.");
}
