use crate::microservice::configuration::strategy::Strategy;
use crate::microservice::configuration::Configuration;
use crate::microservice::Microservice;
use trading::Strategist;

pub fn boot(strategy: Box<dyn Strategist>) {
    // 1. Initialize Logger
    let _ = env_logger::try_init();

    // 2. Create Microservice wrapper
    let service_config = Configuration::new(Strategy::new());

    // 3. Run
    // Microservice::new takes (initial_state_factory, configuration)
    let microservice = Microservice::new(move || strategy, service_config);
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        microservice.run().await;
    });
}
