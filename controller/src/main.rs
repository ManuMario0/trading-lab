use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use orchestrator_protocol::{
    Layout, OrchestratorClient, OrchestratorCommand, OrchestratorResponse, RunMode,
};
use std::fs;

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
    // Default port 5555
    let addr = "tcp://127.0.0.1:5555";

    // Initialize Client
    let mut client = OrchestratorClient::new(addr)?;

    match cli.command {
        Commands::Status => {
            // Not yet implemented in Client helper, so we skip or error.
            eprintln!("Status command not yet implemented in client library.");
        }
        Commands::Deploy { file } => {
            let content = fs::read_to_string(&file).context("Failed to read layout file")?;
            let layout: Layout = serde_json::from_str(&content).context("Invalid Layout JSON")?;
            let mode = RunMode::Live;

            match client.deploy(layout, mode).await {
                Ok(msg) => println!("SUCCESS: {}", msg),
                Err(e) => eprintln!("ERROR: {}", e),
            }
        }
        Commands::Stop { layout } => match client.stop(layout).await {
            Ok(msg) => println!("SUCCESS: {}", msg),
            Err(e) => eprintln!("ERROR: {}", e),
        },
        Commands::Wallet { layout: _ } => {
            eprintln!("Wallet command not yet implemented in client library.");
        }
        Commands::Shutdown => {
            eprintln!("Shutdown command not yet implemented in client library.");
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
        OrchestratorResponse::ServicesList(services) => {
            println!("Available Services:");
            for s in services {
                println!("- {} (v{}): {}", s.service, s.version, s.description);
            }
        }
        OrchestratorResponse::Error(e) => eprintln!("ERROR: {}", e),
    }
}
