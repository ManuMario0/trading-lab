#pragma once
#include "IOutputPublisher.hpp"
#include <string>
#include <zmq.hpp>

class ZmqOutputPublisher : public IOutputPublisher {
public:
  ZmqOutputPublisher(zmq::context_t &context, const std::string &bind_addr);
  ~ZmqOutputPublisher() override = default;

  void publish(const TargetPortfolio &portfolio) override;

private:
  zmq::socket_t socket_;
};
