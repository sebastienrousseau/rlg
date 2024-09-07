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

#![warn(missing_docs)]
#![doc(
    html_favicon_url = "https://kura.pro/rlg/images/favicon.ico",
    html_logo_url = "https://kura.pro/rlg/images/logos/rlg.svg",
    html_root_url = "https://docs.rs/rlg"
)]

// Version information
/// The current version of the RustLogs crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export commonly used items
pub use config::Config;
pub use config::{LogRotation, LoggingDestination};
pub use log::Log;
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
#[macro_use]
pub mod macros;

/// Error handling module
pub mod error;
pub use error::{RlgError, RlgResult};

/// Utility functions module
pub mod utils;
pub use utils::{generate_timestamp, sanitize_log_message};
