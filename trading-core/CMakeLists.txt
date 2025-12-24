cmake_minimum_required(VERSION 3.12)
project(trading_core CXX)

# Detect build type, default to Debug if not specified
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

# Determine Cargo profile based on CMake build type
if(CMAKE_BUILD_TYPE STREQUAL "Release" OR CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
    set(CARGO_PROFILE release)
    set(CARGO_ARGS --release)
else()
    set(CARGO_PROFILE debug)
    set(CARGO_ARGS "")
endif()

# Create a custom target to run cargo build
# This ensures that the Rust library and CXX headers are generated before C++ compilation attempts to use them
add_custom_target(cargo_build
    COMMAND cargo build ${CARGO_ARGS}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library with Cargo"
    VERBATIM
)

# Define the static library location
if(WIN32)
    set(LIB_NAME trading_core.lib)
else()
    set(LIB_NAME libtrading_core.a)
endif()

# Define an imported static library target for the Rust output
add_library(trading_core_lib STATIC IMPORTED)
set_target_properties(trading_core_lib PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/target/${CARGO_PROFILE}/${LIB_NAME}"
)

# Create an Interface library that consumers will link against
# This wraps the static lib and adds include directories and system dependencies
add_library(trading_core INTERFACE)
target_link_libraries(trading_core INTERFACE trading_core_lib)

# Ensure Cargo runs before any target linking trading_core
add_dependencies(trading_core cargo_build)

# Expose the include directories (Handwritten headers + CXX generated headers)
target_include_directories(trading_core INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/target/cxxbridge"
)

# Link necessary system libraries required by Rust std/crates
if(APPLE)
    target_link_libraries(trading_core INTERFACE "-framework CoreFoundation" "-framework Security" "-framework System")
elseif(UNIX AND NOT APPLE)
    target_link_libraries(trading_core INTERFACE pthread dl)
endif()
