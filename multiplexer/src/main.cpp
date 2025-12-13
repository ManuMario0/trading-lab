#include "io/ZmqAdminListener.hpp"
#include "io/ZmqInputListener.hpp"
#include "io/ZmqOutputPublisher.hpp"
#include "logic/KellyMultiplexer.hpp"
#include <chrono>
#include <iostream>
#include <string>
#include <thread>
#include <vector>

// --- Helper for simple flag parsing ---
std::string get_arg(const std::vector<std::string> &args,
                    const std::string &flag, const std::string &default_val) {
  auto it = std::find(args.begin(), args.end(), flag);
  if (it != args.end() && ++it != args.end()) {
    return *it;
  }
  return default_val;
}

// --- Main ---

int main(int argc, char **argv) {
  std::cout << "=== Multiplexer Starting (ZMQ Enabled) ===" << std::endl;

  std::vector<std::string> args(argv, argv + argc);
  std::string input_port = get_arg(args, "--input-port", "5556");
  std::string output_port = get_arg(args, "--output-port", "5557");
  std::string admin_port = get_arg(args, "--admin-port", "5558");

  // 1. Config (Still Hardcoded for V0)
  ClientRegistry registry;
  registry.clients["StratA"] = {0.05, 0.10}; // Mu=5%, Sigma=10%
  registry.clients["StratB"] = {0.10, 0.20}; // Mu=10%, Sigma=20%

  MultiplexerConfig config;
  config.kelly_fraction = 0.3;

  // 2. IO Setup
  zmq::context_t context(1);

  // Input: PULL from Strategies
  std::string input_addr = "tcp://*:" + input_port;
  std::cout << "[ZmqInput] Binding to " << input_addr << std::endl;
  auto input = std::make_unique<ZmqInputListener>(context, input_addr);

  // Output: PUB to Execution Engine
  std::string output_addr = "tcp://*:" + output_port;
  std::cout << "[ZmqOutput] Binding to " << output_addr << std::endl;
  auto output = std::make_unique<ZmqOutputPublisher>(context, output_addr);

  // 3. App Logic
  KellyMultiplexer app(registry, config);

  // Admin: REP for Orchestration
  std::string admin_addr = "tcp://*:" + admin_port;
  std::cout << "[ZmqAdmin] Binding to " << admin_addr << std::endl;
  auto admin = std::make_unique<ZmqAdminListener>(context, admin_addr, app);
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
