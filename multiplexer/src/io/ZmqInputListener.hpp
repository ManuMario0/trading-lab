#pragma once
#include "IInputListener.hpp"
#include <atomic>
#include <string>
#include <thread>
#include <zmq.hpp>

class ZmqInputListener : public IInputListener {
public:
  ZmqInputListener(zmq::context_t &context, const std::string &bind_addr);
  ~ZmqInputListener() override;

  void start(Callback cb) override;
  void stop();

private:
  void listen_loop();

  zmq::socket_t socket_;
  Callback callback_;
  std::atomic<bool> running_;
  std::thread worker_;
};
