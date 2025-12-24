use serde::{Deserialize, Serialize};

/// Represents the type/category of the microservice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    Strategy,
    Gateway,
    Infrastructure,
    Custom(String),
}

/// The Manifest declares the resource requirements of a service.
/// Sent to the Orchestrator during the bootstrap handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    service_name: String,
    service_type: ServiceType,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Manifest {
    /// Creates a new Manifest.
    pub fn new(service_name: impl Into<String>, service_type: ServiceType) -> Self {
        Self {
            service_name: service_name.into(),
            service_type,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Declares that this service requires an Input for the given channel name.
    /// (Typically called with `Channel::name()` or similar).
    pub fn add_input(&mut self, channel_name: &str) {
        if !self.inputs.iter().any(|s| s == channel_name) {
            self.inputs.push(channel_name.to_string());
        }
    }

    /// Declares that this service produces Output for the given channel name.
    pub fn add_output(&mut self, channel_name: &str) {
        if !self.outputs.iter().any(|s| s == channel_name) {
            self.outputs.push(channel_name.to_string());
        }
    }

    // --- Getters ---

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }

    pub fn get_service_type(&self) -> &ServiceType {
        &self.service_type
    }

    pub fn get_inputs(&self) -> &[String] {
        &self.inputs
    }

    pub fn get_outputs(&self) -> &[String] {
        &self.outputs
    }
}
