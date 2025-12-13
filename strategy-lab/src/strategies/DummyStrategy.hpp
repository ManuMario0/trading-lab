#pragma once
#include "IStrategy.hpp"
#include <iostream>
#include <map>

class DummyStrategy : public IStrategy {
public:
  explicit DummyStrategy(const std::string &strategy_id)
      : strategy_id_(strategy_id) {}

  std::optional<TargetPortfolio>
  on_market_update(const MarketUpdate &price) override {
    // MarketUpdate is now alias for Price
    bool signal_generated = false;
    double new_weight = 0.0;

    // Check if it is AAPL
    if (price.instrument.data.symbol == "AAPL") {
      double current_price = price.last;

      if (last_prices_.count("AAPL")) {
        double last_price = last_prices_["AAPL"];

        // Simple Threshold Logic to avoid spamming tiny moves?
        // For now, keep it sensitive to verify flow.
        if (current_price > last_price) {
          // Price went up -> Buy
          std::cout << "[DummyStrategy] AAPL Up (" << last_price << " -> "
                    << current_price << "). BUY." << std::endl;
          new_weight = 1.0;
          signal_generated = true;
        } else if (current_price < last_price) {
          // Price went down -> Sell (Short)
          std::cout << "[DummyStrategy] AAPL Down (" << last_price << " -> "
                    << current_price << "). SELL." << std::endl;
          new_weight = -1.0;
          signal_generated = true;
        }
      } else {
        // First tick, just record it
        std::cout << "[DummyStrategy] AAPL First Tick: " << current_price
                  << std::endl;
      }
      last_prices_["AAPL"] = current_price;
    }

    if (signal_generated) {
      TargetPortfolio portfolio;
      portfolio.strategy_id = strategy_id_;

      // Use the instrument from the update
      portfolio.target_weights[price.instrument] = new_weight;
      return portfolio;
    }

    return std::nullopt;
  }

  std::string on_admin_command(const std::string &cmd) override {
    std::cout << "[DummyStrategy] Received Admin Command: " << cmd << std::endl;
    return "ACK";
  }

private:
  std::string strategy_id_;
  std::map<std::string, double> last_prices_;
};
