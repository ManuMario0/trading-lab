use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub default_value: String,
    pub current_value: String,
    pub is_editable: bool,
}

/// A mechanism to update a value when changed via Admin
pub trait ParameterUpdater: Send + Sync {
    fn update(&self, new_value: &str) -> Result<(), String>;
}

pub struct Registry {
    parameters: HashMap<String, ParameterDefinition>,
    updaters: HashMap<String, Box<dyn ParameterUpdater>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            updaters: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: String,
        description: String,
        param_type: ParameterType,
        default_value: String,
        updater: Option<Box<dyn ParameterUpdater>>,
    ) {
        let def = ParameterDefinition {
            name: name.clone(),
            description,
            param_type,
            default_value: default_value.clone(),
            current_value: default_value,
            is_editable: updater.is_some(),
        };
        self.parameters.insert(name.clone(), def);

        if let Some(up) = updater {
            self.updaters.insert(name, up);
        }
    }

    pub fn get_all(&self) -> Vec<ParameterDefinition> {
        self.parameters.values().cloned().collect()
    }

    pub fn update(&mut self, name: &str, value: &str) -> Result<(), String> {
        if let Some(param) = self.parameters.get_mut(name) {
            if let Some(updater) = self.updaters.get(name) {
                updater.update(value)?;
                param.current_value = value.to_string();
                Ok(())
            } else {
                Err("Parameter is not editable".to_string())
            }
        } else {
            Err("Parameter not found".to_string())
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
}
