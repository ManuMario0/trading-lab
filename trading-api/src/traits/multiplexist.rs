use crate::model::{allocation_batch::AllocationBatch, identity::Id};

pub trait Multiplexist: Send {
    /// Called when the Multiplexer receives a batch of allocations from a source.
    ///
    /// # Arguments
    ///
    /// * `source_id` - The ID of the source.
    /// * `batch` - The batch of allocations.
    ///
    /// # Returns
    ///
    /// * `AllocationBatch` - The aggregated allocation batch.
    fn on_allocation_batch(&mut self, source_id: Id, batch: AllocationBatch) -> AllocationBatch;
}

impl Multiplexist for Box<dyn Multiplexist> {
    fn on_allocation_batch(&mut self, source_id: Id, batch: AllocationBatch) -> AllocationBatch {
        (**self).on_allocation_batch(source_id, batch)
    }
}
