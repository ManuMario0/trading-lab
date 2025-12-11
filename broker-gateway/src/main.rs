use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Starting Broker Gateway...");

    // TODO: Parse command line args for --mode=paper or --mode=live
    // TODO: Connect to Broker API
    // TODO: Listen on ZMQ

    Ok(())
}
