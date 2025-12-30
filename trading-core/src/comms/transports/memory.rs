use crate::comms::transport::{TransportInput, TransportOutput};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Internal memory transport for testing/threading.
///
/// Implements `TransportInput` using Tokio's MPSC channels.
#[allow(dead_code)]
pub(crate) struct MemoryTransportInput {
    receiver: mpsc::Receiver<Vec<u8>>,
}

impl MemoryTransportInput {
    /// Creates a new MemoryTransportInput.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The receiving end of a channel.
    #[allow(dead_code)]
    pub fn new(receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        Self { receiver }
    }
}

#[async_trait]
impl TransportInput for MemoryTransportInput {
    async fn recv_bytes(&mut self) -> Result<Vec<u8>> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Memory channel closed"))
    }

    async fn try_recv(&mut self) -> Result<Vec<u8>> {
        self.receiver.try_recv().map_err(|err| match err {
            mpsc::error::TryRecvError::Empty => anyhow::anyhow!("Memory channel empty"),
            _ => anyhow::anyhow!("Memory channel closed"),
        })
    }

    async fn connect(&mut self, _address: &crate::comms::address::Address) -> Result<()> {
        anyhow::bail!("MemoryTransportInput does not support dynamic connection yet")
    }

    async fn disconnect(&mut self, _address: &crate::comms::address::Address) -> Result<()> {
        anyhow::bail!("MemoryTransportInput does not support dynamic disconnection yet")
    }
}

/// Internal memory transport for testing/threading (Output).
///
/// Implements `TransportOutput` using Tokio's MPSC channels.
#[allow(dead_code)]
pub(crate) struct MemoryTransportOutput {
    sender: mpsc::Sender<Vec<u8>>,
}

impl MemoryTransportOutput {
    /// Creates a new MemoryTransportOutput.
    ///
    /// # Arguments
    ///
    /// * `sender` - The sending end of a channel.
    #[allow(dead_code)]
    pub fn new(sender: mpsc::Sender<Vec<u8>>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl TransportOutput for MemoryTransportOutput {
    async fn send_bytes(&self, data: &[u8]) -> Result<()> {
        self.sender
            .send(data.to_vec())
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send to memory channel"))
    }
}
