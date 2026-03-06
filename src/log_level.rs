// log_level.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Import necessary traits and modules.
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error::Error, fmt, str::FromStr};

/// Custom error type for `LogLevel` parsing with context.
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
    #[must_use]
    pub fn new(invalid_value: &str) -> Self {
        Self {
            invalid_value: invalid_value.to_string(),
        }
    }
}

impl fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid log level: {0}", self.invalid_value)
    }
}

impl Error for ParseLogLevelError {}

/// An enumeration of the different levels that a log message can have, ordered by severity.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
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
    #[default]
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
    #[must_use]
    pub const fn includes(self, other: Self) -> bool {
        match self {
            Self::ALL => true,   // ALL includes everything
            Self::NONE => false, // NONE includes nothing
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
    #[must_use]
    pub const fn to_numeric(self) -> u8 {
        match self {
            Self::ALL => 0,
            Self::NONE => 1,
            Self::DISABLED => 2,
            Self::DEBUG => 3,
            Self::TRACE => 4,
            Self::VERBOSE => 5,
            Self::INFO => 6,
            Self::WARN => 7,
            Self::ERROR => 8,
            Self::FATAL => 9,
            Self::CRITICAL => 10,
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
    #[must_use]
    pub const fn from_numeric(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::ALL),
            1 => Some(Self::NONE),
            2 => Some(Self::DISABLED),
            3 => Some(Self::DEBUG),
            4 => Some(Self::TRACE),
            5 => Some(Self::VERBOSE),
            6 => Some(Self::INFO),
            7 => Some(Self::WARN),
            8 => Some(Self::ERROR),
            9 => Some(Self::FATAL),
            10 => Some(Self::CRITICAL),
            _ => None,
        }
    }
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ALL" => Ok(Self::ALL),
            "NONE" => Ok(Self::NONE),
            "DISABLED" => Ok(Self::DISABLED),
            "DEBUG" => Ok(Self::DEBUG),
            "TRACE" => Ok(Self::TRACE),
            "VERBOSE" => Ok(Self::VERBOSE),
            "INFO" => Ok(Self::INFO),
            "WARN" => Ok(Self::WARN),
            "ERROR" => Ok(Self::ERROR),
            "FATAL" => Ok(Self::FATAL),
            "CRITICAL" => Ok(Self::CRITICAL),
            _ => Err(ParseLogLevelError::new(s)),
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = ParseLogLevelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self {
            Self::ALL => "ALL",
            Self::NONE => "NONE",
            Self::DISABLED => "DISABLED",
            Self::DEBUG => "DEBUG",
            Self::TRACE => "TRACE",
            Self::VERBOSE => "VERBOSE",
            Self::INFO => "INFO",
            Self::WARN => "WARN",
            Self::ERROR => "ERROR",
            Self::FATAL => "FATAL",
            Self::CRITICAL => "CRITICAL",
        };
        write!(f, "{level_str}")
    }
}
