//! Save all the default network configurations for microservices here.
//!
//! This includes:
//! - Strategy
//! - Multiplexer
//! - Execution engine

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::comms;
use crate::{
    admin::command::AdminCommand,
    comms::Address,
    framework::runner_manager::RunnerManager,
    model::{allocation::Allocation, market_data::MarketDataBatch},
};
use std::sync::mpsc::Receiver;

pub struct Configuration<State> {
    config: ConfigurationBuilder<State>,
}

impl<State> Configuration<State> {
    /// Creates a new Strategy configuration.
    ///
    /// # Arguments
    ///
    /// * `market_data_callback` - The core logic function that processes market data and returns an allocation.
    ///
    /// # Returns
    ///
    /// A `Configuration::Strategy` variant.
    pub fn new_strategy(
        market_data_callback: Box<dyn FnMut(&mut State, &MarketDataBatch) -> Allocation + Send>,
    ) -> Self {
        Self {
            config: ConfigurationBuilder::new_strategy(market_data_callback),
        }
    }

    /// Creates a new Multiplexer configuration.
    ///
    /// # Returns
    ///
    /// A `Configuration::Multiplexer` variant.
    pub fn new_multiplexer() -> Self {
        Self {
            config: ConfigurationBuilder::new_multiplexer(),
        }
    }

    /// Creates a new Execution Engine configuration.
    ///
    /// # Returns
    ///
    /// A `Configuration::ExecutionEngine` variant.
    pub fn new_execution_engine() -> Self {
        Self {
            config: ConfigurationBuilder::new_execution_engine(),
        }
    }

    /// Launches the configured service.
    ///
    /// # Arguments
    ///
    /// * `state` - Shared access to the application state.
    /// * `address_book` - Map of component addresses (e.g. "market_data", "allocation").
    ///
    /// # Returns
    ///
    /// * `Ok(())` (never returns in normal operation as it blocks/loops).
    /// * `Err(String)` if initialization or address lookup fails.
    pub fn launch(
        &mut self,
        state: Arc<Mutex<State>>,
        address_book: HashMap<String, Address>,
        admin_rx: Receiver<AdminCommand>,
    ) -> Result<(), String>
    where
        State: Send + 'static,
    {
        self.config.launch(state, address_book, admin_rx)
    }
}

enum ConfigurationBuilder<State> {
    Strategy(Strategy<State>),
    Multiplexer(Multiplexer),
    ExecutionEngine(ExecutionEngine),
}

impl<State> ConfigurationBuilder<State> {
    /// Creates a new Strategy configuration.
    ///
    /// # Arguments
    ///
    /// * `market_data_callback` - The core logic function that processes market data and returns an allocation.
    ///
    /// # Returns
    ///
    /// A `Configuration::Strategy` variant.
    fn new_strategy(
        market_data_callback: Box<dyn FnMut(&mut State, &MarketDataBatch) -> Allocation + Send>,
    ) -> Self {
        Self::Strategy(Strategy {
            market_data_callback,
            runners: RunnerManager::new(),
        })
    }

    /// Creates a new Multiplexer configuration.
    ///
    /// # Returns
    ///
    /// A `Configuration::Multiplexer` variant.
    fn new_multiplexer() -> Self {
        Self::Multiplexer(Multiplexer {
            runners: RunnerManager::new(),
        })
    }

    /// Creates a new Execution Engine configuration.
    ///
    /// # Returns
    ///
    /// A `Configuration::ExecutionEngine` variant.
    fn new_execution_engine() -> Self {
        Self::ExecutionEngine(ExecutionEngine {})
    }

    /// Launches the configured service.
    ///
    /// # Arguments
    ///
    /// * `state` - Shared access to the application state.
    /// * `address_book` - Map of component addresses (e.g. "market_data", "allocation").
    ///
    /// # Returns
    ///
    /// * `Ok(())` (never returns in normal operation as it blocks/loops).
    /// * `Err(String)` if initialization or address lookup fails.
    fn launch(
        &mut self,
        state: Arc<Mutex<State>>,
        address_book: HashMap<String, Address>,
        admin_rx: Receiver<AdminCommand>,
    ) -> Result<(), String>
    where
        State: Send + 'static,
    {
        match self {
            Self::Strategy(strategy) => strategy.run(state, address_book, admin_rx),
            Self::Multiplexer(multiplexer) => multiplexer.run(state, address_book, admin_rx),
            Self::ExecutionEngine(execution_engine) => execution_engine.run(admin_rx),
        }
    }
}

