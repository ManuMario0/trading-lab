use log::info;
use std::collections::HashMap;
use trading_core::model::{Allocation, InstrumentId, TargetPortfolio};

#[derive(Debug, Clone)]
pub struct MultiplexerConfig {
    pub kelly_fraction: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyParams {
    pub mu: f64,
    pub sigma: f64,
}

pub struct KellyMultiplexer {
    config: MultiplexerConfig,
    clients: HashMap<String, StrategyParams>,
    client_portfolios: HashMap<String, TargetPortfolio>,
}

impl KellyMultiplexer {
    pub fn new(config: MultiplexerConfig) -> Self {
        Self {
            config,
            clients: HashMap::new(),
            client_portfolios: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, id: String, mu: f64, sigma: f64) {
        self.clients
            .insert(id.clone(), StrategyParams { mu, sigma });
        info!(
            "[KellyMux] Added/Updated client {} (Mu={}, Sigma={})",
            id, mu, sigma
        );
    }

    pub fn remove_client(&mut self, id: &str) {
        self.clients.remove(id);
        self.client_portfolios.remove(id);
        info!("[KellyMux] Removed client {}", id);
    }

    pub fn on_portfolio_received(&mut self, portfolio: TargetPortfolio) -> Option<TargetPortfolio> {
        self.client_portfolios
            .insert(portfolio.multiplexer_id.clone(), portfolio);
        self.recalculate()
    }

    pub fn recalculate(&mut self) -> Option<TargetPortfolio> {
        if self.client_portfolios.is_empty() {
            return None;
        }

        let mut aggregated_weights: HashMap<InstrumentId, f64> = HashMap::new();

        for (client_id, portfolio) in &self.client_portfolios {
            let params = self.clients.entry(client_id.clone()).or_insert_with(|| {
                info!("[KellyMux] Auto-registering new client: {}", client_id);
                StrategyParams {
                    mu: 0.05,
                    sigma: 0.20,
                }
            });

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

            for (instrument_id, weight) in &portfolio.target_weights {
                let final_weight = weight * scalar;
                *aggregated_weights.entry(*instrument_id).or_insert(0.0) += final_weight;
            }
        }

        Some(TargetPortfolio {
            multiplexer_id: "KellyMux_Aggregated".to_string(),
            target_weights: aggregated_weights,
        })
    }
}
