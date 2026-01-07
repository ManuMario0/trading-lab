//! Configuration and lifecycle management for the `Feeder` microservice.
//!
//! This module defines the `Feeder` service type using the `define_service!` macro.
//! The `Feeder` acts as the source of truth for market data in the system pipeline.
//! It reads from a `DataFeed` implementation and publishes `MarketDataBatch` messages
//! to its output port.

use crate::comms::Address;
use crate::define_service;
use crate::framework::runner::ManagedRunner;
use crate::framework::runner_manager::RunnerManager;
use crate::manifest::ServiceBlueprint;
use crate::microservice::configuration::Configurable;
use log::{error, info};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use trading::model::market_data::MarketDataBatch;
use trading::traits::data_feed::DataFeed;

define_service!(
    name: feeder_gen,
    service_type: "Feeder",
    inputs: {},
    outputs: {
        market_data => MarketDataBatch
    }
);

#[derive(Clone)]
pub struct Feeder;

impl Feeder {
    pub fn new() -> Self {
        Self
    }
}

// Custom Runner for Feeder Source
pub struct FeederRunner {
    handle: Option<JoinHandle<()>>,
    stop_tx: Sender<()>,
}

impl ManagedRunner for FeederRunner {
    fn shutdown(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn update_address(&mut self, _address: Address) {
        // Feeder is a Source, it doesn't have an input address to update
        log::warn!("Feeder received update_address but it is a Source node.");
    }

    fn add_input(&mut self, _address: Address) {
        log::warn!("Feeder received add_input but it is a Source node.");
    }

    fn disconnect_input(&mut self, _address: Address) {
        log::warn!("Feeder received disconnect_input but it is a Source node.");
    }
}

impl Configurable for Feeder {
    type State = Box<dyn DataFeed + Send>;

    fn create_runners(
        &self,
        id: crate::model::identity::Id,
        bindings: crate::manifest::ServiceBindings,
        state: Arc<Mutex<Self::State>>,
    ) -> Result<RunnerManager, String> {
        let mut manager = RunnerManager::new();

        let market_data_out = bindings
            .outputs
            .get("market_data")
            .ok_or("Missing binding for 'market_data' output")?
            .clone();

        let (stop_tx, stop_rx) = mpsc::channel();

        // The Feeder is a simple loop that continually polls the DataFeed trait
        // and publishes to the output.
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let address = match market_data_out {
                    crate::manifest::Binding::Single(source) => source.address,
                    _ => panic!("Feeder output 'market_data' must be a Single binding"),
                };

                let publisher =
                    crate::comms::build_publisher::<MarketDataBatch>(&address, id).unwrap();

                info!("Feeder runner started. Publishing to {:?}", address);

                let mut ticker = tokio::time::interval(std::time::Duration::from_millis(50));

                loop {
                    // Check stop signal non-blocking
                    if stop_rx.try_recv().is_ok() {
                        info!("Feeder received stop signal.");
                        break;
                    }

                    ticker.tick().await;

                    let batch_opt = {
                        let mut guard = state.lock().unwrap();
                        guard.get_market_data()
                    };

                    if let Some(batch) = batch_opt {
                        if batch.get_count() > 0 {
                            if let Err(e) = publisher.send(batch).await {
                                error!("Feeder failed to send batch: {}", e);
                            }
                        }
                    }
                }
            });
        });

        manager.add_managed_runner(
            "feeder_source",
            Box::new(FeederRunner {
                handle: Some(handle),
                stop_tx,
            }),
        );

        Ok(manager)
    }

    fn manifest(&self) -> &ServiceBlueprint {
        &feeder_gen::MANIFEST
    }
}
