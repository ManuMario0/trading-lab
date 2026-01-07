use anyhow::{Context, Result};
use trading_core::admin::command::{AdminCommand, AdminPayload, AdminResponse};
// Explicit full path import
use trading_core::comms::transport::TransportDuplex;
use trading_core::comms::transports::zmq::ZmqClientDuplex;

/// Helper to talk to a running microservice via its Admin Port.
/// This is the "Voice" of the Orchestrator.
pub struct AdminClient {
    transport: ZmqClientDuplex,
}

impl AdminClient {
    pub fn new(address: &str) -> Result<Self> {
        // ZMQ Connect
        let transport =
            ZmqClientDuplex::new(address).context("Failed to create ZmqClientDuplex")?;
        Ok(Self { transport })
    }

    /// Sends a command and expects a response.
    pub async fn send_command(&mut self, cmd: AdminCommand) -> Result<AdminResponse> {
        let payload = AdminPayload::new_command(cmd);

        // Manual Serialization (bincode)
        let bytes = bincode::serialize(&payload).context("Failed to serialize AdminPayload")?;

        // Use TransportDuplex trait methods
        self.transport
            .send_bytes(&bytes)
            .await
            .context("Failed to send admin command")?;

        // Explicitly typed to force compiler resolution
        let response_result: anyhow::Result<Vec<u8>> = self.transport.recv_bytes().await;
        let response_bytes = response_result.context("Failed to receive admin response")?;

        let response: AdminPayload =
            bincode::deserialize(&response_bytes).context("Failed to deserialize AdminPayload")?;

        match response {
            AdminPayload::Response(r) => Ok(r),
            AdminPayload::Command(_) => {
                anyhow::bail!("Received Command on Client port, expected Response")
            }
        }
    }
}
