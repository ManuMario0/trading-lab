#include "ZmqAdminListener.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

ZmqAdminListener::ZmqAdminListener(zmq::context_t &context,
                                   const std::string &bind_addr,
                                   KellyMultiplexer &app)
    : socket_(context, zmq::socket_type::rep), app_(app), running_(false) {
  socket_.bind(bind_addr);
  std::cout << "[ZmqAdmin] Bound to " << bind_addr << std::endl;
}

ZmqAdminListener::~ZmqAdminListener() { stop(); }

void ZmqAdminListener::start() {
  running_ = true;
  worker_ = std::thread(&ZmqAdminListener::listen_loop, this);
}

void ZmqAdminListener::stop() {
  running_ = false;
  if (worker_.joinable()) {
    worker_.detach(); // Simpler for V0
  }
}

void ZmqAdminListener::listen_loop() {
  while (running_) {
    zmq::message_t request;
    try {
      if (!socket_.recv(request, zmq::recv_flags::none))
        continue;

      std::string str_req(static_cast<char *>(request.data()), request.size());
      json j_req = json::parse(str_req);

      std::cout << "[ZmqAdmin] Received: " << str_req << std::endl;

      std::string cmd = j_req.value("cmd", "");
      json response;

      if (cmd == "ADD" || cmd == "UPDATE") {
        std::string id = j_req.at("id");
        double mu = j_req.at("mu");
        double sigma = j_req.at("sigma");
        app_.add_client(id, mu, sigma);
        response = {{"status", "OK"}, {"msg", "Client updated"}};
      } else if (cmd == "REMOVE") {
        std::string id = j_req.at("id");
        app_.remove_client(id);
        response = {{"status", "OK"}, {"msg", "Client removed"}};
      } else {
        response = {{"status", "ERROR"}, {"msg", "Unknown command"}};
      }

      std::string str_resp = response.dump();
      zmq::message_t reply(str_resp.begin(), str_resp.end());
      socket_.send(reply, zmq::send_flags::none);

    } catch (const zmq::error_t &e) {
      if (e.num() != ETERM)
        std::cerr << "[ZmqAdmin] ZMQ Error: " << e.what() << std::endl;
      break;
    } catch (const std::exception &e) {
      std::cerr << "[ZmqAdmin] Logic Error: " << e.what() << std::endl;
      // Try to send error response
      try {
        json err_resp = {{"status", "ERROR"}, {"msg", e.what()}};
        std::string s = err_resp.dump();
        socket_.send(zmq::buffer(s), zmq::send_flags::none);
      } catch (...) {
      }
    }
  }
}
