use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use orchestrator_protocol::{
    Layout, OrchestratorClient, OrchestratorCommand, OrchestratorResponse, RunMode,
};
use std::fs;
use trading_core::comms::transports::zmq::ZmqDuplex;

#[derive(Parser)]
#[command(name = "controller")]
#[command(about = "CLI Controller for the Trading System Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the status of running processes
    Status,
    /// Deploy a layout
    Deploy {
        /// Path to the layout JSON file
        #[arg(short, long)]
        file: String,
    },
    /// Stop a layout (Not fully supported)
    Stop {
        /// Layout ID
        #[arg(short, long)]
        layout: String,
    },
    /// Get wallet info from a running engine
    Wallet {
        /// Layout ID
        #[arg(short, long)]
        layout: String,
    },
    /// Shutdown the daemon
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Connect to Orchestrator
    // Default port 5555 as in daemon
    let addr = "tcp://127.0.0.1:5555";

    // Use REQ socket for client
    let transport = Box::new(trading_core::comms::transports::zmq::ZmqClientDuplex::new(
        addr,
    )?);
    let mut client = OrchestratorClient::new(transport);

    match cli.command {
        Commands::Status => {
            let response = client.send_command(OrchestratorCommand::GetStatus).await?;
            print_response(response);
        }
        Commands::Deploy { file } => {
            let content = fs::read_to_string(&file).context("Failed to read layout file")?;
            let layout: Layout = serde_json::from_str(&content).context("Invalid Layout JSON")?;

            // Mode is hardcoded for now or we could add a flag
            let mode = RunMode::BacktestFast; // Defaulting for now as per user req "fast debugging"? Or maybe Live?
                                              // User requested "fast controller... to start implementing strategies ASAP".
                                              // Let's assume Live for "running microservices" or Paper.
                                              // But since this is a generic controller, maybe default to Paper?
                                              // Or add a --mode flag. I'll stick to Paper as safer default or Live if they want real orchestrator behavior.
                                              // Orchestrator logic "ProcessManager" spawns processes. That's "Live" style (even if paper trading).
                                              // BacktestFast runs in single process.
                                              // The current ProcessManager logic is "Spawn Processes". So it supports Live/Paper.
            let mode = RunMode::Live;

            let response = client
                .send_command(OrchestratorCommand::Deploy { layout, mode })
                .await?;
            print_response(response);
        }
        Commands::Stop { layout } => {
            let response = client
                .send_command(OrchestratorCommand::Stop { layout_id: layout })
                .await?;
            print_response(response);
        }
        Commands::Wallet { layout } => {
            let response = client
                .send_command(OrchestratorCommand::GetWallet { layout_id: layout })
                .await?;
            print_response(response);
        }
        Commands::Shutdown => {
            let response = client.send_command(OrchestratorCommand::Shutdown).await?;
            print_response(response);
        }
    }

    Ok(())
}

fn print_response(resp: OrchestratorResponse) {
    match resp {
        OrchestratorResponse::Success(msg) => println!("SUCCESS: {}", msg),
        OrchestratorResponse::StatusInfo(info) => {
            println!("{:<36} | {:<20} | {:<10}", "ID", "NAME", "STATUS");
            println!("{:-<36}-+-{:-<20}-+-{:-<10}", "", "", "");
            for proc in info {
                println!("{:<36} | {:<20} | {:<10}", proc.id, proc.name, proc.status);
            }
        }
        OrchestratorResponse::WalletInfo(val) => {
            println!("{}", serde_json::to_string_pretty(&val).unwrap())
        }
        OrchestratorResponse::Error(e) => eprintln!("ERROR: {}", e),
    }
}
