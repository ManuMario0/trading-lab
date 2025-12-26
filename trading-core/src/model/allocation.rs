use crate::model::{identity::Identity, instrument::InstrumentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a held position in a specific instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// The ID of the instrument.
    instrument_id: InstrumentId,
    /// The quantity held. Positive for long, negative for short.
    quantity: f64,
}

impl Position {
    /// Creates a new Position instance.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The unique identifier of the instrument.
    /// * `quantity` - The quantity held (positive for long, negative for short).
    ///
    /// # Returns
    ///
    /// A new `Position` instance.
    pub fn new(instrument_id: InstrumentId, quantity: f64) -> Self {
        Self {
            instrument_id,
            quantity,
        }
    }

    pub fn get_instrument_id(&self) -> InstrumentId {
        self.instrument_id
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn set_quantity(&mut self, quantity: f64) {
        self.quantity = quantity;
    }
}

/// Represents a target allocation of instruments.
/// This is used by strategies to communicate their desired holdings to the execution engine.
/// It does not track cash, costs, or PnL.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// The ID of the allocation.
    id: usize,
    /// The source of the allocation, e.g. "strategy" or "portfolio".
    source: String,
    /// The timestamp of the allocation.
    timestamp: u128,
    /// The positions in the allocation.
    positions: HashMap<InstrumentId, Position>,
}

impl Allocation {
    /// Creates a new Allocation instance for a specific identity.
    ///
    /// # Arguments
    ///
    /// * `identity` - The identity of the source generating this allocation (e.g., a strategy).
    ///
    /// # Returns
    ///
    /// A new, empty `Allocation` instance tagged with the identity and current timestamp.
    pub fn new(identity: Identity) -> Self {
        Self {
            id: identity.get_identifier(),
            source: identity.get_name().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            positions: HashMap::new(),
        }
    }

    /// Updates the position for a specific instrument.
    ///
    /// If the quantity is 0.0, the position is removed from the allocation.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The unique identifier of the instrument.
    /// * `quantity` - The new target quantity.
    pub fn update_position(&mut self, instrument_id: InstrumentId, quantity: f64) {
        if quantity == 0.0 {
            self.positions.remove(&instrument_id);
        } else {
            let position = Position::new(instrument_id, quantity);
            self.positions.insert(instrument_id, position);
        }
    }

    /// Retrieves a specific position from the allocation.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The unique identifier of the instrument.
    ///
    /// # Returns
    ///
    /// `Some(&Position)` if found, or `None` if not present.
    pub fn get_position(&self, instrument_id: InstrumentId) -> Option<&Position> {
        self.positions.get(&instrument_id)
    }

    pub fn get_positions(&self) -> &HashMap<InstrumentId, Position> {
        &self.positions
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_source(&self) -> &str {
        &self.source
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::identity::Identity;

    #[test]
    fn test_allocation_update() {
        let mut allocation = Allocation::new(Identity::new("strategy", "1.0"));

        // Add position
        allocation.update_position(1, 10.0);
        let pos = allocation.get_position(1).unwrap();
        assert_eq!(pos.get_quantity(), 10.0);

        // Update position
        allocation.update_position(1, 20.0);
        let pos = allocation.get_position(1).unwrap();
        assert_eq!(pos.get_quantity(), 20.0);

        // Remove position
        allocation.update_position(1, 0.0);
        assert!(allocation.get_position(1).is_none());
    }
}
