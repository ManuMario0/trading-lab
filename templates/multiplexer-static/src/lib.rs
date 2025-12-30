use trading::model::identity::Id;
use trading::prelude::*;

#[derive(Default)]
pub struct MyMultiplexer;

impl Multiplexist for MyMultiplexer {
    fn on_allocation_batch(&mut self, _source_id: Id, batch: AllocationBatch) -> AllocationBatch {
        // Pass through
        batch
    }
}

impl Initiable for MyMultiplexer {
    fn init() -> Self {
        Self::default()
    }
}

trading::export_multiplexer!(MyMultiplexer);
