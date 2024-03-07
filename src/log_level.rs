// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

/// An enumeration of the different levels that a log message can have.
/// Each variant of the enumeration represents a different level of importance.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl fmt::Display for LogLevel {
    /// Implements [`LogLevel`] to display the log level as a string.
    /// It allows the LogLevel enumeration to be used with the write! and print! macros.
    /// It provides a human-readable string representation of the variant, that will be used when displaying the log message.
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