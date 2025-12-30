use serde::{Deserialize, Serialize};
use tokio::process::Child;

#[derive(Debug)]
pub struct RunningProcess {
    pub id: String, // UUID matching Layout Node ID
    pub category: String,
    pub pid: u32,
    pub child: Child,
    pub config_hash: u64, // For diffing
    pub admin_port: u16,
}

impl RunningProcess {
    pub fn new(
        id: String,
        category: String,
        pid: u32,
        child: Child,
        config_hash: u64,
        admin_port: u16,
    ) -> Self {
        Self {
            id,
            category,
            pid,
            child,
            config_hash,
            admin_port,
        }
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn send_command(&self, cmd: &str) -> anyhow::Result<String> {
        // Create a temporary ZeroMQ REQ socket to talk to this process
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;
        let addr = format!("tcp://127.0.0.1:{}", self.admin_port);
        socket.connect(&addr)?;

        socket.send(cmd, 0)?;
        let reply = socket
            .recv_string(0)?
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        Ok(reply)
    }
}

// Public Info struct for API responses (Detached from Child)
pub use orchestrator_protocol::ProcessInfo;
