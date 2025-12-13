#include "ZmqOutputPublisher.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

ZmqOutputPublisher::ZmqOutputPublisher(zmq::context_t &context,
                                       const std::string &bind_addr)
    : socket_(context, zmq::socket_type::pub) {
  socket_.bind(bind_addr);
  std::cout << "[ZmqOutput] Bound to " << bind_addr << std::endl;
}

void ZmqOutputPublisher::publish(const TargetPortfolio &portfolio) {
  try {
    json j = portfolio;
    std::string dump = j.dump(); // Compact JSON
    zmq::message_t msg(dump.begin(), dump.end());
    socket_.send(msg, zmq::send_flags::none);
    // std::cout << "[ZmqOutput] Published " << dump.size() << " bytes." <<
    // std::endl;
  } catch (const std::exception &e) {
    std::cerr << "[ZmqOutput] Error publishing: " << e.what() << std::endl;
  }
}
