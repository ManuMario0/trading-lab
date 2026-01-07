use std::path::PathBuf;
use thiserror::Error;

/// Global error type for the Forge application.
#[derive(Error, Debug)]
pub enum ForgeError {
    /// The skeleton template could not be found internally.
    #[error("Template not found at {0}. Are you running from workspace root?")]
    TemplateNotFound(PathBuf),

    /// The target directory already exists.
    #[error("Directory {0} already exists")]
    DirectoryExists(PathBuf),

    /// Underlying IO failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse or modify TOML.
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml_edit::TomlError),

    /// Failed to copy template files.
    #[error("Copy error: {0}")]
    Copy(#[from] fs_extra::error::Error),

    /// Failed to rename the project folder after copying.
    #[error("Failed to rename project directory from {from} to {to}: {source}")]
    RenameError {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    /// Global logger configuration error.
    #[error("Logger error: {0}")]
    Logger(#[from] log::SetLoggerError),

    /// Missing Cargo.toml in user project.
    #[error("No Cargo.toml found at {0}")]
    MissingCargoToml(PathBuf),

    /// Failed to execute cargo build.
    #[error("Cargo build failed: {0}")]
    CargoBuild(String),

    /// Missing compilation artifact.
    #[error("Artifact not found at {0}")]
    ArtifactNotFound(PathBuf),
}

/// A specialized Result type for Forge operations.
pub type Result<T> = std::result::Result<T, ForgeError>;
