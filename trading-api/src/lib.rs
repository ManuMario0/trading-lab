pub mod macros;
pub mod model;

pub use model::allocation::Allocation;
pub use model::execution::{ExecutionResult, ExecutionStatus};
pub use model::identity::Id;

pub mod traits;
pub use model::instrument::Instrument;
pub use model::instrument::InstrumentId;
pub use model::market_data::PriceUpdate;
pub use model::order::{Order, OrderSide, OrderType};
pub use model::policy::Policy;
pub use model::portfolio::{Actual, Portfolio, Target};
pub use traits::broker::Broker;
pub use traits::executor::Executor;
pub use traits::manager::Manager;
pub use traits::multiplexist::Multiplexist;
pub use traits::strategist::Strategist;

pub mod prelude {
    pub use crate::export_strategy;
    pub use crate::model::allocation::Allocation;
    pub use crate::model::allocation_batch::AllocationBatch;
    pub use crate::model::market_data::MarketDataBatch;
    pub use crate::traits::strategist::Strategist;
}
