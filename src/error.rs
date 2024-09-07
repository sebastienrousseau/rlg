// error.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::config::ConfigError;
use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during the logging process
pub enum RlgError {
    #[error("I/O error: {0}")]
    /// I/O error
    IoError(#[from] io::Error),

    #[error("Configuration error: {0}")]
    /// Configuration error
    ConfigError(#[from] ConfigError),

    #[error("Log format parse error: {0}")]
    /// Log format parse error
    FormatParseError(String),

    #[error("Log level parse error: {0}")]
    /// Log level parse error
    LevelParseError(String),

    #[error("Unsupported log format: {0}")]
    /// Unsupported log format
    UnsupportedFormat(String),

    #[error("Log formatting error: {0}")]
    /// Log formatting error
    FormattingError(String),

    #[error("Log rotation error: {0}")]
    /// Log rotation error
    RotationError(String),

    #[error("Network error: {0}")]
    /// Network error
    NetworkError(String),

    #[error("DateTime parse error: {0}")]
    /// DateTime parse error
    DateTimeParseError(String),

    #[error("{0}")]
    /// Custom error
    Custom(String),
}

impl RlgError {
    /// Create a new custom error with a given message.
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        RlgError::Custom(msg.to_string())
    }
}

/// Type alias for a Result with RlgError as the error type.
pub type RlgResult<T> = Result<T, RlgError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err =
            RlgError::FormatParseError("Invalid format".to_string());
        assert_eq!(
            err.to_string(),
            "Log format parse error: Invalid format"
        );
    }

    #[test]
    fn test_custom_error() {
        let err = RlgError::custom("Custom error message");
        assert_eq!(err.to_string(), "Custom error message");
    }

    #[test]
    fn test_config_error_conversion() {
        let config_err =
            ConfigError::ValidationError("Test error".to_string());
        let rlg_err: RlgError = config_err.into();
        assert!(matches!(rlg_err, RlgError::ConfigError(_)));
    }
}
