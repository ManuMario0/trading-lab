#pragma once
#include <map>
#include <string>

struct StrategyParams {
  double mu;    // Annualized Expected Return
  double sigma; // Annualized Volatility
};

struct MultiplexerConfig {
  double kelly_fraction; // e.g. 0.3
};

// Simple Registry for Demo
struct ClientRegistry {
  std::map<std::string, StrategyParams> clients;
};
