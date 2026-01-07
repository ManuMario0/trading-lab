use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The Trading Engine Developer Toolkit CLI.
#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "The Trading Engine Developer Toolkit", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for the Forge tool.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new strategy or component.
    New {
        /// Name of the project.
        #[arg(help = "The name of the new project (kebab-case recommended)")]
        name: String,

        /// Type of component to generate.
        #[arg(short, long, default_value = "strategy")]
        type_: String,

        /// Output path (default: current directory).
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    /// Fuses a user library into a runnable engine binary.
    Fuse {
        /// Path to the user's project directory.
        #[arg(help = "Path to the strategy library to fuse")]
        path: PathBuf,
    },
}
