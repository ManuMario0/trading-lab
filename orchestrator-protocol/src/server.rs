use crate::messages::{OrchestratorCommand, OrchestratorResponse};
use anyhow::Result;
use trading_core::comms::transport::TransportDuplex;

pub struct OrchestratorServer {
    transport: Box<dyn TransportDuplex>,
}

impl OrchestratorServer {
    pub fn new(transport: Box<dyn TransportDuplex>) -> Self {
        Self { transport }
    }

    pub async fn next_command(&mut self) -> Result<OrchestratorCommand> {
        let msg = self.transport.recv_bytes().await?;
        let command: OrchestratorCommand = serde_json::from_slice(&msg)?;
        Ok(command)
    }

    pub async fn send_response(&mut self, response: OrchestratorResponse) -> Result<()> {
        let msg = serde_json::to_vec(&response)?;
        self.transport.send_bytes(&msg).await?;
        Ok(())
    }
}
