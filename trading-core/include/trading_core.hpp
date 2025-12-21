#pragma once

#include "rust/cxx.h"
#include "trading-core/src/ffi/mod.rs.h"
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

namespace trading {

// Re-export specific types from the FFI namespace for convenience
using ExchangeType = trading_core::ExchangeType;

/**
 * Configuration for an exchange socket.
 */
struct ExchangeConfig {
  std::string name;
  std::string endpoint;
  ExchangeType socket_type;
  bool is_bind;

  // Convert to FFI type
  trading_core::ExchangeConfig to_ffi() const {
    return trading_core::ExchangeConfig{name, endpoint, socket_type, is_bind};
  }
};

/**
 * Admin interface for managing global settings and the admin server.
 */
class Admin {
public:
  static void start_server(uint16_t port) {
    trading_core::admin_start_server(port);
  }

  static void register_param(const std::string &name,
                             const std::string &description,
                             const std::string &default_value, int param_type) {
    trading_core::admin_register_param(name, description, default_value,
                                       param_type);
  }
};

/**
 * Manages ZMQ sockets for the trading core.
 */
class ExchangeManager {
public:
  ExchangeManager() : inner_(trading_core::new_exchange_manager()) {}

  // Prevent copying to avoid multiple ownership issues with the underlying Box
  // (though Box is move-only)
  ExchangeManager(const ExchangeManager &) = delete;
  ExchangeManager &operator=(const ExchangeManager &) = delete;

  // Allow moving
  ExchangeManager(ExchangeManager &&) = default;
  ExchangeManager &operator=(ExchangeManager &&) = default;

  void add_exchange(const ExchangeConfig &config) {
    // We pass the FFI struct to the rust function
    trading_core::exchange_manager_add(*inner_, config.to_ffi());
  }

  void send(const std::string &name, const std::string &data, int flags = 0) {
    // Convert string to bytes slice
    rust::Slice<const uint8_t> slice(
        reinterpret_cast<const uint8_t *>(data.data()), data.size());
    trading_core::exchange_manager_send(*inner_, name, slice, flags);
  }

  void send(const std::string &name, const std::vector<uint8_t> &data,
            int flags = 0) {
    rust::Slice<const uint8_t> slice(data.data(), data.size());
    trading_core::exchange_manager_send(*inner_, name, slice, flags);
  }

  std::vector<uint8_t> recv(const std::string &name, int flags = 0) {
    rust::Vec<uint8_t> result =
        trading_core::exchange_manager_recv(*inner_, name, flags);

    // Convert rust::Vec to std::vector
    std::vector<uint8_t> vec;
    vec.reserve(result.size());
    std::copy(result.begin(), result.end(), std::back_inserter(vec));
    return vec;
  }

  // Access to the raw inner pointer if needed for other FFI calls
  // trading_core::ExchangeManager& inner() { return *inner_; }

private:
  rust::Box<trading_core::ExchangeManager> inner_;
};

} // namespace trading
