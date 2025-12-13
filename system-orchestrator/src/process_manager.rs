use std::process::Stdio;
use tokio::process::{Child, Command};
use log::{info, error, warn};
use anyhow::Result;

pub struct ProcessManager {
    children: Vec<(String, Child)>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn spawn(&mut self, name: &str, cmd_path: &str, args: &[&str]) -> Result<()> {
        info!("Spawning [{}]: {} {:?}", name, cmd_path, args);

        // We use tokio::process::Command
        let child = Command::new(cmd_path)
            .args(args)
            .stdout(Stdio::inherit()) // For now, just pipe to orchestrator stdout
            .stderr(Stdio::inherit())
            .kill_on_drop(true)       // Killing orchestrator kills children
            .spawn();

        match child {
            Ok(c) => {
                info!("[{}] started successfully.", name);
                self.children.push((name.to_string(), c));
                Ok(())
            }
            Err(e) => {
                error!("Failed to spawn [{}]: {}", name, e);
                // We might not want to panic, but return error
                Err(anyhow::anyhow!("Failed to spawn {}: {}", name, e))
            }
        }
    }

    pub async fn check_status(&mut self) {
        // Simple health check: see if any have exited
        // Note: try_wait() is non-blocking
        let mut finished_indices = Vec::new();

        for (i, (name, child)) in self.children.iter_mut().enumerate() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    warn!("Process [{}] exited with: {}", name, status);
                    finished_indices.push(i);
                }
                Ok(None) => {
                    // Still running
                }
                Err(e) => {
                    error!("Error attempting to wait on [{}]: {}", name, e);
                }
            }
        }
        
        // Remove finished children
        // Reverse sort indices to remove safely
        finished_indices.sort_by(|a, b| b.cmp(a));
        for i in finished_indices {
            self.children.remove(i);
        }
    }
}
