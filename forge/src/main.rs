use clap::Parser;
use log::{error, info, warn};
use thiserror::Error;

pub mod args;
pub mod builder;
pub mod error;
pub mod generator;

use args::{Cli, Commands};

// Define Main CLI Errors
#[derive(Error, Debug)]
enum CliError {
    #[error("Generator error: {0}")]
    Generator(#[from] error::ForgeError),
}

fn main() -> Result<(), CliError> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name, type_, path } => {
            info!("Forging new {} named '{}'...", type_, name);
            if type_ != "strategy" {
                warn!("Only 'strategy' type is supported currently. Defaulting to strategy.");
            }
            match generator::generate_strategy(name, path) {
                Ok(_) => info!("Generation successful!"),
                Err(e) => {
                    error!("Generation failed: {}", e);
                    return Err(CliError::Generator(e));
                }
            }
        }
        Commands::Fuse { path } => {
            info!("Fusing project at '{}'...", path.display());
            match builder::fuse_strategy(path) {
                Ok(artifact) => {
                    info!("Fusion complete!");
                    info!("Engine binary available at: {}", artifact.display());
                }
                Err(e) => {
                    error!("Fusion failed: {}", e);
                    return Err(CliError::Generator(e));
                }
            }
        }
    }

    Ok(())
}
