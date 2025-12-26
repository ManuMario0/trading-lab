use clap::Parser;
use std::thread;
use std::time::Duration;
use trading_core::admin::command::{AdminCommand, AdminPayload};
use trading_core::args::CommonArgs;
use trading_core::comms::{build_publisher, build_subscriber, Address};
use trading_core::microservice::configuration::Configuration;
use trading_core::microservice::Microservice;
use trading_core::model::allocation::Allocation;

// Mock State
#[derive(Default)]
struct TestState;

#[test]
fn test_multiplexer_dynamic_add_strategy() {
    // 1. Setup Addresses
    // Use high ports to avoid conflicts
    // Use high ports to avoid conflicts
    let admin_port = 61001;
    let strategy_1_port = 61002;
    let strategy_2_port = 61003;
    let execution_port = 61004;

    let admin_addr_str = format!("tcp://127.0.0.1:{}", admin_port);
    let strategy_1_addr = Address::zmq_tcp("127.0.0.1", strategy_1_port);
    let strategy_2_addr = Address::zmq_tcp("127.0.0.1", strategy_2_port);
    let execution_addr_str = format!("tcp://127.0.0.1:{}", execution_port);
    let execution_addr = Address::zmq_tcp("127.0.0.1", execution_port);

    // 2. Setup Dummy Publishers (Strategies)
    let mut publisher1 = build_publisher::<Allocation>(&strategy_1_addr).unwrap();
    let _publisher2 = build_publisher::<Allocation>(&strategy_2_addr).unwrap();

    // 3. Setup Dummy Subscriber (Execution Engine)
    // We bind here to behave like the execution engine server if it was SUB (wait, normally execution engine is SUB?)
    // In our architecture:
    // Strategy (PUB) -> Multiplexer (SUB / PUB) -> Execution Engine (SUB).
    // So Multiplexer connects to Strategies (SUB connects to PUB).
    // And Multiplexer connects to Execution Engine (PUB connects to SUB).

    // So Multiplexer needs to BIND its output.
    // `run_multiplexer` calls `build_publisher(&output_addr)`.
    // `ZmqPublisher::new` likely binds.
    // So Execution Engine connects.

    // So our Test Subscriber (acting as Execution Engine) should Connect to Multiplexer's output.
    // But Multiplexer output address is `execution_port`.
    // So Multiplexer will BIND to `execution_port`.
    // Test Subscriber should CONNECT to `execution_port`.

    // 4. Launch Multiplexer in a thread
    let handle = thread::spawn(move || {
        // Mock Args: Use parse_from directly to ensure all required args are present and avoid clap panic
        let args = CommonArgs::parse_from(vec![
            "multiplexer_test".to_string(),
            "--admin-route".to_string(),
            admin_addr_str,
            "--output-port".to_string(),
            execution_addr_str,
            "--input-port".to_string(),
            "tcp://127.0.0.1:0".to_string(), // Dummy unused input port
            "--service-name".to_string(),
            "multiplexer".to_string(),
            "--service-id".to_string(),
            "1".to_string(),
            "--config-dir".to_string(),
            "/tmp/trading_test_config".to_string(),
            "--data-dir".to_string(),
            "/tmp/trading_test_data".to_string(),
        ]);

        // Address Book: Map "execution_engine" to output port
        // `Microservice::run` populates "allocation" -> args.get_output_port().

        let config = Configuration::new_multiplexer(Box::new(
            |_state: &mut TestState, allocation: Allocation| allocation,
        ));
        let service = Microservice::new_with_args(args, || TestState, config);
        service.run();
    });

    thread::sleep(Duration::from_millis(500)); // Wait for startup

    // 5. Admin Client
    let ctx = zmq::Context::new();
    let admin_sock = ctx.socket(zmq::REQ).unwrap();
    let admin_connect_str = format!("tcp://127.0.0.1:{}", admin_port);
    admin_sock.connect(&admin_connect_str).unwrap();

    // 6. Test Loop
    let mut test_subscriber = build_subscriber::<Allocation>(&execution_addr).unwrap();

    // Command 1: Add Strategy 1
    let cmd = AdminCommand::AddStrategy {
        address: Address::zmq_tcp("127.0.0.1", strategy_1_port),
    };
    let payload = AdminPayload::new_command(cmd);
    let msg = serde_json::to_string(&payload).unwrap();
    admin_sock.send(&msg, 0).unwrap();
    let resp = admin_sock.recv_string(0).unwrap().unwrap();
    println!("Admin Resp: {}", resp);
    assert!(resp.contains("Ok"));

    thread::sleep(Duration::from_millis(500)); // Allow connect

    // Publish data from Strategy 1
    let alloc1 = Allocation::default();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        publisher1.send(&alloc1).await.unwrap();
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
    let msg = serde_json::to_string(&payload).unwrap();
    admin_sock.send(&msg, 0).unwrap();

    handle.join().unwrap();
}
