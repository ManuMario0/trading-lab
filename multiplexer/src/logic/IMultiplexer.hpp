#include "models/Portfolio.hpp"

class IMultiplexer {
public:
  // Virtual destructor is essential for proper cleanup of derived classes
  virtual ~IMultiplexer() = default;

  // Defines a method that processes portfolio updates.
  // The parameter type (const Portfolio&) is a placeholder
  // and should be adjusted based on the actual Portfolio structure.
  virtual TargetPortfolio
  on_portfolio_received(const TargetPortfolio &portfolio) = 0;

  virtual void add_client(const std::string &clientId, double mu,
                          double sigma) = 0;
  virtual void remove_client(const std::string &clientId) = 0;
};
