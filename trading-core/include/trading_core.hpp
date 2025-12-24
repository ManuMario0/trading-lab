#pragma once

#include "rust/cxx.h"
#include "trading-core/src/ffi/mod.rs.h"
#include <functional>
#include <memory>
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

  // Access inner for passing to Microservice
  rust::Box<trading_core::CommonArgs> into_inner() { return std::move(inner_); }

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
 * @brief Represents a single update to the price.
 * Can be owning (Box) or a view (const ref).
 */
class PriceUpdate {
public:
  // Create new owning PriceUpdate
  PriceUpdate(size_t instrument_id, double price, uint64_t timestamp)
      : inner_(trading_core::new_price_update(instrument_id, price, timestamp)),
        is_owned_(true), ptr_(&*inner_) {}

  // Create view from raw pointer (internal use)
  explicit PriceUpdate(const trading_core::PriceUpdate *ptr)
      : inner_(trading_core::new_price_update(0, 0,
                                              0)), // Dummy init, safely unused
        is_owned_(false), ptr_(ptr) {}

  // Move constructor
  PriceUpdate(PriceUpdate &&other) noexcept
      : inner_(std::move(other.inner_)), is_owned_(other.is_owned_),
        ptr_(other.ptr_) {
    if (is_owned_) {
      ptr_ = &*inner_;
    }
  }

  size_t get_instrument_id() const { return ptr_->get_instrument_id(); }
  double get_price() const { return ptr_->get_price(); }
  uint64_t get_timestamp() const { return ptr_->get_timestamp(); }

  // Check if owned to extract box if needed
  bool is_owned() const { return is_owned_; }

  // Helper to release ownership (move out Box)
  rust::Box<trading_core::PriceUpdate> release() {
    if (!is_owned_)
      throw std::runtime_error("Cannot release non-owned PriceUpdate");
    return std::move(inner_);
  }

private:
  rust::Box<trading_core::PriceUpdate> inner_;
  bool is_owned_;
  const trading_core::PriceUpdate *ptr_;
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
 * Owning version.
 */
class MarketDataBatch {
public:
  MarketDataBatch() : inner_(trading_core::new_market_data_batch()) {}

  // Wrap existing
  explicit MarketDataBatch(const trading_core::MarketDataBatch &raw)
      // We can't easily wrap a reference in an owning box without cloning.
      // But we can support a "View" mode via a separate class or flag.
      // For simplicity, this class OWNS new batches.
      // Callbacks receiving batches should use MarketDataBatchView.
      : inner_(trading_core::new_market_data_batch()) {}

  MarketDataBatch(MarketDataBatch &&) = default;
  MarketDataBatch &operator=(MarketDataBatch &&) = default;

  void add_update(PriceUpdate update) {
    if (update.is_owned()) {
      inner_->add_update(update.release());
    } else {
      throw std::runtime_error("Cannot add non-owned PriceUpdate to Batch");
    }
  }

  void clear() { inner_->clear(); }

  size_t count() const { return inner_->get_count(); }

  // Return view
  PriceUpdateView at(size_t index) const {
    return PriceUpdateView(inner_->get_update_at(index));
  }

  // Access inner for FFI passing
  const trading_core::MarketDataBatch &raw() const { return *inner_; }
  trading_core::MarketDataBatch &raw_mut() { return *inner_; }

private:
  rust::Box<trading_core::MarketDataBatch> inner_;
};

/**
 * @brief Read-only view of a MarketDataBatch.
 */
class MarketDataBatchView {
public:
  explicit MarketDataBatchView(const trading_core::MarketDataBatch &raw)
      : inner_(&raw) {}

  size_t count() const { return inner_->get_count(); }

  PriceUpdateView at(size_t index) const {
    return PriceUpdateView(inner_->get_update_at(index));
  }

  const trading_core::MarketDataBatch *raw_ptr() const { return inner_; }

private:
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
 * @brief Represents a mutable/owning Allocation.
 */
class Allocation {
public:
  Allocation(const std::string &source, size_t id)
      : inner_(trading_core::new_allocation(source, id)) {}

  // Steal inner box (internal use)
  explicit Allocation(rust::Box<trading_core::Allocation> ptr)
      : inner_(std::move(ptr)) {}

