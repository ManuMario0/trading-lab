use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessConfig {
    name: String,
    category: String,
    cmd: String,
    args: Vec<String>,
}

impl ProcessConfig {
    pub fn new(name: String, category: String, cmd: String, args: Vec<String>) -> Self {
        Self {
            name,
            category,
            cmd,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}
