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
//! rlg = "0.0.8"
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
//! use rlg::{LogLevel, Log, LogFormat};
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
//! use rlg::{Config, LogLevel, Log, LogFormat};
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
//! use rlg::{LogLevel, Log, LogFormat};
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

use tokio::io::{self, AsyncWriteExt};
use std::{
    env,
    fmt::{self, Write as FmtWrite},
    fs::OpenOptions,
    io::{stdout, Write}
};
use vrd::Random;
use dtt::DateTime;


/// Configuration struct for logging system.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Config {
    /// Path and name of the log file.
    pub log_file_path: String,
}

impl Config {
    /// Loads configuration from environment variables or defaults.
    pub fn load() -> Config {
        let log_file_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "RLG.log".into());
        Config { log_file_path }
    }
}

/// The `macros` module contains functions for generating macros.
pub mod macros;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// An enumeration of the different log formats that can be used.
pub enum LogFormat {
    /// The log format is set to CLF.
    CLF,
    /// The log format is set to JSON.
    JSON,
    /// The log format is set to CEF.
    CEF,
    /// The log format is set to ELF.
    ELF,
    /// The log format is set to W3C.
    W3C,
    /// The log format is set to GELF.
    GELF,
    /// The log format is set to Apache Access Log.
    ApacheAccessLog,
    /// The log format is set to Logstash.
    Logstash,
    /// The log format is set to Log4j XML.
    Log4jXML,
    /// The log format is set to NDJSON (Newline Delimited JSON).
    NDJSON,
}

impl fmt::Display for LogFormat {
    /// Implements [`LogFormat`] to display the log format as a string.
    /// It allows the LogFormat enumeration to be used with the write!
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::CEF => write!(f, "CEF"),
            LogFormat::CLF => write!(f, "CLF"),
            LogFormat::ELF => write!(f, "ELF"),
            LogFormat::GELF => write!(f, "GELF"),
            LogFormat::JSON => write!(f, "JSON"),
            LogFormat::W3C => write!(f, "W3C"),
            LogFormat::ApacheAccessLog => write!(f, "Apache Access Log"),
            LogFormat::Logstash => write!(f, "Logstash"),
            LogFormat::Log4jXML => write!(f, "Log4j XML"),
            LogFormat::NDJSON => write!(f, "NDJSON"),
        }
    }
}

/// Implements [`Log`] to log a message to the console with a simple,
/// readable output format.
///
/// # Arguments
///
/// * `session_id` - A string slice that holds a session ID. The session
///    ID is a unique identifier for the current session. A random GUID
///    (Globally Unique Identifier) is generated by default.
/// * `time` - A string slice that holds the timestamp in ISO 8601
///    format.
/// * `level` - A string slice that holds the level (INFO, WARN, ERROR,
///     etc.).
/// * `component` - A string slice that holds the component name.
/// * `description` - A string slice that holds the description of the
///    log message.
///

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// An enumeration of the different levels that a log message can have.
/// Each variant of the enumeration represents a different level of
/// importance.
///
/// # Arguments
///
/// * `ALL` - The log level is set to all.
/// * `DEBUG` - The log level is set to debug.
/// * `DISABLED` - The log level is set to disabled.
/// * `ERROR` - The log level is set to error.
/// * `FATAL` - The log level is set to fatal.
/// * `INFO` - The log level is set to info.
/// * `NONE` - The log level is set to none.
/// * `TRACE` - The log level is set to trace.
/// * `VERBOSE` - The log level is set to verbose.
/// * `WARNING` - The log level is set to warning.
///
pub enum LogLevel {
    /// The log level is set to all.
    ALL,
    /// The log level is set to debug.
    DEBUG,
    /// The log level is set to disabled.
    DISABLED,
    /// The log level is set to error.
    ERROR,
    /// The log level is set to fatal.
    FATAL,
    /// The log level is set to info.
    INFO,
    /// The log level is set to none.
    NONE,
    /// The log level is set to trace.
    TRACE,
    /// The log level is set to verbose.
    VERBOSE,
    /// The log level is set to warning.
    WARNING,
}

