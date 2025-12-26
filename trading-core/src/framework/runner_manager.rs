use crate::comms::Address;
use crate::framework::runner::{ManagedRunner, Runner};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages multiple runners.
///
/// It holds types-erased runners and allows controlling them via string identifiers.
pub struct RunnerManager {
    runners: HashMap<String, Box<dyn ManagedRunner>>,
}

impl RunnerManager {
    /// Creates a new, empty RunnerManager.
    ///
    /// # Returns
    ///
    /// A new `RunnerManager`.
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    /// Creates and starts a new runner managed by this manager.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this runner (e.g. "market_data").
    /// * `state` - Shared state.
    /// * `callback` - The message processing function.
    /// * `address` - The input address.
    pub fn add_runner<State, Input>(
        &mut self,
        name: impl Into<String>,
        state: Arc<Mutex<State>>,
        callback: Box<dyn FnMut(&mut State, Input) + Send>,
        address: Address,
    ) where
        State: Send + 'static,
        Input: Sync + Send + DeserializeOwned + 'static,
    {
        let runner = Runner::new(state, callback, address);
        self.runners.insert(name.into(), Box::new(runner));
    }

    /// Stops a specific runner and removes it from the manager.
    pub fn stop_runner(&mut self, name: &str) {
        if let Some(runner) = self.runners.remove(name) {
            runner.stop();
        }
    }

    /// Updates the listening address of a specific runner.
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the runner to update.
    /// * `address` - The new address.
    pub fn update_runner_address(&mut self, name: &str, address: Address) {
        if let Some(runner) = self.runners.get_mut(name) {
            runner.update_address(address);
        }
    }

    /// Adds a new input source to a specific runner.
    pub fn add_runner_input(&mut self, name: &str, address: Address) {
        if let Some(runner) = self.runners.get_mut(name) {
            runner.add_input(address);
        }
    }
}
