// log_level.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Import necessary traits and modules.
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error::Error, fmt, str::FromStr};

/// Custom error type for LogLevel parsing with context.
#[derive(Debug, Clone)]
pub struct ParseLogLevelError {
    /// The invalid log level value.
    pub invalid_value: String,
}

impl ParseLogLevelError {
    /// Creates a new instance of `ParseLogLevelError` with the given invalid log level value.
    ///
    /// # Arguments
    ///
    /// * `invalid_value` - A reference to a string representing the invalid log level value.
    ///
    /// # Returns
    ///
    /// A new instance of `ParseLogLevelError` containing the provided invalid log level value.
    pub fn new(invalid_value: &str) -> Self {
        ParseLogLevelError {
            invalid_value: invalid_value.to_string(),
        }
    }
}

impl fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid log level: {}", self.invalid_value)
    }
}

impl Error for ParseLogLevelError {}

/// An enumeration of the different levels that a log message can have, ordered by severity.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub enum LogLevel {
    /// `ALL`: The log level includes all levels.
    ALL,
    /// `NONE`: No logging.
    NONE,
    /// `DISABLED`: Logging is disabled.
    DISABLED,
    /// `DEBUG`: Debugging information, typically useful for developers.
    DEBUG,
    /// `TRACE`: Finer-grained informational events than `DEBUG`.
    TRACE,
    /// `VERBOSE`: Detailed logging, often more detailed than `INFO`.
    VERBOSE,
    /// `INFO`: Informational messages that highlight the progress of the application.
    INFO,
    /// `WARN`: Potentially harmful situations.
    WARN,
    /// `ERROR`: Error events that might still allow the application to continue running.
    ERROR,
    /// `FATAL`: Very severe error events that will presumably lead the application to abort.
    FATAL,
    /// `CRITICAL`: Critical conditions, often requiring immediate attention.
    CRITICAL,
}

impl LogLevel {
    /// Checks if the current log level includes another log level.
    ///
    /// # Arguments
    ///
    /// * `other` - The log level to compare with.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::log_level::LogLevel;
    /// assert!(LogLevel::ERROR.includes(LogLevel::DEBUG)); // ERROR includes DEBUG
    /// assert!(!LogLevel::DEBUG.includes(LogLevel::WARN)); // DEBUG does not include WARN
    /// assert!(LogLevel::WARN.includes(LogLevel::DEBUG)); // WARN includes DEBUG
    /// ```
    pub fn includes(self, other: LogLevel) -> bool {
        match self {
            LogLevel::ALL => true,   // ALL includes everything
            LogLevel::NONE => false, // NONE includes nothing
            _ => self.to_numeric() >= other.to_numeric(), // Default behavior for other levels
        }
    }

    /// Converts the log level to its corresponding numeric value, similar to syslog severity levels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::log_level::LogLevel;
    /// assert_eq!(LogLevel::ERROR.to_numeric(), 8);
    /// assert_eq!(LogLevel::DEBUG.to_numeric(), 3);
    /// ```
    pub fn to_numeric(self) -> u8 {
        match self {
            LogLevel::ALL => 0,
            LogLevel::NONE => 1,
            LogLevel::DISABLED => 2,
            LogLevel::DEBUG => 3,
            LogLevel::TRACE => 4,
            LogLevel::VERBOSE => 5,
            LogLevel::INFO => 6,
            LogLevel::WARN => 7,
            LogLevel::ERROR => 8,
            LogLevel::FATAL => 9,
            LogLevel::CRITICAL => 10,
        }
    }

    /// Creates a `LogLevel` from a numeric value, similar to syslog severity levels.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::log_level::LogLevel;
    /// assert_eq!(LogLevel::from_numeric(8), Some(LogLevel::ERROR));
    /// assert_eq!(LogLevel::from_numeric(5), Some(LogLevel::VERBOSE));
    /// ```
    pub fn from_numeric(value: u8) -> Option<Self> {
        match value {
            0 => Some(LogLevel::ALL),
            1 => Some(LogLevel::NONE),
            2 => Some(LogLevel::DISABLED),
            3 => Some(LogLevel::DEBUG),
            4 => Some(LogLevel::TRACE),
            5 => Some(LogLevel::VERBOSE),
            6 => Some(LogLevel::INFO),
            7 => Some(LogLevel::WARN),
            8 => Some(LogLevel::ERROR),
            9 => Some(LogLevel::FATAL),
            10 => Some(LogLevel::CRITICAL),
            _ => None,
        }
    }
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ALL" => Ok(LogLevel::ALL),
            "NONE" => Ok(LogLevel::NONE),
            "DISABLED" => Ok(LogLevel::DISABLED),
            "DEBUG" => Ok(LogLevel::DEBUG),
            "TRACE" => Ok(LogLevel::TRACE),
            "VERBOSE" => Ok(LogLevel::VERBOSE),
            "INFO" => Ok(LogLevel::INFO),
            "WARN" => Ok(LogLevel::WARN),
            "ERROR" => Ok(LogLevel::ERROR),
            "FATAL" => Ok(LogLevel::FATAL),
            "CRITICAL" => Ok(LogLevel::CRITICAL),
            _ => Err(ParseLogLevelError::new(s)),
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = ParseLogLevelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        LogLevel::from_str(&value)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self {
            LogLevel::ALL => "ALL",
            LogLevel::NONE => "NONE",
            LogLevel::DISABLED => "DISABLED",
            LogLevel::DEBUG => "DEBUG",
            LogLevel::TRACE => "TRACE",
            LogLevel::VERBOSE => "VERBOSE",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::FATAL => "FATAL",
            LogLevel::CRITICAL => "CRITICAL",
        };
        write!(f, "{}", level_str)
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::INFO
    }
}
