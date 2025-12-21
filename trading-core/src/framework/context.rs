use crate::model::market_data::MarketDataBatch;

/// A container for all data available to a strategy during an event.
///
/// This structure follows the "Pull/Push" enrichment model.
/// Fields are `Option` because a strategy might not subscribe to everything.
/// This allows us to add new data types (e.g., Sentiment, Volatility) without breaking existing strategies.
pub struct Context {
    price_updates: MarketDataBatch,
    // Future fields:
    // pub volatility: Option<VolatilitySurface>,
    // pub news: Option<NewsSentiment>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            price_updates: MarketDataBatch::new(Vec::new()),
        }
    }

    pub fn clear(&mut self) {
        self.price_updates.clear();
    }

    pub fn set_price_updates(&mut self, updates: MarketDataBatch) {
        self.price_updates = updates;
    }

    pub fn get_price_updates(&self) -> &MarketDataBatch {
        &self.price_updates
    }
}

/// Helper to configure which data streams a strategy subscribes to.
///
/// This builder pattern allows the framework to efficienty filter incoming data.
pub struct ContextBuilder {
    pub topics: Vec<String>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self { topics: Vec::new() }
    }

    /// Subscribes to a specific data topic (channel).
    /// e.g., "md.usa.tech"
    pub fn with_topic(mut self, topic: &str) -> Self {
        self.topics.push(topic.to_string());
        self
    }
}
