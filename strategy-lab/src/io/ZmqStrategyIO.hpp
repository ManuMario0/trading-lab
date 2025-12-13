#pragma once
#include "../models/MarketData.hpp"
#include "../models/Portfolio.hpp"
#include <atomic>
#include <functional>
#include <string>
#include <thread>
#include <zmq.hpp>

// Callback types
using MarketDataCallback = std::function<void(const MarketUpdate &)>;
using AdminCallback =
    std::function<std::string(const std::string &)>; // Command -> Response

class ZmqStrategyIO {
public:
  ZmqStrategyIO(const std::string &input_addr, const std::string &output_addr,
                const std::string &admin_addr);
  ~ZmqStrategyIO();

  // Start validity of processing
  void start(MarketDataCallback market_cb, AdminCallback admin_cb);
  void stop();

  // Send portfolio update
  void send_portfolio(const TargetPortfolio &portfolio);

private:
  void input_loop(); // Listens for Market Data
  void admin_loop(); // Listens for Admin commands

  std::string input_addr_;
  std::string output_addr_;
  std::string admin_addr_;

  zmq::context_t context_;
  zmq::socket_type input_type_ =
      zmq::socket_type::sub; // Market Data usually PUB/SUB
  zmq::socket_type output_type_ =
      zmq::socket_type::push; // PUSH to Multiplexer PULL
  zmq::socket_type admin_type_ = zmq::socket_type::rep; // Request/Reply

  zmq::socket_t input_socket_;
  zmq::socket_t output_socket_;
  zmq::socket_t admin_socket_;

  std::atomic<bool> running_;
  std::thread input_thread_;
  std::thread admin_thread_;

  MarketDataCallback market_callback_;
  AdminCallback admin_callback_;
};
