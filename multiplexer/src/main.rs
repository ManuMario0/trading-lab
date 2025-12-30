mod kelly_multiplexer;

use anyhow::Result;
use kelly_multiplexer::{KellyMultiplexer, MultiplexerConfig};
use trading_core::microservice::{
    configuration::{multiplexer::Multiplexer, Configuration},
    Microservice,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Define State
    let initial_state = || {
        let config = MultiplexerConfig {
            kelly_fraction: 1.0,
        };
        KellyMultiplexer::new(config)
    };

    // 2. Define Configuration (Multiplexer)
    // Multiplexer runs differently (has internal logic), so new_multiplexer() sets it up.
    // 2. Define Configuration (Multiplexer)
    let config = Configuration::new(Multiplexer::new());

    // 3. Run Service
    let service = Microservice::new(initial_state, config);
    service.run().await;

    Ok(())
}
