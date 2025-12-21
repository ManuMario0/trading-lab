use crate::model::{instrument::InstrumentId, Instrument, InstrumentDB};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    instrument_id: InstrumentId,
    price: f64,
    timestamp: u64,
}

impl PriceUpdate {
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
    pub fn new(updates: Vec<PriceUpdate>) -> Self {
        Self { updates }
    }

    pub fn get_updates(&self) -> &Vec<PriceUpdate> {
        &self.updates
    }

    pub fn clear(&mut self) {
        self.updates.clear();
    }
}
