use crate::comms::zmq::{GenericPublisher, GenericSubscriber};
use crate::framework::context::{Context, ContextBuilder};
use crate::framework::strategy::Strategy;
use anyhow::{Context as AnyhowContext, Result};

/// Orchestrates the execution of a Strategy.
///
/// The Runner handles:
/// 1. Connecting to ZMQ input streams (Subs).
/// 2. Connecting to ZMQ output streams (Pubs).
/// 3. Deserializing incoming messages.
/// 4. Updating the Context.
/// 5. Invoking the Strategy logic.
/// 6. Publishing the resulting Allocation.
pub struct StrategyRunner {
    subscriber: GenericSubscriber,
    publisher: GenericPublisher,
    context: Context,
}

impl StrategyRunner {
    /// Creates a new StrategyRunner.
    ///
    /// # Arguments
    ///
    /// * `sub_address` - ZMQ Sub address (e.g., "tcp://localhost:5555").
    /// * `pub_address` - ZMQ Pub address (e.g., "tcp://*:5556").
    /// * `builder` - Configuration for which data to subscribe to.
    pub fn new(sub_address: &str, pub_address: &str, builder: ContextBuilder) -> Result<Self> {
        // Subscribe to all topics requested by the builder
        let topics = builder.topics;
        let topic_refs: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();

        let subscriber = GenericSubscriber::new(sub_address, &topic_refs)
            .context("Failed to create subscriber")?;

        let publisher = GenericPublisher::new(pub_address).context("Failed to create publisher")?;

        Ok(Self {
            subscriber,
            publisher,
            context: Context::new(),
        })
    }

    /// Starts the main event loop.
    pub fn run<S: Strategy>(&mut self, strategy: &mut S) -> Result<()> {
        loop {
            // 1. Receive generic bytes
            let (topic, data) = self.subscriber.receive()?;

            // 2. Deserialize Batch
            if topic.starts_with("md.") {
                // Now expecting a BATCH of updates
                let batch: crate::model::market_data::MarketDataBatch =
                    serde_json::from_slice(&data)
                        .context("Failed to deserialize MarketDataBatch")?;

                self.context.clear();
                // Extend the context with the batch updates
                self.context.set_price_updates(batch);

                // 3. Invoke Strategy
                if let Some(allocation) = strategy.on_event(&self.context) {
                    // 4. Serialize & Publish Output
                    let output_bytes = serde_json::to_vec(&allocation)?;
                    self.publisher.publish("allocation", &output_bytes)?;
                }
            }
        }
    }
}
