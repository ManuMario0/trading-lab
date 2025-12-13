#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

struct InstrumentData {
  std::string symbol;
  std::string exchange;
};

struct Instrument {
  std::string type; // "Stock"
  InstrumentData data;

  bool operator<(const Instrument &other) const {
    if (type != other.type)
      return type < other.type;
    if (data.symbol != other.data.symbol)
      return data.symbol < other.data.symbol;
    return data.exchange < other.data.exchange;
  }

  bool operator==(const Instrument &other) const {
    return type == other.type && data.symbol == other.data.symbol &&
           data.exchange == other.data.exchange;
  }
};

struct Price {
  Instrument instrument;
  double last;
  double bid;
  double ask;
  long long timestamp; // Unix ms
};

// Alias for compatibility if needed, or we just use Price
using MarketUpdate = Price;

// JSON Serialization
// ------------------

inline void to_json(nlohmann::json &j, const InstrumentData &p) {
  j = nlohmann::json{{"symbol", p.symbol}, {"exchange", p.exchange}};
}

inline void from_json(const nlohmann::json &j, InstrumentData &p) {
  j.at("symbol").get_to(p.symbol);
  j.at("exchange").get_to(p.exchange);
}

inline void to_json(nlohmann::json &j, const Instrument &p) {
  j = nlohmann::json{{"type", p.type}, {"data", p.data}};
}

inline void from_json(const nlohmann::json &j, Instrument &p) {
  j.at("type").get_to(p.type);
  j.at("data").get_to(p.data);
}

inline void to_json(nlohmann::json &j, const Price &p) {
  j = nlohmann::json{{"instrument", p.instrument},
                     {"last", p.last},
                     {"bid", p.bid},
                     {"ask", p.ask},
                     {"timestamp", p.timestamp}};
}

inline void from_json(const nlohmann::json &j, Price &p) {
  j.at("instrument").get_to(p.instrument);
  j.at("last").get_to(p.last);
  j.at("bid").get_to(p.bid);
  j.at("ask").get_to(p.ask);
  j.at("timestamp").get_to(p.timestamp);
}
