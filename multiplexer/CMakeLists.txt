cmake_minimum_required(VERSION 3.15)
project(multiplexer CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

# 1. nlohmann/json
FetchContent_Declare(
    json
    URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz
    DOWNLOAD_EXTRACT_TIMESTAMP true
)
FetchContent_MakeAvailable(json)

# 2. cppzmq (Header-only wrapper)
set(CPPZMQ_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(CPPZMQ_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
FetchContent_Declare(
    cppzmq
    GIT_REPOSITORY https://github.com/zeromq/cppzmq.git
    GIT_TAG v4.10.0
)
FetchContent_MakeAvailable(cppzmq)

# 3. LibZMQ (System dependency usually, or we could fetch it)
find_package(PkgConfig REQUIRED)
pkg_check_modules(ZeroMQ REQUIRED libzmq)
link_directories(${ZeroMQ_LIBRARY_DIRS})

# Core Library
add_library(multiplexer_core
    src/models/Portfolio.hpp
    src/params/ClientConfig.hpp
    src/logic/KellyMultiplexer.hpp
    src/logic/KellyMultiplexer.cpp
    src/io/IInputListener.hpp
    src/io/IOutputPublisher.hpp
    src/io/ZmqInputListener.hpp
    src/io/ZmqInputListener.cpp
    src/io/ZmqOutputPublisher.hpp
    src/io/ZmqOutputPublisher.cpp
    src/io/ZmqAdminListener.hpp
    src/io/ZmqAdminListener.cpp
)

target_link_libraries(multiplexer_core
    PUBLIC
    nlohmann_json::nlohmann_json
    cppzmq
    ${ZeroMQ_LIBRARIES}
)
target_include_directories(multiplexer_core
    PUBLIC
    src
    ${ZeroMQ_INCLUDE_DIRS}
)

# Executable
add_executable(multiplexer src/main.cpp)
target_link_libraries(multiplexer PRIVATE multiplexer_core)
