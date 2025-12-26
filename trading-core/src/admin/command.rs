use serde::{Deserialize, Serialize};

use crate::comms::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AdminPayload {
    Command(AdminCommand),
    Response(AdminResponse),
}

impl AdminPayload {
    /// Creates a new Command payload.
    ///
    /// # Arguments
    ///
    /// * `command` - The `AdminCommand` to wrap.
    ///
    /// # Returns
    ///
    /// A `AdminPayload::Command`.
    pub fn new_command(command: AdminCommand) -> Self {
        Self::Command(command)
    }

    /// Creates a new Response payload.
    ///
    /// # Arguments
    ///
    /// * `response` - The `AdminResponse` to wrap.
    ///
    /// # Returns
    ///
    /// A `AdminPayload::Response`.
    pub fn new_response(response: AdminResponse) -> Self {
        Self::Response(response)
    }
}

/// Represents a command sent over the Admin channel.
///
/// Using an Enum ensures type safety and prevents invalid command strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum AdminCommand {
    /// Update a configuration parameter.
    UpdateRegistry { key: String, value: String },

    /// Gracefully shutdown the service.
    Shutdown,

    /// Ping request to check service health.
    Ping,

    /// Request for status
    Status,

    /// Request for registry
    Registry,

    /// Adds a new strategy connection dynamically (Multiplexer only).
    AddStrategy { address: Address },

    /// Catch-all for forward compatibility (optional).
    /// If an unknown command is received, it falls here (if using serde_json).
    #[serde(other)]
    Unknown,
}

impl AdminCommand {
    /// Helper to check if this command is a shutdown command.
    ///
    /// # Returns
    ///
    /// `true` if this is a `Shutdown` command, `false` otherwise.
    pub fn is_shutdown(&self) -> bool {
        matches!(self, AdminCommand::Shutdown)
    }
}

/// Represents a response sent back from the service to the Admin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload")]
pub enum AdminResponse {
    /// Command processed successfully.
    Ok,

    /// Command failed with an error message.
    Error(String),

    /// Requested information (e.g., status, config).
    Info(serde_json::Value),

    /// Pong response to a Ping command.
    Pong,
}
