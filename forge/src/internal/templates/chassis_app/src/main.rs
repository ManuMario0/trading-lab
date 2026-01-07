use anyhow::Result;
use trading_core::microservice::{
    configuration::{strategy::Strategy, Configuration},
    Microservice,
};

// This import will be rewritten by Forge to match the user's crate name
// e.g. use my_strategy::entry_point;
use user_crate::entry_point;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Define State Closure
    // We use dynamic dispatch (Box<dyn Strategist>) which IS supported by trading-api traits.
    let initial_state = |_: &_| entry_point();

    // 2. Define Configuration with the Box type
    // We explicitly tell the compiler that the State is Box<dyn trading::Strategist>
    let config = Configuration::new(Strategy::<Box<dyn trading::Strategist>>::new());

    // 3. Run
    let service = Microservice::new(
        initial_state,
        config,
        env!("CARGO_PKG_VERSION").to_string(), // Injected by Forge
        env!("CARGO_PKG_DESCRIPTION").to_string(), // Injected by Forge
    );

    service.run().await;

    Ok(())
}
