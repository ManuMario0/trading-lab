use crate::comms::address::Address;
use crate::comms::transport::{TransportInput, TransportOutput};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Mutex;
use zmq::{Context as ZmqContext, Socket, SocketType};

/// A thread-safe ZMQ Publisher wrapper.
///
/// Implements `TransportOutput` by wrapping a synchronous `zmq::Socket` in a Mutex.
pub(crate) struct ZmqPublisher {
    socket: Mutex<Socket>,
}

impl ZmqPublisher {
    pub fn new(address: &str) -> Result<Self> {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::PUB)?;
        socket.bind(address)?;
        Ok(Self {
            socket: Mutex::new(socket),
        })
    }
}

#[async_trait]
impl TransportOutput for ZmqPublisher {
    async fn send_bytes(&self, data: &[u8]) -> Result<()> {
        let socket = self.socket.lock().unwrap();
        // ZMQ is fast enough that we can use the blocking call inside the lock
        socket
            .send(data, 0)
            .context("Failed to send ZMQ message (Transport)")
    }
}

/// A thread-safe ZMQ Subscriber wrapper.
pub(crate) struct ZmqSubscriber {
    socket: Mutex<Socket>,
}

impl ZmqSubscriber {
    pub fn new(address: &str) -> Result<Self> {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::SUB)?;
        socket.connect(address)?;
        // Subscribe to everything
        socket.set_subscribe(b"")?;
        Ok(Self {
            socket: Mutex::new(socket),
        })
    }

    pub fn new_empty() -> Result<Self> {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::SUB)?;
        // Subscribe to everything
        socket.set_subscribe(b"")?;
        Ok(Self {
            socket: Mutex::new(socket),
        })
    }
}

#[async_trait]
impl TransportInput for ZmqSubscriber {
    async fn recv_bytes(&mut self) -> Result<Vec<u8>> {
        let socket = self.socket.lock().unwrap();
        // Just read the data frame
        let data = socket
            .recv_bytes(0)
            .context("Failed to receive data payload")?;
        Ok(data)
    }

    async fn try_recv(&mut self) -> Result<Vec<u8>> {
        let socket = self.socket.lock().unwrap();
        // Just read the data frame
        let data = socket
            .recv_bytes(zmq::DONTWAIT)
            .context("Failed to receive data payload")?;
        Ok(data)
    }

    async fn connect(&mut self, address: &Address) -> Result<()> {
        let socket = self.socket.lock().unwrap();
        match address {
            Address::Zmq(endpoint) => {
                socket.connect(endpoint)?;
                Ok(())
            }
            _ => anyhow::bail!("ZmqSubscriber only supports Zmq addresses"),
        }
    }
}
