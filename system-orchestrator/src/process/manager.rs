use anyhow::Result;
use log::{error, info, warn};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::process::Stdio;
use tokio::process::Command;

use super::config_resolver::ConfigResolver;
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
    pub active_edges: HashMap<String, Port>, // EdgeKey -> Port (Legacy/Active)
    pub output_registry: HashMap<String, u16>, // Persisted Output Assignments
}

impl ProcessGroup {
    pub fn new(layout_id: String) -> Self {
        Self {
            layout_id,
            processes: HashMap::new(),
            port_allocator: PortAllocator::new(1024, 2048),
            active_edges: HashMap::new(),
            output_registry: HashMap::new(),
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

    pub fn list(&self) -> Vec<ProcessInfo> {
        let mut list = Vec::new();
        for group in self.groups.values() {
            for (id, proc) in &group.processes {
                list.push(ProcessInfo {
                    id: id.clone(),
                    name: proc.category.clone(), // Using category as name for now
                    status: "Running".to_string(),
                    cpu_usage: 0.0,
                    memory_usage: 0,
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

    pub fn check_status(&mut self) {
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

    // --- Reconciliation (Deploy) ---

    pub fn deploy(&mut self, layout: &Layout) -> Result<()> {
        let layout_id = layout.id();
        info!("Reconciling Runtime for Layout [{}]...", layout_id);

        let groups = &mut self.groups;
        let group = groups
            .entry(layout_id.to_string())
            .or_insert_with(|| ProcessGroup::new(layout_id.to_string()));

        // Use standard PortAllocator logic
        // We recreate allocator to ensure clean state based on layout requirements
        // Assuming no persistence of ports across restart of Orchestrator?
        // Or we should use group.port_allocator?
        // Let's use group.port_allocator but maybe reset it if we fully re-deploy?
        // For MVP, lets create a fresh one.

        let mut allocator = PortAllocator::new(1024, 2048);
        // Note: New config resolver uses System default range or we should check SystemConfig?
        // Hardcoded 1024-3072 in allocator new?

        let mut output_registry = HashMap::new();

        // Pass 1: Allocate Outputs
        for node in layout.nodes() {
            ConfigResolver::allocate_outputs(node, &mut allocator, &mut output_registry);
        }

        // Pass 2: Resolve Arguments
        let mut desired_configs = HashMap::new();

        for node in layout.nodes() {
            if let Some((cmd, args, admin)) = ConfigResolver::resolve_args(
                &self.system_config,
                node,
                layout,
                &mut allocator,
                &output_registry,
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
                // Inline Spawn logic: Duplicate from self.spawn to avoid borrow issues
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
        group.port_allocator = allocator;
        group.output_registry = output_registry;
        // active_edges is deprecated/unused in new resolver logic but kept for struct compat

        Ok(())
    }
}
