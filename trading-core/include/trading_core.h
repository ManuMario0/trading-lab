#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Holds the standard configuration parameters parsed from the command line.
 *
 * These arguments are expected to be present for every microservice invocation.
 */
typedef struct CommonArgs CommonArgs;

/**
 * A manager for ZMQ sockets.
 * Note: ZMQ sockets are not thread-safe. This manager should generally be owned
 * by the thread that uses the sockets, or use internal mutexes if shared (which adds contention).
 */
typedef struct ExchangeManager ExchangeManager;

void tc_hello(void);

struct CommonArgs *tc_parse_args(void);

void tc_args_free(struct CommonArgs *ptr);

uint16_t tc_args_get_admin_port(const struct CommonArgs *ptr);

void tc_admin_start_server(uint16_t port);

void tc_admin_register_param(const char *name,
                             const char *description,
                             const char *default_value,
                             int param_type);

struct ExchangeManager *tc_exchange_manager_new(void);

void tc_exchange_manager_free(struct ExchangeManager *ptr);
