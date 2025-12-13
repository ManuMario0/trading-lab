use clap::Parser;
use execution_engine::engine::Engine;
use execution_engine::gateway::Gateway;
use execution_engine::io::{Args, ZmqAdmin, ZmqExchange, ZmqGateway};
use execution_engine::models::AllocationConfig;
use execution_engine::risk_guard::RiskGuard;
use log::info;
use std::sync::{mpsc, Arc, Mutex};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("=== Execution Engine Starting (Real IO Mode) ===");

    // 1. Parse Args
    let args = Args::parse();
    info!("Configuration: {:?}", args);

    // 2. Initialize Core Components
    // 2. Initialize Core Components
    let risk_guard = RiskGuard::new();

    // Create Shared ZMQ Context
    let shared_context = zmq::Context::new();

    let exchange = Box::new(ZmqExchange::new(&shared_context, args.order_port));
    let config = AllocationConfig::default(); // Empty initially, populated via Admin or Config File

    // Engine is shared between Admin (Thread) and Gateway (Main Loop)
    let engine = Arc::new(Mutex::new(Engine::new(risk_guard, exchange, config)));

    // 3. Initialize Admin Listener
    // Note: Admin runs in its own thread to handle synchronous REP/REQ cycles
    let mut admin = ZmqAdmin::new(engine.clone(), args.admin_port);
    admin.start();

    // 4. Initialize Gateway (Input)
    // Gateway runs in the main thread (Event Loop)
    let (_control_tx, control_rx) = mpsc::channel();

    // Wire Admin to Gateway for dynamic ports?
    // Current Admin impl just logs "Port plumbing pending".
    // ideally we pass control_tx to Admin?
    // For now, let's keep it simple as per plan.
    // The Gateway supports dynamic ports via `control_rx`.
    // We can expose a way to send commands later.

    let mut gateway = ZmqGateway::new(
        shared_context.clone(),
        args.data_port,
        args.multiplexer_ports,
        control_rx,
    );

    // 5. Run Event Loop
    info!("Entering Event Loop...");
    while let Some(msg) = gateway.next() {
        // Lock engine for processing
        match engine.lock() {
            Ok(mut guard) => guard.process(msg),
            Err(e) => {
                // Poisoned mutex is fatal
                log::error!("Engine Mutex Poisoned: {}", e);
                break;
            }
        }
    }

    info!("Event stream finished. Shutdown.");
    Ok(())
}
