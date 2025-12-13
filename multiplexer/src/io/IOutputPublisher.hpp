#pragma once
#include "../models/Portfolio.hpp"

// Interface for publishing the Aggregated Portfolio to the Engine
class IOutputPublisher {
public:
  virtual ~IOutputPublisher() = default;

  virtual void publish(const TargetPortfolio &portfolio) = 0;
};
