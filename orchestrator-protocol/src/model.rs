use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Layout {
    id: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Layout {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

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

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RunMode {
    BacktestFast,
    Paper,
    Live,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
}
