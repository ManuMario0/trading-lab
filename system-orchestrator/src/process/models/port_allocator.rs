use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::RangeInclusive};

pub type Port = u16;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortAllocator {
    range: RangeInclusive<Port>,
    used_ports: HashSet<Port>,
}

impl PortAllocator {
    pub fn new(start: Port, end: Port) -> Self {
        Self::new_from_range(start..=end)
    }

    pub fn new_from_allocator(allocator: &PortAllocator) -> Self {
        Self::new_from_range(allocator.range.clone())
    }

    pub fn new_from_range(range: RangeInclusive<Port>) -> Self {
        Self {
            range,
            used_ports: HashSet::new(),
        }
    }

    pub fn reserve(&mut self, port: Port) -> bool {
        if self.range.contains(&port) {
            self.used_ports.insert(port);
            true
        } else {
            false
        }
    }

    pub fn allocate(&mut self) -> Option<Port> {
        for port in self.range.clone() {
            if !self.used_ports.contains(&port) {
                self.used_ports.insert(port);
                return Some(port);
            }
        }
        None
    }

    pub fn release(&mut self, port: Port) {
        self.used_ports.remove(&port);
    }
}
