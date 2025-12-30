use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfig {
    pub server_port: u16,
    pub log_level: String,

    // ZMQ Ports
    pub admin_port: u16,       // Engine Admin (REP)
    pub multiplexer_port: u16, // Mpx Out -> Engine In (PUB)
    pub data_port: u16,        // Data -> Strat (PUB)
    pub order_port: u16,       // Engine Order Out (PUB)

    pub multiplexer_input_port: u16, // Strat Out -> Mpx In (PULL)
    pub multiplexer_admin_port: u16, // Mpx Admin (REP)
    pub strategy_admin_port: u16,    // Strat Admin (REP)

    // Binary Paths
    pub data_pipeline_path: String,
    pub strategy_lab_path: String,
    pub multiplexer_path: String,
    pub execution_engine_path: String,
    pub gateway_paper_path: String,
    pub portfolio_manager_path: String,
    pub broker_gateway_path: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            server_port: 3000,
            log_level: "info".to_string(),

            admin_port: 5560,
            multiplexer_port: 5561,
            data_port: 5562,
            order_port: 5570,

            multiplexer_input_port: 5564,
            multiplexer_admin_port: 5565,
            strategy_admin_port: 5566,

            // Defaults assume we are running from root of repo or specific layout.
            // These will likely be overridden by config.toml
            data_pipeline_path: "../data-pipeline/src/run_replay_mvp.py".to_string(),
            strategy_lab_path: "../strategy-lab/build/strategy_lab".to_string(),
            multiplexer_path: "../multiplexer/build/multiplexer".to_string(),
            execution_engine_path: "../execution-engine/target/debug/execution-engine".to_string(),
            gateway_paper_path: "../gateways/gateway-paper/target/debug/gateway-paper".to_string(),
            portfolio_manager_path: "../portfolio-manager/target/debug/portfolio-manager"
                .to_string(),
            broker_gateway_path: "../broker-gateway/target/debug/broker-gateway".to_string(),
        }
    }
}
