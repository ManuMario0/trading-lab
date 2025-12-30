pub mod configuration;
pub mod registry;

use anyhow::Result;
use log::info;

use crate::{
    admin::{command::AdminPayload, AdminCommand, AdminResponse},
    args::CommonArgs,
    comms::{self, socket::ReplySocket},
    fs::PathManager,
    manifest::Binding,
    microservice::{
        configuration::{Configurable, Configuration},
        registry::Registry,
    },
};
use std::{
    io,
    sync::{Arc, Mutex},
};

pub struct Microservice<Config>
where
    Config: Configurable,
{
    /// Service settings
    configuration: Configuration<Config>,
    registry: Registry,
    on_registry_update: Option<Box<dyn FnMut(&mut Config::State, &Registry) -> () + 'static>>,

    /// Bindings
    args: CommonArgs,
    path_manager: PathManager,

    /// Service state
    state: Arc<Mutex<Config::State>>,
    should_shutdown: bool,
}

impl<Config> Microservice<Config>
where
    Config: Configurable,
{
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
    pub fn new<F>(initial_state: F, configuration: Configuration<Config>) -> Self
    where
        F: FnOnce() -> Config::State,
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
        configuration: Configuration<Config>,
    ) -> Self
    where
        F: FnOnce() -> Config::State,
    {
        Self {
            state: Arc::new(Mutex::new(initial_state())),
            registry: Registry::new(),
            on_registry_update: None,
            path_manager: PathManager::from_args(&args),
            configuration,
            args,
            should_shutdown: false,
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
        G: FnMut(&mut Config::State, &Registry) -> () + 'static,
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
    pub async fn run(mut self) {
        // 1. Ensure required directories exist
        if let Err(e) = self.ensure_dirs() {
            panic!("Failed to verify/create directories: {}", e);
        }

        // 2. Initialize admin comms
        let mut admin = self
            .init_admin_comms()
            .expect("Failed to initialize admin comms");

        // 3. Initial Registry Sync / Callback
        // If a callback was registered, execute it now to set initial state based on registry
        if let Some(callback) = self.on_registry_update.as_mut() {
            let mut state = self.state.lock().unwrap();
            callback(&mut state, &self.registry);
        }

        // 4. Launch the runners for the process
        self.launch_runners();

        // 5. Admin loop
        loop {
            let (msg, response_handler) = match admin.recv().await {
                Ok(res) => res,
                Err(e) => {
                    info!("Admin connection closed or error: {}", e);
                    break;
                }
            };
            let response = self.process_admin_command(msg.data());
            response_handler
                .send_reply(response)
                .await
                .expect("Connection lost");
            if self.should_shutdown {
                break;
            }
        }

        // 6. Clean up
        info!("Shutting down complete");
    }

    /// Ensures that all required directories exist.
    fn ensure_dirs(&self) -> Result<(), io::Error> {
        self.path_manager.ensure_dirs()
    }

    /// Initializes admin comms.
    fn init_admin_comms(&self) -> Result<ReplySocket<AdminPayload>> {
        if let Some(admin) = self.args.get_bindings().inputs.get("admin") {
            if let Binding::Single(source) = admin {
                comms::builder::build_replier(&source.address, self.args.get_service_id())
            } else {
                Err(anyhow::anyhow!("Admin comms not correctly configured: expected single address, found variadic address"))
            }
        } else {
            Err(anyhow::anyhow!("Admin comms not configured"))
        }
    }

    fn launch_runners(&mut self) {
        self.configuration
            .launch(self.state.clone(), self.args.get_bindings())
    }

    fn process_admin_command(&mut self, msg: AdminPayload) -> AdminPayload {
        match msg {
            AdminPayload::Command(cmd) => match cmd {
                AdminCommand::Shutdown => {
                    info!("Shutting down runners");
                    self.configuration.shutdown();
                    self.should_shutdown = true;
                    AdminPayload::new_response(AdminResponse::Ok)
                }
                AdminCommand::Ping => AdminPayload::new_response(AdminResponse::Pong),
                AdminCommand::UpdateBindings { config } => {
                    self.configuration.update_from_service_config(config);
                    AdminPayload::new_response(AdminResponse::Ok)
                }
                AdminCommand::Registry => AdminPayload::new_response(AdminResponse::Info(
                    serde_json::to_value(self.registry.clone()).unwrap(),
                )),
                AdminCommand::Status => AdminPayload::new_response(AdminResponse::Ok),
                AdminCommand::UpdateRegistry { key, value } => {
                    self.registry.update_parameter(&key, value);
                    if let Some(callback) = self.on_registry_update.as_mut() {
                        let mut state = self.state.lock().unwrap();
                        callback(&mut state, &self.registry);
                    }
                    AdminPayload::new_response(AdminResponse::Ok)
                }
                AdminCommand::Unknown => {
                    AdminPayload::new_response(AdminResponse::Error("Unknown command".to_string()))
                }
            },
            _ => AdminPayload::new_response(AdminResponse::Error(
                "Only commands supported".to_string(),
            )),
        }
    }
}
