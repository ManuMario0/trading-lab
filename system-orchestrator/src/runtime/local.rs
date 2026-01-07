use crate::layout::model::ServiceConfig;
use crate::runtime::client::AdminClient;
use crate::runtime::traits::{HealthStatus, ServiceProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use libc;
use log::{info, warn};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use trading_core::admin::command::AdminCommand; // Explicit import to be safe

/// Manages processes on the local machine using `std::process`.
pub struct LocalServiceProvider {
    /// Map NodeId -> ProcessMetadata
    processes: Arc<Mutex<HashMap<String, ProcessMetadata>>>,
}

struct ProcessMetadata {
    pid: u32,
    admin_addr: Option<String>,
}

impl LocalServiceProvider {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ServiceProvider for LocalServiceProvider {
    async fn spawn(&self, config: &ServiceConfig) -> Result<()> {
        let mut map = self.processes.lock().unwrap();
        if map.contains_key(config.node_id()) {
            info!(
                "Runtime: Service '{}' already known running.",
                config.node_id()
            );
            return Ok(());
        }

        info!(
            "Runtime: Spawning '{}' ({})",
            config.node_id(),
            config.service_type()
        );

        let mut cmd = Command::new(config.binary_path());
        cmd.args(config.args());
        cmd.envs(config.env());

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let child = cmd.spawn().context("Failed to spawn process")?;
        let pid = child.id();

        let admin_addr = config.admin_api().cloned();

        // Log the admin addr for debugging
        if let Some(ref addr) = admin_addr {
            info!(
                "Runtime: '{}' (PID {}) Admin Access at {}",
                config.node_id(),
                pid,
                addr
            );
        }

        map.insert(
            config.node_id().to_string(),
            ProcessMetadata { pid, admin_addr },
        );

        Ok(())
    }

    async fn stop(&self, id: &str) -> Result<()> {
        let metadata_opt = { self.processes.lock().unwrap().remove(id) };

        if let Some(meta) = metadata_opt {
            let pid = meta.pid;

            // 1. Try Graceful (Admin)
            if let Some(addr) = meta.admin_addr {
                info!(
                    "Runtime: Attempting graceful shutdown for '{}' within 500ms at {}",
                    id, addr
                );
                match AdminClient::new(&addr) {
                    Ok(mut client) => {
                        if let Err(e) = client.send_command(AdminCommand::Shutdown).await {
                            warn!(
                                "Runtime: Graceful shutdown command failed for '{}': {}",
                                id, e
                            );
                        } else {
                            // Wait 500ms for it to exit on its own
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    Err(e) => warn!("Runtime: Failed to connect to admin for '{}': {}", id, e),
                }
            }

            // 2. Force Kill (Safety Net)
            // We check if it's still running using kill -0
            let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };
            if is_alive {
                info!("Runtime: Force killing PID {}", pid);
                #[cfg(unix)]
                {
                    let _ = Command::new("kill").arg(pid.to_string()).output();
                }
            } else {
                info!("Runtime: Service '{}' exited gracefully.", id);
            }
        }
        Ok(())
    }

    async fn probe(&self, id: &str) -> Result<HealthStatus> {
        let map = self.processes.lock().unwrap();
        if let Some(meta) = map.get(id) {
            let pid = meta.pid;
            let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };

            if is_alive {
                return Ok(HealthStatus::Running(pid));
            } else {
                return Ok(HealthStatus::Failed("PID not found".into()));
            }
        }
        Ok(HealthStatus::Stopped)
    }

    async fn list(&self) -> Result<Vec<String>> {
        let map = self.processes.lock().unwrap();
        Ok(map.keys().cloned().collect())
    }
}
