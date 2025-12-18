use serde::{Deserialize, Serialize};
use tokio::process::Child;

#[derive(Debug)]
pub struct RunningProcess {
    pub id: String, // UUID matching Layout Node ID
    pub pid: u32,
    pub child: Child,
    pub config_hash: u64, // For diffing
}

impl RunningProcess {
    pub fn new(id: String, pid: u32, child: Child, config_hash: u64) -> Self {
        Self {
            id,
            pid,
            child,
            config_hash,
        }
    }
}

// Public Info struct for API responses (Detached from Child)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub status: String,
}
