use crate::messages::{OrchestratorCommand, OrchestratorResponse};
use crate::model::Layout;
use anyhow::{Context, Result};
use trading_core::comms::transport::TransportDuplex;
use trading_core::comms::transports::zmq::ZmqClientDuplex;

pub struct OrchestratorClient {
    transport: ZmqClientDuplex,
}

impl OrchestratorClient {
    pub fn new(address: &str) -> Result<Self> {
        let transport = ZmqClientDuplex::new(address)?;
        Ok(Self { transport })
    }

    pub async fn deploy(&mut self, layout: Layout, mode: crate::model::RunMode) -> Result<String> {
        let cmd = OrchestratorCommand::Deploy { layout, mode };
        match self.send_command(cmd).await? {
            OrchestratorResponse::Success(msg) => Ok(msg),
            OrchestratorResponse::Error(e) => anyhow::bail!("Orchestrator Error: {}", e),
            _ => anyhow::bail!("Unexpected response type"),
        }
    }

    pub async fn stop(&mut self, layout_id: String) -> Result<String> {
        let cmd = OrchestratorCommand::Stop { layout_id };
        match self.send_command(cmd).await? {
            OrchestratorResponse::Success(msg) => Ok(msg),
            OrchestratorResponse::Error(e) => anyhow::bail!("Orchestrator Error: {}", e),
            _ => anyhow::bail!("Unexpected response type"),
        }
    }

    async fn send_command(&mut self, cmd: OrchestratorCommand) -> Result<OrchestratorResponse> {
        let req_bytes = bincode::serialize(&cmd).context("Failed to serialize command")?;
        self.transport.send_bytes(&req_bytes).await?;

        let resp_bytes = self
            .transport
            .recv_bytes()
            .await
            .context("Failed to receive response")?;
        let resp: OrchestratorResponse =
            bincode::deserialize(&resp_bytes).context("Failed to deserialize response")?;
        Ok(resp)
    }
}
