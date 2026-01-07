use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Deployment Model (The "Output" of the Layout Engine) ---
// This is internal to the Orchestrator.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeploymentPlan {
    layout_id: String,
    services: HashMap<String, ServiceConfig>,
    // Maps "NodeId:PortName" -> "Address"
    // We use a composite string key for JSON compatibility.
    allocations: HashMap<String, String>,
}

impl DeploymentPlan {
    pub fn new(
        layout_id: String,
        services: HashMap<String, ServiceConfig>,
        allocations: HashMap<String, String>,
    ) -> Self {
        Self {
            layout_id,
            services,
            allocations,
        }
    }

    pub fn layout_id(&self) -> &str {
        &self.layout_id
    }

    pub fn services(&self) -> &HashMap<String, ServiceConfig> {
        &self.services
    }

    pub fn allocations(&self) -> &HashMap<String, String> {
        &self.allocations
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ServiceConfig {
    node_id: String,
    service_type: String,
    binary_path: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    admin_api: Option<String>,
}

impl ServiceConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        service_type: String,
        binary_path: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        admin_api: Option<String>,
    ) -> Self {
        Self {
            node_id,
            service_type,
            binary_path,
            args,
            env,
            admin_api,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    pub fn binary_path(&self) -> &str {
        &self.binary_path
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    pub fn admin_api(&self) -> Option<&String> {
        self.admin_api.as_ref()
    }
}