struct Strategy<State> {
    /// Callbacks
    market_data_callback: Box<dyn FnMut(&mut State, &MarketDataBatch) -> Allocation + Send>,

    /// Runners
    ///
    /// A Strategy has only one runner: the market data runner.
    runners: RunnerManager,
}

impl<State> Strategy<State> {
    pub fn run(
        &mut self,
        state: Arc<Mutex<State>>,
        address_book: HashMap<String, Address>,
        _admin_rx: Receiver<AdminCommand>,
    ) -> Result<(), String>
    where
        State: Send + 'static,
    {
        // 1. Get Addresses
        let input_addr = address_book
            .get("market_data")
            .ok_or("Missing 'market_data' address")?
            .clone();

        let output_addr = address_book
            .get("allocation")
            .ok_or("Missing 'allocation' address")?;

        // 2. Create Publisher (Output)
        let publisher = comms::build_publisher::<Allocation>(output_addr)
            .map_err(|e| format!("Failed to create publisher: {}", e))?;

        // 3. Setup logic
        // We take ownership of the callback to move it into the runner logic
        // For multiple runs, we'd need to assume `Box<dyn FnMut + Clone>` or wrap in Arc<Mutex>
        // But run() is called once at startup.
        let mut cb = std::mem::replace(
            &mut self.market_data_callback,
            Box::new(|_, _| Allocation::default()),
        );

        let callback = Box::new(move |state: &mut State, data: MarketDataBatch| {
            let allocation = cb(state, &data);

            // Send allocation strictly
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = publisher.send(&allocation).await;
                });
            });
        });

        self.runners
            .add_runner("market_data", state, callback, input_addr);
        Ok(())
    }
}

struct Multiplexer {
    runners: RunnerManager,
}

impl Multiplexer {
    pub fn run<State>(
        &mut self,
        state: Arc<Mutex<State>>,
        address_book: HashMap<String, Address>,
        admin_rx: Receiver<AdminCommand>,
    ) -> Result<(), String>
    where
        State: Send + 'static,
    {
        // 1. Get Output Address (Allocation)
        let output_addr = address_book
            .get("allocation")
            .ok_or("Missing 'allocation' address for Multiplexer output")?
            .clone();

        // 2. Create Publisher to Allocation
        let publisher = comms::build_publisher::<Allocation>(&output_addr)
            .map_err(|e| format!("Failed to create publisher: {}", e))?;

        // 3. Define the Callback
        // The Multiplexer simply forwards received Allocations to the Execution Engine.
        // It might also log or aggregate in the future.
        let callback = Box::new(move |_state: &mut State, allocation: Allocation| {
            // Forwarding logic
            // We use block_in_place to bridge async send with the sync callback
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Err(e) = publisher.send(&allocation).await {
                        eprintln!("Multiplexer failed to forward allocation: {}", e);
                    }
                });
            });
        });

        // 4. Start the Runner on an Empty Input
        // The Runner will start disconnected. Admin commands will add inputs later.
        // Note: "strategies" is the ID of this runner.
        self.runners
            .add_runner("strategies", state, callback, Address::Empty);

        // 5. Admin Loop (Blocking)
        // We listen for Admin commands dynamically
        println!("Multiplexer running. Waiting for Admin commands...");
        for cmd in admin_rx {
            match cmd {
                AdminCommand::AddStrategy { address } => {
                    println!("Multiplexer adding strategy input: {}", address);
                    self.runners.add_runner_input("strategies", address);
                }
                AdminCommand::Shutdown => {
                    println!("Multiplexer shutting down...");
                    break;
                }
                _ => {
                    // Ignore other commands or implement registry updates here
                }
            }
        }

        Ok(())
    }
}

struct ExecutionEngine {}

impl ExecutionEngine {
    pub fn run(&self, _admin_rx: Receiver<AdminCommand>) -> Result<(), String> {
        Ok(())
    }
}
