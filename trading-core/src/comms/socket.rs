//! Typed socket abstractions.
//!
//! Provides `ReceiverSocket` and `SenderSocket` which handle serialization/deserialization automatically.

use crate::comms::packet::Packet;
use crate::comms::transport::{TransportDuplex, TransportInput, TransportOutput};
use crate::model::identity::Id;
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
    /// * `Ok(Packet<C>)` containing the deserialized message with sender info.
    /// * `Err` if transport fails or deserialization error occurs.
    pub async fn recv(&mut self) -> Result<Packet<C>> {
        let bytes = self.transport.recv_bytes().await?;
        let packet: Packet<C> = bincode::deserialize(&bytes)?;
        Ok(packet)
    }

    /// Receives the next message and deserializes it (Non-blocking attempt).
    ///
    /// # Returns
    ///
    /// * `Ok(Packet<C>)` if a message is immediately available.
    /// * `Err` if no message is available (`EAGAIN` equivalent) or other error.
    pub async fn try_recv(&mut self) -> Result<Packet<C>> {
        let bytes = self.transport.try_recv().await?;
        let packet: Packet<C> = bincode::deserialize(&bytes)?;
        Ok(packet)
    }

    /// Connects to a new publisher/source dynamically.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to connect to.
    pub async fn connect(&mut self, address: &crate::comms::address::Address) -> Result<()> {
        self.transport.connect(address).await
    }

    /// Disconnects from a publisher/source dynamically.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to disconnect from.
    pub async fn disconnect(&mut self, address: &crate::comms::address::Address) -> Result<()> {
        self.transport.disconnect(address).await
    }
}

/// A strongly-typed output socket.
pub struct SenderSocket<C> {
    transport: Box<dyn TransportOutput>,
    id: Id,
    _marker: PhantomData<C>,
}

impl<C> SenderSocket<C>
where
    C: Serialize + DeserializeOwned,
{
    /// Creates a new SenderSocket from a raw transport backend.
    ///
    /// # Arguments
    ///
    /// * `transport` - The underlying transport implementation.
    pub fn new(transport: Box<dyn TransportOutput>, id: Id) -> Self {
        Self {
            transport,
            id,
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
    pub async fn send(&self, data: C) -> Result<()> {
        let packet = Packet::new(self.id, data);
        let bytes = bincode::serialize(&packet)?;
        self.transport.send_bytes(&bytes).await
    }
}

pub(crate) struct ResponseHandle<'a, C> {
    transport: &'a Box<dyn TransportDuplex>,
    id: Id,
    _marker: PhantomData<C>,
}

impl<'a, C> ResponseHandle<'a, C>
where
    C: Serialize + DeserializeOwned,
{
    /// We ensure we consume the object to prevent double use.
    pub async fn send_reply(self, data: C) -> Result<()> {
        let packet = Packet::new(self.id, data);
        let bytes = bincode::serialize(&packet)?;
        self.transport.send_bytes(&bytes).await
    }
}

pub struct ReplySocket<C> {
    transport: Box<dyn TransportDuplex>,
    id: Id,
    _marker: PhantomData<C>,
}

impl<C> ReplySocket<C>
where
    C: DeserializeOwned + Serialize,
{
    /// Creates a new ReplySocket from a raw transport backend.
    ///
    /// # Arguments
    ///
    /// * `transport` - The underlying transport implementation.
    pub(crate) fn new(transport: Box<dyn TransportDuplex>, id: Id) -> Self {
        Self {
            transport,
            id,
            _marker: PhantomData,
        }
    }

    /// Receives the next message and deserializes it.
    ///
    /// # Returns
    ///
    /// * `Ok((C, ResponseHandle<'a, C>))` containing the deserialized message and a response handle.
    /// * `Err` if transport fails or deserialization error occurs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut socket = ReplySocket::new(Box::new(ZmqSubscriber::new("tcp://127.0.0.1:5555")));
    /// let (message, response_handle) = socket.recv().await?;
    /// response_handle.send_reply(&message).await?;
    /// ```
    pub(crate) async fn recv<'a>(&'a mut self) -> Result<(Packet<C>, ResponseHandle<'a, C>)> {
        let bytes = self.transport.recv_bytes().await?;
        let packet: Packet<C> = match bincode::deserialize(&bytes) {
            Ok(p) => p,
            Err(e) => {
                // If deserialization fails, the REP socket is still in SEND state.
                // We must send a reply to reset it to RECV state for the next request.
                // We send an empty frame which will likely cause a deserialization error on the other side,
                // but that's better than deadlocking the service.
                let _ = self.transport.send_bytes(b"").await;
                return Err(e.into());
            }
        };

        let response_handle = ResponseHandle {
            transport: &self.transport,
            id: self.id,
            _marker: PhantomData,
        };
        Ok((packet, response_handle))
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
        let output: SenderSocket<MarketDataBatch> =
            SenderSocket::new(Box::new(output_transport), Id::from(10usize));
        let mut input: ReceiverSocket<MarketDataBatch> =
            ReceiverSocket::new(Box::new(input_transport));

        // 3. Create Typed Data
        let update = PriceUpdate::new(1, 150.0, 150.0, 150.0, 1000);
        let batch = MarketDataBatch::new(vec![update]);

        // 4. Send
        output.send(batch).await?;

        // 5. Receive & Verify
        // The received item is a Packet<MarketDataBatch>
        let received_packet = input.recv().await?;
        // Check the sender ID
        assert_eq!(received_packet.id(), Id::from(10usize));

        // Check the data
        let received_data = received_packet.data();
        assert_eq!(received_data.get_count(), 1);
        assert_eq!(received_data.get_update_at(0).last, 150.0);

        Ok(())
    }
}
