//! In-memory database for managing instrument definitions.
//!
//! This module handles querying instruments. Persistence is handled by the runtime.

use crate::model::instrument::InstrumentId;
use crate::model::Instrument;
use std::collections::HashMap;

/// A database for trading instruments.
#[derive(Debug, Default)]
pub struct InstrumentDB {
    instruments: HashMap<InstrumentId, Instrument>,
}

impl InstrumentDB {
    /// Creates a new, empty InstrumentDB.
    pub fn new() -> Self {
        Self {
            instruments: HashMap::new(),
        }
    }

    /// Retrieves an instrument by its ID.
    pub fn get(&self, id: InstrumentId) -> Option<&Instrument> {
        self.instruments.get(&id)
    }

    /// Adds or updates an instrument in the in-memory database.
    pub fn insert(&mut self, id: InstrumentId, instrument: Instrument) {
        self.instruments.insert(id, instrument);
    }

    /// Returns an iterator over the instruments.
    pub fn iter(&self) -> impl Iterator<Item = (&InstrumentId, &Instrument)> {
        self.instruments.iter()
    }
}
