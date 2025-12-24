pub mod configuration;
pub mod registry;

use crate::{
    args::CommonArgs,
    comms::Address,
    fs::PathManager,
    microservice::{configuration::Configuration, registry::Registry},
};
use std::sync::{Arc, Mutex};

pub struct Microservice<State> {
    admin_address: Address,
    state: Arc<Mutex<State>>,
    registry: Registry,
    on_registry_update: Option<Box<dyn FnOnce(&mut State, &Registry) -> () + 'static>>,
    path_manager: PathManager,
    configuration: Configuration<State>,
}

impl<State> Microservice<State> {
    /// Creates a new microservice instance.
    ///
    /// The microservice acts as the central hub for the application, initializing
    /// core components like the path manager, admin registry, and state container.
    ///
    /// # Arguments
    ///
    /// * `args` - Parsed command-line arguments.
    /// * `initial_state` - A closure that produces the initial state.
    /// * `configuration` - The network/strategy configuration.
    ///
    /// # Returns
    ///
    /// A new `Microservice` instance.
    pub fn new<F>(args: CommonArgs, initial_state: F, configuration: Configuration<State>) -> Self
    where
        F: FnOnce() -> State,
    {
        Self {
            admin_address: args.get_admin_route(),
            state: Arc::new(Mutex::new(initial_state())),
            registry: Registry::new(),
            on_registry_update: None,
            path_manager: PathManager::from_args(&args),
            configuration,
        }
    }

    /// Registers a callback for parameter updates.
    ///
    /// # Arguments
    ///
    /// * `registry` - A closure that returns the initial Registry.
    /// * `callback` - Function to execute when a parameter is updated via Admin.
    ///
    /// # Returns
    ///
    /// The modified `Microservice` instance (builder pattern).
    pub fn with_registry<F, G>(mut self, registry: F, callback: G) -> Self
    where
        F: FnOnce() -> Registry,
        G: FnOnce(&mut State, &Registry) -> () + 'static,
    {
        let registry = registry();
        self.registry = registry;
        self.on_registry_update = Some(Box::new(callback));
        self
    }

    /// Starts the microservice main loop.
    ///
    /// This function:
    /// 1. Initializes the admin listener.
    /// 2. Launches configured runners (Strategy, Multiplexer, etc.).
    /// 3. Blocks indefinitely while monitoring health.
    ///
    /// # Panics
    ///
    /// Panics if binding to ports fails or initialization errors occur.
    pub fn run(mut self)
    where
        State: Send + 'static,
    {
        // 1. Ensure required directories exist
        if let Err(e) = self.path_manager.ensure_dirs() {
            panic!("Failed to verify/create directories: {}", e);
        }

        // 2. Initial Registry Sync / Callback
        // If a callback was registered, execute it now to set initial state based on registry
        if let Some(callback) = self.on_registry_update.take() {
            let mut state = self.state.lock().unwrap();
            callback(&mut state, &self.registry);
        }

        // 3. Start Admin Interface (Stub)
        // In a real implementation, this would spawn a thread listening on self.admin_address
        // and exposing self.registry.
        println!("Microservice starting...");
        println!("Admin interface on: {}", self.admin_address);

        // 4. Launch the Configuration
        // TODO: The address book should be loaded from a config file or discovery service.
        // For now, we use an empty map, potentially preventing correct connection in this stub.
        let address_book = std::collections::HashMap::new();

        match self.configuration.launch(self.state, address_book) {
            Ok(_) => {
                // Should not happen if launch loops
                println!("Service exited gracefully.");
            }
            Err(e) => {
                panic!("Service crashed: {}", e);
            }
        }
    }
}
