use std::collections::HashMap;
use thiserror::Error;
use zmq::{Context, Error as ZmqError, Socket, SocketType};

#[derive(Error, Debug)]
pub enum CommsError {
    #[error("ZMQ Error: {0}")]
    Zmq(#[from] ZmqError),
    #[error("Socket not found: {0}")]
    SocketNotFound(String),
    #[error("Invalid socket type")]
    InvalidSocketType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExchangeType {
    Pub,
    Sub,
    Req,
    Rep,
    Dealer,
    Router,
    Push,
    Pull,
}

impl From<ExchangeType> for SocketType {
    fn from(t: ExchangeType) -> Self {
        match t {
            ExchangeType::Pub => zmq::PUB,
            ExchangeType::Sub => zmq::SUB,
            ExchangeType::Req => zmq::REQ,
            ExchangeType::Rep => zmq::REP,
            ExchangeType::Dealer => zmq::DEALER,
            ExchangeType::Router => zmq::ROUTER,
            ExchangeType::Push => zmq::PUSH,
            ExchangeType::Pull => zmq::PULL,
        }
    }
}

pub struct ExchangeConfig {
    pub name: String,
    pub endpoint: String,
    pub socket_type: ExchangeType,
    pub is_bind: bool,
}

/// A manager for ZMQ sockets.
/// Note: ZMQ sockets are not thread-safe. This manager should generally be owned
/// by the thread that uses the sockets, or use internal mutexes if shared (which adds contention).
pub struct ExchangeManager {
    context: Context,
    sockets: HashMap<String, Socket>,
}

impl ExchangeManager {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            sockets: HashMap::new(),
        }
    }

    pub fn add_exchange(&mut self, config: &ExchangeConfig) -> Result<(), CommsError> {
        let socket = self.context.socket(config.socket_type.into())?;

        if config.is_bind {
            socket.bind(&config.endpoint)?;
        } else {
            socket.connect(&config.endpoint)?;
        }

        if config.socket_type == ExchangeType::Sub {
            socket.set_subscribe(b"")?; // Default subscribe to all
        }

        self.sockets.insert(config.name.clone(), socket);
        Ok(())
    }

    pub fn get_socket(&self, name: &str) -> Option<&Socket> {
        self.sockets.get(name)
    }

    pub fn get_mut_socket(&mut self, name: &str) -> Option<&mut Socket> {
        self.sockets.get_mut(name)
    }

    pub fn send(&self, name: &str, data: &[u8], flags: i32) -> Result<(), CommsError> {
        let socket = self
            .sockets
            .get(name)
            .ok_or_else(|| CommsError::SocketNotFound(name.to_string()))?;
        socket.send(data, flags)?;
        Ok(())
    }

    pub fn recv(&self, name: &str, flags: i32) -> Result<Vec<u8>, CommsError> {
        let socket = self
            .sockets
            .get(name)
            .ok_or_else(|| CommsError::SocketNotFound(name.to_string()))?;
        let msg = socket.recv_bytes(flags)?;
        Ok(msg)
    }
}
