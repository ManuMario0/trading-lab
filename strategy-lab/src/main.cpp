#include "io/ZmqStrategyIO.hpp"
#include "strategies/DummyStrategy.hpp"
#include <atomic>
#include <chrono>
#include <csignal>
#include <iostream>
#include <thread>

std::atomic<bool> keep_running(true);

void signal_handler(int signal) {
  if (signal == SIGINT || signal == SIGTERM) {
    std::cout << "\n[Main] Signal received, shutting down..." << std::endl;
    keep_running = false;
  }
}

int main(int argc, char **argv) {
  // Basic argument parsing or defaults
  // Usage: ./strategy_lab [input_addr] [output_addr] [admin_addr]
  std::string input_addr = "tcp://127.0.0.1:5555"; // Example default
  std::string output_addr = "tcp://127.0.0.1:5556";
  std::string admin_addr = "tcp://*:5557";

  if (argc > 1)
    input_addr = argv[1];
  if (argc > 2)
    output_addr = argv[2];
  if (argc > 3)
    admin_addr = argv[3];

  // Signal handling
  std::signal(SIGINT, signal_handler);
  std::signal(SIGTERM, signal_handler);

  std::cout << "[Main] Starting Strategy Lab..." << std::endl;
  std::cout << "  Input: " << input_addr << std::endl;
  std::cout << "  Output: " << output_addr << std::endl;
  std::cout << "  Admin: " << admin_addr << std::endl;

  // Instantiate Strategy
  DummyStrategy strategy("dummy_strategy_01");

  // Instantiate IO
  ZmqStrategyIO io(input_addr, output_addr, admin_addr);

  // Callbacks
  auto market_cb = [&](const MarketUpdate &update) {
    // Run strategy logic
    auto result = strategy.on_market_update(update);
    // If result produced, send it
    if (result) {
      io.send_portfolio(*result);
    }
  };

  auto admin_cb = [&](const std::string &cmd) {
    return strategy.on_admin_command(cmd);
  };

  // Start IO
  io.start(market_cb, admin_cb);

  std::cout << "[Main] Service running. Press Ctrl+C to stop." << std::endl;

  // Main loop
  while (keep_running) {
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }

  // Cleanup
  std::cout << "[Main] Stopping IO..." << std::endl;
  io.stop();
  std::cout << "[Main] Shutdown complete." << std::endl;

  return 0;
}
