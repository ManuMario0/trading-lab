use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for the source of signals/portfolio goals.
/// e.g. "Trading-Core", "Investing-LongTerm", "Paper-Test-1"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MultiplexerId(String);

impl MultiplexerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MultiplexerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a basic ticker symbol (Equity, Future, etc).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId {
    symbol: String,
    exchange: String, // e.g. "NASDAQ", "NYSE", "Crypto"
}

impl SymbolId {
    pub fn new(symbol: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            exchange: exchange.into(),
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }
}
