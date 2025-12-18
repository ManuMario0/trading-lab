use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    id: String,
    name: String,
    category: String,
    position: (f64, f64),
    status: String,
}

impl Node {
    pub fn new(
        id: String,
        name: String,
        category: String,
        position: (f64, f64),
        status: String,
    ) -> Self {
        Self {
            id,
            name,
            category,
            position,
            status,
        }
    }

    // Accessors
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn position(&self) -> (f64, f64) {
        self.position
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    // Mutators (if needed, or keep immutable)
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}
