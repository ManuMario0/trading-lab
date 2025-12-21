use anyhow::Result;
use zmq::{Context, Socket, SocketType};

/// A generic ZMQ Publisher wrapper.
///
/// This struct wraps a ZMQ PUB socket and provides a simple API to publish messages to topics.
pub struct GenericPublisher {
    socket: Socket,
}

impl GenericPublisher {
    /// Creates a new GenericPublisher bound to the specified address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind to (e.g., "tcp://*:5555").
    pub fn new(address: &str) -> Result<Self> {
        let context = Context::new();
        let socket = context.socket(SocketType::PUB)?;
        socket.bind(address)?;
        Ok(Self { socket })
    }

    /// Publishes a message to a specific topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to publish to.
    /// * `message` - The message payload as bytes.
    pub fn publish(&self, topic: &str, message: &[u8]) -> Result<()> {
        self.socket
            .send_multipart(&[topic.as_bytes(), message], 0)?;
        Ok(())
    }
}

/// A generic ZMQ Subscriber wrapper.
///
/// This struct wraps a ZMQ SUB socket and provides a simple API to receive messages from topics.
pub struct GenericSubscriber {
    socket: Socket,
}

impl GenericSubscriber {
    /// Creates a new GenericSubscriber connected to the specified address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to connect to (e.g., "tcp://localhost:5555").
    /// * `topics` - A list of topics to subscribe to.
    pub fn new(address: &str, topics: &[&str]) -> Result<Self> {
        let context = Context::new();
        let socket = context.socket(SocketType::SUB)?;
        socket.connect(address)?;

        for topic in topics {
            socket.set_subscribe(topic.as_bytes())?;
        }

        Ok(Self { socket })
    }

    /// Receives a message from the subscribed topics.
    ///
    /// This method blocks until a message is received.
    ///
    /// # Returns
    ///
    /// A tuple containing the topic (String) and the message payload (Vec<u8>).
    pub fn receive(&self) -> Result<(String, Vec<u8>)> {
        let msg = self.socket.recv_multipart(0)?;
        let topic = String::from_utf8(msg[0].clone())?;
        let data = msg[1].clone();
        Ok((topic, data))
    }
}
