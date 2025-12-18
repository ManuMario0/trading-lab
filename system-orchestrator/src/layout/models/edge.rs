use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    id: String,
    source: String,
    target: String,
}

impl Edge {
    pub fn new(id: String, source: String, target: String) -> Self {
        Self { id, source, target }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}
