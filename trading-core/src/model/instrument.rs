use serde::{Deserialize, Serialize};

pub type InstrumentId = usize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instrument {
    Stock(Stock),
    Future(Future),
    Option(OptionContract),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Future {
    // Stub
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionContract {
    // Stub
}

/// Represents a Stock/Equity instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    /// Unique identifier for the stock.
    ///
    /// This ID is used for efficient data transmission (e.g., in market data streams).
    /// Instead of transmitting the full static symbol information repeatedly,
    /// we only send this integer ID after the initial handshake/definition.
    id: InstrumentId,

    /// The human-readable ticker symbol (e.g., "AAPL").
    symbol: String,

    /// The exchange where this stock trades (e.g., "NASDAQ").
    exchange: String,

    /// The broad economic sector (e.g., "Technology").
    sector: String,

    /// The specific industry categorization (e.g., "Consumer Electronics").
    industry: String,

    /// Country of domicile (e.g., "USA").
    country: String,

    /// Trading currency (e.g., "USD").
    currency: String,
}

impl Stock {
    /// Creates a new Stock instance.
    pub fn new(
        id: usize,
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        sector: impl Into<String>,
        industry: impl Into<String>,
        country: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            id,
            symbol: symbol.into(),
            exchange: exchange.into(),
            sector: sector.into(),
            industry: industry.into(),
            country: country.into(),
            currency: currency.into(),
        }
    }
}
