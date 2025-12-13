use super::instrument::Instrument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    instrument: Instrument,
    last: f64,
    bid: f64,
    ask: f64,
    timestamp: i64,
}

impl Price {
    pub fn new(instrument: Instrument, last: f64, bid: f64, ask: f64, timestamp: i64) -> Self {
        Self {
            instrument,
            last,
            bid,
            ask,
            timestamp,
        }
    }

    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    pub fn last(&self) -> f64 {
        self.last
    }

    pub fn bid(&self) -> f64 {
        self.bid
    }

    pub fn ask(&self) -> f64 {
        self.ask
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prices {
    market_data: HashMap<Instrument, Price>,
}

impl Prices {
    pub fn insert(&mut self, instrument: Instrument, price: Price) {
        self.market_data.insert(instrument, price);
    }

    pub fn get(&self, instrument: &Instrument) -> Option<&Price> {
        self.market_data.get(instrument)
    }

    // Helper to check existence (used in Engine logic)
    pub fn contains_key(&self, instrument: &Instrument) -> bool {
        self.market_data.contains_key(instrument)
    }

    // Expose inner map for read-only iteration if needed? Or better:
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Instrument, Price> {
        self.market_data.iter()
    }
}
