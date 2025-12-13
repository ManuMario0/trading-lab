use super::ids::SymbolId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    underlying_symbol: SymbolId,
    strike_price: f64,
    option_type: OptionType,
    expiration_date: String, // YYYY-MM-DD format
    contract_size: f64,      // e.g. 100 for standard equity options
}

impl OptionContract {
    pub fn new(
        underlying_symbol: SymbolId,
        strike_price: f64,
        option_type: OptionType,
        expiration_date: impl Into<String>,
        contract_size: f64,
    ) -> Self {
        Self {
            underlying_symbol,
            strike_price,
            option_type,
            expiration_date: expiration_date.into(),
            contract_size,
        }
    }

    pub fn underlying_symbol(&self) -> &SymbolId {
        &self.underlying_symbol
    }

    pub fn strike_price(&self) -> f64 {
        self.strike_price
    }

    pub fn option_type(&self) -> OptionType {
        self.option_type
    }

    pub fn expiration_date(&self) -> &str {
        &self.expiration_date
    }

    pub fn contract_size(&self) -> f64 {
        self.contract_size
    }
}

impl PartialEq for OptionContract {
    fn eq(&self, other: &Self) -> bool {
        self.underlying_symbol == other.underlying_symbol
            && self.option_type == other.option_type
            && self.expiration_date == other.expiration_date
            && self.strike_price.to_bits() == other.strike_price.to_bits()
            && self.contract_size.to_bits() == other.contract_size.to_bits()
    }
}

impl Eq for OptionContract {}

impl std::hash::Hash for OptionContract {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.underlying_symbol.hash(state);
        self.option_type.hash(state);
        self.expiration_date.hash(state);
        self.strike_price.to_bits().hash(state);
        self.contract_size.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurrencyPair {
    base: String,
    quote: String,
}

impl CurrencyPair {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn quote(&self) -> &str {
        &self.quote
    }
}

impl fmt::Display for CurrencyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

/// Unified Instrument Definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Instrument {
    Stock(SymbolId),
    Future(SymbolId),
    Option(OptionContract),
    Forex(CurrencyPair),
}
