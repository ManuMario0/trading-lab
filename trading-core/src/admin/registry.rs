use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Defines the data type of a configuration parameter.
///
/// This enum is used to validate and parse parameter values in the admin interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// A text string value.
    String,
    /// An integer number.
    Integer,
    /// A floating-point number.
    Float,
    /// A boolean flag (true/false).
    Boolean,
}

/// Represents the definition and current state of a configuration parameter.
///
/// This struct allows the admin interface to display parameter metadata (like
/// description and type) and its current value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// The unique identifier for the parameter.
    pub name: String,
    /// A human-readable description of what the parameter controls.
    pub description: String,
    /// The data type of the parameter.
    pub param_type: ParameterType,
    /// The default value for the parameter.
    pub default_value: String,
    /// The current active value of the parameter.
    pub current_value: String,
    /// Indicates whether this parameter can be modified at runtime.
    pub is_editable: bool,
}

/// A mechanism to update a value when changed via the Admin interface.
///
/// Implement this trait to define how a specific parameter should update the system
/// state when its value is modified.
pub trait ParameterUpdater: Send + Sync {
    /// Applies the new value to the system.
    ///
    /// # Arguments
    ///
    /// * `new_value` - The new value as a string. The implementation should parse
    ///   this string into the appropriate type.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the update was successful.
    /// * `Err(String)` if the update failed (e.g., parsing error or invalid value).
    fn update(&self, new_value: &str) -> Result<(), String>;
}

/// A central registry for managing system configuration parameters.
///
/// The Registry holds parameter definitions and their associated updaters. It allows
/// components to register their settings and the admin interface to query and modify
/// them securely.
pub struct Registry {
    /// Maps parameter names to their definitions.
    parameters: HashMap<String, ParameterDefinition>,
    /// Maps parameter names to their updater logic.
    updaters: HashMap<String, Box<dyn ParameterUpdater>>,
}

impl Registry {
    /// Creates a new, empty Registry.
    ///
    /// # Returns
    ///
    /// A new, empty `Registry` ready for params.
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            updaters: HashMap::new(),
        }
    }

    /// Registers a new parameter with the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the parameter.
    /// * `description` - A description for the admin UI.
    /// * `param_type` - The expected data type.
    /// * `default_value` - The initial value.
    /// * `updater` - An optional `ParameterUpdater`. If provided, the parameter
    ///   will be marked as editable, and this updater will be called when `update`
    ///   is invoked.
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

    /// Retrieves a list of all registered parameter definitions.
    ///
    /// # Returns
    ///
    /// A vector containing definitions for all parameters in the registry.
    pub fn get_all(&self) -> Vec<ParameterDefinition> {
        self.parameters.values().cloned().collect()
    }

    /// Updates the value of a registered parameter.
    ///
    /// This method checks if the parameter exists and is editable. If so, it invokes
    /// the associated `ParameterUpdater` and updates the stored current value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the parameter to update.
    /// * `value` - The new value as a string.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err(String)` if the parameter is not found, not editable, or if the
    ///   update fails.
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
    /// A global, thread-safe instance of the Registry.
    ///
    /// accessing `GLOBAL_REGISTRY` allows any component in the application to
    /// register parameters or for the admin server to list/update them.
    pub static ref GLOBAL_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
}
