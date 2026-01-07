use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Layout {
    id: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Layout {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    id: String,
    name: String,
    service: String,
    status: String,
}

impl Node {
    pub fn new(id: String, name: String, service: String, status: String) -> Self {
        Self {
            id,
            name,
            service,
            status,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    id: String,
    source: String,      // Node ID
    source_port: String, // Output Port Name
    target: String,      // Node ID
    target_port: String, // Input Port Name
}

impl Edge {
    pub fn new(
        id: String,
        source: String,
        source_port: String,
        target: String,
        target_port: String,
    ) -> Self {
        Self {
            id,
            source,
            source_port,
            target,
            target_port,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn source_port(&self) -> &str {
        &self.source_port
    }

    pub fn target_port(&self) -> &str {
        &self.target_port
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RunMode {
    BacktestFast,
    Paper,
    Live,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

/// Represents a discovered service available for deployment.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDescriptor {
    pub service: String, // The "category" or "type" ID
    pub description: String,
    pub version: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub binary_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub data_type: String, // Type of data (e.g. "MarketDataBatch")
    pub required: bool,
    pub is_variadic: bool,
}

// --- Deployment Model (The "Output" of the Layout Engine) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeploymentPlan {
    pub layout_id: String,
    /// Map of NodeID -> Configuration
    pub services: std::collections::HashMap<String, ServiceConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfig {
    pub node_id: String,
    pub service_type: String,
    pub binary_path: String, // Resolved path to executable
    pub args: Vec<String>,   // CLI arguments
    pub env: std::collections::HashMap<String, String>, // Environment variables

    // We might add specific port mappings here if needed for debugging,
    // though usually they are embedded in `args`.
    pub inputs: std::collections::HashMap<String, String>, // PortName -> Address
    pub outputs: std::collections::HashMap<String, String>, // PortName -> Address
}
