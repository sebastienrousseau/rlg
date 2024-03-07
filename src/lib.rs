// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
//!
//! # RustLogs (RLG)
//!
//! RustLogs (RLG) is a library that implements application-level logging in a simple, readable output format. 
//! The library provides logging APIs and various helper macros that simplify many common logging tasks.
//!
//! [![Rust](https://kura.pro/rlg/images/titles/title-rlg.svg)](https://rustlogs.com/)
//!
//! ## Overview
//!
//! RustLogs (RLG) is a library that implements application-level
//! logging in a simple, readable output format. The library provides
//! logging APIs and various helper macros that simplify many common
//! logging tasks.
//!
//! ## Features
//!
//! - Supports many log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`,
//!   `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE`, and `WARNING`.
//! - Provides structured log formats that are easy to parse and filter.
//! - Compatible with multiple output formats including:
//!    - Common Event Format (CEF)
//!    - Extended Log Format (ELF)
//!    - Graylog Extended Log Format (GELF)
//!    - JavaScript Object Notation (JSON)
//!    - NCSA Common Log Format (CLF)
//!    - W3C Extended Log File Format (W3C)
//!    - Syslog Format
//!    - Apache Access Log Format
//!    - Logstash Format
//!    - Log4j XML Format
//!    - NDJSON (Newline Delimited JSON)
//!    - and many more.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rlg = "0.0.3"
//! ```
//!
//! ## Configuration
//!
//! By default, RustLogs (RLG) logs to a file named "RLG.log" in the current directory. You can customize the log file path by setting the `LOG_FILE_PATH` environment variable.
//!
//! ## Examples
//!
//! ### Basic Logging
//!
//! ```rust
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//!
//! // Create a new log entry
//! let log_entry = Log::new(
//!     "12345",
//!     "2023-01-01T12:00:00Z",
//!     &LogLevel::INFO,
//!     "MyComponent",
//!     "This is a sample log message",
//!     &LogFormat::JSON, // Choose from various formats like JSON, Syslog, NDJSON, etc.
//! );
//!
//! // Log the entry asynchronously
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     log_entry.log().await.unwrap();
//! });
//! ```
//!
//! ### Custom Log Configuration
//!
//! ```rust,no_run
//! use rlg::config::Config;
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//!
//! // Customize log file path
//! std::env::set_var("LOG_FILE_PATH", "/path/to/log/file.log");
//!
//! // Load custom configuration
//! let config = Config::load();
//!
//! // Create a new log entry with custom configuration
//! let log_entry = Log::new(
//!     "12345",
//!     "2023-01-01T12:00:00Z",
//!     &LogLevel::INFO,
//!     "MyComponent",
//!     "This is a sample log message",
//!     &LogFormat::ApacheAccessLog
//! );
//!
//! // Log the entry asynchronously
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     log_entry.log().await.unwrap();
//! });
//! ```
//! ## Error Handling
//!
//! Errors can occur during logging operations, such as file I/O errors or formatting errors. The `log()` method returns a `Result<(), io::Error>` that indicates the outcome of the logging operation. You should handle potential errors appropriately in your code.
//!
//! ```rust,no_run
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//!
//! // Create a new log entry
//! let log_entry = Log::new(
//!     "12345",
//!     "2023-01-01T12:00:00Z",
//!     &LogLevel::INFO,
//!     "MyComponent",
//!     "This is a sample log message",
//!     &LogFormat::NDJSON, // Using NDJSON format for this example
//! );
//!
//! // Log the entry asynchronously and handle potential errors
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     match log_entry.log().await {
//!         Ok(_) => println!("Log entry successfully written"),
//!         Err(err) => eprintln!("Error logging entry: {}", err),
//!     }
//! });
//! ```
#![cfg_attr(feature = "bench", feature(test))]
#![deny(dead_code)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![doc(
    html_favicon_url = "https://kura.pro/rlg/images/favicon.ico",
    html_logo_url = "https://kura.pro/rlg/images/logos/rlg.svg",
    html_root_url = "https://docs.rs/rlg"
)]
#![crate_name = "rlg"]
#![crate_type = "lib"]

use crate::log_format::LogFormat;
use crate::config::Config;
use crate::log_level::LogLevel;


/// The `config` module contains the configuration struct for the logging system.
pub mod config;

/// The `log` module contains the log struct and its implementation.
pub mod log;

/// The `log_format` module contains the log format enumeration and its implementation.
pub mod log_format;

/// The `log_level` module contains the log level enumeration and its implementation.
pub mod log_level;

/// The `macros` module contains functions for generating macros.
pub mod macros;

