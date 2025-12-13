#pragma once
#include "../logic/KellyMultiplexer.hpp"
#include <atomic>
#include <string>
#include <thread>
#include <zmq.hpp>

class ZmqAdminListener {
public:
  ZmqAdminListener(zmq::context_t &context, const std::string &bind_addr,
                   KellyMultiplexer &app);
  ~ZmqAdminListener();

  void start();
  void stop();

private:
  void listen_loop();

  zmq::socket_t socket_;
  KellyMultiplexer &app_;
  std::atomic<bool> running_;
  std::thread worker_;
};
