use crate::model::instrument::InstrumentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    instrument_id: InstrumentId,
    quantity: f64,
}

impl Position {
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Allocation {
    positions: HashMap<InstrumentId, Position>,
}

impl Allocation {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    pub fn update_position(&mut self, instrument_id: InstrumentId, quantity: f64) {
        if quantity == 0.0 {
            self.positions.remove(&instrument_id);
        } else {
            let position = Position::new(instrument_id, quantity);
            self.positions.insert(instrument_id, position);
        }
    }

    pub fn get_position(&self, instrument_id: InstrumentId) -> Option<&Position> {
        self.positions.get(&instrument_id)
    }

    pub fn get_positions(&self) -> &HashMap<InstrumentId, Position> {
        &self.positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_update() {
        let mut allocation = Allocation::new();

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
