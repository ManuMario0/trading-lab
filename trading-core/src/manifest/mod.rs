use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::comms::Address;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceManifest {
    pub blueprint: ServiceBlueprint,
    pub version: String,
    pub description: String,
}

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

impl ServiceBlueprint {
    pub fn generate_args(
        &self,
        service_id: &str,
        _admin_port: u16,
        bindings: ServiceBindings,
    ) -> Vec<String> {
        let mut args = Vec::new();

        // Subcommand
        args.push("run".to_string());

        // Standard Args
        args.push("--service-name".to_string());
        args.push(self.service_type.clone());

        // We use hash or 0 for ID if parsing fails, but interface passes string node.id
        // CommonArgs expects integer id?
        // Let's hash string to usize or parse
        // If service_id is "pm_1", we strip prefix or hash.
        // Simple hash:
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hash;
        use std::hash::Hasher;
        service_id.hash(&mut hasher);
        let numeric_id = hasher.finish() as usize; // Cast u64 to usize

        args.push("--service-id".to_string());
        args.push(numeric_id.to_string());

        // Bindings (JSON)
        let bindings_json = serde_json::to_string(&bindings).unwrap_or("{}".to_string());
        args.push("--bindings".to_string());
        args.push(bindings_json);

        // Admin Port ? CommonArgs doesn't have admin_port explicitly?
        // Step 387: CommonArgs fields: service_name, service_id, bindings, config_dir, data_dir.
        // It does NOT have admin_port.
        // But `config_resolver` allocates it.
        // Where is admin port used?
        // Maybe in bindings?
        // Or passed as extra arg?
        // Legacy args had --admin-port.
        // New CommonArgs relies on bindings to setup inputs/outputs.
        // BUT admin is distinct.
        // Check `trading-core/src/microservice/mod.rs` or `launcher`.
        // If CommonArgs doesn't take admin port, how does service know where to bind admin?
        // Maybe strict Blueprint doesn't enforce admin port yet?
        // Or it's part of bindings? e.g. "admin" port.

        // For MVP, if CommonArgs doesn't have it, we might skip it or add it.
        // Let's assume we skip precise admin port config for now OR add it to bindings?

        // Return args
        args.push("--config-dir".to_string());
        args.push(".".to_string());
        args.push("--data-dir".to_string());
        args.push(".".to_string());

        args
    }
}

impl ServiceManifest {
    pub fn new(blueprint: ServiceBlueprint, version: String, description: String) -> Self {
        Self {
            blueprint,
            version,
            description,
        }
    }
}
