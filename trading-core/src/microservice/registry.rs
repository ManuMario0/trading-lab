//! This will hold the custom configuration arguments for the microservice.

use crate::fs::PathManager;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

/// File to backup the registry current configuration.
/// This allow for a fast recovery of the service in case of crash.
///
/// We will be able to make a fast startup by simply loading the file.
/// This allow the end-user to not loose its configuration in case of crash.
pub const PARAMETER_REGISTRY_BACKUP: &str = "parameter_registry_backup";

/// The value of a parameter.
///
/// This enum is used both to store the value of a parameter and its type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

// Helper functions to create ParameterValue instances
// Those helper functions are particularly useful for proper interaction with C++ extraction.
impl ParameterValue {
    pub fn new_string(str: String) -> Self {
        Self::String(str)
    }

    pub fn new_integer(int: i64) -> Self {
        Self::Integer(int)
    }

    pub fn new_float(float: f64) -> Self {
        Self::Float(float)
    }

    pub fn new_boolean(bool: bool) -> Self {
        Self::Boolean(bool)
    }
}

/// A parameter of the running microservice.
///
/// This struct is used to store the value of a parameter and its type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// The name of the parameter.
    name: String,

    /// A description of the parameter.
    description: String,

    /// The current value of the parameter.
    value: ParameterValue,

    /// Whether the parameter can be updated at runtime.
    ///
    /// Note: this is not implemented yet, it is here for future usage.
    updatable: bool,
}

impl Parameter {
    /// Creates a new parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier of the parameter.
    /// * `description` - Human-readable description.
    /// * `startup_value` - The initial value.
    /// * `updatable` - Whether this can be changed at runtime.
    ///
    /// # Returns
    ///
    /// A new `Parameter` instance.
    pub fn new(
        name: String,
        description: String,
        startup_value: ParameterValue,
        updatable: bool,
    ) -> Self {
        Self {
            name,
            description,
            value: startup_value,
            updatable,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_value(&self) -> &ParameterValue {
        &self.value
    }

    pub fn is_updatable(&self) -> bool {
        self.updatable
    }

    pub fn get_value_as_string(&self) -> String {
        match &self.value {
            ParameterValue::String(s) => s.clone(),
            ParameterValue::Integer(i) => i.to_string(),
            ParameterValue::Float(f) => f.to_string(),
            ParameterValue::Boolean(b) => b.to_string(),
        }
    }
}

/// A registry of parameters.
///
/// This struct is used to store the value of a parameter and its type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// A map of parameters.
    parameters: HashMap<String, Parameter>,
}

impl Registry {
    /// Creates a new empty Registry.
    ///
    /// # Returns
    ///
    /// An empty `Registry`.
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    /// Inserts a parameter into the registry.
    ///
    /// # Arguments
    ///
    /// * `parameter` - The `Parameter` to store.
    pub fn insert_parameter(&mut self, parameter: Parameter) {
        self.parameters.insert(parameter.name.clone(), parameter);
    }

    /// Retrieves a parameter by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name key to lookup.
    ///
    /// # Returns
    ///
    /// * `Some(&Parameter)` if found.
    /// * `None` if not found.
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    /// Backups the registry to a file.
    ///
    /// This file is unique to the process and will be used only by the perticular process.
    /// In particular, if multiple instances of the same process are running, they will each have their own backup file.
    ///
    /// This allow for a fast recovery of the service in case of crash.
    ///
    /// # Arguments
    ///
    /// * `paths` - The path manager to use for saving the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err` if file writing fails.
    pub fn backup(&self, paths: PathManager) -> std::io::Result<()> {
        paths.save_config(Path::new(PARAMETER_REGISTRY_BACKUP), self)
    }

    /// Loads the registry from a backup file.
    ///
    /// # Arguments
    ///
    /// * `paths` - The path manager to use for loading the backup.
    ///
    /// # Returns
    ///
    /// * `Ok(Registry)` if parsing succeeds.
    /// * `Err` if file reading or parsing fails.
    pub fn load_backup(paths: PathManager) -> std::io::Result<Self> {
        paths.load_config(Path::new(PARAMETER_REGISTRY_BACKUP))
    }
    /// Returns a list of all parameters.
    pub fn get_parameters(&self) -> Vec<&Parameter> {
        self.parameters.values().collect()
    }
}