  Allocation(Allocation &&) = default;

  void update_position(size_t instrument_id, double quantity) {
    inner_->update_position(instrument_id, quantity);
  }

  size_t get_id() const { return inner_->get_id(); }
  std::string get_source() const { return std::string(inner_->get_source()); }
  uint64_t get_timestamp() const { return inner_->get_timestamp_u64(); }

  bool has_position(size_t instrument_id) const {
    return inner_->has_position(instrument_id);
  }

  double get_position_quantity(size_t instrument_id) const {
    return inner_->get_position_quantity(instrument_id);
  }

  // Release ownership for FFI return
  rust::Box<trading_core::Allocation> release() { return std::move(inner_); }

private:
  rust::Box<trading_core::Allocation> inner_;
};

/**
 * @brief Read-only view of an Allocation.
 */
class AllocationView {
public:
  // Usually passed by reference from Rust
  explicit AllocationView(const trading_core::Allocation &raw) : inner_(&raw) {}

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

// --- Strategy Configuration ---

// Callback type: receives a batch view, returns an optional Allocation (owned).
// If returned Allocation is empty/null, no trade is made.
using StrategyCallback =
    std::function<std::unique_ptr<Allocation>(const MarketDataBatchView &)>;

extern "C" {
// This static thunk is called by Rust. It casts user_data back to the
// std::function. It creates a new owned Allocation on heap (Box) and returns
// the raw pointer to Rust.
static trading_core::Allocation *
strategy_callback_thunk(const trading_core::MarketDataBatch *batch_ptr,
                        size_t user_data) {
  auto *cb = reinterpret_cast<StrategyCallback *>(user_data);
  MarketDataBatchView batch_view(*batch_ptr);

  try {
    std::unique_ptr<Allocation> result = (*cb)(batch_view);
    if (result) {
      // Return the raw Box pointer. We release the Rust Box from our C++
      // wrapper.
      return result->release().into_raw();
    }
  } catch (...) {
    // Suppress exceptions
  }
  return nullptr;
}
}

/**
 * @brief Configuration for the Strategy Microservice.
 */
class Configuration {
public:
  static Configuration create_strategy(StrategyCallback callback) {
    // We clone the callback onto the heap to persist it.
    // NOTE: This leaks once if we don't clean it up, but for a service that
    // runs forever it's acceptable. Ideally we'd wrap this in a struct that
    // Rust owns and drops, but FFI with C++ lambdas is hard.
    auto *cb_ptr = new StrategyCallback(std::move(callback));

    return Configuration(trading_core::new_strategy_configuration(
        reinterpret_cast<size_t>(&strategy_callback_thunk),
        reinterpret_cast<size_t>(cb_ptr)));
  }

  rust::Box<trading_core::Configuration> into_inner() {
    return std::move(inner_);
  }

private:
  explicit Configuration(rust::Box<trading_core::Configuration> ptr)
      : inner_(std::move(ptr)) {}
  rust::Box<trading_core::Configuration> inner_;
};

/**
 * @brief Microservice application instance.
 */
class Microservice {
public:
  Microservice(CommonArgs args, Configuration config)
      : inner_(trading_core::new_microservice(args.into_inner(),
                                              config.into_inner())) {}

  /**
   * @brief Runs the microservice. Blocks indefinitely.
   */
  void run() { inner_->run(); }

private:
  rust::Box<trading_core::Microservice> inner_;
};

// --- Registry ---

class Registry {
public:
  static Registry create() { return Registry(trading_core::new_registry()); }

  Registry(Registry &&) = default;
  Registry &operator=(Registry &&) = default;
  Registry(const Registry &) = delete;
  Registry &operator=(const Registry &) = delete;

  std::vector<std::string> get_parameter_names() const {
    auto rust_vec = inner_->get_parameters_list();
    std::vector<std::string> names;
    names.reserve(rust_vec.size());
    for (const auto &s : rust_vec) {
      names.push_back(std::string(s));
    }
    return names;
  }

private:
  explicit Registry(rust::Box<trading_core::Registry> ptr)
      : inner_(std::move(ptr)) {}
  rust::Box<trading_core::Registry> inner_;
};

} // namespace trading
