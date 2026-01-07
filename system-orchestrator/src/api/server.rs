use crate::event_bus::{EventBus, SystemEvent};
use anyhow::Result;
use log::{error, info};
use orchestrator_protocol::messages::{OrchestratorCommand, OrchestratorResponse};
use trading_core::comms::transport::TransportDuplex;
use trading_core::comms::transports::zmq::ZmqDuplex;

pub struct ApiServer {
    transport: ZmqDuplex,
    event_bus: EventBus,
}

impl ApiServer {
    pub fn new(bind_address: &str, event_bus: EventBus) -> Result<Self> {
        let transport = ZmqDuplex::new(bind_address)?;
        Ok(Self {
            transport,
            event_bus,
        })
    }

    pub async fn run(&mut self) {
        info!("API Server: Listening for commands...");
        loop {
            match self.transport.recv_bytes().await {
                Ok(bytes) => {
                    match bincode::deserialize::<OrchestratorCommand>(&bytes) {
                        Ok(cmd) => {
                            let response = self.handle_command(cmd);
                            if let Err(e) = self.send_response(response).await {
                                error!("API Server: Failed to send response: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("API Server: Failed to deserialize command: {}", e);
                            // Try to send generic error
                            let _ = self
                                .send_response(OrchestratorResponse::Error(
                                    "Invalid Protocol Format".into(),
                                ))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    error!("API Server: Transport error: {}", e);
                    // Minimal failure backoff
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: OrchestratorCommand) -> OrchestratorResponse {
        info!("API Server: Received Command: {:?}", cmd);

        match cmd {
            OrchestratorCommand::Deploy { layout, mode: _ } => {
                // SMR: We turn the external Command into an internal Event.
                self.event_bus
                    .publish(SystemEvent::DeployRequested { layout });
                OrchestratorResponse::Success("Deployment Scheduled".into())
            }
            OrchestratorCommand::Stop { layout_id } => OrchestratorResponse::Error(format!(
                "Stop command not yet implemented for {}",
                layout_id
            )),
            OrchestratorCommand::GetStatus => {
                OrchestratorResponse::Error("GetStatus not yet implemented".into())
            }
            OrchestratorCommand::GetWallet { .. } => {
                OrchestratorResponse::Error("GetWallet not yet implemented".into())
            }
            OrchestratorCommand::Shutdown => {
                // In a real SMR system, Shutdown is an event too.
                info!("API Server: Shutdown Requested (Ignored for now)");
                OrchestratorResponse::Success("Shutdown Ignored".into())
            }
            OrchestratorCommand::GetServices => {
                OrchestratorResponse::Error("GetServices not yet implemented".into())
            }
        }
    }

    async fn send_response(&self, resp: OrchestratorResponse) -> Result<()> {
        let bytes = bincode::serialize(&resp)?;
        self.transport.send_bytes(&bytes).await?;
        Ok(())
    }
}
