mod kelly_multiplexer;

use anyhow::Result;
use clap::Parser;
use kelly_multiplexer::{KellyMultiplexer, MultiplexerConfig};
use log::info;
use trading_core::{
    args::CommonArgs,
    microservice::{configuration::Configuration, Microservice},
};

fn main() -> Result<()> {
    env_logger::init();

    // 1. Parse Args
    let args = CommonArgs::parse();
    info!("Starting Multiplexer (Rust): {}", args.get_service_name());

    // 2. Define State
    let initial_state = || {
        let config = MultiplexerConfig {
            kelly_fraction: 1.0,
        };
        let mut mux = KellyMultiplexer::new(config);
        // Pre-register our strategy (in a real app this would be via Admin, or auto-discovery)
        // mux.add_client("dummy_strategy".to_string(), 0.05, 0.20);
        mux
    };

    // 3. Define Configuration (Multiplexer)
    // Multiplexer runs differently (has internal logic), so new_multiplexer() sets it up.
    let config = Configuration::new_multiplexer();

    // 4. Run Service
    let service = Microservice::new(args, initial_state, config);
    service.run();

    Ok(())
}
