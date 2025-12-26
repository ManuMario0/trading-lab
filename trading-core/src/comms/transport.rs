use crate::comms::address::Address;
use anyhow::Result;
use async_trait::async_trait;

/// Abstraction for the incoming transport layer (reading raw bytes).
/// Implementation details (ZMQ, TCP, Memory) are hidden behind this trait.
#[async_trait]
pub trait TransportInput: Send + Sync {
    /// Receive the next full frame/message as bytes.
    async fn recv_bytes(&mut self) -> Result<Vec<u8>>;

    /// Try to receive the next full frame/message as bytes.
    /// If no message is available, returns an error.
    async fn try_recv(&mut self) -> Result<Vec<u8>>;

    /// Connects to a new publisher/source dynamically.
    /// This allows a single input to aggregate multiple sources (Multiplexing).
    ///
    /// # Arguments
    ///
    /// * `address` - The address to connect to.
    async fn connect(&mut self, address: &Address) -> Result<()>;
}

/// Abstraction for the outgoing transport layer (sending raw bytes).
#[async_trait]
pub trait TransportOutput: Send + Sync {
    /// Send a full frame/message.
    async fn send_bytes(&self, data: &[u8]) -> Result<()>;
}
