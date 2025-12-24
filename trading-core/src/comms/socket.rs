//! Typed socket abstractions.
//!
//! Provides `ReceiverSocket` and `SenderSocket` which handle serialization/deserialization automatically.

use crate::comms::transport::{TransportInput, TransportOutput};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// A strongly-typed input socket.
pub struct ReceiverSocket<C> {
    transport: Box<dyn TransportInput>,
    _marker: PhantomData<C>,
}

impl<C> ReceiverSocket<C>
where
    C: DeserializeOwned,
{
    /// Creates a new ReceiverSocket from a raw transport backend.
    ///
    /// # Arguments
    ///
    /// * `transport` - The underlying transport implementation (e.g. ZMQ, Memory).
    pub fn new(transport: Box<dyn TransportInput>) -> Self {
        Self {
            transport,
            _marker: PhantomData,
        }
    }

    /// Receives the next message and deserializes it.
    ///
    /// This is a blocking call (asynchronous).
    ///
    /// # Returns
    ///
    /// * `Ok(C)` containing the deserialized message.
    /// * `Err` if transport fails or deserialization error occurs.
    pub async fn recv(&mut self) -> Result<C> {
        let bytes = self.transport.recv_bytes().await?;
        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }

    /// Receives the next message and deserializes it (Non-blocking attempt).
    ///
    /// # Returns
    ///
    /// * `Ok(C)` if a message is immediately available.
    /// * `Err` if no message is available (`EAGAIN` equivalent) or other error.
    pub async fn try_recv(&mut self) -> Result<C> {
        let bytes = self.transport.try_recv().await?;
        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }
}

/// A strongly-typed output socket.
pub struct SenderSocket<C> {
    transport: Box<dyn TransportOutput>,
    _marker: PhantomData<C>,
}

impl<C> SenderSocket<C>
where
    C: Serialize,
{
    /// Creates a new SenderSocket from a raw transport backend.
    ///
    /// # Arguments
    ///
    /// * `transport` - The underlying transport implementation.
    pub fn new(transport: Box<dyn TransportOutput>) -> Self {
        Self {
            transport,
            _marker: PhantomData,
        }
    }

    /// Serializes and sends the message.
    ///
    /// # Arguments
    ///
    /// * `data` - The strongly-typed message to send.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err` if serialization or transport fails.
    pub async fn send(&self, data: &C) -> Result<()> {
        let bytes = bincode::serialize(data)?;
        self.transport.send_bytes(&bytes).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comms::transports::memory::{MemoryTransportInput, MemoryTransportOutput};
    use crate::model::market_data::{MarketDataBatch, PriceUpdate};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_typed_socket_memory() -> Result<()> {
        // 1. Setup Memory Transport
        let (tx, rx) = mpsc::channel(100);
        let output_transport = MemoryTransportOutput::new(tx);
        let input_transport = MemoryTransportInput::new(rx);

        // 2. Wrap in Typed Sockets
        let output: SenderSocket<MarketDataBatch> = SenderSocket::new(Box::new(output_transport));
        let mut input: ReceiverSocket<MarketDataBatch> =
            ReceiverSocket::new(Box::new(input_transport));

        // 3. Create Typed Data
        let update = PriceUpdate::new(1, 150.0, 1000);
        let batch = MarketDataBatch::new(vec![update]);

        // 4. Send
        output.send(&batch).await?;

        // 5. Receive & Verify
        let received = input.recv().await?;
        assert_eq!(received.get_updates().len(), 1);
        assert_eq!(received.get_updates()[0].get_price(), 150.0);

        Ok(())
    }
}
