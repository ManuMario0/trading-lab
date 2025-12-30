//! Generic Runner for processing data streams.
//!
//! A `Runner` wraps an input socket, a state container, and a callback function.
//! It manages the event loop, thread spawning, and control messages (stop, update).

use crate::comms::socket::ReceiverSocket;
use crate::comms::{build_subscriber, builder, Address};
use crate::model::identity::Id;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

/// Commands sent to control the runner's lifecycle and configuration.
pub enum RunnerCommand {
    /// Stop the runner loop and exit.
    Stop,

    /// Update the listening address at runtime.
    UpdateAddress(Address),

    /// Add a new input source dynamically (Multiplexing).
    AddInput(Address),

    /// Remove an input source dynamically (Multiplexing).
    DisconnectInput(Address),
}

/// The structure used for a runner.
///
/// A runner is a component that handles exactly one input channel.
/// This is usefull as I can abstract hotswapping of listening/writing ports for the runners.
pub struct Runner<State, Input> {
    handle: Option<JoinHandle<()>>,
    control_tx: Sender<RunnerCommand>,
    _input_marker: std::marker::PhantomData<Input>,
    _state_marker: std::marker::PhantomData<State>,
}

impl<State, Input> Runner<State, Input> {
    /// This will create the runner and start it in a separate thread.
    ///
    /// # Arguments
    ///
    /// * `state` - Shared thread-safe access to the microservice state.
    /// * `callback` - To be executed for each incoming message.
    /// * `address` - The initial address to listen on.
    ///
    /// # Returns
    ///
    /// A new `Runner` instance holding the thread handle and control channel.
    pub(super) fn new(
        state: Arc<Mutex<State>>,
        callback: Box<dyn FnMut(&mut State, Id, Input) + Send>,
    ) -> Self
    where
        State: Send + 'static,
        Input: Sync + Send + Serialize + DeserializeOwned + 'static,
    {
        let (control_tx, control_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            // Create a runtime for the async runner loop
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(runner_loop(control_rx, state, callback));
        });
        Self {
            handle: Some(handle),
            control_tx,
            _input_marker: std::marker::PhantomData,
            _state_marker: std::marker::PhantomData,
        }
    }

    /// Updates the listening address of the running loop.
    ///
    /// # Arguments
    ///
    /// * `address` - The new address to bind/connect to.
    fn update_address(&mut self, address: Address) {
        self.control_tx
            .send(RunnerCommand::UpdateAddress(address))
            .unwrap();
    }

    /// Adds a new input source to the runner.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to connect to.
    fn add_input(&mut self, address: Address) {
        self.control_tx
            .send(RunnerCommand::AddInput(address))
            .unwrap();
    }

    /// Disconnects an input source from the runner.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to disconnect from.
    fn disconnect_input(&mut self, address: Address) {
        self.control_tx
            .send(RunnerCommand::DisconnectInput(address))
            .unwrap();
    }

    /// Shuts down the runner.
    ///
    /// # Panics
    ///
    /// Panics if the thread join fails or channel send fails (which shouldn't happen in normal operation).
    fn shutdown(&mut self) {
        self.control_tx.send(RunnerCommand::Stop).unwrap();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

/// A trait for type-erased runners, allowing them to be stored in a homogeneous collection.
pub(crate) trait ManagedRunner: Send {
    /// Shuts down the runner.
    fn shutdown(&mut self);
    /// Updates the runner's address.
    fn update_address(&mut self, address: Address);
    /// Adds a connection source.
    fn add_input(&mut self, address: Address);
    /// Disconnects an input source from the runner.
    fn disconnect_input(&mut self, address: Address);
}

impl<State, Input> ManagedRunner for Runner<State, Input>
where
    State: Send + 'static,
    Input: Sync + Send + DeserializeOwned + 'static,
{
    fn shutdown(&mut self) {
        self.shutdown()
    }

    fn update_address(&mut self, address: Address) {
        self.update_address(address)
    }

    fn add_input(&mut self, address: Address) {
        self.add_input(address)
    }

    fn disconnect_input(&mut self, address: Address) {
        self.disconnect_input(address)
    }
}

async fn runner_loop<State, Input>(
    control_rx: mpsc::Receiver<RunnerCommand>,
    state: Arc<Mutex<State>>,
    mut callback: Box<dyn FnMut(&mut State, Id, Input) + Send>,
) where
    Input: Serialize + DeserializeOwned + Sync + Send + 'static,
{
    // First setup the listener on the address
    let mut listener: ReceiverSocket<Input> = builder::build_empty_subscriber().unwrap();
    let mut busy_count = 0;

    loop {
        let mut received_work = false;

        // 1. Try to receive a message
        match listener.try_recv().await {
            Ok(packet) => {
                // Call the callback
                callback(&mut state.lock().unwrap(), packet.id(), packet.data());
                received_work = true;
            }
            Err(_) => (),
        }

        // 2. Try to receive a control message
        match control_rx.try_recv() {
            Ok(RunnerCommand::Stop) => break,
            Ok(RunnerCommand::UpdateAddress(addr)) => {
                // For replacing the listener, we rebuild it
                listener = build_subscriber(&addr).unwrap();
                received_work = true;
            }
            Ok(RunnerCommand::AddInput(addr)) => {
                // For adding inputs, we use connect on the existing listener
                // We unwrap here as this is a critical configuration error if it fails
                listener.connect(&addr).await.unwrap();
                received_work = true;
            }
            Ok(RunnerCommand::DisconnectInput(addr)) => {
                listener.disconnect(&addr).await.unwrap();
                received_work = true;
            }
            Err(_) => (),
        }

        // 3. Backoff Strategy (Hybrid Spin/Sleep)
        if received_work {
            busy_count = 0;
        } else {
            busy_count += 1;
            if busy_count < 2000 {
                // High Perf: Yield to OS but stay scheduled (Nanoseconds/Microseconds latency)
                std::thread::yield_now();
            } else {
                // Low Power: Sleep if really idle (1ms latency)
                // Cap the counter to avoid overflow, just stay in sleep mode
                busy_count = 2000;
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        }
    }
}
