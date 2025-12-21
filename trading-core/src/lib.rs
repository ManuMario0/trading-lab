//! # Trading Core Library
//!
//! A shared foundation for Rust and C++ microservices in the specialized trading platform.
//!
//! ## Modules
//! - `model`: Common data types (Order, Instrument) with identical serialization.
//! - `args`: Standardized argument parsing.
//! - `comms`: Generic ZMQ Exchange management.
//! - `fs`: Centralized file system paths and state persistence.
//! - `admin`: Dynamic parameter registry and HTTP Admin API.
//! - `ffi`: C-compatible bindings for C++ integration.

pub mod admin;
pub mod args;
pub mod comms;
pub mod ffi;
pub mod framework;
pub mod fs;
pub mod model;
