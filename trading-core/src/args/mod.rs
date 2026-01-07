//! Defines the standard command-line arguments shared across all microservices.
//!
//! This module uses `clap` to parse common configuration parameters such as the
//! admin port, service name, and directory paths. By enforcing a common argument
//! structure, we ensure uniform configuration and behavior across both Rust and
//! C++ components of the trading platform.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
#[cfg(feature = "test-utils")]
use std::cell::RefCell;
use std::path::PathBuf;

use crate::manifest::ServiceBindings;

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Parses command-line arguments into a `Cli` struct.
    ///
    /// This function automatically handles `--help` and `--version` flags via `clap`.
    /// If required arguments are missing or invalid, it will print an error and exit.
    pub fn new() -> Self {
        #[cfg(feature = "test-utils")]
        {
            // Check if a mock is set for this thread
            let mock = MOCK_ARGS.with(|m| m.borrow().clone());
            if let Some(args) = mock {
                return Cli {
                    command: Commands::Run(args),
                };
            }
            // Fallback in unit tests if no mock is set
            return Cli {
                command: Commands::Run(CommonArgs::default_for_test()),
            };
        }

        #[cfg(not(feature = "test-utils"))]
        Cli::parse()
    }

    #[cfg(feature = "test-utils")]
    pub fn set_mock(args: CommonArgs) {
        MOCK_ARGS.with(|m| *m.borrow_mut() = Some(args));
    }

    pub fn process(self, manifest: &crate::manifest::ServiceManifest) -> CommonArgs {
        match self.command {
            Commands::Run(args) => args,
            Commands::Manifest => {
                println!("{}", serde_json::to_string(manifest).unwrap());
                std::process::exit(0);
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
enum Commands {
    Run(CommonArgs),
    Manifest,
}

impl From<Cli> for Commands {
    fn from(value: Cli) -> Self {
        value.command
    }
}

#[cfg(feature = "test-utils")]
thread_local! {
    static MOCK_ARGS: RefCell<Option<CommonArgs>> = RefCell::new(None);
}

/// Holds the standard configuration parameters parsed from the command line.
///
/// These arguments are expected to be present for every microservice invocation.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct CommonArgs {
    /// Name of the service (used for logging and admin registry)
    #[arg(short, long)]
    service_name: String,

    #[arg(short = 'i', long)]
    service_id: usize,

    #[arg(long)]
    bindings: String,

    /// Path to the configuration directory
    #[arg(long)]
    config_dir: PathBuf,

    /// Path to the data directory (for state saving)
    #[arg(long)]
    data_dir: PathBuf,
}

impl CommonArgs {
    /// Returns the service bindings.
    pub fn get_bindings(&self) -> ServiceBindings {
        serde_json::from_str(&self.bindings).expect("Failed to parse bindings")
    }

    /// Returns the path to the configuration directory.
    ///
    /// This directory should contain static config files (e.g., `algo_params.json`).
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    /// Returns the path to the data directory.
    ///
    /// This directory is used for runtime state persistence (e.g., `state.json`).
    pub fn get_data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    /// Returns the name of the service.
    ///
    /// This is used for logging identification and registering with the Admin functionality.
    pub fn get_service_name(&self) -> String {
        self.service_name.clone()
    }

    /// Returns the ID of the service.
    ///
    /// This is used for logging identification and registering with the Admin functionality.
    pub fn get_service_id(&self) -> usize {
        self.service_id
    }

    /// Creates a default Configuration for testing.
    pub fn default_for_test() -> Self {
        Self {
            service_name: "test_service".to_string(),
            service_id: 0,
            bindings: "".to_string(),
            config_dir: PathBuf::from("./test_config"),
            data_dir: PathBuf::from("./test_data"),
        }
    }
}
