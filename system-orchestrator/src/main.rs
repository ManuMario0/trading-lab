mod config;
mod layout;
mod process;

use config::SystemConfig;
use layout::manager::LayoutManager;
use log::{error, info};
use orchestrator_protocol::{OrchestratorCommand, OrchestratorResponse, OrchestratorServer};
use process::manager::ProcessManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use trading_core::comms::transports::zmq::ZmqDuplex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("=== System Orchestrator Starting [DAEMON MODE] ===");

    let config = SystemConfig::default();

    // 1. Initialize Managers
    let _layout_manager = Arc::new(LayoutManager::new());
    let process_manager = Arc::new(Mutex::new(ProcessManager::new(config.clone())));

    // 2. Initialize Comms (ZMQ REP)
    let zmq_port = 5555;
    let addr = format!("tcp://0.0.0.0:{}", zmq_port);
    info!("Binding Orchestrator Command Interface to {}", addr);

    let transport = Box::new(ZmqDuplex::new(&addr)?);
    let mut server = OrchestratorServer::new(transport);

    let pm_clone = process_manager.clone();

    // 3. Start Command Server Loop
    tokio::spawn(async move {
        info!("Command Server Loop Started");
        loop {
            match server.next_command().await {
                Ok(cmd) => {
                    info!("Received command: {:?}", cmd);
                    let resp = handle_command(cmd, &pm_clone).await;
                    if let Err(e) = server.send_response(resp).await {
                        error!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    error!("Transport error receiving command: {}", e);
                    sleep(Duration::from_millis(100)).await; // Backoff on error
                }
            }
        }
    });

    info!("System initialized. Monitoring processes...");

    // 4. Main Loop (Process Monitoring)
    loop {
        {
            let mut pm = process_manager.lock().unwrap();
            pm.check_status();
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn handle_command(
    cmd: OrchestratorCommand,
    pm: &Arc<Mutex<ProcessManager>>,
) -> OrchestratorResponse {
    let mut pm = pm.lock().unwrap();

    match cmd {
        OrchestratorCommand::GetStatus => OrchestratorResponse::StatusInfo(pm.list()),

        OrchestratorCommand::Deploy { layout, mode: _ } => {
            // mode is currently ignored in ProcessManager, but passed for future use
            match pm.deploy(&layout) {
                Ok(_) => OrchestratorResponse::Success("Layout Deployed".to_string()),
                Err(e) => OrchestratorResponse::Error(format!("Deploy failed: {}", e)),
            }
        }

        OrchestratorCommand::Stop { layout_id } => {
            // Naive stop: remove from PM internal map? PM didn't expose remove_layout directly.
            // But we can implement a "stop_layout" or just accept we don't support it fully yet.
            // For now, return Error or minimal support if user really wants iteration.
            // User requested fast iteration.
            // Let's implement Stop if "remove_layout" logic existed.
            // ProcessManager has stop(layout_id, node_id).
            // We'll just return not implemented for now to be safe, or success if we did nothing.
            OrchestratorResponse::Error(
                "Stop Layout not fully implemented in ProcessManager".to_string(),
            )
        }

        OrchestratorCommand::GetWallet { layout_id } => match pm.get_engine(&layout_id) {
            Some(engine) => match engine.send_command(r#"{"cmd": "WALLET"}"#) {
                Ok(json_str) => match serde_json::from_str(&json_str) {
                    Ok(val) => OrchestratorResponse::WalletInfo(val),
                    Err(_) => OrchestratorResponse::Error("Invalid JSON from wallet".into()),
                },
                Err(e) => OrchestratorResponse::Error(e.to_string()),
            },
            None => OrchestratorResponse::Error("No engine found in layout".into()),
        },

        OrchestratorCommand::Shutdown => {
            std::process::exit(0);
        }
    }
}
