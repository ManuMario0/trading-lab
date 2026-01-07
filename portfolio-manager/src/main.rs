use anyhow::Result;
use portfolio_manager::{model::config::AllocationConfig, risk_guard::RiskGuard, PortfolioManager};
use trading_core::microservice::{
    configuration::{portfolio_manager::PortfolioManager as ServiceWrapper, Configuration},
    Microservice,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Define Initialization Logic
    let initial_state = |_: &_| {
        // Initialize RiskGuard
        let mut risk_guard = RiskGuard::new();
        risk_guard.add_policy(Box::new(
            portfolio_manager::risk_guard::max_allocation::MaxAllocationPolicy,
        ));
        risk_guard.add_policy(Box::new(
            portfolio_manager::risk_guard::max_position_size::MaxPositionSizePolicy {
                max_percent: 0.10, // 10% max pos size
            },
        ));

        // Seed default config for testing/dummy support
        let mut seeded_config = AllocationConfig::default();
        seeded_config.insert(
            portfolio_manager::model::ids::MultiplexerId::new("KellyMux_Aggregated"),
            portfolio_manager::model::config::StrategyConfig::new(1.0, 0.20, 0.0), // 100% alloc
        );

        PortfolioManager::new(seeded_config, risk_guard)
    };

    // 2. Wrap Config
    let config = Configuration::new(ServiceWrapper::new());

    // 3. Create and Run Microservice
    // Microservice handles args, admin port, runners launch, and shutdown loop
    let service = Microservice::new(
        initial_state,
        config,
        env!("CARGO_PKG_VERSION").to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );
    service.run().await;

    Ok(())
}
