use crate::model::{Layout, ProcessInfo, RunMode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum OrchestratorCommand {
    Deploy { layout: Layout, mode: RunMode },
    Stop { layout_id: String },
    GetStatus,
    GetWallet { layout_id: String },
    Shutdown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OrchestratorResponse {
    Success(String),
    StatusInfo(Vec<ProcessInfo>),
    WalletInfo(serde_json::Value),
    Error(String),
}
