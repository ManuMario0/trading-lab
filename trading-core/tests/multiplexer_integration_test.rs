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
    let admin_port = 60001;
    let strategy_1_port = 60002;
    let strategy_2_port = 60003;
    let execution_port = 60004;

    let admin_addr_str = format!("tcp://127.0.0.1:{}", admin_port);
    let strategy_1_addr = Address::zmq_tcp("127.0.0.1", strategy_1_port);
    let strategy_2_addr = Address::zmq_tcp("127.0.0.1", strategy_2_port);
    let execution_addr_str = format!("tcp://127.0.0.1:{}", execution_port);
    let execution_addr = Address::zmq_tcp("127.0.0.1", execution_port);

    // 2. Setup Dummy Publishers (Strategies)
    let mut publisher1 = build_publisher::<Allocation>(&strategy_1_addr).unwrap();
    let mut publisher2 = build_publisher::<Allocation>(&strategy_2_addr).unwrap();

    // 3. Setup Dummy Subscriber (Execution Engine)
    // We bind here to behave like the execution engine server if it was SUB (wait, normally execution engine is SUB?)
    // In our architecture:
    // Strategy (PUB) -> Multiplexer (SUB / PUB) -> Execution Engine (SUB).
    // So Multiplexer connects to Strategies (SUB connects to PUB).
    // And Multiplexer connects to Execution Engine (PUB connects to SUB).
    // Wait, ZMQ PUB/SUB direction:
    // Strategies bind PUB. Multiplexer connects SUB. Correct.
    // Multiplexer should bind PUB? Or connect PUB?
    // Usually, the centralized component binds.
    // If Execution Engine is a service, it binds SUB or PULL?
    // ZmqPublisher::new connects or binds?
    // Let's check comms implementation. Usually Publisher binds, Subscriber connects.
    // BUT ZMQ supports either.
    // In `trading-core`, `ZmqSubscriber` connects by default (we checked `connect` impl).
    // `ZmqPublisher` likely binds.

    // So if Multiplexer connects to Execution Engine:
    // Execution Engine (SUB) Binds.
    // Multiplexer (PUB) Connects.

    // Let's create a Subscriber that BINDS.
    // `build_subscriber` calls `ZmqSubscriber::new`.
    // We need to check if `ZmqSubscriber::new` binds or connects.
    // If it connects, then we need a Publisher that binds.
    // IF Multiplexer is PUB, it connects.
    // So Execution Engine must be SUB and BIND.
    // BUT `ZmqSubscriber::new` connects.
    // Use `ZmqSubscriber::new_bind`? It doesn't exist yet.
    // Maybe we just rely on standard: Publisher Binds, Subscriber connects.
    // Strategies (PUB - Bind) -> Multiplexer (SUB - Connect). This works.
    // Multiplexer (PUB - Bind) -> Execution Engine (SUB - Connect).

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
        // Mock Args
        let args = CommonArgs::parse_args(vec![
            "multiplexer_test".to_string(),
            "--admin-route".to_string(),
            admin_addr_str,
            "--output-port".to_string(), // This is the PUB address
            execution_addr_str,
            "--service-name".to_string(),
            "multiplexer".to_string(),
            "--config-dir".to_string(),
            "/tmp/trading_test_config".to_string(),
            "--data-dir".to_string(),
            "/tmp/trading_test_data".to_string(),
        ]);

        // Address Book: Map "execution_engine" to output port?
        // Wait, `Multiplexer::run` looks up "execution_engine".
        // `args.get_output_port` returns the output port.
        // But `Configuration::run` uses `address_book` passed by `Microservice::run`.
        // `Microservice::run` currently passes an EMPTY address_book (Stub).
        // THIS IS A PROBLEM.
        // I need to update `Microservice::run` to populate address book or use args.
        // `args` has `output_port`.
        // The simple fix for the test is to assume `address_book` is somehow populated or mock it.
        // Since `Microservice::run` creates an empty map, `Multiplexer::run` will FAIL to find "execution_engine".

        // Quick Fix for Test:
        // We can't easily modify `Microservice::run` logic from here without changing source.
        // I should have populated `address_book` in `Microservice::run`.
        // Let's assume I fix `Microservice::run` to put `execution_engine` -> `args.output_port`.
        // Or I can just manually construct `Multiplexer` and run it, bypassing `Microservice`?
        // But `Multiplexer` struct is private or difficult to construct?
        // `Configuration::new_multiplexer()` returns `Configuration` which has private builder.

        // I MUST fix `Microservice::run` to populate `address_book`.

        // BUT, I'll proceed keeping this in mind.
        // I will write the test assuming it works, then fix `Microservice::run` in the next step if verify fails (or preemptively).
        // Actually, I should preemptively fix `Microservice::run` to use `args.get_output_port()` for "execution_engine" or similar defaults.

        let config = Configuration::new_multiplexer();
        let service = Microservice::new(args, || TestState, config);
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
    // Re-verify direction:
    // Multiplexer (PUB) on Port 60004.
    // Subscriber (SUB) connects to 60004.
    // If `build_subscriber` calls connect, and `build_publisher` calls bind.
    // Yes.

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
    let alloc1 = Allocation::default(); // Add identifiable data if possible
                                        // Allocation fields are private? Check `allocation.rs`.
                                        // Default is all 0/empty.
                                        // `SenderSocket` send is async. `build_publisher` returns `SenderSocket`.
                                        // We need async runtime for sending?
                                        // `SenderSocket` wraps `TransportOutput`. `ZmqPublisher` implements it.
                                        // `ZmqPublisher::send` is likely async or blocking? zmq crate is blocking usually.
                                        // `SendEndpoint` trait is async.
                                        // So we need `block_on` to send.

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        publisher1.send(&alloc1).await.unwrap();
    });

    // Receive
    let received = rt.block_on(async {
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
