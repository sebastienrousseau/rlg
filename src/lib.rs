// lib.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! # RustLogs (RLG)
//!
//! RustLogs (RLG) is a robust and flexible logging library for Rust applications.
//! It provides a simple, readable output format and offers various features to
//! enhance your application's logging capabilities.
//!
//! ## Features
//!
//! - Multiple log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`, `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE`, and `WARN`.
//! - Structured log formats for easy parsing and filtering.
//! - Support for multiple output formats including:
//!   - Common Event Format (CEF)
//!   - Extended Log Format (ELF)
//!   - Graylog Extended Log Format (GELF)
//!   - JavaScript Object Notation (JSON)
//!   - NCSA Common Log Format (CLF)
//!   - W3C Extended Log File Format (W3C)
//!   - Syslog Format
//!   - Apache Access Log Format
//!   - Logstash Format
//!   - Log4j XML Format
//!   - NDJSON (Newline Delimited JSON)
//! - Configurable logging destinations (file, stdout, network).
//! - Log rotation support.
//! - Asynchronous logging for improved performance.
//!
//! ## Optional Features
//!
//! The following optional features can be enabled via feature flags in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rlg = { version = "0.0.6", features = ["syslog", "logstash", "log4j"] }
//! ```
//!
//! - `syslog`: Enables support for Syslog format.
//! - `logstash`: Enables support for Logstash format.
//! - `log4j`: Enables support for Log4j XML format.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rlg = "0.0.6"
//! ```
//!
//! ### Basic Logging
//!
//! ```rust
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//! use std::error::Error;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Set environment variables for configuration
//!     env::set_var("LOG_FILE_PATH", "RLG.log");
//!     env::set_var("LOG_LEVEL", "DEBUG");
//!     env::set_var("LOG_ROTATION", "size:10485760"); // 10 MB
//!     env::set_var("LOG_FORMAT", "%time - %level - %message");
//!     env::set_var("LOG_DESTINATIONS", "file,stdout");
//!
//!     // Write a log entry
//!     Log::write_log_entry(
//!         LogLevel::INFO,
//!         "MyComponent",
//!         "This is a sample log message",
//!         LogFormat::JSON
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Log Configuration
//!
//! ```rust
//! use rlg::config::{Config, LogRotation, LoggingDestination};
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//! use std::error::Error;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Set environment variables for configuration
//!     env::set_var("LOG_FILE_PATH", "RLG.log");
//!     env::set_var("LOG_LEVEL", "DEBUG");
//!     env::set_var("LOG_ROTATION", "size:10485760"); // 10 MB
//!     env::set_var("LOG_FORMAT", "%time - %level - %message");
//!     env::set_var("LOG_DESTINATIONS", "file,stdout");
//!
//!     // Write a log entry
//!     Log::write_log_entry(
//!         LogLevel::INFO,
//!         "MyComponent",
//!         "This is a sample log message with custom configuration",
//!         LogFormat::JSON
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For more detailed information about each module and its functionalities,
//! please refer to the respective module documentation.

#![warn(missing_docs)]
#![doc(
    html_favicon_url = "https://kura.pro/rlg/images/favicon.ico",
    html_logo_url = "https://kura.pro/rlg/images/logos/rlg.svg",
    html_root_url = "https://docs.rs/rlg"
)]

/// The current version of the RustLogs crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use config::Config;
pub use log_format::LogFormat;
pub use log_level::LogLevel;

/// Configuration module for RustLogs.
pub mod config;

/// Core logging functionality.
pub mod log;

/// Log format definitions and implementations.
pub mod log_format;

/// Log level definitions and implementations.
pub mod log_level;

/// Macros for convenient logging.
pub mod macros;

// Re-export commonly used items
pub use config::{LogRotation, LoggingDestination};
