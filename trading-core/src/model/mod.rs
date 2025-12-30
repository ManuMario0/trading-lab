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
pub mod allocation_batch;
pub mod execution;
pub mod identity;
pub mod instrument;
pub mod instrument_db;
pub mod market_data;
pub mod order;
pub mod policy;
pub mod portfolio;

pub use allocation::Allocation;
pub use execution::{ExecutionResult, ExecutionStatus};
pub use instrument::Instrument;
pub use instrument::InstrumentId;
pub use instrument_db::InstrumentDB;
pub use market_data::PriceUpdate;
pub use order::{Order, OrderSide, OrderType};
pub use policy::Policy;
pub use portfolio::{Actual, Portfolio, Target};
