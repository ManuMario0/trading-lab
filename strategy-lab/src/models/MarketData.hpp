#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

struct InstrumentData {
  std::string symbol;
  std::string exchange;
};

// Simplified Market Update for V0
// It receives a list of assets and their current prices/state
struct MarketUpdate {
  struct AssetUpdate {
    std::string symbol;
    std::string exchange;
    double price;
    // extended fields like bid/ask/last/volume can be added here
  };

  std::vector<AssetUpdate> updates;
  std::string timestamp; // Optional, for logging/sync
};

// JSON Serialization
// ------------------

inline void to_json(nlohmann::json &j, const MarketUpdate::AssetUpdate &p) {
  j = nlohmann::json{
      {"symbol", p.symbol}, {"exchange", p.exchange}, {"price", p.price}};
}

inline void from_json(const nlohmann::json &j, MarketUpdate::AssetUpdate &p) {
  j.at("symbol").get_to(p.symbol);
  j.at("exchange").get_to(p.exchange);
  j.at("price").get_to(p.price);
}

inline void to_json(nlohmann::json &j, const MarketUpdate &p) {
  j = nlohmann::json{{"updates", p.updates}, {"timestamp", p.timestamp}};
}

inline void from_json(const nlohmann::json &j, MarketUpdate &p) {
  j.at("updates").get_to(p.updates);
  if (j.contains("timestamp")) {
    j.at("timestamp").get_to(p.timestamp);
  }
}
