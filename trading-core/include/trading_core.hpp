#pragma once

#include "rust/cxx.h"
#include "trading-core/src/ffi/mod.rs.h"
#include <string>
#include <vector>

namespace trading {

/**
 * @brief Represents the side of an order (Buy or Sell).
 */
enum class OrderSide { Buy = 0, Sell = 1 };

/**
 * @brief Represents the type of order (how it should be executed).
 */
enum class OrderType { Limit = 0, Market = 1, Stop = 2 };

/**
 * @brief Holds the standard configuration parameters parsed from the command
 * line.
 */
class CommonArgs {
public:
  /**
   * @brief Parses command-line arguments into a CommonArgs instance.
   * @param args The command-line arguments.
   * @return A new CommonArgs instance.
   */
  static CommonArgs parse(const std::vector<std::string> &args) {
    // deep copy args for Rust vec
    rust::Vec<rust::String> rust_args;
    for (const auto &arg : args) {
      rust_args.push_back(arg);
    }
    return CommonArgs(trading_core::args_parse(std::move(rust_args)));
  }

  CommonArgs(CommonArgs &&) = default;
  CommonArgs &operator=(CommonArgs &&) = default;
  CommonArgs(const CommonArgs &) = delete;
  CommonArgs &operator=(const CommonArgs &) = delete;

  /**
   * @brief Returns the name of the service.
   */
  std::string get_service_name() const {
    return std::string(inner_->get_service_name());
  }

  /**
   * @brief Returns the admin port (SUB) as a string.
   */
  std::string get_admin_route() const {
    return std::string(inner_->get_admin_route_str());
  }

  /**
   * @brief Returns the output port (PUB) as a string.
   */
  std::string get_output_port() const {
    return std::string(inner_->get_output_port_str());
  }

  /**
   * @brief Returns the path to the configuration directory.
   */
  std::string get_config_dir() const {
    return std::string(inner_->get_config_dir_str());
  }

  /**
   * @brief Returns the path to the data directory.
   */
  std::string get_data_dir() const {
    return std::string(inner_->get_data_dir_str());
  }

private:
  explicit CommonArgs(rust::Box<trading_core::CommonArgs> ptr)
      : inner_(std::move(ptr)) {}
  rust::Box<trading_core::CommonArgs> inner_;
};

/**
 * @brief Represents a trading order to buy or sell an instrument.
 */
class Order {
public:
  /**
   * @brief Creates a new Order.
   */
  Order(const std::string &id, const std::string &instrument_id, OrderSide side,
        OrderType type, double price, double quantity, int64_t timestamp)
      : inner_(trading_core::new_order(
            id, instrument_id, static_cast<int32_t>(side),
            static_cast<int32_t>(type), price, quantity, timestamp)) {}

  Order(Order &&) = default;
  Order &operator=(Order &&) = default;
  Order(const Order &) = delete;
  Order &operator=(const Order &) = delete;

  std::string get_id() const { return std::string(inner_->get_id()); }
  std::string get_instrument_id() const {
    return std::string(inner_->get_instrument_id());
  }

  OrderSide get_side() const {
    return static_cast<OrderSide>(inner_->get_side_i32());
  }

  OrderType get_type() const {
    return static_cast<OrderType>(inner_->get_type_i32());
  }

  double get_price() const { return inner_->get_price(); }
  double get_quantity() const { return inner_->get_quantity(); }
  int64_t get_timestamp() const { return inner_->get_timestamp(); }

  const trading_core::Order &raw() const { return *inner_; }

private:
  rust::Box<trading_core::Order> inner_;
};

/**
 * @brief Represents a Stock/Equity instrument.
 */
class Stock {
public:
  Stock(size_t id, const std::string &symbol, const std::string &exchange,
        const std::string &sector, const std::string &industry,
        const std::string &country, const std::string &currency)
      : inner_(trading_core::new_stock(id, symbol, exchange, sector, industry,
                                       country, currency)) {}

  Stock(Stock &&) = default;
  Stock &operator=(Stock &&) = default;
  Stock(const Stock &) = delete;
  Stock &operator=(const Stock &) = delete;

