// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Import necessary traits and modules.
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};

/// An enumeration of the different levels that a log message can have.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
    /// The log level is set to warning but indicates a potential problem.
    WARN,
    /// The log level is set to critical.
    CRITICAL,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::INFO
    }
}

impl fmt::Display for LogLevel {
    /// Implements [`LogLevel`] to display the log level as a string.
    /// It allows the LogLevel enumeration to be used with the write! and
    /// print! macros.
    /// It provides a human-readable string representation of the variant,
    /// that will be used when displaying the log message.
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
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::CRITICAL => write!(f, "CRITICAL"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "all" => Ok(LogLevel::ALL),
            "debug" => Ok(LogLevel::DEBUG),
            "disabled" => Ok(LogLevel::DISABLED),
            "error" => Ok(LogLevel::ERROR),
            "fatal" => Ok(LogLevel::FATAL),
            "info" => Ok(LogLevel::INFO),
            "none" => Ok(LogLevel::NONE),
            "trace" => Ok(LogLevel::TRACE),
            "verbose" => Ok(LogLevel::VERBOSE),
            "warning" => Ok(LogLevel::WARNING),
            "warn" => Ok(LogLevel::WARN),
            "critical" => Ok(LogLevel::CRITICAL),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

impl TryFrom<&str> for LogLevel {
    type Error = LogLevel;

    /// Attempts to convert a string slice into a LogLevel enum variant.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse().map_err(|_| LogLevel::INFO)
    }
}

impl TryFrom<String> for LogLevel {
    type Error = LogLevel;

    /// Attempts to convert a String into a LogLevel enum variant.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}
