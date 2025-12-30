use crate::layout::models::{layout::Layout, node::Node};
use crate::process::models::port_allocator::{Port, PortAllocator};
use std::collections::HashMap;
use trading_core::{
    comms::Address,
    manifest::{Binding, ServiceBindings, ServiceBlueprint, Source},
    microservice::configuration::{
        multiplexer::MULTIPLEXER_MANIFEST,
        portfolio_manager::PORTFOLIO_MANAGER_MANIFEST,
        // strategy::STRATEGY_MANIFEST,
    },
};

pub struct ConfigResolver;

impl ConfigResolver {
    /// Pass 1: Allocate Ports for Outputs and register them
    pub fn allocate_outputs(
        node: &Node,
        allocator: &mut PortAllocator,
        output_registry: &mut HashMap<String, u16>,
    ) {
        let blueprint = match node.category() {
            "PortfolioManager" => Some(PORTFOLIO_MANAGER_MANIFEST.clone()),
            "Multiplexer" => Some(MULTIPLEXER_MANIFEST.clone()),
            _ => None,
        };

        if let Some(bp) = blueprint {
            for port_def in &bp.outputs {
                let port = allocator.allocate().unwrap_or(0);
                let key = format!("{}:{}", node.id(), port_def.name);
                output_registry.insert(key, port);
            }
        } else {
            // Legacy Output Allocation
            let output_port = allocator.allocate().unwrap_or(0);
            match node.category() {
                "Strategy" => {
                    output_registry.insert(format!("{}:allocation", node.id()), output_port);
                }
                "ExecutionEngine" => {
                    output_registry.insert(format!("{}:portfolio", node.id()), output_port);
                    output_registry.insert(format!("{}:default", node.id()), output_port);
                }
                "BrokerGateway" => {
                    output_registry.insert(format!("{}:service", node.id()), output_port);
                    output_registry.insert(format!("{}:execution", node.id()), output_port);
                }
                _ => {}
            }
        }
    }

    /// Pass 2: Resolve Inputs and Generate Arguments
    pub fn resolve_args(
        config: &crate::config::SystemConfig,
        node: &Node,
        layout: &Layout,
        allocator: &mut PortAllocator,
        output_registry: &HashMap<String, u16>,
    ) -> Option<(String, Vec<String>, u16)> {
        let blueprint = match node.category() {
            "PortfolioManager" => Some(PORTFOLIO_MANAGER_MANIFEST.clone()),
            "Multiplexer" => Some(MULTIPLEXER_MANIFEST.clone()),
            _ => None,
        };

        let admin_port = allocator.allocate().unwrap_or(0);

        if let Some(bp) = blueprint {
            let mut bindings = ServiceBindings {
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            };

            // Re-construct Outputs from Registry
            for port_def in &bp.outputs {
                let key = format!("{}:{}", node.id(), port_def.name);
                if let Some(port) = output_registry.get(&key) {
                    bindings.outputs.insert(
                        port_def.name.clone(),
                        Binding::Single(Source {
                            address: Address::zmq_tcp("0.0.0.0", *port),
                            id: 0,
                        }),
                    );
                }
            }

            // Resolve Inputs
            for port_def in &bp.inputs {
                let sources: Vec<&str> = layout
                    .edges()
                    .iter()
                    .filter(|e| e.target() == node.id())
                    .map(|e| e.source())
                    .collect();

                for source_id in sources {
                    if let Some(source_node) = layout.nodes().iter().find(|n| n.id() == source_id) {
                        let output_port_name = match source_node.category() {
                            "Multiplexer" => "allocation",
                            "Strategy" => "allocation",
                            "ExecutionEngine" => "portfolio",
                            "BrokerGateway" => "execution",
                            "PortfolioManager" => "target",
                            _ => "default", // Legacy Fallback
                        };

                        let key = format!("{}:{}", source_id, output_port_name);

                        let port = output_registry
                            .get(&key)
                            .or_else(|| output_registry.get(&format!("{}:default", source_id)))
                            .or_else(|| output_registry.get(&format!("{}:service", source_id)));

                        if let Some(port) = port {
                            bindings.inputs.insert(
                                port_def.name.clone(),
                                Binding::Single(Source {
                                    address: Address::zmq_tcp("127.0.0.1", *port),
                                    id: 0,
                                }),
                            );
                            break;
                        }
                    }
                }
            }

            let args_vec = bp.generate_args(&node.id(), admin_port, bindings);

            let cmd = match node.category() {
                "PortfolioManager" => config.portfolio_manager_path.clone(),
                "Multiplexer" => config.multiplexer_path.clone(),
                _ => "unknown".to_string(),
            };

            Some((cmd, args_vec, admin_port))
        } else {
            Self::resolve_legacy(config, node, output_registry, admin_port)
        }
    }

    fn resolve_legacy(
        config: &crate::config::SystemConfig,
        node: &Node,
        output_registry: &HashMap<String, u16>,
        admin_port: u16,
    ) -> Option<(String, Vec<String>, u16)> {
        let mut args = Vec::new();

        let output_port = output_registry
            .get(&format!("{}:allocation", node.id()))
            .or_else(|| output_registry.get(&format!("{}:service", node.id())))
            .or_else(|| output_registry.get(&format!("{}:default", node.id())))
            .cloned()
            .unwrap_or(0);

        match node.category() {
            "Strategy" => {
                let input = config.data_port;
                let output = output_port;

                args.push(format!("tcp://127.0.0.1:{}", input));
                args.push(format!("tcp://127.0.0.1:{}", output));
                args.push(format!("tcp://*:{}", admin_port));
                Some((config.strategy_lab_path.clone(), args, admin_port))
            }
            "ExecutionEngine" => {
                args.push("--admin-port".to_string());
                args.push(admin_port.to_string());
                args.push("--multiplexer-ports".to_string());
                args.push(config.multiplexer_port.to_string());
                args.push("--data-port".to_string());
                args.push(config.data_port.to_string());
                args.push("--order-port".to_string());
                args.push(config.order_port.to_string());
                Some((config.execution_engine_path.clone(), args, admin_port))
            }
            "BrokerGateway" => {
                let port = output_port;
                args.push("--port".to_string());
                args.push(port.to_string());
                Some((config.broker_gateway_path.clone(), args, admin_port))
            }
            "DataPipeline" => {
                args.push("-u".to_string());
                args.push(config.data_pipeline_path.clone());
                args.push("--port".to_string());
                args.push(config.data_port.to_string());
                Some(("python3".to_string(), args, admin_port))
            }
            _ => None,
        }
    }
}
