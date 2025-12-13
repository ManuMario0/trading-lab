mod config;
mod process_manager;
mod state;
mod zmq_monitor;
mod admin_client;
mod api;

use config::SystemConfig;
use process_manager::ProcessManager;
use zmq_monitor::ZmqMonitor;
use admin_client::AdminClient;
use api::run_api_server;
use state::create_state;
use log::{info, error};
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::broadcast;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("=== System Orchestrator Starting ===");

    let config = SystemConfig::default(); // Load default for now
    let state = create_state();
    
    // Create Process Manager
    let mut pm = ProcessManager::new();

    // 1. Start Multiplexer (C++)
    // Args: --input-port X --output-port Y --admin-port Z
    pm.spawn(
        "Multiplexer", 
        &config.multiplexer_path, 
        &[
            "--input-port", &config.multiplexer_input_port.to_string(), // 5564
            "--output-port", &config.multiplexer_port.to_string(),      // 5561
            "--admin-port", &config.multiplexer_admin_port.to_string()  // 5565
        ]
    )?;

    // 2. Start Execution Engine (Rust)
    // Args: --admin-port X --multiplexer-ports Y --data-port Z --order-port W
    pm.spawn(
        "ExecutionEngine", 
        &config.execution_engine_path, 
        &[
            "--admin-port", &config.admin_port.to_string(),             // 5560
            "--multiplexer-ports", &config.multiplexer_port.to_string(), // 5561
            "--data-port", &config.data_port.to_string(),               // 5562
            "--order-port", &config.order_port.to_string(),             // 5563
        ]
    )?;

    // 3. Start Data Pipeline (Python)
    pm.spawn(
        "DataPipeline",
        "python3",
        &[
            "-u",
            &config.data_pipeline_path,
            "--port", &config.data_port.to_string()                     // 5562
        ]
    )?;

    // 4. Start Strategy Lab (C++)
    // Usage: ./strategy_lab [input_addr] [output_addr] [admin_addr]
    // Input: Data (5562)
    // Output: Mpx Input (5564) (Mpx PULLs, so Strat PUSHes to that address)
    // Admin: Strat Admin (5566)
    
    let strat_input = format!("tcp://127.0.0.1:{}", config.data_port);
    let strat_output = format!("tcp://127.0.0.1:{}", config.multiplexer_input_port);
    let strat_admin = format!("tcp://*:{}", config.strategy_admin_port);

    pm.spawn(
        "StrategyLab",
        &config.strategy_lab_path,
        &[&strat_input, &strat_output, &strat_admin] 
    )?;

    // Create Broadcast Channel for WS
    let (tx, _rx) = broadcast::channel(100);

    // 5. Init Admin Client (Needed for API)
    let admin_client = Arc::new(AdminClient::new(&config).expect("Failed to init AdminClient"));

    // 6. Start ZMQ Monitor (Bridge)
    // Pass tx to monitor so it can broadcast updates
    let mut monitor = ZmqMonitor::new(config.clone(), state.clone(), tx.clone());
    monitor.start();
    
    // 7. Start API Server (Async Task)
    let state_clone = state.clone();
    let admin_clone = admin_client.clone();
    let tx_clone = tx.clone();
    let api_port = 3000; // Default API port for now, could be in config
    
    tokio::spawn(async move {
        run_api_server(state_clone, admin_clone, tx_clone, api_port).await;
    });

    info!("All systems initiated. Entering Supervisor Loop...");

    // Main Control Loop
    loop {
        pm.check_status().await;
        // ...

        // ... rest of loop
        
        // Log State Stats
        {
            let read_guard = state.read().unwrap();
            if !read_guard.last_prices.is_empty() {
                // Info log might be too spammy, maybe debug usually, but for verification:
                info!("Orchestrator State: Tracking {} symbols. AAPL: {:?}", 
                    read_guard.last_prices.len(),
                    read_guard.last_prices.get("AAPL").map(|p| p.last));
            }
        }
        
        sleep(Duration::from_secs(5)).await;
    }
}
