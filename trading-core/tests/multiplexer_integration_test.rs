use clap::Parser;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use trading::Multiplexist;
use trading_core::admin::command::{AdminCommand, AdminPayload};
use trading_core::args::CommonArgs;
use trading_core::comms::{build_publisher, build_subscriber, Address, Packet};
use trading_core::manifest::{Binding, ServiceBindings, Source};
use trading_core::microservice::configuration::multiplexer::Multiplexer;
use trading_core::microservice::configuration::Configuration;
use trading_core::microservice::Microservice;
use trading_core::model::{
    allocation::Allocation, allocation_batch::AllocationBatch, identity::Identity,
};

// Mock State
#[derive(Default)]
struct TestState;

impl Multiplexist for TestState {
    fn on_allocation_batch(&mut self, source_id: usize, batch: AllocationBatch) -> AllocationBatch {
        println!("Received allocation batch from source_id: {}", source_id);
        batch // Echo back
    }
}

#[test]
fn test_multiplexer_dynamic_add_strategy() {
    // 1. Setup Addresses
    // Use high ports to avoid conflicts
    let admin_port = 62001;
    let strategy_1_port = 62002;
    let strategy_2_port = 62003;
    let execution_port = 62004;

    let admin_addr_str = format!("tcp://127.0.0.1:{}", admin_port);
    let admin_addr = Address::zmq_tcp("127.0.0.1", admin_port);
    let strategy_1_addr = Address::zmq_tcp("127.0.0.1", strategy_1_port);
    let strategy_2_addr = Address::zmq_tcp("127.0.0.1", strategy_2_port);
    let execution_addr = Address::zmq_tcp("127.0.0.1", execution_port);

    // 2. Setup Dummy Publishers (Strategies)
    let id1 = Identity::new("strategy_1", "1.0", 1);
    let publisher1 =
        build_publisher::<AllocationBatch>(&strategy_1_addr, id1.get_identifier()).unwrap();
    let id2 = Identity::new("strategy_2", "1.0", 2);
    let _publisher2 =
        build_publisher::<AllocationBatch>(&strategy_2_addr, id2.get_identifier()).unwrap();

    // 3. Setup Dummy Subscriber (Execution Engine)
    // The subscriber connects to the multiplexer's output
    // Multiplexer binds to execution_port
    // Subscriber connects to execution_addr

    let execution_addr_clone = execution_addr.clone();
    // 4. Launch Multiplexer in a thread
    let handle = thread::spawn(move || {
        // Construct Bindings
        let mut inputs = HashMap::new();
        inputs.insert(
            "admin".to_string(),
            Binding::Single(Source {
                address: admin_addr,
                id: 0,
            }),
        );
        // Initial dummy connection for allocation input
        // Initial empty variadic connection
        inputs.insert("strategies".to_string(), Binding::Variadic(HashMap::new()));

        let mut outputs = HashMap::new();
        outputs.insert(
            "allocation".to_string(),
            Binding::Single(Source {
                address: execution_addr_clone,
                id: 0,
            }),
        );

        let bindings = ServiceBindings { inputs, outputs };
        let bindings_json = serde_json::to_string(&bindings).unwrap();

        // Mock Args: Use parse_from directly with --bindings
        let args = CommonArgs::parse_from(vec![
            "multiplexer_test",
            "--service-name",
            "multiplexer",
            "--service-id",
            "1",
            "--config-dir",
            "/tmp/trading_test_config",
            "--data-dir",
            "/tmp/trading_test_data",
            "--bindings",
            &bindings_json,
        ]);

        let config = Configuration::new(Multiplexer::new());
        let service = Microservice::new_with_args(args, |_: &_| TestState, config);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(service.run());
    });

    thread::sleep(Duration::from_millis(500)); // Wait for startup

    // 5. Admin Client
    let ctx = zmq::Context::new();
    let admin_sock = ctx.socket(zmq::REQ).unwrap();
    admin_sock.connect(&admin_addr_str).unwrap();
    println!("Test: Admin client connected to {}", admin_addr_str);

    // 6. Test Loop
    let mut test_subscriber = build_subscriber::<AllocationBatch>(&execution_addr).unwrap();

    // Command 1: Add Strategy 1 (Update Bindings)
    let mut new_inputs = HashMap::new();
    let mut strategies_map = HashMap::new();
    strategies_map.insert(
        "strategy_1".to_string(),
        Source {
            address: Address::zmq_tcp("127.0.0.1", strategy_1_port),
            id: 1,
        },
    );
    new_inputs.insert("strategies".to_string(), Binding::Variadic(strategies_map));

    let update_config = ServiceBindings {
        inputs: new_inputs,
        outputs: HashMap::new(),
    };

    let cmd = AdminCommand::UpdateBindings {
        config: update_config,
    };
    let payload = AdminPayload::Command(cmd);
    let packet = Packet::new(0, payload); // ID 0 for admin
    let msg = bincode::serialize(&packet).unwrap();
    admin_sock.send(&msg, 0).unwrap();
    let resp_bytes = admin_sock.recv_bytes(0).unwrap();
    let resp: AdminPayload = bincode::deserialize(&resp_bytes).unwrap();
    match resp {
        AdminPayload::Response(trading_core::admin::command::AdminResponse::Ok) => {}
        _ => panic!("Expected AdminResponse::Ok, got {:?}", resp),
    }

    thread::sleep(Duration::from_millis(500)); // Allow connect

    // Publish data from Strategy 1
    let alloc1 = Allocation::default();
    let batch1 = AllocationBatch::new(vec![alloc1]);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        publisher1.send(batch1).await.unwrap();
    });

    // Receive
    let _received = rt.block_on(async {
        // try_recv or recv
        test_subscriber.recv().await // Should wait
    });
    // If success, we got it.

    // Command 2: Shutdown
    let cmd = AdminCommand::Shutdown;
    let payload = AdminPayload::new_command(cmd);
    let msg = bincode::serialize(&payload).unwrap();
    admin_sock.send(&msg, 0).unwrap();

    handle.join().unwrap();
}
