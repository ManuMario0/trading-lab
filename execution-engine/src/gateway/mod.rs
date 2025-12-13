use crate::models::ingress::IngressMessage;

/// Trait for receiving input messages from external sources.
/// This decouples the Engine from the specific transport (Mock, ZMQ, TCP).
pub trait Gateway: Send {
    /// Fetch the next message. Returns None if connection closed or empty.
    /// In a real system, this might be async.
    fn next(&mut self) -> Option<IngressMessage>;
}

/// A simple mock gateway that replays a vector of messages.
pub struct MockGateway {
    messages: std::vec::IntoIter<IngressMessage>,
}

impl MockGateway {
    pub fn new(messages: Vec<IngressMessage>) -> Self {
        Self {
            messages: messages.into_iter(),
        }
    }
}

impl Gateway for MockGateway {
    fn next(&mut self) -> Option<IngressMessage> {
        self.messages.next()
    }
}
