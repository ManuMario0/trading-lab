use execution_engine::engine::Engine;
use execution_engine::exchange::mock::MockExchange;
use execution_engine::gateway::MockGateway;
use execution_engine::models::{
    AllocationConfig, IngressMessage, Instrument, MultiplexerId, Price, StrategyConfig, SymbolId,
    TargetPortfolio,
};
use execution_engine::risk_guard::RiskGuard;
use log::info;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("=== Execution Engine Starting (Event-Driven Mode) ===");

    // 1. Initialize Components
    let risk_guard = RiskGuard::new();
    let exchange = Box::new(MockExchange::new(0.001)); // 10 bps fee
    let mut config = AllocationConfig::default();

    // Setup Strat A (50% Allocation)
    let strategy_id = MultiplexerId::new("StratA");
    config.insert(
        strategy_id.clone(),
        StrategyConfig::new(0.50, 0.50, 0.0), // 50% target, 50% drawdown limit
    );

    let mut engine = Engine::new(risk_guard, exchange, config);

    // 2. Prepare Mock Data Sequence
    let aapl_id = SymbolId::new("AAPL", "NASDAQ");
    let aapl_inst = Instrument::Stock(aapl_id.clone());

    let messages = vec![
        // A. Market Data Update (AAPL starts at $150)
        IngressMessage::MarketData(Price::new(aapl_inst.clone(), 150.0, 149.9, 150.1, 1000)),
        // B. Target Portfolio (Buy 10 AAPL)
        IngressMessage::TargetPortfolio(TargetPortfolio::new(
            strategy_id.clone(),
            vec![(aapl_inst.clone(), 10.0)].into_iter().collect(),
            None,
        )),
        // C. Market Data Update (AAPL moves to $155 - Profit!)
        IngressMessage::MarketData(Price::new(aapl_inst.clone(), 155.0, 154.9, 155.1, 2000)),
        // D. Admin Command: Rebalance
        IngressMessage::Command(execution_engine::models::AdminCommand::RebalanceCapital {
            tolerance: 0.01,
        }),
        // E. Add Strategy B
        IngressMessage::Command(execution_engine::models::AdminCommand::AddStrategy {
            id: MultiplexerId::new("StratB"),
            config: StrategyConfig::new(0.50, 0.50, 0.0),
        }),
        // F. Target for Strategy B
        IngressMessage::TargetPortfolio(TargetPortfolio::new(
            MultiplexerId::new("StratB"),
            vec![(aapl_inst.clone(), 5.0)].into_iter().collect(),
            None,
        )),
    ];

    // 3. Initialize Gateway
    let mut gateway = MockGateway::new(messages);

    // 4. Run Event Loop
    use execution_engine::gateway::Gateway;

    info!("Entering Event Loop...");
    while let Some(msg) = gateway.next() {
        engine.process(msg);
    }

    info!("Event stream finished. Shutdown.");
    Ok(())
}
