//! Market Data models.
//!
//! Includes `PriceUpdate` for individual ticks and `MarketDataBatch` for efficient network transmission.

use crate::model::{instrument::InstrumentId, Instrument, InstrumentDB};
use serde::{Deserialize, Serialize};

/// Represents a single update to the price of an instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    /// The ID of the instrument.
    instrument_id: InstrumentId,
    /// The new price.
    price: f64,
    /// The timestamp of the update (Unix timestamp).
    timestamp: u64,
}

impl PriceUpdate {
    /// Creates a new PriceUpdate.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The ID of the instrument.
    /// * `price` - The current price.
    /// * `timestamp` - The unix timestamp of the tick.
    ///
    /// # Returns
    ///
    /// A new `PriceUpdate`.
    pub fn new(instrument_id: InstrumentId, price: f64, timestamp: u64) -> Self {
        Self {
            instrument_id,
            price,
            timestamp,
        }
    }

    pub fn get_instrument_id(&self) -> InstrumentId {
        self.instrument_id
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Resolves the associated Instrument from a database.
    ///
    /// # Arguments
    ///
    /// * `instrument_db` - Reference to the `InstrumentDB`.
    ///
    /// # Returns
    ///
    /// * `Some(&Instrument)` if the ID is found.
    /// * `None` otherwise.
    pub fn get_instrument<'a>(&self, instrument_db: &'a InstrumentDB) -> Option<&'a Instrument> {
        instrument_db.get(self.instrument_id)
    }
}

/// Represents a batch of market data updates sent over the network.
/// Vectorized for performance (one packet = many updates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataBatch {
    updates: Vec<PriceUpdate>,
}

impl MarketDataBatch {
    /// Creates a new batch of market data updates.
    ///
    /// # Arguments
    ///
    /// * `updates` - A vector of `PriceUpdate`.
    ///
    /// # Returns
    ///
    /// A new `MarketDataBatch`.
    pub fn new(updates: Vec<PriceUpdate>) -> Self {
        Self { updates }
    }

    pub fn get_updates(&self) -> &Vec<PriceUpdate> {
        &self.updates
    }

    pub fn get_count(&self) -> usize {
        self.updates.len()
    }

    pub fn get_update_at(&self, index: usize) -> &PriceUpdate {
        &self.updates[index]
    }

    pub fn clear(&mut self) {
        self.updates.clear();
    }
}
