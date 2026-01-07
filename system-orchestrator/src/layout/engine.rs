use crate::layout::model::{DeploymentPlan, ServiceConfig};
use crate::registry::ServiceCatalog;
use anyhow::{Context, Result};
use orchestrator_protocol::model::{Layout, Node, ServiceDescriptor};
use std::collections::{HashMap, HashSet};
use trading_core::manifest::{Binding, ServiceBindings, Source};

/// The Pure Logic Core.
pub struct LayoutEngine;

/// Represents the delta between two deployment plans.
#[derive(Debug)]
pub struct PlanDiff {
    pub to_spawn: Vec<ServiceConfig>,
    pub to_kill: Vec<String>,                         // Node IDs
    pub to_reconfigure: Vec<(String, ServiceConfig)>, // Node ID -> New Config
}

impl LayoutEngine {
    /// Resolves a Layout into a Deployment Plan using composed pure functions.
    pub fn resolve(
        layout: &Layout,
        catalog: &ServiceCatalog,
        base_port: u16,
        prev_plan: Option<&DeploymentPlan>,
    ) -> Result<DeploymentPlan> {
        // 1. Validation
        Self::validate_services(layout, catalog)?;

        // 2. Address Allocation
        // (NodeId, PortName) -> Address
        // Node-Driven Allocation with State Preservation
        let prev_allocations = prev_plan.map(|p| p.allocations());
        let output_addresses =
            Self::allocate_addresses(layout, catalog, base_port, prev_allocations);

        // 3. Admin Allocation
        let admin_addresses = Self::allocate_admin_addresses(
            layout,
            base_port + (output_addresses.len() as u16) + 100,
        );

        // 4. Build Configs
        let mut services = HashMap::new();
        for node in layout.nodes() {
            let descriptor = catalog.get(node.service()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Service '{}' not found (validation failed?)",
                    node.service()
                )
            })?;
            let config = Self::resolve_node_config(
                node,
                layout,
                descriptor,
                &output_addresses,
                &admin_addresses,
            )?;
            services.insert(node.id().to_string(), config);
        }

        // Convert to composite key map for DeploymentPlan
        let allocation_map: HashMap<String, String> = output_addresses
            .iter()
            .map(|((node, port), addr)| (format!("{}:{}", node, port), addr.clone()))
            .collect();

        Ok(DeploymentPlan::new(
            layout.id().to_string(),
            services,
            allocation_map,
        ))
    }

    /// Calculates the difference between two plans to minimize downtime.
    pub fn diff(old_plan: &DeploymentPlan, new_plan: &DeploymentPlan) -> PlanDiff {
        let mut diff = PlanDiff {
            to_spawn: Vec::new(),
            to_kill: Vec::new(),
            to_reconfigure: Vec::new(),
        };

        // 1. Identify Spawns (New in NewPlan)
        for (id, config) in new_plan.services() {
            if !old_plan.services().contains_key(id) {
                diff.to_spawn.push(config.clone());
            } else {
                // Check for updates
                let old_config = old_plan
                    .services()
                    .get(id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Service '{}' missing from old plan despite check", id)
                    })
                    .expect("Logic error: id checked by contains_key");
                if old_config != config {
                    // Can we just reconfigure?
                    // Rule: If Binary, Env, or ServiceType changed -> Restart (Spawn + Kill old implicitly handled?)
                    // If only Inputs/Outputs changed -> Reconfigure.

                    if Self::can_hot_reload(old_config, config) {
                        diff.to_reconfigure.push((id.clone(), config.clone()));
                    } else {
                        // Must kill and respawn
                        diff.to_kill.push(id.clone());
                        diff.to_spawn.push(config.clone());
                    }
                }
            }
        }

        // 2. Identify Kills (Missing in NewPlan)
        for id in old_plan.services().keys() {
            if !new_plan.services().contains_key(id) {
                diff.to_kill.push(id.clone());
            }
        }

        diff
    }

    // --- Internal Helpers ---

    fn can_hot_reload(old: &ServiceConfig, new: &ServiceConfig) -> bool {
        // We can only hot reload if binary and env are same.
        old.binary_path() == new.binary_path()
            && old.service_type() == new.service_type()
            && old.env() == new.env()
        // Note: Args usually contain ports. If args changed, typically we need restart unless logic parses args.
        // The user said: "You can ask them to change the port... though not the ones they write to".
        // This implies we have a `UpdateAddress` command.
        // Ideally we should check if `inputs` changed.
    }

    fn validate_services(layout: &Layout, catalog: &ServiceCatalog) -> Result<()> {
        for node in layout.nodes() {
            if catalog.get(node.service()).is_none() {
                anyhow::bail!("Service type '{}' not found in registry", node.service());
            }
        }
        Ok(())
    }

    /// Allocates addresses for OUTPUT ports (Sources).
    /// Returns Map<(NodeID, OutputPortName), Address>
    ///
    /// Implements Node-Driven Allocation:
    /// - Iterates over all Nodes in the layout.
    /// - Looks up their Descriptor.
    /// - allocates an address for EVERY output port defined.
    /// - Reuses address if found in `prev_allocations`.
    fn allocate_addresses(
        layout: &Layout,
        catalog: &ServiceCatalog,
        start_port: u16,
        prev_allocations: Option<&HashMap<String, String>>,
    ) -> HashMap<(String, String), String> {
        let mut addresses = HashMap::new();
        let mut current = start_port;

        // 1. Identify all required outputs from the Layout + Catalog
        // We use a Vec to ensure deterministic ordering (sort by NodeId, then PortName)
        let mut required_outputs: Vec<(String, String)> = Vec::new();

        for node in layout.nodes() {
            if let Some(descriptor) = catalog.get(node.service()) {
                for output in &descriptor.outputs {
                    required_outputs.push((node.id().to_string(), output.name.clone()));
                }
            }
        }

        // Sort to ensure deterministic allocation for new ports
        required_outputs.sort();
        required_outputs.dedup();

        // 2. Allocate
        for (node_id, port_name) in required_outputs {
            let composite_key = format!("{}:{}", node_id, port_name);
            let address = if let Some(prev) = prev_allocations.and_then(|m| m.get(&composite_key)) {
                // Reuse existing
                // We MUST ensure `current` skips this if it was generated sequentially?
                // Actually, if we reuse, we risk collision if we reset `current`.
                // For MVP: We just allocate linearly for NEW ones.
                // Ideally, we mark used ports.
                // But for now, let's assume `base_port` is stable.
                // If `prev` exists, we use it.
                prev.clone()
            } else {
                // Allocate new
                // Simplification for MVP: We just increment.
                // Collision risk if we mix preserved with new sequential?
                // Yes. If we preserve 6000, and current starts at 6000, we collide.
                // FIX: Identify MAX used port in prev?
                // OR: Just allocate linearly and check if used?

                // Let's loop until we find a free port.
                // Check if `tcp://127.0.0.1:{current}` is already in `addresses` (from previous iterations)
                // - OR in `prev_allocations` values?

                let mut candidate;
                let mut tries = 0;
                loop {
                    tries += 1;
                    if tries > 5000 {
                        eprintln!("[CRITICAL] Infinite loop detected in allocate_addresses for node {} port {}. Current: {}", node_id, port_name, current);
                        panic!("Infinite loop detected in allocate_addresses");
                    }

                    candidate = format!("tcp://127.0.0.1:{}", current);
                    current += 1;

                    // Check if candidate is already used by a preserved allocation
                    let mut used = false;
                    if let Some(prev) = prev_allocations {
                        if prev.values().any(|addr| addr == &candidate) {
                            used = true;
                            // eprintln!("DEBUG: Port conflict {} is used.", candidate);
                        }
                    }
                    if !used {
                        break;
                    }
                }
                candidate
            };

            addresses.insert((node_id, port_name), address);
        }

        addresses
    }

    fn allocate_admin_addresses(layout: &Layout, start_port: u16) -> HashMap<String, String> {
        let mut addresses = HashMap::new();
        let mut current = start_port;
        for node in layout.nodes() {
            let addr = format!("tcp://0.0.0.0:{}", current);
            addresses.insert(node.id().to_string(), addr);
            current += 1;
        }
        addresses
    }

    fn resolve_node_config(
        node: &Node,
        layout: &Layout,
        descriptor: &ServiceDescriptor,
        output_addresses: &HashMap<(String, String), String>, // Keyed by Source (Output)
        admin_addresses: &HashMap<String, String>,
    ) -> Result<ServiceConfig> {
        let mut bindings = ServiceBindings::new();

        // 1. Resolve Inputs
        // For each input port of this node, find the edge connected to it.
        // Then look up the address of that edge's SOURCE.
        for port in &descriptor.inputs {
            // Find edge where target == node.id and target_port == port.name
            if let Some(edge) = layout
                .edges()
                .iter()
                .find(|e| e.target() == node.id() && e.target_port() == port.name)
            {
                let source_key = (edge.source().to_string(), edge.source_port().to_string());
                if let Some(addr) = output_addresses.get(&source_key) {
                    // Create a Binding::Single
                    // We need a dummy ID for Source? Or use something real?
                    // In `trading-core::manifest::Source`, id is usize.
                    // But Node IDs are Strings.
                    // MVP hack: Hash the string to usize.
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    use std::hash::{Hash, Hasher};
                    edge.source().hash(&mut hasher);
                    let source_id = hasher.finish() as usize;

                    let binding = Binding::Single(Source {
                        address: trading_core::comms::Address::from_str(addr).unwrap(),
                        id: source_id,
                    });

                    bindings.inputs.insert(port.name.clone(), binding);
                } else {
                    // Allocator failed?
                    anyhow::bail!("No address allocated for source {:?}", source_key);
                }
            } else if port.required {
                anyhow::bail!(
                    "Node '{}' missing connection for required input '{}'",
                    node.id(),
                    port.name
                );
            }
        }

        // 2. Resolve Outputs
        // For each output port of this node, find if we allocated an address (if it's used).
        // Since we allocate for all Used outputs, just checking map is enough.
        // Actually, verify against descriptor.outputs?
        for port in &descriptor.outputs {
            let key = (node.id().to_string(), port.name.clone());
            if let Some(addr) = output_addresses.get(&key) {
                // For Output Binding, ID is usually *MY* ID?
                // Whatever, let's use my ID.
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::{Hash, Hasher};
                node.id().hash(&mut hasher);
                let my_id = hasher.finish() as usize;

                let binding = Binding::Single(Source {
                    address: trading_core::comms::Address::from_str(addr).unwrap(),
                    id: my_id,
                });
                bindings.outputs.insert(port.name.clone(), binding);
            }
        }

        // 3. Admin Address Injection
        // We must add the "admin" input binding so trading-core can init the AdminListener
        if let Some(admin_addr) = admin_addresses.get(node.id()) {
            let binding = Binding::Single(Source {
                address: trading_core::comms::Address::from_str(admin_addr).unwrap(),
                id: 0, // Admin doesn't track source IDs
            });
            bindings.inputs.insert("admin".to_string(), binding);
        }

        // 3. Admin (Not in ServiceBindings yet, handled separately or via explicit port?)
        // The ServiceBindings struct in trading-core might not have an 'admin' field.
        // But for now we rely on the implementation detail that `ServiceConfig` has inputs/outputs maps suitable for legacy start?
        // Wait, `ServiceConfig` args are passed to process.
        // We actally want to pass the JSON `--bindings`.

        use std::str::FromStr; // For Address::from_str

        // Generate a numeric ID by hashing the string ID for core compatibility
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        node.id().hash(&mut hasher);
        let service_id_numeric = hasher.finish() as usize;

        // Args generation
        let args = vec![
            "run".to_string(),
            "--service-name".to_string(),
            node.id().to_string(),
            "--service-id".to_string(),
            service_id_numeric.to_string(),
            "--bindings".to_string(),
            serde_json::to_string(&bindings).unwrap(),
            "--config-dir".to_string(),
            "./config".to_string(),
            "--data-dir".to_string(),
            "./data".to_string(),
        ];

        // Add Admin Port as a separate arg if needed, OR put it in bindings if we define a standard "admin" input?
        // Let's stick to standard args for now.

        Ok(ServiceConfig::new(
            node.id().to_string(),
            node.service().to_string(),
            descriptor
                .binary_path
                .clone()
                .unwrap_or_else(|| "TODO_BINARY_PATH".to_string()),
            args,
            HashMap::new(),
            admin_addresses.get(node.id()).cloned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orchestrator_protocol::model::{PortInfo, ServiceDescriptor};
    use std::collections::HashMap;

    #[test]
    fn test_resolve_node_config_bindings() {
        // 1. Setup Data
        let node_id = "strat_1";
        let node = Node::new(
            node_id.to_string(),
            "MyStrategy".to_string(),
            "MomentumStrategy".to_string(),
            "Stopped".to_string(),
        );

        // We need to put edges in Layout to make `resolve_node_config` find the connection.
        let mut layout = Layout::new("test_layout");
        layout.add_edge(orchestrator_protocol::model::Edge::new(
            "e1".to_string(),
            "feeder_1".to_string(),
            "ticks".to_string(),
            "strat_1".to_string(),
            "market_data".to_string(),
        ));

        // Descriptor matches what we expect
        let descriptor = ServiceDescriptor {
            service: "MomentumStrategy".to_string(),
            description: "Test Strat".to_string(),
            version: "0.1.0".to_string(),
            inputs: vec![PortInfo {
                name: "market_data".to_string(),
                data_type: "Tick".to_string(),
                required: true,
                is_variadic: false,
            }],
            outputs: vec![PortInfo {
                name: "orders".to_string(),
                data_type: "Order".to_string(),
                required: false,
                is_variadic: false,
            }],
            binary_path: Some("/bin/strat".to_string()),
        };

        let mut output_addresses = HashMap::new();
        // Address for the Source of the input (Feeder)
        output_addresses.insert(
            ("feeder_1".to_string(), "ticks".to_string()),
            "tcp://127.0.0.1:6000".to_string(),
        );
        // Address for the Output of this node (Strategy)
        output_addresses.insert(
            ("strat_1".to_string(), "orders".to_string()),
            "tcp://127.0.0.1:6001".to_string(),
        );

        let admin_addresses = HashMap::new();

        // 3. Execute
        let config = LayoutEngine::resolve_node_config(
            &node,
            &layout,
            &descriptor,
            &output_addresses,
            &admin_addresses,
        )
        .expect("Resolution failed");

        // 4. Assert
        println!("Generated Args: {:?}", config.args());

        // Parse the CLI arg for bindings
        let bindings_flag_index = config
            .args()
            .iter()
            .position(|r| r == "--bindings")
            .expect("Missing --bindings flag");
        let json_str = &config.args()[bindings_flag_index + 1];
        let bindings: ServiceBindings =
            serde_json::from_str(json_str).expect("Failed to parse JSON");

        // Check Input
        let input = bindings
            .inputs
            .get("market_data")
            .expect("Missing input binding");
        match input {
            Binding::Single(src) => {
                assert_eq!(src.address.to_string(), "zmq:tcp://127.0.0.1:6000");
                // ID depends on hash of "feeder_1"
            }
            _ => panic!("Expected Single binding"),
        }

        // Check Output
        let output = bindings
            .outputs
            .get("orders")
            .expect("Missing output binding");
        match output {
            Binding::Single(src) => {
                assert_eq!(src.address.to_string(), "zmq:tcp://127.0.0.1:6001");
            }
            _ => panic!("Expected Single binding"),
        }
    }
}
