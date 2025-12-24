cmake_minimum_required(VERSION 3.15)
project(strategy_lab CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Include Directories
# Assuming trading-core is at ../trading-core
include_directories(src ../trading-core/include ../trading-core/target/cxxbridge)

# Find System Libraries Required by Rust
find_library(CORE_FOUNDATION CoreFoundation)
find_library(SECURITY Security)
find_library(SYSTEM_CONFIGURATION SystemConfiguration)

# Find ZeroMQ (Required by trading-core)
find_package(PkgConfig REQUIRED)
pkg_check_modules(ZeroMQ REQUIRED libzmq)
link_directories(${ZeroMQ_LIBRARY_DIRS})

# Add Executable
add_executable(strategy_lab
    src/main.cpp
    # DummyStrategy is header-only or included
)

# Link Libraries
target_link_libraries(strategy_lab
    PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../trading-core/target/debug/libtrading_core.a
    ${CORE_FOUNDATION}
    ${SECURITY}
    ${SYSTEM_CONFIGURATION}
    ${ZeroMQ_LIBRARIES}
)
