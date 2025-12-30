use crate::comms::Address;
use crate::framework::runner::{ManagedRunner, Runner};
use crate::manifest::Binding;
use crate::model::identity::Id;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages multiple runners.
///
/// It holds types-erased runners and allows controlling them via string identifiers.
pub struct RunnerManager {
    runners: HashMap<String, Box<dyn ManagedRunner>>,
    active_bindings: HashMap<String, Binding>,
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
            active_bindings: HashMap::new(),
        }
    }

    /// Reconciles the running state with the desired binding
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the runner to update.
    /// * `binding` - The binding information.
    pub(crate) fn update_from_binding(&mut self, name: &str, binding: Binding) {
        if let Some(runner) = self.runners.get_mut(name) {
            let old_binding = self.active_bindings.get(name);

            match (old_binding, &binding) {
                // Case: Variadic -> Variadic (Diff Logic)
                (Some(Binding::Variadic(old_map)), Binding::Variadic(new_map)) => {
                    // 1. Remove inputs that are in OLD but not in NEW
                    for (id, source) in old_map {
                        if !new_map.contains_key(id) {
                            runner.disconnect_input(source.address.clone());
                        }
                    }

                    // 2. Add or Update inputs from NEW
                    for (id, source) in new_map {
                        // Check if it existed
                        if let Some(old_source) = old_map.get(id) {
                            if old_source.address != source.address || old_source.id != source.id {
                                // Changed: Disconnect old, Connect new
                                runner.disconnect_input(old_source.address.clone());
                                runner.add_input(source.address.clone());
                            }
                        } else {
                            // New: Connect
                            runner.add_input(source.address.clone());
                        }
                    }
                }

                // Case: Any -> Single (Hot Swap)
                (_, Binding::Single(source)) => {
                    runner.update_address(source.address.clone());
                }

                // Case: Single (or None) -> Variadic
                (_, Binding::Variadic(new_map)) => {
                    for (_key, source) in new_map {
                        runner.add_input(source.address.clone());
                    }
                }
            }
        }
        // Update the state
        self.active_bindings.insert(name.to_string(), binding);
    }
    /// Creates and starts a new runner managed by this manager.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this runner (e.g. "market_data").
    /// * `state` - Shared state.
    /// * `callback` - The message processing function.
    /// * `address` - The input address, if needed.
    pub(crate) fn add_runner<State, Input>(
        &mut self,
        name: impl Into<String>,
        state: Arc<Mutex<State>>,
        callback: Box<dyn FnMut(&mut State, Id, Input) + Send>,
        address: Option<Address>,
    ) where
        State: Send + 'static,
        Input: Sync + Send + Serialize + DeserializeOwned + 'static,
    {
        let mut runner = Runner::new(state, callback);
        if let Some(address) = address {
            runner.add_input(address);
        }
        self.runners.insert(name.into(), Box::new(runner));
    }

    /// Shuts down all runners managed by this manager.
    pub(crate) fn shutdown(self) {
        for mut runner in self.runners.into_values() {
            runner.shutdown();
        }
    }
}