  size_t get_id() const { return inner_->get_id(); }
  std::string get_symbol() const { return std::string(inner_->get_symbol()); }
  std::string get_exchange() const {
    return std::string(inner_->get_exchange());
  }
  std::string get_sector() const { return std::string(inner_->get_sector()); }
  std::string get_industry() const {
    return std::string(inner_->get_industry());
  }
  std::string get_country() const { return std::string(inner_->get_country()); }
  std::string get_currency() const {
    return std::string(inner_->get_currency());
  }

private:
  rust::Box<trading_core::Stock> inner_;
};

/**
 * @brief Lightweight view of a PriceUpdate.
 * Does not own the underlying data.
 */
class PriceUpdateView {
public:
  explicit PriceUpdateView(const trading_core::PriceUpdate &raw)
      : inner_(&raw) {}

  size_t get_instrument_id() const { return inner_->get_instrument_id(); }
  double get_price() const { return inner_->get_price(); }
  uint64_t get_timestamp() const { return inner_->get_timestamp(); }

private:
  const trading_core::PriceUpdate *inner_;
};

/**
 * @brief Represents a batch of price updates.
 */
class MarketDataBatch {
public:
  // Wraps an existing batch (usually received from callback)
  explicit MarketDataBatch(const trading_core::MarketDataBatch &raw)
      : inner_(&raw) {}

  size_t count() const { return inner_->get_count(); }

  PriceUpdateView at(size_t index) const {
    return PriceUpdateView(inner_->get_update_at(index));
  }

private:
  // Reference view mainly, as batches are usually passed into C++ callbacks
  const trading_core::MarketDataBatch *inner_;
};

/**
 * @brief Represents a Position in an Allocation.
 */
class Position {
public:
  explicit Position(rust::Box<trading_core::Position> ptr)
      : inner_(std::move(ptr)) {}

  Position(Position &&) = default;
  Position &operator=(Position &&) = default;
  Position(const Position &) = delete;
  Position &operator=(const Position &) = delete;

  size_t get_instrument_id() const { return inner_->get_instrument_id(); }
  double get_quantity() const { return inner_->get_quantity(); }

private:
  rust::Box<trading_core::Position> inner_;
};

/**
 * @brief Represents an Allocation (target portfolio).
 */
class Allocation {
public:
  // Usually passed by reference from Rust
  explicit Allocation(const trading_core::Allocation &raw) : inner_(&raw) {}

  size_t get_id() const { return inner_->get_id(); }
  std::string get_source() const { return std::string(inner_->get_source()); }
  uint64_t get_timestamp() const { return inner_->get_timestamp_u64(); }

  bool has_position(size_t instrument_id) const {
    return inner_->has_position(instrument_id);
  }

  /**
   * @brief Retrieves a copy of the position for an instrument.
   * @return Position object (owned copy).
   */
  Position get_position_copy(size_t instrument_id) const {
    return Position(inner_->get_position_copy(instrument_id));
  }

private:
  const trading_core::Allocation *inner_;
};

/**
 * @brief View of a microservice Parameter.
 */
class ParameterView {
public:
  explicit ParameterView(const trading_core::Parameter &raw) : inner_(&raw) {}

  std::string get_name() const { return std::string(inner_->get_name()); }
  std::string get_description() const {
    return std::string(inner_->get_description());
  }
  std::string get_value_string() const {
    return std::string(inner_->get_value_as_string());
  }
  bool is_updatable() const { return inner_->is_updatable(); }

private:
  const trading_core::Parameter *inner_;
};

/**
 * @brief Registry of microservice parameters.
 */
class Registry {
public:
  static Registry create() { return Registry(trading_core::new_registry()); }

  Registry(Registry &&) = default;
  Registry &operator=(Registry &&) = default;
  Registry(const Registry &) = delete;
  Registry &operator=(const Registry &) = delete;

  /**
   * @brief Returns a list of parameter names.
   */
  std::vector<std::string> get_parameter_names() const {
    auto rust_vec = inner_->get_parameters_list();
    std::vector<std::string> names;
    names.reserve(rust_vec.size());
    for (const auto &s : rust_vec) {
      names.push_back(std::string(s));
    }
    return names;
  }

  // Access raw inner box if needed for other FFI calls
  // const trading_core::Registry& raw() const { return *inner_; }
  // trading_core::Registry& raw_mut() { return *inner_; }

private:
  explicit Registry(rust::Box<trading_core::Registry> ptr)
      : inner_(std::move(ptr)) {}
  rust::Box<trading_core::Registry> inner_;
};

} // namespace trading
