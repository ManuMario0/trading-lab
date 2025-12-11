use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Starting Execution Engine...");

    // TODO: Initialize ZMQ Context
    // TODO: Initialize Risk Guard
    // TODO: Initialize Paper Exchange

    Ok(())
}
