use log::info;
use std::collections::HashMap;
use trading_core::model::{identity::Identity, Allocation};

#[derive(Debug, Clone)]
pub struct MultiplexerConfig {
    pub kelly_fraction: f64,
}

#[derive(Debug)]
pub struct Client {
    id: usize,
    strategy_params: StrategyParams,
    portfolio: Allocation,
}

#[derive(Debug)]
pub struct StrategyParams {
    pub mu: f64,
    pub sigma: f64,
}

pub struct KellyMultiplexer {
    identity: Identity,
    config: MultiplexerConfig,
    clients: HashMap<usize, Client>,
}

impl KellyMultiplexer {
    pub fn new(config: MultiplexerConfig) -> Self {
        Self {
            identity: Identity::new("kelly_multiplexer", "1.0.0"),
            config,
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, id: usize, mu: f64, sigma: f64) {
        self.clients.insert(
            id,
            Client {
                id,
                strategy_params: StrategyParams { mu, sigma },
                portfolio: Allocation::new(self.identity.clone()),
            },
        );
        info!(
            "[KellyMux] Added/Updated client {} (Mu={}, Sigma={})",
            id, mu, sigma
        );
    }

    pub fn remove_client(&mut self, id: usize) {
        self.clients.remove(&id);
        info!("[KellyMux] Removed client {}", id);
    }

    pub fn on_portfolio_received(&mut self, portfolio: Allocation) -> Option<Allocation> {
        let id = portfolio.get_id();
        self.clients.get_mut(&id).unwrap().portfolio = portfolio;
        self.recalculate()
    }

    pub fn recalculate(&mut self) -> Option<Allocation> {
        if self.clients.is_empty() {
            return None;
        }

        let mut allocation = Allocation::new(self.identity.clone());

        for (_client_id, client) in &self.clients {
            let params = &client.strategy_params;

            // Kelly Formula: f = (mu - r) / sigma^2
            // Assuming r = 0 for simplicity or embedded in mu (excess return)
            let raw_kelly = if params.sigma > 1e-6 {
                params.mu / (params.sigma * params.sigma)
            } else {
                0.0
            };

            let mut scalar = self.config.kelly_fraction * raw_kelly;

            // Safety clamp
            if scalar > 2.0 {
                scalar = 2.0;
            }
            if scalar < -2.0 {
                scalar = -2.0;
            }

            for (instrument_id, position) in client.portfolio.get_positions() {
                let final_weight = position.get_quantity() * scalar;
                allocation.update_position(*instrument_id, final_weight);
            }
        }

        Some(allocation)
    }
}
