#pragma once
#include "../models/Portfolio.hpp"
#include "../params/ClientConfig.hpp"
#include "IMultiplexer.hpp"
#include <map>
#include <mutex>
#include <string>

class KellyMultiplexer : public IMultiplexer {
public:
  KellyMultiplexer(ClientRegistry registry, MultiplexerConfig config);

  TargetPortfolio
  on_portfolio_received(const TargetPortfolio &received_portfolio);

  // Admin Methods
  void add_client(const std::string &id, double mu, double sigma);
  void remove_client(const std::string &id);

private:
  TargetPortfolio recalculate_and_publish();

  ClientRegistry registry_;
  MultiplexerConfig config_;

  // Thread-safety for map updates
  std::mutex state_mutex_;
  // Store latest portfolio from each client
  std::map<std::string, TargetPortfolio> client_portfolios_;
};
