
#pragma once
#include "trading_core.hpp"
#include <iostream>
#include <map>
#include <memory>
#include <string>

class DummyStrategy {
public:
  explicit DummyStrategy(const std::string &strategy_id)
      : strategy_id_(strategy_id) {}

  std::unique_ptr<trading::Allocation>
  on_market_update(const trading::MarketDataBatchView &batch) {
    bool signal_generated = false;
    double new_weight = 0.0;
    size_t signal_instrument_id = 0;

    for (size_t i = 0; i < batch.count(); ++i) {
      auto update = batch.at(i);
      size_t instrument_id = update.get_instrument_id();
      double current_price = update.get_price();

      // Simple logic: Assume ID 1 is our target (e.g. AAPL)
      // Or just trade on everything.
      // Let's print.
      // std::cout << "[DummyStrategy] Update for " << instrument_id << ": " <<
      // current_price << std::endl;

      if (last_prices_.count(instrument_id)) {
        double last_price = last_prices_[instrument_id];
        if (current_price > last_price) {
          // Up -> Buy
          std::cout << "[DummyStrategy] ID " << instrument_id << " Up ("
                    << last_price << " -> " << current_price << "). BUY."
                    << std::endl;
          new_weight = 1.0;
          signal_generated = true;
          signal_instrument_id = instrument_id;
        } else if (current_price < last_price) {
          // Down -> Sell
          std::cout << "[DummyStrategy] ID " << instrument_id << " Down ("
                    << last_price << " -> " << current_price << "). SELL."
                    << std::endl;
          new_weight = -1.0;
          signal_generated = true;
          signal_instrument_id = instrument_id;
        }
      } else {
        std::cout << "[DummyStrategy] ID " << instrument_id
                  << " First Tick: " << current_price << std::endl;
      }
      last_prices_[instrument_id] = current_price;
    }

    if (signal_generated) {
      // Create separate allocation for each signal?
      // Or one allocation with one position?
      // For now, just send one.
      auto allocation =
          std::make_unique<trading::Allocation>("dummy_strategy", 0);
      allocation->update_position(signal_instrument_id, new_weight);
      return allocation;
    }

    return nullptr;
  }

  std::string on_admin_command(const std::string &cmd) {
    std::cout << "[DummyStrategy] Received Admin Command: " << cmd << std::endl;
    return "ACK";
  }

private:
  std::string strategy_id_;
  std::map<size_t, double> last_prices_;
};
