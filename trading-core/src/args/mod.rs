//! Defines the standard command-line arguments shared across all microservices.
//!
//! This module uses `clap` to parse common configuration parameters such as the
//! admin port, service name, and directory paths. By enforcing a common argument
//! structure, we ensure uniform configuration and behavior across both Rust and
//! C++ components of the trading platform.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::comms::Address;

/// Holds the standard configuration parameters parsed from the command line.
///
/// These arguments are expected to be present for every microservice invocation.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct CommonArgs {
    /// Name of the service (used for logging and admin registry)
    #[arg(short, long, default_value = "unknown_service")]
    service_name: String,

    /// Port for incoming ZMQ messages (SUB) - default 0 implies ephemeral/unused in some contexts, but usually required.
    #[arg(long, default_value = "tcp://127.0.0.1:5555")]
    admin_route: String,

    /// Port for outgoing ZMQ messages (PUB)
    #[arg(long, default_value = "tcp://127.0.0.1:5556")]
    output_port: String,

    /// Port for incoming market data (SUB)
    #[arg(long, default_value = "tcp://127.0.0.1:5557")]
    input_port: String,

    /// Path to the configuration directory
    #[arg(long, default_value = "./config")]
    config_dir: PathBuf,

    /// Path to the data directory (for state saving)
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,
}

impl CommonArgs {
    /// Parses command-line arguments into a `CommonArgs` struct.
    ///
    /// This function automatically handles `--help` and `--version` flags via `clap`.
    /// If required arguments are missing or invalid, it will print an error and exit.
    pub fn parse_args(args: Vec<String>) -> Self {
        CommonArgs::parse_from(args)
    }

    /// Returns the admin port (SUB).
    pub fn get_admin_route(&self) -> Address {
        Address::Zmq(self.admin_route.clone())
    }

    /// Returns the output port (PUB).
    pub fn get_output_port(&self) -> Address {
        Address::Zmq(self.output_port.clone())
    }

    /// Returns the input port (SUB).
    pub fn get_input_port(&self) -> Address {
        Address::Zmq(self.input_port.clone())
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
}
