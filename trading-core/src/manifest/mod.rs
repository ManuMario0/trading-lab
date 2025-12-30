use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::comms::Address;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceBlueprint {
    /// Unique identifier for the service type (e.g., "MomentumStrategy", "GeoMultiplexer")
    pub service_type: String,

    /// Inputs this service expects
    pub inputs: Vec<PortDefinition>,

    /// Outputs this service produces
    pub outputs: Vec<PortDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortDefinition {
    pub name: String,      // e.g., "market_data", "signals"
    pub data_type: String, // e.g., "MarketDataBatch", "Allocation"
    pub required: bool,

    /// If true, this port accepts N dynamic incoming connections.
    /// The Orchestrator will map specific instances to this port using a naming convention
    /// (e.g. "strategies" -> ["strategy_A", "strategy_B"]).
    #[serde(default)]
    pub is_variadic: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    pub address: Address,
    pub id: usize, // Unique Process ID / Strategy ID
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Binding {
    /// A single connection (e.g. "tcp://localhost:5555")
    Single(Source),
    /// A map of named connections for variadic ports (e.g. {"strat1": "tcp://...", "strat2": "tcp://..."})
    Variadic(std::collections::HashMap<String, Source>),
}

/// The concrete configuration for a specific service instance.
/// This tells the service exactly where to bind/connect for each logical port.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceBindings {
    /// Map of Manifest Input Name -> Binding Info
    pub inputs: std::collections::HashMap<String, Binding>,

    /// Map of Manifest Output Name -> Binding Info
    pub outputs: std::collections::HashMap<String, Binding>,
}

impl FromStr for Binding {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ServiceBindings {
    pub fn new() -> Self {
        Self {
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        }
    }
}
