//! In-Memory Database of what services are available to run.
//!
//! This struct is the "Knowledge Base" (Registry).
//! It is PURE data. It does not do I/O.
//!
//! This should not be updated directly but through events, allowing clean state machine design.
use orchestrator_protocol::model::ServiceDescriptor;
use std::collections::HashMap;

/// The In-Memory Database of what services are available to run.
///
/// This struct is the "Knowledge Base" (Registry).
/// It is PURE data. It does not do I/O.
#[derive(Debug, Default, Clone)]
pub struct ServiceCatalog {
    /// Maps ServiceType (e.g. "strategy") -> Descriptor
    services: HashMap<String, ServiceDescriptor>,
}

impl ServiceCatalog {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register or Update a service definition.
    pub fn register(&mut self, descriptor: ServiceDescriptor) {
        // In the future, we could add validation logic here (e.g. semantic version check)
        self.services.insert(descriptor.service.clone(), descriptor);
    }

    /// Remove a service definition.
    pub fn unregister(&mut self, service_type: &str) {
        self.services.remove(service_type);
    }

    /// Retrieve a service definition.
    pub fn get(&self, service_type: &str) -> Option<&ServiceDescriptor> {
        self.services.get(service_type)
    }

    /// List all known service types.
    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    /// Returns a list of all descriptors (cloned).
    /// Useful for API responses.
    pub fn get_all_descriptors(&self) -> Vec<ServiceDescriptor> {
        self.services.values().cloned().collect()
    }
}
