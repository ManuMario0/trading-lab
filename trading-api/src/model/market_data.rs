//! Market Data models.
//!
//! Includes `PriceUpdate` for individual ticks and `MarketDataBatch` for efficient network transmission.

use crate::model::{instrument::InstrumentId, Instrument, InstrumentDB};
use serde::{Deserialize, Serialize};

/// Represents a single update to the price of an instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    /// The ID of the instrument.
    pub instrument_id: InstrumentId,
    /// The best bid price.
    pub bid: f64,
    /// The best ask price.
    pub ask: f64,
    /// The last traded price.
    pub last: f64,
    /// The timestamp of the update (Unix timestamp).
    pub timestamp: u64,
}

impl PriceUpdate {
    /// Creates a new PriceUpdate.
    pub fn new(instrument_id: InstrumentId, bid: f64, ask: f64, last: f64, timestamp: u64) -> Self {
        Self {
            instrument_id,
            bid,
            ask,
            last,
            timestamp,
        }
    }

    pub fn get_instrument_id(&self) -> InstrumentId {
        self.instrument_id
    }

    pub fn get_bid(&self) -> f64 {
        self.bid
    }

    pub fn get_ask(&self) -> f64 {
        self.ask
    }

    pub fn get_last(&self) -> f64 {
        self.last
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Resolves the associated Instrument from a database.
    pub fn get_instrument<'a>(&self, instrument_db: &'a InstrumentDB) -> Option<&'a Instrument> {
        instrument_db.get(self.instrument_id)
    }
}

/// Represents a batch of market data updates sent over the network.
/// Vectorized for performance (Structure of Arrays layout).
/// This layout is cache-friendly and allows direct access to vectors for ML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataBatch {
    instrument_ids: Vec<InstrumentId>,
    bid_prices: Vec<f64>,
    ask_prices: Vec<f64>,
    last_prices: Vec<f64>,
    timestamps: Vec<u64>,
}

impl MarketDataBatch {
    /// Creates a new batch from a vector of updates.
    /// Helpful for backward compatibility and manual construction.
    pub fn new(updates: Vec<PriceUpdate>) -> Self {
        let count = updates.len();
        let mut instrument_ids = Vec::with_capacity(count);
        let mut bid_prices = Vec::with_capacity(count);
        let mut ask_prices = Vec::with_capacity(count);
        let mut last_prices = Vec::with_capacity(count);
        let mut timestamps = Vec::with_capacity(count);

        for update in updates {
            instrument_ids.push(update.instrument_id);
            bid_prices.push(update.bid);
            ask_prices.push(update.ask);
            last_prices.push(update.last);
            timestamps.push(update.timestamp);
        }

        Self {
            instrument_ids,
            bid_prices,
            ask_prices,
            last_prices,
            timestamps,
        }
    }

    /// Creates a new batch directly from vectors.
    /// Unsafe because it assumes all vectors have the same length.
    pub fn from_vectors(
        instrument_ids: Vec<InstrumentId>,
        bid_prices: Vec<f64>,
        ask_prices: Vec<f64>,
        last_prices: Vec<f64>,
        timestamps: Vec<u64>,
    ) -> Self {
        // Basic sanity check
        assert_eq!(instrument_ids.len(), bid_prices.len());
        assert_eq!(instrument_ids.len(), ask_prices.len());
        assert_eq!(instrument_ids.len(), last_prices.len());
        assert_eq!(instrument_ids.len(), timestamps.len());

        Self {
            instrument_ids,
            bid_prices,
            ask_prices,
            last_prices,
            timestamps,
        }
    }

    pub fn get_count(&self) -> usize {
        self.instrument_ids.len()
    }

    pub fn clear(&mut self) {
        self.instrument_ids.clear();
        self.bid_prices.clear();
        self.ask_prices.clear();
        self.last_prices.clear();
        self.timestamps.clear();
    }

    pub fn add_update(&mut self, update: PriceUpdate) {
        self.instrument_ids.push(update.instrument_id);
        self.bid_prices.push(update.bid);
        self.ask_prices.push(update.ask);
        self.last_prices.push(update.last);
        self.timestamps.push(update.timestamp);
    }

    pub fn get_update_at(&self, index: usize) -> PriceUpdate {
        PriceUpdate {
            instrument_id: self.instrument_ids[index],
            bid: self.bid_prices[index],
            ask: self.ask_prices[index],
            last: self.last_prices[index],
            timestamp: self.timestamps[index],
        }
    }

    /// Returns an iterator over the updates in the batch.
    pub fn iter(&'_ self) -> MarketDataBatchIterator<'_> {
        MarketDataBatchIterator {
            batch: self,
            index: 0,
        }
    }
}

pub struct MarketDataBatchIterator<'a> {
    batch: &'a MarketDataBatch,
    index: usize,
}

impl<'a> Iterator for MarketDataBatchIterator<'a> {
    type Item = PriceUpdate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.batch.get_count() {
            let update = PriceUpdate {
                instrument_id: self.batch.instrument_ids[self.index],
                bid: self.batch.bid_prices[self.index],
                ask: self.batch.ask_prices[self.index],
                last: self.batch.last_prices[self.index],
                timestamp: self.batch.timestamps[self.index],
            };
            self.index += 1;
            Some(update)
        } else {
            None
        }
    }
}
