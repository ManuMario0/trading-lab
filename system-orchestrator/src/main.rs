mod api;
mod event_bus;
mod layout;
mod registry;
mod runtime;
mod supervisor;

use crate::event_bus::SystemEvent;
use anyhow::Result;
use clap::Parser;
use log::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory containing the microservice manifests and binaries
    #[arg(short, long, default_value = "services")]
    service_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logger
    // This satisfies "Clean Logging". We use env_logger for now,
    // but we will also attach a listener to the EventBus below.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    // Set global panic hook to ensure crashes in background tasks are logged
    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL: System Panic! Details: {}", info);
    }));

    info!("Starting System Orchestrator V2");
    info!("Service Directory: {}", args.service_dir);

    // 2. Initialize the Central Nervous System (Event Bus)
    let event_bus = crate::event_bus::EventBus::new();

    // 3. Spawn the "Black Box Recorder"
    let mut log_rx = event_bus.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = log_rx.recv().await {
            match event {
                SystemEvent::ServiceCrashed { id, exit_code } => {
                    log::error!("[CRASH] Service {} died with code {:?}", id, exit_code);
                }
                _ => {
                    info!("[EVENT] {:?}", event);
                }
            }
        }
    });

    // 4. Initialize Runtime (The Hand)
    // We use the LocalServiceProvider for this node.
    let runtime = std::sync::Arc::new(crate::runtime::LocalServiceProvider::new());

    // 5. Initialize Registry (The Eyes)
    let disk_watcher = crate::registry::DiskWatcher::new(
        std::path::PathBuf::from(&args.service_dir),
        event_bus.clone(),
    );
    // Fix: Run DiskWatcher in a real OS thread because it performs heavy blocking I/O (file scanning)
    // that starves the tokio async runtime workers.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(disk_watcher.run());
    });

    // 6. Initialize Supervisor (The Brain)
    let supervisor = crate::supervisor::Supervisor::new(event_bus.clone(), runtime);

    // 7. Start the Supervisor (The Brain)
    // We spawn it in a background task
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(supervisor.run());
    });

    // 8. Initialize API Server (The Mouth/Ears)
    // Listens on 0.0.0.0:5555
    // TODO: Make port configurable via args
    let mut api_server = crate::api::ApiServer::new("tcp://0.0.0.0:5555", event_bus.clone())?;

    info!("Starting API Server on tcp://0.0.0.0:5555 ...");
    tokio::spawn(async move {
        api_server.run().await;
    });

    info!("System Initialized. Waiting for events...");

    // Notify readiness (optional)

    // Keep the main thread alive.
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
