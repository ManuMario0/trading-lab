#include "KellyMultiplexer.hpp"
#include <iostream>

KellyMultiplexer::KellyMultiplexer(ClientRegistry registry,
                                   MultiplexerConfig config)
    : registry_(std::move(registry)), config_(config) {}

void KellyMultiplexer::add_client(const std::string &id, double mu,
                                  double sigma) {
  std::lock_guard<std::mutex> lock(state_mutex_);
  registry_.clients[id] = {mu, sigma};
  std::cout << "[KellyMux] Added/Updated client " << id << " (Mu=" << mu
            << ", Sigma=" << sigma << ")" << std::endl;
  // Note: We don't recalculate immediately, next tick will pick it up
}

void KellyMultiplexer::remove_client(const std::string &id) {
  std::lock_guard<std::mutex> lock(state_mutex_);
  registry_.clients.erase(id);
  client_portfolios_.erase(id); // Also remove stale data
  std::cout << "[KellyMux] Removed client " << id << std::endl;
}

TargetPortfolio
KellyMultiplexer::on_portfolio_received(const TargetPortfolio &p) {
  std::lock_guard<std::mutex> lock(state_mutex_);

  // 1. Update State
  client_portfolios_[p.multiplexer_id] = p;

  // 2. Recalculate
  return recalculate_and_publish();
}

TargetPortfolio KellyMultiplexer::recalculate_and_publish() {
  // Check if we have clients
  if (client_portfolios_.empty())
    return {};

  TargetPortfolio aggregated;
  aggregated.multiplexer_id = "KellyMux_Aggregated";

  // Accumulate weights
  // Formula: Weight_Mux = Sum(Weight_Strat * KellyScalar)

  for (const auto &[client_id, portfolio] : client_portfolios_) {
    // Look up config
    auto it = registry_.clients.find(client_id);
    if (it == registry_.clients.end()) {
      std::cerr << "Warning: No config for client " << client_id
                << ", ignoring." << std::endl;
      continue;
    }

    const StrategyParams &params = it->second;

    // Kelly Formula: f = (mu - r) / sigma^2
    // Assuming r (risk free) = 0 for simplicity or embedded in mu (excess
    // return).
    double raw_kelly = 0.0;
    if (params.sigma > 1e-6) {
      raw_kelly = params.mu / (params.sigma * params.sigma);
    }

    // Apply Global Scalar (e.g. 0.3 * f)
    double scalar = config_.kelly_fraction * raw_kelly;

    // Scale and Add Weights
    for (const auto &[instrument, heavy_weight] : portfolio.target_weights) {
      double final_weight = heavy_weight * scalar;
      aggregated.target_weights[instrument] += final_weight;
    }
  }

  return aggregated;
}