/// Implements [`LogLevel`] to display the log level as a string. It
/// allows the LogLevel enumeration to be used with the write! and
/// print! macros. It provides a human-readable string representation of
/// the variant, that will be used when displaying the log message.
impl fmt::Display for LogLevel {
    /// Implements [`LogLevel`] to display the log level as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::ALL => write!(f, "ALL"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::DISABLED => write!(f, "DISABLED"),
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::FATAL => write!(f, "FATAL"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::NONE => write!(f, "NONE"),
            LogLevel::TRACE => write!(f, "TRACE"),
            LogLevel::VERBOSE => write!(f, "VERBOSE"),
            LogLevel::WARNING => write!(f, "WARNING"),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// The `Log` struct provides an easy way to log a message to the
/// console. It contains a set of defined fields to create a simple,
/// log message with a readable output format.
///
/// # Arguments
///
/// * `session_id` - A string slice that holds a session ID. The session
///    ID is a unique identifier for the current session. A random GUID
///    (Globally Unique Identifier) is generated by default.
/// * `time` - A string slice that holds the timestamp in ISO 8601
///    format.
/// * `level` - A string slice that holds the level (INFO, WARN, ERROR,
///     etc.).
/// * `component` - A string slice that holds the component name.
/// * `description` - A string slice that holds the description of the
///    log message.
/// * `format` - A string slice that holds the log format.
///
///
pub struct Log {
    /// A string that holds a session ID. The session ID is a unique
    /// identifier for the current session. A random GUID (Globally
    /// Unique Identifier) is generated by default.
    pub session_id: String,
    /// A string that holds the timestamp in ISO 8601 format.
    pub time: String,
    /// A string that holds the level (INFO, WARN, ERROR, etc.).
    pub level: LogLevel,
    /// A string that holds the component name.
    pub component: String,
    /// A string that holds the description of the log message.
    pub description: String,
    /// A string that holds the log format.
    pub format: LogFormat,
}

/// This implementation allows the Log struct to be created with default
/// values. It creates a new instance of the Log struct with empty
/// strings for the session_id, time, component and description fields,
///  and LogLevel::INFO for level field. This is useful when creating a
/// new instance of the Log struct. It allows the struct to be created
/// with default values, and then the fields can be set to the desired
/// values.
impl Default for Log {
    /// This implementation allows the Log struct to be created with
    /// default values.
    fn default() -> Log {
        Log {
            session_id: String::default(),
            time: String::default(),
            level: LogLevel::INFO,
            component: String::default(),
            description: String::default(),
            format: LogFormat::CLF,
        }
    }
}

impl Log {
    /// Logs a message asynchronously using a pre-allocated buffer to reduce memory allocation.
    /// The message is formatted according to the specified log format and then written to a file.
    /// Additionally, the message is printed to the standard output and the output buffer is flushed.
    ///
    /// # Errors
    ///
    /// Returns an `io::Result<()>` indicating the outcome of the logging operation.
    /// An error is returned if there's an issue with string formatting or IO operations (file writing or flushing stdout).
    ///
    pub async fn log(&self) -> io::Result<()> {
        let mut log_message = String::with_capacity(256);

        let write_result = match self.format {
            LogFormat::CLF => write!(
                log_message,
                "SessionID={} Timestamp={} Description={} Level={} Component={} Format=CLF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::JSON => write!(
                log_message,
                "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\"}} Format=JSON",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::CEF => write!(
                log_message,
                "CEF:0|{}|{}|{}|{}|{}|CEF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ELF => write!(
                log_message,
                "ELF:0|{}|{}|{}|{}|{}|ELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::W3C => write!(
                log_message,
                "W3C:0|{}|{}|{}|{}|{}|W3C",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::GELF => write!(
                log_message,
                "GELF:0|{}|{}|{}|{}|{}|GELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ApacheAccessLog => write!(
                log_message,
                "{} - - [{}] \"{}\" {} {}",
                hostname::get().unwrap().to_string_lossy(),
                self.time,
                self.description,
                self.level,
                self.component
            ),
            LogFormat::Logstash => write!(
                log_message,
                "{{\"@timestamp\":\"{}\",\"level\":\"{}\",\"component\":\"{}\",\"message\":\"{}\"}}",
                self.time,
                self.level,
                self.component,
                self.description
            ),
            LogFormat::Log4jXML => write!(
                log_message,
                "<log4j:event logger=\"{}\" timestamp=\"{}\" level=\"{}\" thread=\"{}\"><log4j:message>{}</log4j:message></log4j:event>",
                self.component,
                self.time,
                self.level,
                self.session_id,
                self.description
            ),
            LogFormat::NDJSON => write!(
                log_message,
                "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"component\":\"{}\",\"message\":\"{}\"}}",
                self.time,
                self.level,
                self.component,
                self.description
            ),
        };

        // Converting std::fmt::Error to std::io::Error
        if let Err(e) = write_result {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }

        // Writing to a file asynchronously
        let mut file = tokio::fs::File::create("RLG.log").await?;
        file.write_all(log_message.as_bytes()).await?;

        // Printing to stdout and flushing
        println!("{log_message}");
        stdout().flush()?;

        Ok(())
    }

    /// Creates a new log entry with provided details.
    ///
    /// # Parameters
    ///
    /// - `session_id`: A unique identifier for the session.
    /// - `time`: The timestamp in ISO 8601 format.
    /// - `level`: The logging level.
    /// - `component`: The component generating the log.
    /// - `description`: The log message.
    /// - `format`: The format for the log message.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Log`.
    pub fn new(session_id: &str, time: &str, level: &LogLevel, component: &str, description: &str, format: &LogFormat) -> Self {
        Self {
            session_id: session_id.to_string(),
            time: time.to_string(),
            level: level.clone(),
            component: component.to_string(),
            description: description.to_string(),
            format: format.clone(),
        }
    }
    /// Writes a log entry to the log file using the provided details.
    ///
    /// # Parameters
    ///
    /// - `log_level`: The severity level of the log.
    /// - `process`: The process name generating the log.
    /// - `message`: The log message.
    /// - `log_format`: The format of the log message.
    ///
    /// # Returns
    ///
    /// A `std::io::Result<()>` indicating the success or failure of writing the log entry.
    pub fn write_log_entry(log_level: LogLevel, process: &str, message: &str, log_format: LogFormat) -> io::Result<()> {
        // Configuration is loaded at the beginning of the function
        let config = Config::load();


        let date = DateTime::new();
        let iso = date.iso_8601;
        let uuid = Random::default().int(0, 1_000_000_000).to_string();


        let mut log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(config.log_file_path)?; // Use the configured path

        // Construct a new log entry and write it to the log file
        let log_entry = Log::new(
            &uuid, &iso, &log_level,
            process, message, &log_format,
        );

        writeln!(log_file, "{}", log_entry)?;

        Ok(())
    }
}

impl fmt::Display for Log {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => {
                write!(
                    f,
                    "SessionID={} Timestamp={} Description={} Level={} Component={}",
                    self.session_id, self.time, self.description, self.level, self.component
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::JSON => {
                write!(
                f,
                "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\",\"Format\":\"JSON\"}}",
                self.session_id, self.time, self.level, self.component, self.description)
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::CEF => {
                write!(
                    f,
                    "CEF:0|{}|{}|{}|{}|{}|CEF",
                    self.session_id, self.time, self.level, self.component, self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::ELF => {
                write!(
                    f,
                    "ELF:0|{}|{}|{}|{}|{}|ELF",
                    self.session_id, self.time, self.level, self.component, self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::W3C => {
                write!(
                    f,
                    "W3C:0|{}|{}|{}|{}|{}|W3C",
                    self.session_id, self.time, self.level, self.component, self.description
                )
                // self.session_id, self.time, self.level, self.component, self.description)
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::GELF => {
                write!(
                    f,
                    r#"{{
                            "version": "1.1",
                            "host": "{}",
                            "short_message": "{}",
                            "level": "{:?}",
                            "timestamp": "{}",
                            "component": "{}",
                            "session_id": "{}"
                        }}"#,
                    self.component,
                    self.description,
                    self.level,
                    self.time,
                    self.component,
                    self.session_id
                )
                .expect("Unable to write log message");
                Ok(())
            },
            LogFormat::ApacheAccessLog => {
                write!(
                    f,
                    "{} - - [{}] \"{}\" {} {}",
                    hostname::get().unwrap().to_string_lossy(),
                    self.time,
                    self.description,
                    self.level,
                    self.component
                )
                .expect("Unable to write log message");
                Ok(())
            },
            LogFormat::Logstash => {
                write!(
                    f,
                    r#"{{
                            "@timestamp": "{}",
                            "level": "{}",
                            "component": "{}",
                            "message": "{}"
                        }}"#,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            },
            LogFormat::Log4jXML => {
                write!(
                    f,
                    r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
                    self.component,
                    self.time,
                    self.level,
                    self.session_id,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            },
            LogFormat::NDJSON => {
                write!(
                    f,
                    r#"{{
                            "timestamp": "{}",
                            "level": "{}",
                            "component": "{}",
                            "message": "{}"
                        }}"#,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
        }
    }
}
