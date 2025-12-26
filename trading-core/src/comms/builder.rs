//! Factory functions for creating communication endpoints.
//!
//! Abstracts the creation of `SenderSocket` and `ReceiverSocket` based on `Address`.

use super::address::Address;
use super::socket::{ReceiverSocket, SenderSocket};
use super::transports::zmq::{ZmqPublisher, ZmqSubscriber};
use crate::comms::transport::{TransportInput, TransportOutput};
use anyhow::{bail, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Factory to create Sender endpoints.
///
/// # Arguments
///
/// * `address` - The target address to publish to.
///
/// # Returns
///
/// * `Ok(SenderSocket)` if successful.
/// * `Err` if the address type is unsupported or initialization fails.
pub fn build_publisher<T>(address: &Address) -> Result<SenderSocket<T>>
where
    T: Serialize + Send + Sync + 'static,
{
    let transport: Box<dyn TransportOutput> = match address {
        Address::Zmq(addr_str) => {
            let p = ZmqPublisher::new(addr_str)?;
            Box::new(p)
        }
        Address::Memory(_) => {
            bail!("Memory channels not yet implemented for Publisher");
        }
        Address::Empty => {
            bail!("Cannot build a publisher with an empty address");
        }
    };
    Ok(SenderSocket::new(transport))
}

/// Factory to create Receiver endpoints.
///
/// # Arguments
///
/// * `address` - The address to subscribe/listen to.
///
/// # Returns
///
/// * `Ok(ReceiverSocket)` if successful.
/// * `Err` if initialization fails.
pub fn build_subscriber<T>(address: &Address) -> Result<ReceiverSocket<T>>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    let transport: Box<dyn TransportInput> = match address {
        Address::Zmq(addr_str) => {
            let s = ZmqSubscriber::new(addr_str)?;
            Box::new(s)
        }
        Address::Memory(_) => {
            bail!("Memory channels not yet implemented for Subscriber");
        }
        Address::Empty => return build_empty_subscriber(),
    };
    Ok(ReceiverSocket::new(transport))
}

/// Factory to create an empty Subscriber endpoint (no initial connection).
///
/// # Returns
///
/// * `Ok(ReceiverSocket)` if successful.
pub fn build_empty_subscriber<T>() -> Result<ReceiverSocket<T>>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    // Zmq implementation
    let transport: Box<dyn TransportInput> = Box::new(ZmqSubscriber::new_empty()?);
    Ok(ReceiverSocket::new(transport))
}
