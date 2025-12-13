use crate::models::{MultiplexerId, Price, StrategyConfig, TargetPortfolio};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminCommand {
    RebalanceCapital {
        tolerance: f64,
    },
    AddStrategy {
        id: MultiplexerId,
        config: StrategyConfig,
    },
    RemoveStrategy {
        id: MultiplexerId,
    },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngressMessage {
    MarketData(Price),
    TargetPortfolio(TargetPortfolio),
    Command(AdminCommand),
}
