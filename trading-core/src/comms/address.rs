//! Address models for network configuration.
//!
//! Defines the `Address` enum for abstracting over different transport protocols (ZMQ, Memory).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a network address for communication endpoints.
///
/// This enum allows shielding the application from specific transport implementations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Address {
    /// ZeroMQ Transport (Inter-Process)
    /// Format: "tcp://ip:port" or "ipc://path"
    Zmq(String),

    /// Internal Memory Channel (Intra-Process)
    /// Format: "channel_name"
    Memory(String),

    /// No connection (used for dynamic Multiplexers starting empty)
    Empty,
}

impl Address {
    /// Creates a new ZMQ TCP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address (e.g., "127.0.0.1").
    /// * `port` - The TCP port.
    ///
    /// # Returns
    ///
    /// A `Address::Zmq` variant.
    pub fn zmq_tcp(ip: &str, port: u16) -> Self {
        Address::Zmq(format!("tcp://{}:{}", ip, port))
    }

    /// Creates a new Memory Channel address.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the memory channel.
    ///
    /// # Returns
    ///
    /// A `Address::Memory` variant.
    pub fn memory(name: &str) -> Self {
        Address::Memory(name.to_string())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Zmq(addr) => write!(f, "zmq:{}", addr),
            Address::Memory(name) => write!(f, "mem:{}", name),
            Address::Empty => write!(f, "empty"),
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Address::Zmq("tcp://127.0.0.1:5555".to_string())
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("zmq:") {
            Ok(Address::Zmq(stripped.to_string()))
        } else if let Some(stripped) = s.strip_prefix("mem:") {
            Ok(Address::Memory(stripped.to_string()))
        } else if s == "empty" {
            Ok(Address::Empty)
        } else if s.starts_with("tcp://") || s.starts_with("ipc://") {
            Ok(Address::Zmq(s.to_string()))
        } else {
            Err(format!("Unknown address format: {}", s))
        }
    }
}
