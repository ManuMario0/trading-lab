#include "strategies/DummyStrategy.hpp"
#include "trading_core.hpp"
#include <iostream>
#include <vector>

int main(int argc, char **argv) {
  // 1. Parse Args
  std::vector<std::string> args_vec;
  for (int i = 0; i < argc; ++i) {
    args_vec.push_back(argv[i]);
  }

  auto args = trading::CommonArgs::parse(args_vec);

  std::cout << "Starting Strategy Service: " << args.get_service_name()
            << std::endl;

  // 2. Create Strategy
  // We use a specific generic 'DummyStrategy' here.
  static auto strategy = std::make_unique<DummyStrategy>("dummy_strategy_1");

  // 3. Define Callback
  // This lambda will be wrapped in std::function and passed to Rust.
  auto callback = [](const trading::MarketDataBatchView &batch)
      -> std::unique_ptr<trading::Allocation> {
    return strategy->on_market_update(batch);
  };

  // 4. Create Configuration
  auto config = trading::Configuration::create_strategy(callback);

  // 5. Create Microservice
  // Note: args is moved in.
  trading::Microservice service(std::move(args), std::move(config));

  // 6. Run
  service.run();

  return 0;
}
