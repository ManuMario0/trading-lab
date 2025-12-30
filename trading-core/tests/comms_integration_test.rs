use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use trading_core::comms::builder::{build_publisher, build_subscriber};
use trading_core::comms::socket::{ReceiverSocket, SenderSocket};
use trading_core::model::allocation::Allocation;
use trading_core::model::identity::Identity;
use trading_core::model::market_data::{MarketDataBatch, PriceUpdate};

// This test verifies that we can:
// 1. Serialize our data models (MarketDataBatch, Allocation) using bincode
// 2. Transmit them over ZMQ (TCP)
// 3. Deserialize them correctly on the other side
#[tokio::test]
async fn test_zmq_pipeline_end_to_end() -> Result<()> {
    use trading_core::comms::address::Address;

    // 1. Setup Identities
    use trading_core::args::CommonArgs;
    CommonArgs::set_mock(CommonArgs::default_for_test());
    let source_identity = Identity::new("test_strategy", "1.0", 999);

    // 2. Setup Ports (Use non-standard ports to avoid conflicts)
    // Note: Address::zmq_tcp adds "tcp://" prefix and port.
    let pub_addr = Address::zmq_tcp("127.0.0.1", 5995);

    // 3. Create Publisher (Simulating Engine/Market Data Source)
    // builder returns Result<SenderSocket<T>> directly
    let publisher: SenderSocket<MarketDataBatch> =
        build_publisher(&pub_addr, source_identity.get_identifier())?;

    // 4. Create Subscriber (Simulating Strategy)
    let mut subscriber: ReceiverSocket<MarketDataBatch> = build_subscriber(&pub_addr)?;

    // Allow ZMQ strict time to connect
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 5. Create Data
    let update = PriceUpdate::new(1, 100.50, 100.55, 100.50, 123456789);
    let batch = MarketDataBatch::new(vec![update]);

    // 6. Send From Publisher
    println!("Sending MarketDataBatch...");
    publisher.send(batch).await?;

    // 7. Receive at Subscriber
    println!("Waiting for receive...");
    let received_batch = timeout(Duration::from_secs(1), subscriber.recv()).await??;

    // 8. Verify
    let batch_data = received_batch.data();
    assert_eq!(batch_data.get_count(), 1);
    let rx_update = batch_data.get_update_at(0);
    assert_eq!(rx_update.get_instrument_id(), 1);
    assert_eq!(rx_update.get_bid(), 100.50);

    println!("Successfully verified MarketDataBatch pipeline!");

    // 9. Verify Allocation (New Port)
    let alloc_addr = Address::zmq_tcp("127.0.0.1", 5996);

    let alloc_sender: SenderSocket<Allocation> =
        build_publisher(&alloc_addr, source_identity.get_identifier())?;
    let mut alloc_receiver: ReceiverSocket<Allocation> = build_subscriber(&alloc_addr)?;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut allocation = Allocation::new();
    allocation.update_position(1, 50.0);

    println!("Sending Allocation...");
    alloc_sender.send(allocation).await?;

    let received_alloc = timeout(Duration::from_secs(1), alloc_receiver.recv()).await??;

    // assert_eq!(received_alloc.get_source(), "test_strategy");
    // .get_position(1) -> Option<&Position>, then .get_quantity()
    assert_eq!(
        received_alloc
            .data()
            .get_position(1)
            .unwrap()
            .get_quantity(),
        50.0
    );

    println!("Successfully verified Allocation pipeline!");

    Ok(())
}
