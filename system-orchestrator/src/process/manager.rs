use anyhow::Result;
use log::{error, info, warn};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::process::Stdio;
use tokio::process::Command;

use super::models::config::ProcessConfig;
use super::models::port_allocator::{Port, PortAllocator};
use super::models::state::{ProcessInfo, RunningProcess};
use crate::config::SystemConfig;
use crate::layout::models::layout::Layout;
use crate::layout::models::node::Node;

/// Represents the runtime state of a single deployed Layout
pub struct ProcessGroup {
    pub layout_id: String,
    pub processes: HashMap<String, RunningProcess>, // NodeID -> Process
    pub port_allocator: PortAllocator,
    pub active_edges: HashMap<String, Port>, // EdgeKey -> Port
}

impl ProcessGroup {
    pub fn new(layout_id: String) -> Self {
        Self {
            layout_id,
            processes: HashMap::new(),
            port_allocator: PortAllocator::new(1024, 2048),
            active_edges: HashMap::new(),
        }
    }
}

pub struct ProcessManager {
    // Map LayoutID -> ProcessGroup
    groups: HashMap<String, ProcessGroup>,
    system_config: SystemConfig,
}

impl ProcessManager {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            groups: HashMap::new(),
            system_config: config,
        }
    }

    // --- Runtime Lifecycle ---

    pub fn spawn(&mut self, layout_id: &str, node_id: String, config: ProcessConfig) -> Result<()> {
        info!(
            "Spawning [{}::{}] {} {:?}",
            layout_id,
            config.name(),
            config.cmd(),
            config.args()
        );

        // Ensure group exists (should be created by deploy, but good for safety)
        let group = self
            .groups
            .entry(layout_id.to_string())
            .or_insert_with(|| ProcessGroup::new(layout_id.to_string()));

        // Calculate Hash
        let mut hasher = DefaultHasher::new();
        config.cmd().hash(&mut hasher);
        config.args().hash(&mut hasher);
        let config_hash = hasher.finish();

        let child = Command::new(config.cmd())
            .args(config.args())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn();

        match child {
            Ok(c) => {
                info!(
                    "[{}] started successfully (PID: {:?})",
                    config.name(),
                    c.id()
                );
                if let Some(pid) = c.id() {
                    let running = RunningProcess::new(
                        node_id.clone(),
                        config.category().to_string(),
                        pid,
                        c,
                        config_hash,
                        config.admin_port(),
                    );
                    group.processes.insert(node_id, running);
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to spawn [{}]: {}", config.name(), e);
                Err(anyhow::anyhow!("Failed to spawn {}: {}", config.name(), e))
            }
        }
    }

    pub fn stop(&mut self, layout_id: &str, node_id: &str) -> Result<()> {
        if let Some(group) = self.groups.get_mut(layout_id) {
            if let Some(mut proc) = group.processes.remove(node_id) {
                info!(
                    "Stopping process [{}/{}] (PID: {})...",
                    layout_id, node_id, proc.pid
                );
                let _ = proc.child.kill();
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Process {} not found in layout {}",
                    node_id,
                    layout_id
                ))
            }
        } else {
            Err(anyhow::anyhow!("Layout {} not found", layout_id))
        }
    }

    // List all processes across all layouts? Or specific?
    // Let's list all flattened for now to keep API compat where possible,
    // or maybe scoped. The existing API caller expected a flat list.
    pub fn list(&self) -> Vec<ProcessInfo> {
        let mut list = Vec::new();
        for group in self.groups.values() {
            for (id, proc) in &group.processes {
                list.push(ProcessInfo {
                    id: id.clone(),
                    status: "Running".to_string(),
                });
            }
        }
        list
    }

    pub fn get_engine(&self, layout_id: &str) -> Option<&RunningProcess> {
        if let Some(group) = self.groups.get(layout_id) {
            for proc in group.processes.values() {
                if proc.category() == "ExecutionEngine" {
                    return Some(proc);
                }
            }
        }
        None
    }

    // --- Reconciliation (Deploy) ---

    pub fn deploy(&mut self, layout: &Layout) -> Result<()> {
        let layout_id = layout.id();
        info!("Reconciling Runtime for Layout [{}]...", layout_id);

        // Get or Create Group
        // We need to be careful about ownership here.
        // We can extract the group to work on it, then put it back?
        // Or just work via mutable reference.

        // Note: To make the borrow checker happy when calling self.resolve_config inside loop,
        // we might need to separate the group data from 'self'.
        // But resolve_config mostly needs SystemConfig.

        let groups = &mut self.groups;
        let group = groups
            .entry(layout_id.to_string())
            .or_insert_with(|| ProcessGroup::new(layout_id.to_string()));

        // 1. Resolve Ports (Within this Group)
        let mut next_allocator = PortAllocator::new_from_allocator(&group.port_allocator);

        let mut new_active_edges = HashMap::new();
        let mut edge_to_port = HashMap::new();

        for edge in layout.edges() {
            let key = format!("{}:{}", edge.source(), edge.target());
            let port = if let Some(p) = group.active_edges.get(&key) {
                *p
            } else {
                next_allocator.allocate().unwrap_or(0)
            };

            if port != 0 {
                new_active_edges.insert(key.clone(), port);
                edge_to_port.insert(key, port);
            }
        }

        // Commit Allocator (simplified, assuming monotonic growth or perfect reconstruction)
        // Actually, let's just create a fresh allocator based on reserved ports
        let mut final_allocator = PortAllocator::new(1024, 2048);
        for (_, port) in &new_active_edges {
            final_allocator.reserve(*port);
        }
        // *Issue*: resolve_config needs to allocate *more* ports (admin ports) dynamically.
        // So we should pass 'final_allocator' to resolve_config?
        // Previously verify_config called the allocator on 'self'.

        // Let's temporary swap the group's allocator so helper can use it?
        // Or refactor helper. Helper calls `self.port_allocator.allocate()`.

        // Let's refactor resolve_config to take the allocator as generic or mutable arg.

        // 2. Resolve Configs
        let mut desired_configs = HashMap::new();

        // We will do allocation inline here for admin ports
        for node in layout.nodes() {
            // Note: self.resolve_config now requires &mut self to mutate internal config/allocator if needed?
            // Actually, we passed allocator explicitly.
            // Issue: 'resolve_config' might have been defined as mutable or not.
            // Let's check signature.

            if let Some((cmd, args, admin)) = ProcessManager::resolve_config(
                &self.system_config,
                node,
                &edge_to_port,
                layout,
                &mut final_allocator,
            ) {
                desired_configs.insert(
                    node.id().to_string(),
                    ProcessConfig::new(
                        node.name().to_string(),
                        node.category().to_string(),
                        cmd,
                        args,
                        admin,
                    ),
                );
            }
        }

        // 3. Diff & Execute
        // Key: NodeID
        let mut to_stop = Vec::new();

        // Check running processes in this group
        for (id, proc) in &group.processes {
            if let Some(new_conf) = desired_configs.get(id) {
                let mut hasher = DefaultHasher::new();
                new_conf.cmd().hash(&mut hasher);
                new_conf.args().hash(&mut hasher);
                let new_hash = hasher.finish();

                if proc.config_hash != new_hash {
                    info!("Config changed for {} in layout {}", id, layout_id);
                    to_stop.push(id.clone());
                }
            } else {
                info!("Node {} removed from layout {}", id, layout_id);
                to_stop.push(id.clone());
            }
        }

        // Stop
        for id in to_stop {
            if let Some(mut proc) = group.processes.remove(&id) {
                let _ = proc.child.kill();
            }
        }

        // Start
        for (id, config) in desired_configs {
            if !group.processes.contains_key(&id) {
                // Inline Spawn logic to avoid fighting borrow checker with `self.spawn` while holding grouped ref
                // Or just Clone config and do it after?

                // Let's clone what we need to spawn and do it here
                // Or carefully re-use code.
                // We'll duplicate the spawn logic slightly to avoid 'self' borrow issues.

                // Hash
                let mut hasher = DefaultHasher::new();
                config.cmd().hash(&mut hasher);
                config.args().hash(&mut hasher);
                let config_hash = hasher.finish();

                let child = Command::new(config.cmd())
                    .args(config.args())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .kill_on_drop(true)
                    .spawn();

                match child {
                    Ok(c) => {
                        info!("[{}] started (PID: {:?})", config.name(), c.id());
                        if let Some(pid) = c.id() {
                            let running = RunningProcess::new(
                                id.clone(),
                                config.category().to_string(),
                                pid,
                                c,
                                config_hash,
                                config.admin_port(),
                            );
                            group.processes.insert(id.clone(), running);
                        }
                    }
                    Err(e) => error!("Failed to spawn {}: {}", config.name(), e),
                }
            }
        }

        // 4. Commit State
        group.port_allocator = final_allocator;
        group.active_edges = new_active_edges;

        Ok(())
    }

    // --- Helper: Blueprint -> Runtime Config ---
    // Now static-like or just using system_config, + mutable allocator
    // --- Helper: Blueprint -> Runtime Config ---
    // Made associated function to avoid borrowing issues
    fn resolve_config(
        system_config: &SystemConfig,
        node: &Node,
        edge_ports: &HashMap<String, u16>,
        layout: &Layout,
        allocator: &mut PortAllocator,
    ) -> Option<(String, Vec<String>, u16)> {
        let mut args = Vec::new();

        let find_port = |is_target: bool| -> Option<u16> {
            for edge in layout.edges() {
                let matches = if is_target {
                    edge.target() == node.id()
                } else {
                    edge.source() == node.id()
                };
                if matches {
                    let key = format!("{}:{}", edge.source(), edge.target());
                    if let Some(p) = edge_ports.get(&key) {
                        return Some(*p);
                    }
                }
            }
            None
        };

        let admin = allocator.allocate().unwrap(); // Use the passed allocator

        match node.category() {
            "Strategy" => {
                let input = find_port(true).unwrap_or(system_config.data_port);
                let output = find_port(false).unwrap_or(system_config.multiplexer_input_port);

                args.push(format!("tcp://127.0.0.1:{}", input));
                args.push(format!("tcp://127.0.0.1:{}", output));
                args.push(format!("tcp://*:{}", admin));
                Some((system_config.strategy_lab_path.clone(), args, admin))
            }
            "Multiplexer" => {
                let input = find_port(true).unwrap_or(system_config.data_port);
                let output = find_port(false).unwrap_or(system_config.multiplexer_input_port);

                args.push("--input-port".to_string());
                args.push(input.to_string());
                args.push("--output-port".to_string());
                args.push(output.to_string());
                args.push("--admin-port".to_string());
                args.push(admin.to_string());
                Some((system_config.multiplexer_path.clone(), args, admin))
            }
            "ExecutionEngine" => {
                args.push("--admin-port".to_string());
                args.push(admin.to_string());
                args.push("--multiplexer-ports".to_string());
                args.push(system_config.multiplexer_port.to_string());
                args.push("--data-port".to_string());
                args.push(system_config.data_port.to_string());
                args.push("--order-port".to_string());
                args.push(system_config.order_port.to_string());
                Some((system_config.execution_engine_path.clone(), args, admin))
            }
            "DataPipeline" => {
                args.push("-u".to_string());
                args.push(system_config.data_pipeline_path.clone());
                args.push("--port".to_string());
                args.push(system_config.data_port.to_string());
                Some(("python3".to_string(), args, admin))
            }
            "Gateway" => Some((system_config.gateway_paper_path.clone(), args, admin)),
            _ => None,
        }
    }

    pub async fn check_status(&mut self) {
        for group in self.groups.values_mut() {
            let mut finished = Vec::new();
            for (id, proc) in group.processes.iter_mut() {
                match proc.child.try_wait() {
                    Ok(Some(status)) => {
                        warn!(
                            "Process {} in layout {} exited: {}",
                            id, group.layout_id, status
                        );
                        finished.push(id.clone());
                    }
                    Ok(None) => {}
                    Err(e) => error!("Wait error {}: {}", id, e),
                }
            }
            for id in finished {
                group.processes.remove(&id);
            }
        }
    }
}
