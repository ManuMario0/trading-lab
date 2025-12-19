use super::{edge::Edge, node::Node};
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
