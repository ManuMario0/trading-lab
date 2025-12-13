cmake_minimum_required(VERSION 3.15)
project(strategy_lab CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Dependencies
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

# 3. LibZMQ
find_package(PkgConfig REQUIRED)
pkg_check_modules(ZeroMQ REQUIRED libzmq)
find_library(ACCELERATE_FRAMEWORK Accelerate)

# Include Directories
include_directories(src ${ZeroMQ_INCLUDE_DIRS})
link_directories(${ZeroMQ_LIBRARY_DIRS})

# Strategy Lab Executable
add_executable(strategy_lab
    src/main.cpp
    src/io/ZmqStrategyIO.cpp
    # Strategy headers are included in main.cpp
)

# Link Libraries
target_link_libraries(strategy_lab
    PRIVATE
    nlohmann_json::nlohmann_json
    cppzmq
    ${ZeroMQ_LIBRARIES}
    ${ACCELERATE_FRAMEWORK}
)
