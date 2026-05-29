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
    /// `TRACE`: Finer-grained informational events than `DEBUG`.
    TRACE,
    /// `DEBUG`: Debugging information, typically useful for developers.
    DEBUG,
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

macro_rules! define_log_levels {
    ( $( $variant:ident, $num:expr, $upper:expr, $lower:expr );+ $(;)? ) => {
        impl LogLevel {
            /// Converts the log level to its corresponding numeric value, similar to syslog severity levels.
            ///
            /// # Examples
            ///
            /// ```
            /// use rlg::log_level::LogLevel;
            /// assert_eq!(LogLevel::ERROR.to_numeric(), 8);
            /// assert_eq!(LogLevel::DEBUG.to_numeric(), 4);
            /// ```
            #[must_use]
            pub const fn to_numeric(self) -> u8 {
                match self { $( Self::$variant => $num, )+ }
            }

            /// Returns the uppercase string representation of the log level.
            #[must_use]
            pub const fn as_str(&self) -> &'static str {
                match self { $( Self::$variant => $upper, )+ }
            }

            /// Returns the lowercase string representation of the log level.
            #[must_use]
            pub const fn as_str_lowercase(&self) -> &'static str {
                match self { $( Self::$variant => $lower, )+ }
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
                    $( $num => Some(Self::$variant), )+
                    _ => None,
                }
            }
        }

        impl FromStr for LogLevel {
            type Err = ParseLogLevelError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $( $upper => Ok(Self::$variant), )+
                    _ => Err(ParseLogLevelError::new(s)),
                }
            }
        }
    };
}

define_log_levels! {
    ALL, 0, "ALL", "all";
    NONE, 1, "NONE", "none";
    DISABLED, 2, "DISABLED", "disabled";
    TRACE, 3, "TRACE", "trace";
    DEBUG, 4, "DEBUG", "debug";
    VERBOSE, 5, "VERBOSE", "verbose";
    INFO, 6, "INFO", "info";
    WARN, 7, "WARN", "warn";
    ERROR, 8, "ERROR", "error";
    FATAL, 9, "FATAL", "fatal";
    CRITICAL, 10, "CRITICAL", "critical";
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
}

impl TryFrom<String> for LogLevel {
    type Error = ParseLogLevelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
