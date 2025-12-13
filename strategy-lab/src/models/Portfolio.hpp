#pragma once
#include "MarketData.hpp" // For InstrumentData re-use if needed, though we redefine Instrument here matching Portfolio.hpp in multiplexer
#include <map>
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

// Re-using the exact structure from Multiplexer's Portfolio.hpp to ensure
// compatibility In a shared library scenario, this would be a common
// dependency.

// Instrument and InstrumentData are defined in MarketData.hpp

struct TargetPortfolio {
  std::string strategy_id; // Maps to 'multiplexer_id' effectively
  std::map<Instrument, double> target_weights;

  TargetPortfolio() = default;
};

inline void to_json(nlohmann::json &j, const TargetPortfolio &p) {
  std::vector<std::pair<Instrument, double>> weight_list;
  for (auto const &[key, val] : p.target_weights) {
    weight_list.push_back({key, val});
  }

  // Outputting in the format expected by Multiplexer/Engine
  // The Multiplexer expects "multiplexer_id", but we are a strategy.
  // The multiplexer will likely forward this, OR this strategy output connects
  // to a multiplexer input. If we connect to a Multiplexer, the Multiplexer
  // might expect a specific format. The user prompt said: "output socket (that
  // will connect mostlikely to a multiplexer) that sends TargetProfolio"
  j = nlohmann::json{
      {"multiplexer_id", p.strategy_id}, // Using strategy_id as the ID
      {"target_weights", weight_list},
      {"target_positions", nullptr}};
}

inline void from_json(const nlohmann::json &j, TargetPortfolio &p) {
  const nlohmann::json *data_ptr = &j;
  if (j.contains("data")) {
    data_ptr = &j["data"];
  }

  if (data_ptr->contains("multiplexer_id"))
    data_ptr->at("multiplexer_id").get_to(p.strategy_id);
  else if (data_ptr->contains("strategy_id"))
    data_ptr->at("strategy_id").get_to(p.strategy_id);

  if (data_ptr->contains("target_weights")) {
    auto &weights_array = data_ptr->at("target_weights");
    for (auto &item : weights_array) {
      Instrument inst = item[0].get<Instrument>();
      double w = item[1].get<double>();
      p.target_weights[inst] = w;
    }
  }
}
