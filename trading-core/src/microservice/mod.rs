pub mod configuration;
pub mod registry;

use crate::{
    admin::command::AdminPayload,
    args::CommonArgs,
    comms::Address,
    fs::PathManager,
    microservice::{configuration::Configuration, registry::Registry},
};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct Microservice<State> {
    admin_address: Address,
    state: Arc<Mutex<State>>,
    registry: Registry,
    on_registry_update: Option<Box<dyn FnOnce(&mut State, &Registry) -> () + 'static>>,
    path_manager: PathManager,
    configuration: Configuration<State>,
    args: CommonArgs,
}

impl<State> Microservice<State> {
    /// Creates a new microservice instance.
    ///
    /// This constructor will automatically parse command-line arguments.
    ///
    /// # Arguments
    ///
    /// * `initial_state` - A closure that produces the initial state.
    /// * `configuration` - The network/strategy configuration.
    ///
    /// # Returns
    ///
    /// A new `Microservice` instance.
    pub fn new<F>(initial_state: F, configuration: Configuration<State>) -> Self
    where
        F: FnOnce() -> State,
    {
        let args = CommonArgs::new();
        Self::new_with_args(args, initial_state, configuration)
    }

    /// Creates a new microservice instance with provided arguments.
    ///
    /// Useful for testing or manual injection.
    pub fn new_with_args<F>(
        args: CommonArgs,
        initial_state: F,
        configuration: Configuration<State>,
    ) -> Self
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
            args,
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

        // 3. Start Admin Interface
        let admin_addr = self.admin_address.clone();
        let (admin_tx, admin_rx) = mpsc::channel();

        thread::spawn(move || {
            let context = zmq::Context::new();
            let socket = context
                .socket(zmq::REP)
                .expect("Failed to create Admin REP socket");

            let addr_str = match admin_addr {
                Address::Zmq(s) => s,
                Address::Memory(s) => format!("inproc://{}", s),
                Address::Empty => return, // No admin
            };

            socket.bind(&addr_str).expect("Failed to bind Admin socket");
            println!("Admin interface listening on: {}", addr_str);

            loop {
                // 1. Receive Request
                let msg = match socket.recv_bytes(0) {
                    Ok(m) => m,
                    Err(_) => break, // Context terminated
                };

                // 2. Parse (JSON)
                let payload: AdminPayload = match serde_json::from_slice(&msg) {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = socket.send(&format!("{{\"error\": \"{}\"}}", e), 0);
                        continue;
                    }
                };

                // 3. Process
                match payload {
                    AdminPayload::Command(cmd) => {
                        // Forward to Service
                        // We check for Shutdown here to maybe break the listener loop?
                        let is_shutdown = cmd.is_shutdown();
                        if let Err(e) = admin_tx.send(cmd) {
                            eprintln!("Failed to forward admin command: {}", e);
                            break;
                        }

                        // Send Response
                        // For now we just ack
                        let _ = socket.send(r#"{"status": "Ok", "payload": null}"#, 0);

                        if is_shutdown {
                            break;
                        }
                    }
                    _ => {
                        let _ = socket.send(
                            r#"{"status": "Error", "payload": "Only commands supported"}"#,
                            0,
                        );
                    }
                }
            }
        });

        // 4. Launch the Configuration
        // Default Address Book from Args
        let mut address_book = std::collections::HashMap::new();
        // For Multiplexer: output is execution engine
        address_book.insert("execution_engine".to_string(), self.args.get_output_port());
        // For Strategy: output is allocation, input is market_data
        address_book.insert("allocation".to_string(), self.args.get_output_port());
        address_book.insert("market_data".to_string(), self.args.get_input_port());

        match self
            .configuration
            .launch(self.state, address_book, admin_rx)
        {
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
