#pragma once
#include "IStrategy.hpp"
#include <iostream>
#include <map>

class DummyStrategy : public IStrategy {
public:
  explicit DummyStrategy(const std::string &strategy_id)
      : strategy_id_(strategy_id) {}

  std::optional<TargetPortfolio>
  on_market_update(const MarketUpdate &update) override {
    bool signal_generated = false;
    double new_weight = 0.0;
    bool aapl_found = false;

    for (const auto &asset : update.updates) {
      if (asset.symbol == "AAPL") {
        aapl_found = true;
        if (last_prices_.count("AAPL")) {
          double last_price = last_prices_["AAPL"];
          if (asset.price > last_price) {
            // Price went up -> Buy
            std::cout << "[DummyStrategy] AAPL Up (" << last_price << " -> "
                      << asset.price << "). BUY." << std::endl;
            new_weight = 1.0;
            signal_generated = true;
          } else if (asset.price < last_price) {
            // Price went down -> Sell (Short)
            std::cout << "[DummyStrategy] AAPL Down (" << last_price << " -> "
                      << asset.price << "). SELL." << std::endl;
            new_weight = -1.0;
            signal_generated = true;
          }
        } else {
          // First tick, just record it
          std::cout << "[DummyStrategy] AAPL First Tick: " << asset.price
                    << std::endl;
        }
        last_prices_["AAPL"] = asset.price;
      }
    }

    if (signal_generated) {
      TargetPortfolio portfolio;
      portfolio.strategy_id = strategy_id_;

      Instrument aapl;
      aapl.type = "Stock";
      aapl.data.symbol = "AAPL";
      aapl.data.exchange = "NASDAQ"; // Assumption

      portfolio.target_weights[aapl] = new_weight;
      return portfolio;
    }

    return std::nullopt;
  }

private:
  std::string strategy_id_;
  std::map<std::string, double> last_prices_;
};
