use super::*;
use crate::exchange::mock::MockExchange;
use crate::models::{
    AllocationConfig, Instrument, MultiplexerId, StrategyConfig, SymbolId, TargetPortfolio,
};
use crate::risk_guard::RiskGuard;

fn create_test_engine() -> Engine {
    let rg = RiskGuard::new();
    // 10bps fee
    let exchange = Box::new(MockExchange::new(0.001));
    Engine::new(rg, exchange, Default::default())
}

fn create_test_engine_with_config(config: AllocationConfig) -> Engine {
    let rg = RiskGuard::new();
    let exchange = Box::new(MockExchange::new(0.001));
    Engine::new(rg, exchange, config)
}

struct RejectPolicy;
impl crate::risk_guard::Policy for RejectPolicy {
    fn name(&self) -> &str {
        "RejectAll"
    }
    fn check(
        &self,
        _: &Order,
        _: &crate::risk_guard::RiskContext,
    ) -> crate::risk_guard::RiskDecision {
        crate::risk_guard::RiskDecision::Rejected("Test Reject".into())
    }
}

fn instrument(s: &str) -> Instrument {
    Instrument::Stock(SymbolId::new(s, "TEST"))
}

#[test]
fn test_generate_orders_from_weights() {
    let mut engine = create_test_engine();
    let id = MultiplexerId::new("Test");

    // Setup Market Data
    let a = instrument("A");
    let b = instrument("B");
    engine.update_market_price(a.clone(), 100.0);
    engine.update_market_price(b.clone(), 50.0);

    // Setup Portfolio (Cash only)
    {
        let p = engine.portfolios.entry(id.clone()).or_default();
        p.cash_mut().deposit("USD", 10000.0);
    }

    // Target: 50% A, 50% B
    let weights = vec![(a.clone(), 0.5), (b.clone(), 0.5)];

    let target = TargetPortfolio::new(id.clone(), weights, None);

    // Run
    engine.on_target_portfolio(target);

    // Verify Orders Executed (Check Portfolio State)
    let p = engine.portfolios.get(&id).unwrap();
    // With 1% cash buffer: 10000 * 0.99 = 9900 available.
    // A: (9900 * 0.5) / 100 = 49.5 units
    // B: (9900 * 0.5) / 50 = 99.0 units
    let qty_a = p.positions().get_quantity(&a);
    let qty_b = p.positions().get_quantity(&b);

    assert!(
        (qty_a - 49.5).abs() < 1e-6,
        "Expected 49.5 A, got {}",
        qty_a
    );
    assert!(
        (qty_b - 99.0).abs() < 1e-6,
        "Expected 99.0 B, got {}",
        qty_b
    );
}

#[test]
fn test_atomic_batch_rejection() {
    let mut rg = RiskGuard::new();
    rg.add_policy(Box::new(RejectPolicy));
    let exchange = Box::new(MockExchange::new(0.001));
    let mut engine = Engine::new(rg, exchange, Default::default());

    let id = MultiplexerId::new("TestReject");
    let a = instrument("A");
    engine.update_market_price(a.clone(), 100.0);

    {
        let p = engine.portfolios.entry(id.clone()).or_default();
        p.cash_mut().deposit("USD", 10000.0);
    }

    let weights = vec![(a.clone(), 0.5)];

    // This batch (1 order) should be rejected
    engine.on_target_portfolio(TargetPortfolio::new(id.clone(), weights, None));

    // Verify NO Execution
    let p = engine.portfolios.get(&id).unwrap();
    let qty_a = p.positions().get_quantity(&a);
    assert_eq!(
        qty_a, 0.0,
        "Batch should be rejected, no position change expected"
    );
}

#[test]
fn test_min_global_equity_block() {
    let id = MultiplexerId::new("RiskyStrat");
    let mut config = AllocationConfig::default();
    config.insert(
        id.clone(),
        StrategyConfig::new(1.0, 0.5, 20000.0), // Requires 20k
    );

    let mut engine = create_test_engine_with_config(config);
    let a = instrument("A");
    engine.update_market_price(a.clone(), 100.0);

    // Initial State: 10,000 Cash (Below 20k limit)
    {
        let p = engine.portfolios.entry(id.clone()).or_default();
        p.cash_mut().deposit("USD", 10000.0);
        p.metrics_mut().cur_equity = 10000.0;
    }

    // Attempt to BUY (Increase Exposure)
    let weights = vec![(a.clone(), 1.0)]; // 100% into Asset A

    engine.on_target_portfolio(TargetPortfolio::new(id.clone(), weights, None));

    // Verify NO Execution (Blocked)
    let p = engine.portfolios.get(&id).unwrap();
    let qty_a = p.positions().get_quantity(&a);
    assert_eq!(qty_a, 0.0, "Should block new risk when global equity < min");
}

