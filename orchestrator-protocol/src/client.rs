use crate::messages::{OrchestratorCommand, OrchestratorResponse};
use anyhow::Result;
use trading_core::comms::transport::TransportDuplex;

pub struct OrchestratorClient {
    transport: Box<dyn TransportDuplex>,
}

impl OrchestratorClient {
    pub fn new(transport: Box<dyn TransportDuplex>) -> Self {
        Self { transport }
    }

    pub async fn send_command(
        &mut self,
        command: OrchestratorCommand,
    ) -> Result<OrchestratorResponse> {
        let msg = serde_json::to_vec(&command)?;
        self.transport.send_bytes(&msg).await?;

        let response_bytes = self.transport.recv_bytes().await?;
        let response: OrchestratorResponse = serde_json::from_slice(&response_bytes)?;

        Ok(response)
    }
}
