//! A dummy implementation of a `DataFeed` microservice.
//!
//! This service generates synthetic random market data for testing purposes.
//! It simulates a simple random walk for a set of instruments.

use anyhow::Result;
use rand::Rng;
use trading::model::market_data::{MarketDataBatch, PriceUpdate};
use trading::traits::data_feed::DataFeed;
use trading_core::microservice::configuration::feeder::Feeder;
use trading_core::microservice::configuration::Configuration;
use trading_core::microservice::Microservice;

/// A simple random walk data generator.
struct RandomFeed {
    instruments: Vec<usize>,
    prices: Vec<f64>,
}

impl RandomFeed {
    fn new(num_instruments: usize, start_price: f64) -> Self {
        let instruments: Vec<usize> = (1..=num_instruments).collect();
        let prices = vec![start_price; num_instruments];
        Self {
            instruments,
            prices,
        }
    }
}

impl DataFeed for RandomFeed {
    fn get_market_data(&mut self) -> Option<MarketDataBatch> {
        let mut rng = rand::thread_rng();
        let mut updates = Vec::new();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (i, &instrument_id) in self.instruments.iter().enumerate() {
            // Random walk: +/- 1%
            let change_pct = rng.gen_range(-0.01..0.01);
            self.prices[i] *= 1.0 + change_pct;

            // Ensure price stays positive
            if self.prices[i] < 0.01 {
                self.prices[i] = 0.01;
            }

            updates.push(PriceUpdate::new(
                instrument_id,
                self.prices[i] * 0.999, // Bid
                self.prices[i] * 1.001, // Ask
                self.prices[i],         // Last
                now,
            ));
        }

        // Return a batch with updates for all instruments
        Some(MarketDataBatch::new(updates))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Initial State Factory
    // We pass the args using the new signature, though we don't need them here.
    let initial_state = |_: &_| -> Box<dyn DataFeed + Send> {
        // Create a feed with 3 instruments starting at $100.00
        Box::new(RandomFeed::new(3, 100.0))
    };

    // 2. Configuration
    let config = Configuration::new(Feeder::new());

    // 3. Run Microservice
    let service = Microservice::new(
        initial_state,
        config,
        env!("CARGO_PKG_VERSION").to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );
    service.run().await;

    Ok(())
}
