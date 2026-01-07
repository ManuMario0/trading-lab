//! # Event Bus
//!
//! The Central Nervous System of the Orchestrator. It is the Single Source of Truth for system state changes.
//!
//! ## Purpose
//! This module decouples the components. Instead of `Registry` calling `Supervisor` directly,
//! it emits a `ServiceDiscovered` event. The Supervisor listens to it.
//!
//! ## Why this is better
//! 1. **Traceability**: We can log every single event flowing through the system.
//! 2. **SMR Ready**: These events correspond 1:1 to the "Log Entries" we would replicate in Raft.
//! 3. **Testing**: We can unit test the Supervisor by just sending fake events.

use orchestrator_protocol::model::{Layout, ServiceDescriptor};
use tokio::sync::broadcast;

/// The Single Source of Truth for system state changes.
///
/// If it didn't happen as an Event, it didn't happen.
///
/// # Usage
///
/// You should always emit an Event before modifying any state. This ensures that
/// we can always replay the events to get to the current state. It prevent also state
/// corruption if a component crashes.
///
/// # Examples
///
/// ```
/// let event_bus = EventBus::new();
/// // event_bus.publish(SystemEvent::ServiceDiscovered { descriptor: ... });
/// ```
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// **Registry**: Use this when a new service file is found on disk.
    ServiceDiscovered { descriptor: ServiceDescriptor },

    /// **API**: Use this when a User asks to deploy a layout.
    DeployRequested { layout: Layout },

    /// **Supervisor**: Use this when a PID exits unexpectedly.
    ServiceCrashed {
        id: String, // e.g., "strategy-alpha"
        exit_code: Option<i32>,
    },

    /// **Runtime**: Use this when a process is successfully spawned.
    ServiceStarted { id: String, pid: u32 },

    /// **System**: Use this to report non-fatal or fatal errors in components.
    Error { error: SystemError },
}

#[derive(Debug, Clone)]
pub enum SystemError {
    /// IO-related errors (file system, network keys).
    Io(String),
    /// Configuration errors (parsing, missing fields).
    Configuration(String),
    /// unexpected crashes or logic errors.
    Fatal(String),
}

impl SystemError {
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn fatal(msg: impl Into<String>) -> Self {
        Self::Fatal(msg.into())
    }
}

/// A wrapper around a tokio broadcast channel.
///
/// We wrap it struct to allow for easy mocking/replacing later,
/// and to enforce strong typing on the events.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    /// Creates a new EventBus.
    ///
    /// See [`tokio::sync::broadcast::channel`] for details.
    pub fn new() -> Self {
        // Capacity of 100 events. If the receiver is too slow, it will skip old events (Lagging).
        // This is acceptable for logging, but for SMR we might want a persistent queue later.
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    /// Publishes an event to all subscribers.
    ///
    /// See [`tokio::sync::broadcast::Sender::send`] for details.
    pub fn publish(&self, event: SystemEvent) {
        // We ignore the error if there are no active subscribers (e.g., during startup)
        let _ = self.sender.send(event);
    }

    /// Creates a new subscriber.
    ///
    /// See [`tokio::sync::broadcast::Sender::subscribe`] for details.
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
}