#[test]
fn test_min_global_equity_allow_reduction() {
    let id = MultiplexerId::new("RiskyStrat");
    let mut config = AllocationConfig::default();
    config.insert(id.clone(), StrategyConfig::new(1.0, 0.5, 20000.0));

    let mut engine = create_test_engine_with_config(config);
    let a = instrument("A");
    engine.update_market_price(a.clone(), 100.0);

    // Initial State: 0 Cash + 100 Units @ 100 = 10k Equity.
    {
        let p = engine.portfolios.entry(id.clone()).or_default();
        p.cash_mut().deposit("USD", 0.0);
        p.positions_mut().set_quantity(a.clone(), 100.0);
        p.metrics_mut().cur_equity = 10000.0;
    }

    // Target: Sell Half (Reduce Exposure).
    let pos = vec![(a.clone(), 50.0)]; // Target 50 units

    engine.on_target_portfolio(TargetPortfolio::new(id.clone(), Vec::new(), Some(pos)));

    // Verify Execution (Sold 50 units)
    let p = engine.portfolios.get(&id).unwrap();
    let qty_a = p.positions().get_quantity(&a);
    assert!(
        (qty_a - 50.0).abs() < 1e-6,
        "Should allow reducing exposure 100->50"
    );
    // Check cash increase
    let cash = p.cash().get_balance("USD");
    assert!(
        cash > 4000.0,
        "Cash should increase (approx 5000 minus fees), got {}",
        cash
    );
}

#[test]
fn test_realtime_kill_switch() {
    let id = MultiplexerId::new("Crasher");
    let mut config = AllocationConfig::default();
    config.insert(
        id.clone(),
        StrategyConfig::new(1.0, 0.10, 0.0), // 10% Kill Switch
    );

    let mut engine = create_test_engine_with_config(config);
    let a = instrument("A");

    // Start Price 100.
    engine.update_market_price(a.clone(), 100.0);

    // Setup: 100 units of A ($10,000). HWM = 10,000.
    {
        let mut p = Portfolio::new();
        p.positions_mut().set_quantity(a.clone(), 100.0);
        p.metrics_mut().cur_equity = 10000.0;
        p.metrics_mut().high_water_mark = 10000.0;
        p.metrics_mut().drawdown = 0.0;

        engine.portfolios.insert(id.clone(), p);
    }

    // CRASH: Price drops to 85 (-15%).
    // Drawdown will be (10000 - 8500) / 10000 = 0.15 > 0.10.
    engine.update_market_price(a.clone(), 85.0);

    // Expect: Liquidation. Position = 0. Cash ~= 8500.
    let p = engine.portfolios.get(&id).unwrap();
    let qty_a = p.positions().get_quantity(&a);

    assert_eq!(qty_a, 0.0, "Kill switch should close position");
    let cash = p.cash().get_balance("USD");
    assert!(
        cash > 8000.0,
        "Cash should be recovered (approx 8500), got {}",
        cash
    );
}

#[test]
fn test_capital_rebalancing() {
    let id_a = MultiplexerId::new("StratA");
    let id_treasury = MultiplexerId::new("TREASURY");

    let mut config = AllocationConfig::default();
    // Strat A: Target 50%
    config.insert(id_a.clone(), StrategyConfig::new(0.50, 0.50, 0.0));

    let mut engine = create_test_engine_with_config(config);

    // Initial State:
    // Firm Equity: 20k
    // Treasury: 5k
    // Strat A: 15k (Overweight! Target is 50% of 20k = 10k)
    // Drift = 15k - 10k = 5k.
    {
        let t = engine.portfolios.entry(id_treasury.clone()).or_default();
        t.cash_mut().deposit("USD", 5000.0);
        t.metrics_mut().cur_equity = 5000.0;
    }
    {
        let p = engine.portfolios.entry(id_a.clone()).or_default();
        p.cash_mut().deposit("USD", 15000.0);
        p.metrics_mut().cur_equity = 15000.0;
    }

    engine.recalculate_consolidated();
    assert_eq!(engine.consolidated_portfolio.total_equity, 20000.0);

    // Run Rebalance (Tolerance 10% = 1k. Drift 5k > 1k.)
    engine.rebalance_capital(0.10);

    // Verify:
    // Strat A: Should be reduced by 5k -> 10k.
    // Treasury: Should be increased by 5k -> 10k.
    let p_a = engine.portfolios.get(&id_a).unwrap();
    assert!(
        (p_a.metrics().cur_equity - 10000.0).abs() < 1e-6,
        "Strat A should be swept to 10k, got {}",
        p_a.metrics().cur_equity
    );

    let p_t = engine.portfolios.get(&id_treasury).unwrap();
    assert!(
        (p_t.metrics().cur_equity - 10000.0).abs() < 1e-6,
        "Treasury should have 10k, got {}",
        p_t.metrics().cur_equity
    );
}
