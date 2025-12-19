use crate::exchange::Exchange;
use crate::models::{
    AdminCommand, AllocationConfig, ConsolidatedPortfolio, IngressMessage, Instrument, LedgerEntry,
    MultiplexerId, Order, OrderType, Portfolio, Price, Prices, Side, TargetPortfolio, Transaction,
    TransactionLogger,
};
use crate::risk_guard::{RiskContext, RiskDecision, RiskGuard};
use log::{info, warn};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

pub struct Engine {
    portfolios: HashMap<MultiplexerId, Portfolio>,
    risk_guard: RiskGuard,
    exchange: Box<dyn Exchange>,
    prices: Prices,
    allocation_config: AllocationConfig,
    consolidated_portfolio: ConsolidatedPortfolio,
    ledger: TransactionLogger,
}

impl Engine {
    pub fn new(
        risk_guard: RiskGuard,
        exchange: Box<dyn Exchange>,
        allocation_config: AllocationConfig,
    ) -> Self {
        let ledger_path = PathBuf::from("transactions.csv");
        Self {
            portfolios: HashMap::new(),
            risk_guard,
            exchange,
            prices: Prices::default(),
            allocation_config,
            consolidated_portfolio: ConsolidatedPortfolio::default(),
            ledger: TransactionLogger::new(ledger_path),
        }
    }

    pub fn get_portfolio(&self, id: &MultiplexerId) -> Option<&Portfolio> {
        self.portfolios.get(id)
    }

    pub fn consolidated_portfolio(&self) -> &ConsolidatedPortfolio {
        &self.consolidated_portfolio
    }

    fn recalculate_consolidated(&mut self) {
        self.consolidated_portfolio = ConsolidatedPortfolio::aggregate(&self.portfolios);
    }

    pub fn portfolios(&self) -> &HashMap<MultiplexerId, Portfolio> {
        &self.portfolios
    }

    /// DEBUG/TEST: Manually deposit cash
    pub fn deposit(&mut self, id: &MultiplexerId, currency: &str, amount: f64) {
        let p = self.portfolios.entry(id.clone()).or_default();
        p.cash_mut().deposit(currency, amount);
        // Force update equity metric
        // let eq = p.metrics().cur_equity + amount; // Approximate update or full
        // recalc needed? Let's just do full recalc if prices exist, else just
        // add cash value For simplicity:
        p.metrics_mut().cur_equity += amount;
    }

    /// DEBUG/TEST: Manually set config
    pub fn set_strategy_config(
        &mut self,
        id: MultiplexerId,
        config: crate::models::StrategyConfig,
    ) {
        self.allocation_config.insert(id, config);
    }

    pub fn update_market_price(&mut self, instrument: Instrument, price_val: f64) {
        let price = Price::new(
            instrument.clone(),
            price_val,
            price_val,
            price_val,
            chrono::Utc::now().timestamp_millis(),
        );

        self.prices.insert(instrument, price);

        let mut kill_list: Vec<MultiplexerId> = Vec::new();

        for (id, portfolio) in &mut self.portfolios {
            let eq = portfolio.calculate_equity("USD", &self.prices);
            portfolio.metrics_mut().cur_equity = eq;

            if eq > portfolio.metrics().high_water_mark {
                portfolio.metrics_mut().high_water_mark = eq;
                portfolio.metrics_mut().drawdown = 0.0;
            } else if portfolio.metrics().high_water_mark > 0.0 {
                let dd = (portfolio.metrics().high_water_mark - eq)
                    / portfolio.metrics().high_water_mark;
                portfolio.metrics_mut().drawdown = dd;
            }

            if let Some(config) = self.allocation_config.get(id) {
                if portfolio.metrics().drawdown > config.max_drawdown() {
                    warn!(
                        "KILL SWITCH TRIGGERED for {}: Drawdown {:.2}% > Max {:.2}%",
                        id,
                        portfolio.metrics().drawdown * 100.0,
                        config.max_drawdown() * 100.0
                    );
                    kill_list.push(id.clone());
                }
            }
        }

        for id in kill_list {
            self.liquidate_strategy(&id);
        }

        self.recalculate_consolidated();
    }

