#include "ZmqStrategyIO.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

ZmqStrategyIO::ZmqStrategyIO(const std::string &input_addr,
                             const std::string &output_addr,
                             const std::string &admin_addr)
    : input_addr_(input_addr), output_addr_(output_addr),
      admin_addr_(admin_addr), context_(1),
      input_socket_(context_, input_type_),
      output_socket_(context_, output_type_),
      admin_socket_(context_, admin_type_), running_(false) {
  // Setup Input (Subscriber)
  // Note: If using SUB, we must subscribe to a topic. "" subscribes to all.
  input_socket_.connect(input_addr_);
  input_socket_.set(zmq::sockopt::subscribe, "");
  std::cout << "[StrategyIO] Connected Input (SUB) to " << input_addr_
            << std::endl;

  // Setup Output (Push)
  output_socket_.connect(output_addr_);
  std::cout << "[StrategyIO] Connected Output (PUSH) to " << output_addr_
            << std::endl;

  // Setup Admin (Rep) - Binds to a port to accept connections
  admin_socket_.bind(admin_addr_);
  std::cout << "[StrategyIO] Bound Admin (REP) to " << admin_addr_ << std::endl;
}

ZmqStrategyIO::~ZmqStrategyIO() { stop(); }

void ZmqStrategyIO::start(MarketDataCallback market_cb,
                          AdminCallback admin_cb) {
  market_callback_ = market_cb;
  admin_callback_ = admin_cb;
  running_ = true;

  input_thread_ = std::thread(&ZmqStrategyIO::input_loop, this);
  admin_thread_ = std::thread(&ZmqStrategyIO::admin_loop, this);
}

void ZmqStrategyIO::stop() {
  running_ = false;
  // Context shutdown or close sockets to break blocking calls
  // In a robust app, we'd handle this better (e.g. zmq_term or sending signal)
  // For V0, detaching is acceptable as destructor cleans up context
  if (input_thread_.joinable())
    input_thread_.detach();
  if (admin_thread_.joinable())
    admin_thread_.detach();
}

void ZmqStrategyIO::send_portfolio(const TargetPortfolio &portfolio) {
  try {
    json j = portfolio;
    std::string payload = j.dump();
    zmq::message_t msg(payload.data(), payload.size());
    output_socket_.send(msg, zmq::send_flags::dontwait); // Non-blocking send
  } catch (const std::exception &e) {
    std::cerr << "[StrategyIO] Error sending portfolio: " << e.what()
              << std::endl;
  }
}

void ZmqStrategyIO::input_loop() {
  while (running_) {
    try {
      zmq::message_t msg;
      auto res = input_socket_.recv(msg, zmq::recv_flags::none);
      if (!res)
        continue;

      std::string str_msg(static_cast<char *>(msg.data()), msg.size());
      // Assuming direct JSON payload. In PUB/SUB, sometimes there's a topic
      // frame first. If subscribed to "", it might just be the payload if the
      // sender doesn't send multi-part. We'll assume simple JSON for now.
      try {
        json j = json::parse(str_msg);
        MarketUpdate update = j;
        if (market_callback_) {
          market_callback_(update);
        }
      } catch (const std::exception &e) {
        // Might be a topic string if multi-part, or just bad data
        // std::cerr << "[StrategyIO] JSON Parse Error: " << e.what() << " Msg:
        // " << str_msg << std::endl; Silencing to avoid spam on random noise
      }

    } catch (const zmq::error_t &e) {
      if (e.num() != ETERM) {
        std::cerr << "[StrategyIO] Input ZMQ Error: " << e.what() << std::endl;
      }
      break;
    }
  }
}

void ZmqStrategyIO::admin_loop() {
  while (running_) {
    try {
      zmq::message_t request;
      auto res = admin_socket_.recv(request, zmq::recv_flags::none);
      if (!res)
        continue;

      std::string cmd(static_cast<char *>(request.data()), request.size());
      std::string response = "UNKNOWN";

      if (admin_callback_) {
        response = admin_callback_(cmd);
      }

      zmq::message_t reply(response.data(), response.size());
      admin_socket_.send(reply, zmq::send_flags::none);

    } catch (const zmq::error_t &e) {
      if (e.num() != ETERM) {
        std::cerr << "[StrategyIO] Admin ZMQ Error: " << e.what() << std::endl;
      }
      break;
    }
  }
}
