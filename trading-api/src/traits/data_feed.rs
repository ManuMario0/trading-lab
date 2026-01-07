//! Defines the `DataFeed` trait for market data ingestion.
//!
//! This module specifies the contract that all data feed adapters must implement.
//! It allows the system to abstract over different data sources (e.g., historical replay,
//! live exchange connections, synthetic generators) while providing a consistent stream
//! of `MarketDataBatch` to downstream strategies.

use crate::model::market_data::MarketDataBatch;

/// A trait for components that produce market data.
///
/// Implementors of this trait are responsible for fetching, parsing, and normalizing
/// market data updates into the standard `MarketDataBatch` format.
///
/// # Examples
///
/// ```
/// use trading::traits::data_feed::DataFeed;
/// use trading::model::market_data::MarketDataBatch;
///
/// struct MyFeed;
///
/// impl DataFeed for MyFeed {
///     fn get_market_data(&mut self) -> Option<MarketDataBatch> {
///         // Fetch data...
///         None
///     }
/// }
/// ```
pub trait DataFeed {
    /// Retrieves the next batch of market data updates.
    ///
    /// This method is intended to be polled by the microservice runner.
    /// It should return `Some(batch)` when new data is available, or `None`
    /// if no data is currently ready or the feed has ended.
    ///
    /// # Returns
    ///
    /// * `Option<MarketDataBatch>` - A batch of market updates, or None.
    fn get_market_data(&mut self) -> Option<MarketDataBatch>;
}
