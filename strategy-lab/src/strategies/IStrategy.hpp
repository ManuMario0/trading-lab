#pragma once
#include "../models/MarketData.hpp"
#include "../models/Portfolio.hpp"
#include <optional>

class IStrategy {
public:
  virtual ~IStrategy() = default;

  // Returns a TargetPortfolio if the strategy decides to change positions, or
  // nullopt otherwise.
  virtual std::optional<TargetPortfolio>
  on_market_update(const MarketUpdate &update) = 0;

  // Administrative commands
  virtual std::string on_admin_command(const std::string &cmd) { return "OK"; }
};
