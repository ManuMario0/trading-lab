#pragma once
#include "../models/Portfolio.hpp"
#include <functional>

// Interface for receiving TargetPortfolios from Strategies
class IInputListener {
public:
  virtual ~IInputListener() = default;

  using Callback = std::function<void(const TargetPortfolio &)>;

  // Start listening and invoke callback for each message
  virtual void start(Callback cb) = 0;
};
