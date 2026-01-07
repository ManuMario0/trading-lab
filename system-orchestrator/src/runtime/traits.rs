use crate::layout::model::ServiceConfig;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Running(u32), // PID
    Stopped,
    Failed(String),
}

/// The abstraction for "The Hands".
/// Implement this for LocalProcess, Docker, SSH, etc.
#[async_trait]
pub trait ServiceProvider: Send + Sync {
    /// Spawns a new service instance based on the config.
    /// Should be idempotent (if already running with same config, do nothing).
    async fn spawn(&self, config: &ServiceConfig) -> Result<()>;

    /// Gracefully stops the service.
    /// 1. Send Admin Shutdown command.
    /// 2. Wait.
    /// 3. Force Kill if necessary.
    async fn stop(&self, id: &str) -> Result<()>;

    /// Checks the health of the service.
    /// This should include:
    /// - Is the PID alive?
    /// - Can we ping the Admin port? (If applicable)
    async fn probe(&self, id: &str) -> Result<HealthStatus>;

    /// List all known services managed by this provider.
    /// Used for orphan detection (reconciliation).
    async fn list(&self) -> Result<Vec<String>>;
}
