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

pub struct ProcessManager {
    processes: HashMap<String, RunningProcess>, // Key: Node ID (UUID)
    port_allocator: PortAllocator,
    active_edges: HashMap<String, Port>, // Key: "source_id:target_id"
    system_config: SystemConfig,
}

impl ProcessManager {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            processes: HashMap::new(),
            port_allocator: PortAllocator::new(1024, 2048), // Wider range
            active_edges: HashMap::new(),
            system_config: config,
        }
    }

    // --- Runtime Lifecycle ---

    pub fn spawn(&mut self, id: String, config: ProcessConfig) -> Result<()> {
        info!(
            "Spawning [{}]: {} {:?}",
            config.name(),
            config.cmd(),
            config.args()
        );

        // Calculate Hash for Diffing later
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
                // For diffing, we need to map id -> PID
                if let Some(pid) = c.id() {
                    let running = RunningProcess::new(id.clone(), pid, c, config_hash);
                    self.processes.insert(id, running);
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to spawn [{}]: {}", config.name(), e);
                Err(anyhow::anyhow!("Failed to spawn {}: {}", config.name(), e))
            }
        }
    }

    pub fn stop(&mut self, id: &str) -> Result<()> {
        if let Some(mut proc) = self.processes.remove(id) {
            info!("Stopping process [{}] (PID: {})...", id, proc.pid);
            // Try graceful kill first? For now hard kill or let Drop handle it?
            // "kill_on_drop(true)" on Command handles it when we drop 'proc.child'.
            // But we can explicitly kill to be sure.
            let _ = proc.child.kill();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Process {} not found", id))
        }
    }

    pub fn list(&self) -> Vec<ProcessInfo> {
        // We'll need to join with Layout ideally to get names if we don't store them in RunningProcess.
        // But RunningProcess only has ID.
        // For this refactor, we simplify. We return what we know.
        // Or we might need to store 'name' and 'category' in RunningProcess too for easy listing?
        // Let's assume the API or LayoutManager supplements this data, OR we add it to RunningProcess.
        // For strict correctness, Runtime only cares about PID.
        // But listing usually needs names.
        // Let's iterate and return PIDs.
        self.processes
            .iter()
            .map(|(id, proc)| ProcessInfo {
                id: id.clone(),
                status: "Running".to_string(),
            })
            .collect()
    }

    // --- Reconciliation (Deploy) ---

    pub fn deploy(&mut self, layout: &Layout) -> Result<()> {
        info!("Reconciling Runtime with Layout...");

        // 1. Resolve Ports (Stable Allocation)
        let mut next_allocator = PortAllocator::new_from_allocator(&self.port_allocator);
        // Reset usage tracking for this pass
        // Actually, we want to KEEP existing allocations if they are still valid in new layout
        // But 'new_allocator' copies state?
        // Let's refine: We need to build the *Desired* port map.
        // If an edge exists in old and new, keep port.
        // If new edge, allocate new port.
        // If old edge removed, port becomes free (by not being in new map).

        let mut new_active_edges = HashMap::new();
        let mut edge_to_port = HashMap::new();

        // Re-verify all desired edges
        for edge in layout.edges() {
            let key = format!("{}:{}", edge.source(), edge.target());
            let port = if let Some(p) = self.active_edges.get(&key) {
                *p // Keep existing
            } else {
                next_allocator.allocate().unwrap_or(0) // Allocate new
            };

            if port != 0 {
                new_active_edges.insert(key.clone(), port);
                edge_to_port.insert(key, port);
            }
        }

        // Update allocator state to match new reality
        // (Simplified: We just use the 'next_allocator' which has tracked allocations?)
        // Wait, if we reused 'p', we didn't mark it used in 'next_allocator' yet?
        // We need a fresh allocator but marked with preserved ports.
        // Let's correct:

        let mut final_allocator = PortAllocator::new(1024, 2048);
        for (_, port) in &new_active_edges {
            final_allocator.reserve(*port);
        }

        // 2. Resolve Configs
        let mut desired_configs = HashMap::new();
        for node in layout.nodes() {
            if let Some((cmd, args)) = self.resolve_config(node, &edge_to_port, layout) {
                desired_configs.insert(
                    node.id().to_string(),
                    ProcessConfig::new(
                        node.name().to_string(),
                        node.category().to_string(),
                        cmd,
                        args,
                    ),
                );
            }
        }

        // 3. Diff & Execute

        // A. Stop removed or changed
        let mut to_stop = Vec::new();
        for (id, proc) in &self.processes {
            if let Some(new_conf) = desired_configs.get(id) {
                // Check Hash
                let mut hasher = DefaultHasher::new();
                new_conf.cmd().hash(&mut hasher);
                new_conf.args().hash(&mut hasher);
                let new_hash = hasher.finish();

                if proc.config_hash != new_hash {
                    info!("Config changed for {}", id);
                    to_stop.push(id.clone());
                }
            } else {
                info!("Node {} removed", id);
                to_stop.push(id.clone());
            }
        }

        for id in to_stop {
            self.stop(&id)?;
        }

        // B. Spawn new or restarted
        for (id, config) in desired_configs {
            if !self.processes.contains_key(&id) {
                self.spawn(id, config)?;
            }
        }

        // 4. Commit State
        self.port_allocator = final_allocator;
        self.active_edges = new_active_edges;

        Ok(())
    }

    // --- Helper: Blueprint -> Runtime Config ---
    fn resolve_config(
        &self,
        node: &Node,
        edge_ports: &HashMap<String, u16>,
        layout: &Layout,
    ) -> Option<(String, Vec<String>)> {
        let mut args = Vec::new();

        // Helper to find ports
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

        match node.category() {
            "Strategy" => {
                let input = find_port(true).unwrap_or(self.system_config.data_port);
                let output = find_port(false).unwrap_or(self.system_config.multiplexer_input_port);
                let admin = self.system_config.strategy_admin_port; // Single strategy MVP

                args.push(format!("tcp://127.0.0.1:{}", input));
                args.push(format!("tcp://127.0.0.1:{}", output));
                args.push(format!("tcp://*:{}", admin));
                Some((self.system_config.strategy_lab_path.clone(), args))
            }
            "Multiplexer" => {
                args.push("--input-port".to_string());
                args.push(self.system_config.multiplexer_input_port.to_string());
                args.push("--output-port".to_string());
                args.push(self.system_config.multiplexer_port.to_string());
                args.push("--admin-port".to_string());
                args.push(self.system_config.multiplexer_admin_port.to_string());
                Some((self.system_config.multiplexer_path.clone(), args))
            }
            "ExecutionEngine" => {
                args.push("--admin-port".to_string());
                args.push(self.system_config.admin_port.to_string());
                args.push("--multiplexer-ports".to_string());
                args.push(self.system_config.multiplexer_port.to_string());
                args.push("--data-port".to_string());
                args.push(self.system_config.data_port.to_string());
                args.push("--order-port".to_string());
                args.push(self.system_config.order_port.to_string());
                Some((self.system_config.execution_engine_path.clone(), args))
            }
            "DataPipeline" => {
                args.push("-u".to_string());
                args.push(self.system_config.data_pipeline_path.clone());
                args.push("--port".to_string());
                args.push(self.system_config.data_port.to_string());
                Some(("python3".to_string(), args))
            }
            "Gateway" => Some((self.system_config.gateway_paper_path.clone(), args)),
            _ => None,
        }
    }

    pub async fn check_status(&mut self) {
        let mut finished = Vec::new();
        for (id, proc) in self.processes.iter_mut() {
            match proc.child.try_wait() {
                Ok(Some(status)) => {
                    warn!("Process {} exited: {}", id, status);
                    finished.push(id.clone());
                }
                Ok(None) => {}
                Err(e) => error!("Wait error {}: {}", id, e),
            }
        }
        for id in finished {
            self.processes.remove(&id);
        }
    }
}
