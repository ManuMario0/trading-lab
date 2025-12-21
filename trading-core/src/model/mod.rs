//! Core data models and types shared across the trading platform.
//!
//! This module defines the fundamental data structures (Orders, Instruments, etc.)
//! used in the system. By defining them in a shared location, we ensure that
//! serialization formats (JSON, binary) are identical across Rust and C++ services.
//!
//! # Submodules
//! - [`instrument`]: Defines tradable assets (Spot, Future, Option).
//! - [`order`]: Defines trading orders (Limit, Market, Stop).

pub mod allocation;
pub mod instrument;
pub mod instrument_db;
pub mod market_data;
pub mod order;

pub use instrument::Instrument;
pub use instrument_db::InstrumentDB;
pub use market_data::PriceUpdate;
pub use order::{Order, OrderSide, OrderType};
