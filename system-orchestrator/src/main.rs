mod api;
mod config;
mod layout;
mod process;

use api::run_api_server;
use config::SystemConfig;
use layout::manager::LayoutManager;
use log::info;
use process::manager::ProcessManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("=== System Orchestrator Starting [MODULAR ARCHITECTURE] ===");

    let config = SystemConfig::default();

    // 1. Initialize Managers
    let layout_manager = Arc::new(LayoutManager::new());
    let process_manager = Arc::new(Mutex::new(ProcessManager::new(config.clone())));

    // 2. Initialize Comms
    let (tx, _rx) = broadcast::channel(100);

    // 3. Start API Server
    let pm_clone = process_manager.clone();
    let lm_clone = layout_manager.clone();
    let tx_clone = tx.clone();
    let api_port = 3000;

    tokio::spawn(async move {
        run_api_server(pm_clone, lm_clone, tx_clone, api_port).await;
    });

    info!("System initialized. Waiting for commands...");

    // 5. Main Loop (Process Monitoring)
    loop {
        {
            let mut pm = process_manager.lock().unwrap();
            pm.check_status().await;
        }

        sleep(Duration::from_secs(5)).await;
    }
}
