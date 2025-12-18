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
    // Look up config or Auto-Register
    auto it = registry_.clients.find(client_id);
    if (it == registry_.clients.end()) {
      std::cout << "[KellyMux] Auto-registering new client: " << client_id
                << std::endl;
      StrategyParams default_params;
      default_params.mu = 0.05;    // 5% expected excess return
      default_params.sigma = 0.20; // 20% volatility
      registry_.clients[client_id] = default_params;
      it = registry_.clients.find(client_id);
    }

    const StrategyParams &params = it->second;

    // Kelly Formula: f = (mu - r) / sigma^2
    double raw_kelly = 0.0;
    if (params.sigma > 1e-6) {
      raw_kelly = params.mu / (params.sigma * params.sigma);
    }

    // Apply Global Scalar (e.g. 0.3 * f)
    double scalar = config_.kelly_fraction * raw_kelly;

    // Safety clamp (optional but good for testing)
    if (scalar > 2.0)
      scalar = 2.0;
    if (scalar < -2.0)
      scalar = -2.0;

    // Scale and Add Weights
    for (const auto &[instrument, heavy_weight] : portfolio.target_weights) {
      // heavy_weight is usually -1.0 to 1.0 from Strategy usually representing
      // 'conviction' If Strategy sends 1.0 means "Full Position". We scale it
      // by Kelly.
      double final_weight = heavy_weight * scalar;
      aggregated.target_weights[instrument] += final_weight;
    }
  }

  return aggregated;
}
