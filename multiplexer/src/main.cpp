#include "io/ZmqAdminListener.hpp"
#include "io/ZmqInputListener.hpp"
#include "io/ZmqOutputPublisher.hpp"
#include "logic/KellyMultiplexer.hpp"
#include <chrono>
#include <iostream>
#include <thread>

// --- Main ---

int main() {
  std::cout << "=== Multiplexer Starting (ZMQ Enabled) ===" << std::endl;

  // 1. Config (Still Hardcoded for V0)
  ClientRegistry registry;
  registry.clients["StratA"] = {0.05, 0.10}; // Mu=5%, Sigma=10%
  registry.clients["StratB"] = {0.10, 0.20}; // Mu=10%, Sigma=20%

  MultiplexerConfig config;
  config.kelly_fraction = 0.3;

  // 2. IO Setup
  zmq::context_t context(1);

  // Input: PULL from Strategies
  auto input = std::make_unique<ZmqInputListener>(context, "tcp://*:5556");

  // Output: PUB to Execution Engine
  auto output = std::make_unique<ZmqOutputPublisher>(context, "tcp://*:5557");

  // 3. App Logic
  KellyMultiplexer app(registry, config);

  // Admin: REP for Orchestration
  auto admin = std::make_unique<ZmqAdminListener>(context, "tcp://*:5558", app);
  admin->start();

  // 4. Wiring
  // When input receives a portfolio...
  input->start([&](const TargetPortfolio &p) {
    std::cout << "[Main] Received portfolio from " << p.multiplexer_id
              << std::endl;

    // Process it
    TargetPortfolio aggregated = app.on_portfolio_received(p);

    // Publish result
    // Only publish if we have a valid aggregation (simplified check)
    if (!aggregated.multiplexer_id.empty()) {
      output->publish(aggregated);
      std::cout << "[Main] Published aggregated portfolio." << std::endl;
    }
  });

  std::cout << "=== Multiplexer Running... Press Ctrl+C to stop ==="
            << std::endl;

  // Keep main thread alive
  // In a real production app, we would handle signals (SIGINT) to exit
  // gracefully
  while (true) {
    std::this_thread::sleep_for(std::chrono::seconds(1));
  }

  return 0;
}
