#include "ZmqInputListener.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

ZmqInputListener::ZmqInputListener(zmq::context_t &context,
                                   const std::string &bind_addr)
    : socket_(context, zmq::socket_type::pull), running_(false) {
  socket_.bind(bind_addr);
  std::cout << "[ZmqInput] Bound to " << bind_addr << std::endl;
}

ZmqInputListener::~ZmqInputListener() { stop(); }

void ZmqInputListener::start(Callback cb) {
  callback_ = cb;
  running_ = true;
  worker_ = std::thread(&ZmqInputListener::listen_loop, this);
}

void ZmqInputListener::stop() {
  running_ = false;
  // In a real app we might need to wake up the socket or use polling with
  // timeout For V0, we'll just detach or let it hang on join if blocked
  // (imperfect but simple) Ideally, context shutdown handles this.
  if (worker_.joinable()) {
    // Context termination usually unblocks recv, but we rely on simple
    // destruction here
    worker_.detach();
  }
}

void ZmqInputListener::listen_loop() {
  while (running_) {
    zmq::message_t msg;
    try {
      // Blocking receive
      auto res = socket_.recv(msg, zmq::recv_flags::none);
      if (!res)
        continue;

      std::string str_msg(static_cast<char *>(msg.data()), msg.size());
      json j = json::parse(str_msg);
      TargetPortfolio p = j; // Implicit conversion via from_json

      if (callback_) {
        callback_(p);
      }
    } catch (const zmq::error_t &e) {
      // Context closed or other error
      if (e.num() != ETERM) {
        std::cerr << "[ZmqInput] ZMQ Error: " << e.what() << std::endl;
      }
      break;
    } catch (const std::exception &e) {
      std::cerr << "[ZmqInput] JSON/Logic Error: " << e.what() << std::endl;
    }
  }
}