    pub fn on_target_portfolio(&mut self, target: TargetPortfolio) {
        let multiplexer_id = target.multiplexer_id().clone();
        info!("Received TargetPortfolio from {}", multiplexer_id);

        let total_global_equity: f64 = self
            .portfolios
            .values()
            .map(|p| p.metrics().cur_equity)
            .sum();

        let config = self.allocation_config.get(&multiplexer_id).cloned();

        // Check Min Global Liquidity
        if let Some(cfg) = &config {
            if total_global_equity < cfg.min_global_equity() {
                // If we are below min equity, we only allow reducing risk (Sells).
                // We'll filter this later in generate_rebalance_orders or just reject pure
                // increases? For now, let's keep it simple: strict block if
                // target NAV > current NAV.
                let current_portfolio = self.portfolios.entry(multiplexer_id.clone()).or_default();
                let current_nav = current_portfolio.calculate_equity("USD", &self.prices);

                // Estimate Target NAV
                let target_nav = if let Some(positions) = target.target_positions() {
                    positions
                        .iter()
                        .map(|(instrument, qty)| {
                            let price =
                                self.prices.get(instrument).map(|p| p.last()).unwrap_or(0.0);
                            qty * price
                        })
                        .sum::<f64>()
                } else if !target.target_weights().is_empty() {
                    let total_weight: f64 = target.target_weights().iter().map(|(_, w)| w).sum();
                    current_nav * total_weight
                } else {
                    0.0
                };

                if target_nav >= current_nav - 1e-2 {
                    warn!(
                        "Min Global Liquidity Block for {}: Global {:.2} < Min {:.2}. Target NAV {:.2} >= Curr {:.2}. REJECTED.",
                        multiplexer_id, total_global_equity, cfg.min_global_equity(), target_nav, current_nav
                    );
                    return;
                }
            }
        }

        let equity = {
            let current_portfolio = self.portfolios.entry(multiplexer_id.clone()).or_default();
            let eq = current_portfolio.calculate_equity("USD", &self.prices);
            current_portfolio.metrics_mut().cur_equity = eq;

            if eq > current_portfolio.metrics().high_water_mark {
                current_portfolio.metrics_mut().high_water_mark = eq;
                current_portfolio.metrics_mut().drawdown = 0.0;
            } else if current_portfolio.metrics().high_water_mark > 0.0 {
                let hwm = current_portfolio.metrics().high_water_mark;
                current_portfolio.metrics_mut().drawdown = (hwm - eq) / hwm;
            }

            if let Some(cfg) = &config {
                if current_portfolio.metrics().drawdown > cfg.max_drawdown() {
                    warn!(
                        "Kill Switch Active for {}: Drawdown {:.2}% > {:.2}%. Liquidating and rejecting target.",
                        multiplexer_id, current_portfolio.metrics().drawdown * 100.0, cfg.max_drawdown() * 100.0
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if equity {
            self.liquidate_strategy(&multiplexer_id);
            return;
        }

        let equity_val = self
            .portfolios
            .get(&multiplexer_id)
            .unwrap()
            .metrics()
            .cur_equity;

        let orders = {
            let current_portfolio = self
                .portfolios
                .get(&multiplexer_id)
                .expect("Portfolio should exist");
            Self::generate_rebalance_orders(
                &multiplexer_id,
                current_portfolio,
                &target,
                equity_val,
                &self.prices,
                config.as_ref(),
            )
        };

        if orders.is_empty() {
            return;
        }

        // Smart Rebalancing: Sort Sells (Risk Reducing) before Buys (Risk Increasing)
        // Order::side() is Buy/Sell.
        // We want Sell first.
        let mut orders = orders;
        orders.sort_by(|a, b| {
            // Sell < Buy
            // Sort key: Sell=0, Buy=1
            let score = |o: &Order| match o.side() {
                Side::Sell => 0,
                Side::Buy => 1,
            };
            score(a).cmp(&score(b))
        });

        let risk_decision = {
            let current_portfolio = self
                .portfolios
                .get(&multiplexer_id)
                .expect("Portfolio should exist");

            let risk_ctx = RiskContext {
                portfolio: current_portfolio,
                prices: &self.prices,
                total_equity: total_global_equity,
                allocation_config: &self.allocation_config,
                consolidated: &self.consolidated_portfolio,
            };

            self.risk_guard.check_batch(&orders, &risk_ctx)
        };

        match risk_decision {
            RiskDecision::Approved => {
                info!("Batch approved. Executing {} orders.", orders.len());
                for order in orders {
                    self.execute_order(order);
                }
            }
            RiskDecision::Rejected(reason) => {
                warn!("Batch rejected for {}: {}", multiplexer_id, reason);
            }
        }
    }

    fn liquidate_strategy(&mut self, multiplexer_id: &MultiplexerId) {
        if let Some(portfolio) = self.portfolios.get_mut(multiplexer_id) {
            info!("LIQUIDATING STRATEGY: {}", multiplexer_id);
            let mut orders = Vec::new();

            for (instrument, qty) in portfolio.positions().iter() {
                if *qty != 0.0 {
                    orders.push(Self::create_liquidation_order(
                        multiplexer_id,
                        instrument,
                        *qty,
                    ));
                }
            }

            for order in orders {
                self.execute_order(order);
            }
        }
    }

    fn create_liquidation_order(
        multiplexer_id: &MultiplexerId,
        instrument: &Instrument,
        qty: f64,
    ) -> Order {
        Order::new(
            Uuid::new_v4(),
            multiplexer_id.clone(),
            instrument.clone(),
            if qty > 0.0 { Side::Sell } else { Side::Buy },
            qty.abs(),
            OrderType::Market,
            chrono::Utc::now().timestamp_millis(),
        )
    }

    fn generate_rebalance_orders(
        id: &MultiplexerId,
        current: &Portfolio,
        target: &TargetPortfolio,
        equity: f64,
        prices: &Prices,
        config: Option<&crate::models::StrategyConfig>,
    ) -> Vec<Order> {
        let mut orders = Vec::new();
        let mut target_quantities: HashMap<Instrument, f64> = HashMap::new();

        // 1. Calculate Available Equity (Use Cash Buffer)
        let cash_buffer = config.map(|c| c.cash_buffer()).unwrap_or(0.01);
        let available_equity = equity * (1.0 - cash_buffer);

        if !target.target_weights().is_empty() {
            for (instrument, weight) in target.target_weights() {
                let price = prices.get(instrument).map(|p| p.last()).unwrap_or(0.0);

                if price <= 0.0 {
                    warn!(
                        "Cannot allocate weight for {:?}: No price available.",
                        instrument
                    );
                    continue;
                }

                // Use buffered equity
                let target_value = available_equity * weight;
                let target_qty = target_value / price;
                target_quantities.insert(instrument.clone(), target_qty);
            }
        } else if let Some(positions) = target.target_positions() {
            target_quantities = positions.iter().cloned().collect();
        } else {
            return orders;
        }

        let mut all_instruments: HashSet<&Instrument> = HashSet::new();
        all_instruments.extend(target_quantities.keys());
        all_instruments.extend(current.positions().iter().map(|(k, _)| k));

        for instrument in all_instruments {
            let target_qty = *target_quantities.get(instrument).unwrap_or(&0.0);
            let current_qty = current.positions().get_quantity(instrument);

            let diff = target_qty - current_qty;

            if diff.abs() > 1e-6 {
                let side = if diff > 0.0 { Side::Buy } else { Side::Sell };
                orders.push(Order::new(
                    Uuid::new_v4(),
                    id.clone(),
                    instrument.clone(),
                    side,
                    diff.abs(),
                    OrderType::Market,
                    chrono::Utc::now().timestamp_millis(),
                ));
            }
        }

        orders
    }

    fn execute_order(&mut self, order: Order) {
        let price = self
            .prices
            .get(order.instrument())
            .map(|p| p.last())
            .unwrap_or(0.0);

        if price == 0.0 {
            warn!(
                "Cannot execute order for {:?}: No price",
                order.instrument()
            );
            return;
        }

        // Abstract Exchange Call
        let fill = self.exchange.submit_order(&order, price);

        self.apply_fill(
            order.multiplexer_id(),
            order.instrument(),
            order.side(),
            fill.quantity,
            fill.price, // Use Fill Price
            fill.fee,
        );
    }

    fn apply_fill(
        &mut self,
        id: &MultiplexerId,
        instrument: &Instrument,
        side: Side,
        quantity: f64,
        price: f64,
        commission: f64,
    ) {
        let portfolio = self.portfolios.entry(id.clone()).or_default();

        let cost = quantity * price;

        match side {
            Side::Buy => {
                portfolio
                    .positions_mut()
                    .update_quantity(instrument.clone(), quantity);
                portfolio.cash_mut().withdraw("USD", cost);
            }
            Side::Sell => {
                portfolio
                    .positions_mut()
                    .update_quantity(instrument.clone(), -quantity);
                portfolio.cash_mut().deposit("USD", cost);
            }
        }
        portfolio.cash_mut().withdraw("USD", commission);

        self.recalculate_consolidated();
    }

    /// Checks if strategies have drifted from their capital allocation targets
    /// and performs cash transfers to/from the Treasury to rebalance.
    pub fn rebalance_capital(&mut self, tolerance: f64) {
        let treasury_id = MultiplexerId::new("TREASURY");
        let total_equity = self.consolidated_portfolio.total_equity;

        // Skip if firm has no equity
        if total_equity <= 0.0 {
            return;
        }

        let mut transfers: Vec<(MultiplexerId, f64)> = Vec::new();

        // 1. Calculate Transfers
        for (id, config) in self.allocation_config.iter() {
            // Treasury does not have a config, nor do we rebalance it against itself
            if *id == treasury_id {
                continue;
            }

            if let Some(portfolio) = self.portfolios.get(id) {
                let current_equity = portfolio.metrics().cur_equity;
                let target_equity = total_equity * config.allocation_fraction();
                let threshold_val = target_equity * tolerance;

                let drift = current_equity - target_equity;

                if drift.abs() > threshold_val {
                    // Significant Drift Detected
                    // drift > 0: Overweight -> Withdraw -> Send to Treasury
                    // drift < 0: Underweight -> Deposit -> Take from Treasury
                    // Transfer amount is the ENTIRE drift to bring it back to target perfectly?
                    // Or just enough to be within threshold?
                    // "Target - Current" logic usually implies correcting fully to target.
                    // Given we have a buffer, let's correct fully.
                    transfers.push((id.clone(), -drift));
                }
            }
        }

        if !transfers.is_empty() {
            // 2. Execute Transfers
            for (id, amount) in transfers {
                if amount == 0.0 {
                    continue;
                }

                // A. Handle Strategy Side
                let strat_portfolio = self.portfolios.entry(id.clone()).or_default();
                if amount > 0.0 {
                    strat_portfolio.cash_mut().deposit("USD", amount);
                } else {
                    // Withdraw negative amount? No, amount is net change.
                    // If amount is -100, we withdraw 100.
                    strat_portfolio.cash_mut().withdraw("USD", amount.abs());
                }
                // Is this enough? We updated cash, but Metrics?
                // Engine update loop handles metrics usually, but here we modify Cash directly.
                // We should update metric immediately to keep state consistent for next iter?
                // Or just leave it for next tick. Let's update equity roughly.
                strat_portfolio.metrics_mut().cur_equity += amount;

                // B. Handle Treasury Side
                let treasury = self.portfolios.entry(treasury_id.clone()).or_default();
                if amount > 0.0 {
                    // Strategy got money, Treasury pays
                    treasury.cash_mut().withdraw("USD", amount);
                    treasury.metrics_mut().cur_equity -= amount;
                    info!(
                        "Capital Rebalance: Sent ${:.2} from Treasury to {}",
                        amount, id
                    );
                } else {
                    // Strategy lost money (negative amount), Treasury receives
                    treasury.cash_mut().deposit("USD", amount.abs());
                    treasury.metrics_mut().cur_equity += amount.abs();
                    info!(
                        "Capital Rebalance: Swept ${:.2} from {} to Treasury",
                        amount.abs(),
                        id
                    );
                }

                // Log Transaction
                let entries = vec![
                    LedgerEntry {
                        account: format!("Assets:{}:Cash", id),
                        amount,
                        currency: "USD".into(),
                    },
                    LedgerEntry {
                        account: format!("Assets:{}:Cash", treasury_id),
                        amount: -amount,
                        currency: "USD".into(),
                    },
                ];

                let tx = Transaction::new(format!("Rebalance Capital for {}", id), entries);

                if let Err(e) = self.ledger.log(&tx) {
                    warn!("Failed to log transaction: {}", e);
                }
            }

            self.recalculate_consolidated();
        }
    }

    /// Unified Event Processor
    pub fn process(&mut self, msg: IngressMessage) {
        match msg {
            IngressMessage::MarketData(price) => {
                // Forward updates to existing method
                self.update_market_price(price.instrument().clone(), price.last());
            }
            IngressMessage::TargetPortfolio(target) => {
                self.on_target_portfolio(target);
            }
            IngressMessage::Command(cmd) => match cmd {
                AdminCommand::RebalanceCapital { tolerance } => {
                    info!("Received Command: RebalanceCapital tolerance={}", tolerance);
                    self.rebalance_capital(tolerance);
                }
                AdminCommand::AddStrategy { id, config } => {
                    info!("Received Command: AddStrategy id={}", id);
                    self.allocation_config.insert(id, config);
                }
                AdminCommand::RemoveStrategy { id } => {
                    info!("Received Command: RemoveStrategy id={} - LIQUIDATING", id);

                    // 1. Liquidate Positions (Market Orders)
                    self.liquidate_strategy(&id);

                    // 2. Sweep Remaining Cash
                    if let Some(mut portfolio) = self.portfolios.remove(&id) {
                        let cash = portfolio.cash_mut().get_balance("USD");
                        if cash > 0.0 {
                            let treasury_id = MultiplexerId::new("TREASURY");
                            let treasury = self.portfolios.entry(treasury_id.clone()).or_default();

                            treasury.cash_mut().deposit("USD", cash);
                            treasury.metrics_mut().cur_equity += cash;

                            info!("RemoveStrategy: Swept ${:.2} from {} to Treasury", cash, id);

                            // Log Transaction
                            let entries = vec![
                                LedgerEntry {
                                    account: format!("Assets:{}:Cash", id),
                                    amount: -cash,
                                    currency: "USD".into(),
                                },
                                LedgerEntry {
                                    account: format!("Assets:{}:Cash", treasury_id),
                                    amount: cash,
                                    currency: "USD".into(),
                                },
                            ];
                            let tx =
                                Transaction::new(format!("Liquidation Sweep for {}", id), entries);
                            if let Err(e) = self.ledger.log(&tx) {
                                warn!("Failed to log sweep: {}", e);
                            }
                        }
                    }

                    // 3. Remove Config
                    self.allocation_config.remove(&id);
                    self.recalculate_consolidated();
                }
                AdminCommand::Shutdown => {
                    info!("Received Command: Shutdown (No-op engine side)");
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
