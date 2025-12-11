cmake_minimum_required(VERSION 3.15)
project(multiplexer CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Dependencies
find_package(PkgConfig REQUIRED)
pkg_check_modules(ZeroMQ REQUIRED libzmq)

# Include Directories
include_directories(src ${ZeroMQ_INCLUDE_DIRS})
link_directories(${ZeroMQ_LIBRARY_DIRS})

# Multiplexer Executable
add_executable(multiplexer
    src/main.cpp
    # Add logic files here
)

# Link Libraries
target_link_libraries(multiplexer
    PRIVATE
    ${ZeroMQ_LIBRARIES}
)
