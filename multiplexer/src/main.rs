mod kelly_multiplexer;

use anyhow::Result;
use kelly_multiplexer::{KellyMultiplexer, MultiplexerConfig};
use trading_core::{
    microservice::{configuration::Configuration, Microservice},
    model::Allocation,
};

fn main() -> Result<()> {
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
    let config = Configuration::new_multiplexer(Box::new(
        |_state: &mut KellyMultiplexer, allocation: Allocation| {
            _state.on_portfolio_received(allocation).unwrap()
        },
    ));

    // 3. Run Service
    let service = Microservice::new(initial_state, config);
    service.run();

    Ok(())
}
