use crate::event_bus::{EventBus, SystemEvent};
use crate::layout::{DeploymentPlan, LayoutEngine};
use crate::registry::ServiceCatalog;
use crate::runtime::{HealthStatus, ServiceProvider};
use anyhow::Result;
use log::{error, info, warn};
use orchestrator_protocol::model::Layout;
use std::sync::Arc;
use tokio::time::{self, Duration};

/// The Manager that ensures Reality matches the Plan.
///
/// Use `Supervisor::new(...)` to create it, and `supervisor.run()` to start the loop.
pub struct Supervisor<P: ServiceProvider> {
    event_bus: EventBus,
    runtime: Arc<P>,

    // State (In Memory SMR)
    // The Catalog is our view of "What binaries are available"
    catalog: ServiceCatalog,

    // The Desired Plan is our view of "What SHOULD be running"
    desired_plan: Option<DeploymentPlan>,

    // We keep the raw Layout to re-resolve if needed (e.g. if Catalog updates)
    current_layout: Option<Layout>,

    // Configuration for port allocation
    base_port: u16,
}

impl<P: ServiceProvider + 'static> Supervisor<P> {
    pub fn new(event_bus: EventBus, runtime: Arc<P>) -> Self {
        Self {
            event_bus,
            runtime,
            catalog: ServiceCatalog::new(),
            desired_plan: None,
            current_layout: None,
            base_port: 6000,
        }
    }

    /// Start the Supervisor Loop.
    /// This consumes the struct and runs forever.
    pub async fn run(mut self) {
        info!("Supervisor: Starting Reconciliation Loop");
        let mut rx = self.event_bus.subscribe();
        let mut ticker = time::interval(Duration::from_secs(1));

        loop {
            // tokio::select! {
            // 1. Handle Events
            info!("DEBUG: Waiting for event...");
            match rx.recv().await {
                Ok(event) => {
                    info!("DEBUG: Event received: {:?}", event);
                    // Flush stdout
                    self.handle_event(event);
                    // Trigger reconcile after every event for now to simulate the loop
                    if let Err(e) = self.reconcile().await {
                        error!("Supervisor: Reconcile Loop Error: {:?}", e);
                    }
                }
                Err(e) => {
                    warn!("Supervisor: EventBus receive error: {:?}", e);
                }
            }

            /*
            _ = ticker.tick() => {
                info!("[TRACE] Supervisor: Starting Reconcile...");
                if let Err(e) = self.reconcile().await {
                    error!("Supervisor: Reconcile Loop Error: {:?}", e);
                }
                info!("[TRACE] Supervisor: Reconcile Complete.");
            }
            */
            // }
        }
    }

    fn handle_event(&mut self, event: SystemEvent) {
        info!("[TRACE] Supervisor: Handling Event...");
        match event {
            SystemEvent::ServiceDiscovered { descriptor } => {
                info!("Supervisor: Service Discovered: {}", descriptor.service);
                self.catalog.register(descriptor);
                info!(
                    "[TRACE] Catalog updated. Total services: {}",
                    self.catalog.get_all_descriptors().len()
                );

                // Reactive: Attempt to resolve pending layout if new information is available
                if self.current_layout.is_some() {
                    info!("Supervisor: Service Discovered. Retrying pending layout resolution...");
                    self.try_resolve_layout();
                }
            }
            SystemEvent::DeployRequested { layout } => {
                info!("Supervisor: Deploy Requested for Layout '{}'", layout.id());
                self.current_layout = Some(layout.clone());
                self.try_resolve_layout();
            }
            SystemEvent::ServiceCrashed { id, .. } => {
                warn!(
                    "Supervisor: Service '{}' Crashed. Will heal on next tick.",
                    id
                );
            }
            _ => {
                info!("[TRACE] Supervisor: Ignored Event: {:?}", event);
            }
        }
        info!("[TRACE] Supervisor: Finished Handling Event");
    }

    /// Attempts to resolve the current layout into a desired plan.
    /// Updates `self.desired_plan` on success.
    fn try_resolve_layout(&mut self) {
        if let Some(layout) = &self.current_layout {
            info!("Supervisor: Attempting to resolve layout...");
            match LayoutEngine::resolve(
                layout,
                &self.catalog,
                self.base_port,
                self.desired_plan.as_ref(),
            ) {
                Ok(plan) => {
                    info!(
                        "Supervisor: Layout Resolved. Desired Plan Updated with {} services.",
                        plan.services().len()
                    );
                    self.desired_plan = Some(plan);
                }
                Err(e) => {
                    // This is expected if not all services are discovered yet.
                    warn!(
                        "Supervisor: Failed to resolve layout (waiting for services?): {:?}",
                        e
                    );
                }
            }
        }
    }

    /// The Core Function: Make Reality == Plan
    async fn reconcile(&self) -> Result<()> {
        info!(
            "DEBUG: Reconcile starting. Plan exists: {}",
            self.desired_plan.is_some()
        );
        let plan = match &self.desired_plan {
            Some(p) => p,
            None => {
                info!("DEBUG: Reconcile early return (no plan)");
                return Ok(());
            }
        };

        // 1. Check Reality (Missing Services)
        for (id, config) in plan.services() {
            let status = self.runtime.probe(id).await?;
            match status {
                HealthStatus::Running(_) => {
                    // It is running. Ideally we check if config matches (reconfiguration).
                    // For now, we assume if it exists, it is good.
                }
                HealthStatus::Stopped | HealthStatus::Failed(_) => {
                    warn!("Supervisor: Service '{}' is NOT running. Spawning...", id);
                    if let Err(e) = self.runtime.spawn(config).await {
                        error!("Supervisor: Failed to spawn '{}': {}", id, e);
                    } else {
                        info!("Supervisor: Service '{}' spawned successfully.", id);
                        // Note: We don't get the PID back from spawn() yet in the trait signature effectively,
                        // but LocalServiceProvider tracks it.

                        // We could emit ServiceStarted here, but let's wait for the next probe to confirm it?
                        // "Level Triggered" vs "Edge Triggered".
                        // Better to emit it now.
                        self.event_bus.publish(SystemEvent::ServiceStarted {
                            id: id.clone(),
                            pid: 0, // Placeholder
                        });
                    }
                }
            }
        }

        // 2. Kill Orphans (Garbage Collection)
        // We list all running processes.
        let running_services = self.runtime.list().await?;
        for id in running_services {
            if !plan.services().contains_key(&id) {
                if let Err(e) = self.runtime.stop(&id).await {
                    error!("Supervisor: Failed to stop orphan '{}': {}", id, e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::model::ServiceConfig; // Internal type
    use async_trait::async_trait;
    use orchestrator_protocol::model::{Layout, Node, ServiceDescriptor};
    use std::sync::{Arc, Mutex};

    // --- Mock Service Provider ---
    #[derive(Clone)]
    struct MockRuntime {
        spawned: Arc<Mutex<Vec<String>>>,
    }

    impl MockRuntime {
        fn new() -> Self {
            Self {
                spawned: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl ServiceProvider for MockRuntime {
        async fn spawn(&self, config: &ServiceConfig) -> Result<()> {
            self.spawned
                .lock()
                .unwrap()
                .push(config.node_id().to_string());
            Ok(())
        }

        async fn stop(&self, _id: &str) -> Result<()> {
            Ok(())
        }

        async fn probe(&self, id: &str) -> Result<HealthStatus> {
            let spawned = self.spawned.lock().unwrap();
            if spawned.contains(&id.to_string()) {
                return Ok(HealthStatus::Running(123));
            }
            Ok(HealthStatus::Stopped)
        }

        async fn list(&self) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    /// Verification Test: Supervisor Reconciliation Loop
    ///
    /// **Objective**: Verify that the Supervisor correctly subscribes to events, updates its internal state
    /// (Catalog and Desired Plan), and triggers the `reconcile()` loop to spawn missing services.
    ///
    /// **Setup**:
    /// 1. `EventBus`: Real in-memory event bus.
    /// 2. `MockRuntime`: A simulated runtime that tracks `spawn` calls instead of starting processes.
    /// 3. `Supervisor`: Real supervisor instance running in a background task.
    ///
    /// **Scenario**:
    /// 1. **Discovery**: Publish `ServiceDiscovered` to register "fake-service".
    /// 2. **deployment**: Publish `DeployRequested` with a Layout requesting "fake-service".
    /// 3. **Reconciliation**: Wait for the Supervisor to tick, resolve the layout, and detect that "node-1" is missing.
    /// 4. **Assertion**: Verify that `MockRuntime.spawned` contains "node-1".
    #[tokio::test]
    async fn test_supervisor_reconciliation() {
        // Initialize logger to capture "Supervisor: ..." logs during test failure/debugging.
        let _ = env_logger::builder().is_test(true).try_init();

        // Setup
        let event_bus = EventBus::new();
        let runtime = Arc::new(MockRuntime::new());
        let supervisor = Supervisor::new(event_bus.clone(), runtime.clone());

        // Run Supervisor background
        let s_handle = tokio::spawn(async move {
            supervisor.run().await;
        });

        // Wait for Supervisor to subscribe
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 1. Service Discovered
        let descriptor = ServiceDescriptor {
            service: "fake-service".to_string(),
            description: "Test".into(),
            version: "1.0".into(),
            binary_path: Some("/tmp/fake-service".into()),
            inputs: vec![],
            outputs: vec![],
        };
        event_bus.publish(SystemEvent::ServiceDiscovered { descriptor });
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 2. Deploy Requested
        // We use the Node struct as defined in `orchestrator-protocol::model`
        let mut layout = Layout::new("test-layout");
        layout.add_node(Node::new(
            "node-1".to_string(),
            "Node 1".to_string(),
            "fake-service".to_string(),
            "Stopped".to_string(),
        ));

        event_bus.publish(SystemEvent::DeployRequested { layout });

        // 3. Wait for Reconcile Loop
        // Ticks every 1s, so we wait 1.2s to be safe
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // 4. Verify
        {
            let spawned = runtime.spawned.lock().unwrap();
            assert!(
                spawned.contains(&"node-1".to_string()),
                "Supervisor FAILED to spawn node-1. Spawned list: {:?}",
                *spawned
            );
        }

        s_handle.abort();
    }

    /// Verification Test: Supervisor Handles Missing Required Inputs gracefully
    ///
    /// **Objective**: Verify that the Supervisor refuses to resolve a layout if required edges are missing,
    /// and DOES NOT spawn the service.
    ///
    /// **Scenario**:
    /// 1. Register "strict-service" with a REQUIRED input port "data_in".
    /// 2. Request deployment of "strict-service" WITHOUT any edges connecting to "data_in".
    /// 3. Assert that NO services are spawned.
    #[tokio::test]
    async fn test_deploy_missing_required_input() {
        let _ = env_logger::builder().is_test(true).try_init();

        let event_bus = EventBus::new();
        let runtime = Arc::new(MockRuntime::new());
        let supervisor = Supervisor::new(event_bus.clone(), runtime.clone());

        let s_handle = tokio::spawn(async move {
            supervisor.run().await;
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 1. Register Strict Service
        let descriptor = ServiceDescriptor {
            service: "strict-service".to_string(),
            description: "Strict Service".into(),
            version: "1.0".into(),
            binary_path: Some("/bin/strict".into()),
            inputs: vec![orchestrator_protocol::model::PortInfo {
                name: "data_in".to_string(),
                data_type: "Any".to_string(),
                required: true,
                is_variadic: false,
            }],
            outputs: vec![],
        };
        event_bus.publish(SystemEvent::ServiceDiscovered { descriptor });
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 2. Request Deployment (Missing Edge)
        let mut layout = Layout::new("broken-layout");
        layout.add_node(Node::new(
            "strict-node".to_string(),
            "Strict Node".to_string(),
            "strict-service".to_string(),
            "Stopped".to_string(),
        ));
        // No edges added!

        event_bus.publish(SystemEvent::DeployRequested { layout });

        // 3. Wait for Reconcile
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // 4. Verify NOTHING spawned
        {
            let spawned = runtime.spawned.lock().unwrap();
            assert!(
                spawned.is_empty(),
                "Supervisor SHOULD NOT have spawned strict-node because required input was missing! Spawned: {:?}",
                *spawned
            );
        }
        s_handle.abort();
    }

    /// Verification Test: Supervisor Unknown Service
    ///
    /// **Objective**: Verify that if a layout requests a service that has NOT been discovered,
    /// the Supervisor logs an error and does not crash or spawn ghosts.
    #[tokio::test]
    async fn test_deploy_unknown_service() {
        let _ = env_logger::builder().is_test(true).try_init();

        let event_bus = EventBus::new();
        let runtime = Arc::new(MockRuntime::new());
        let supervisor = Supervisor::new(event_bus.clone(), runtime.clone());

        let s_handle = tokio::spawn(async move {
            supervisor.run().await;
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 1. Request Unknown Service
        let mut layout = Layout::new("ghost-layout");
        layout.add_node(Node::new(
            "ghost-node".to_string(),
            "Ghost Node".to_string(),
            "unknown-service".to_string(),
            "Stopped".to_string(),
        ));

        event_bus.publish(SystemEvent::DeployRequested { layout });
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // 2. Verify Empty
        {
            let spawned = runtime.spawned.lock().unwrap();
            assert!(spawned.is_empty(), "Supervisor spawned a ghost!");
        }
        s_handle.abort();
    }

    /// Verification Test: Repros the Integration Test Scenario
    ///
    /// **Objective**: Exact replica of the integration test layout to check for panics.
    #[tokio::test]
    async fn test_repro_integration_failure() {
        let _ = env_logger::builder().is_test(true).try_init();

        let event_bus = EventBus::new();
        let runtime = Arc::new(MockRuntime::new());
        let supervisor = Supervisor::new(event_bus.clone(), runtime.clone());

        let s_handle = tokio::spawn(async move {
            supervisor.run().await;
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 1. Discovery (Mocking what DiskWatcher does)
        // BrokerGateway
        event_bus.publish(SystemEvent::ServiceDiscovered {
            descriptor: ServiceDescriptor {
                service: "BrokerGateway".into(),
                description: "D".into(),
                version: "1".into(),
                binary_path: Some("/bin/bg".into()),
                inputs: vec![],
                outputs: vec![orchestrator_protocol::model::PortInfo {
                    name: "market_data".into(),
                    data_type: "Any".into(),
                    required: true,
                    is_variadic: false,
                }],
            },
        });
        // Strategy
        event_bus.publish(SystemEvent::ServiceDiscovered {
            descriptor: ServiceDescriptor {
                service: "Strategy".into(),
                description: "D".into(),
                version: "1".into(),
                binary_path: Some("/bin/st".into()),
                inputs: vec![orchestrator_protocol::model::PortInfo {
                    name: "market_data".into(),
                    data_type: "Any".into(),
                    required: true,
                    is_variadic: false,
                }],
                outputs: vec![orchestrator_protocol::model::PortInfo {
                    name: "allocation".into(),
                    data_type: "Any".into(),
                    required: true,
                    is_variadic: false,
                }],
            },
        });
        // PortfolioManager
        event_bus.publish(SystemEvent::ServiceDiscovered {
            descriptor: ServiceDescriptor {
                service: "PortfolioManager".into(),
                description: "D".into(),
                version: "1".into(),
                binary_path: Some("/bin/pm".into()),
                inputs: vec![
                    orchestrator_protocol::model::PortInfo {
                        name: "market_data".into(),
                        data_type: "Any".into(),
                        required: false,
                        is_variadic: false,
                    },
                    orchestrator_protocol::model::PortInfo {
                        name: "allocation".into(),
                        data_type: "Any".into(),
                        required: true,
                        is_variadic: false,
                    },
                ],
                outputs: vec![],
            },
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // 2. Deploy Request (Exact JSON structure from simple_layout.json mapped to Structs)
        let mut layout = Layout::new("integration-test-01");
        // Nodes
        layout.add_node(Node::new(
            "gateway-node".into(),
            "My Broker Gateway".into(),
            "BrokerGateway".into(),
            "Stopped".into(),
        ));
        layout.add_node(Node::new(
            "strategy-node".into(),
            "My Strategy".into(),
            "Strategy".into(),
            "Stopped".into(),
        ));
        layout.add_node(Node::new(
            "portfolio-node".into(),
            "My Portfolio Manager".into(),
            "PortfolioManager".into(),
            "Stopped".into(),
        ));
        // Edges
        use orchestrator_protocol::model::Edge;
        layout.add_edge(Edge::new(
            "edge-md-1".into(),
            "gateway-node".into(),
            "market_data".into(),
            "strategy-node".into(),
            "market_data".into(),
        ));
        layout.add_edge(Edge::new(
            "edge-md-2".into(),
            "gateway-node".into(),
            "market_data".into(),
            "portfolio-node".into(),
            "market_data".into(),
        ));
        layout.add_edge(Edge::new(
            "edge-alloc".into(),
            "strategy-node".into(),
            "allocation".into(),
            "portfolio-node".into(),
            "allocation".into(),
        ));

        event_bus.publish(SystemEvent::DeployRequested { layout });

        // 3. Verify
        tokio::time::sleep(Duration::from_millis(1000)).await;

        {
            let spawned = runtime.spawned.lock().unwrap();
            assert!(
                spawned.contains(&"gateway-node".to_string()),
                "Gateway not spawned: {:?}",
                *spawned
            );
            assert!(
                spawned.contains(&"strategy-node".to_string()),
                "Strategy not spawned: {:?}",
                *spawned
            );
            assert!(
                spawned.contains(&"portfolio-node".to_string()),
                "Portfolio not spawned: {:?}",
                *spawned
            );
        }
        s_handle.abort();
    }
}
