use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstrumentData {
    pub symbol: String,
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceUpdate {
    pub instrument: InstrumentData,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub id: String,
    pub symbol: String,
    pub action: String, // BUY/SELL
    pub quantity: f64,
    pub price: f64,
    pub status: String, // NEW, FILLED
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemView {
    pub last_prices: HashMap<String, PriceUpdate>, // Key: Symbol
    pub recent_orders: Vec<OrderUpdate>,
    pub strategies_active: bool,
}

pub type SharedSystemState = Arc<RwLock<SystemView>>;

pub fn create_state() -> SharedSystemState {
    Arc::new(RwLock::new(SystemView::default()))
}
