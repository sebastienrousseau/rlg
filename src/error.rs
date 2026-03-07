// error.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::config::ConfigError;
#[cfg(feature = "miette")]
use miette::Diagnostic;
use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
/// Errors that can occur during the logging process
pub enum RlgError {
    #[error("I/O error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::io_error),
            help("Ensure the log directory exists and is writable.")
        )
    )]
    /// I/O error
    IoError(#[from] io::Error),

    #[error("Configuration error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::config_error),
            help(
                "Check your configuration file or environment variables."
            )
        )
    )]
    /// Configuration error
    ConfigError(#[from] ConfigError),

    #[error("Log format parse error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::format_parse_error),
            help(
                "Ensure the format string matches supported variants (JSON, OTLP, MCP, etc.)."
            )
        )
    )]
    /// Log format parse error
    FormatParseError(String),

    #[error("Log level parse error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::level_parse_error),
            help(
                "Supported levels: ALL, TRACE, DEBUG, INFO, WARN, ERROR, FATAL."
            )
        )
    )]
    /// Log level parse error
    LevelParseError(String),

    #[error("Unsupported log format: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::unsupported_format),
            help(
                "Visit docs.rs/rlg for a list of supported industry formats."
            )
        )
    )]
    /// Unsupported log format
    UnsupportedFormat(String),

    #[error("Log formatting error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::formatting_error),
            help(
                "This may happen if attributes contain non-serializable data."
            )
        )
    )]
    /// Log formatting error
    FormattingError(String),

    #[error("Log rotation error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::rotation_error),
            help(
                "Ensure RLG has permission to rename or delete old log files."
            )
        )
    )]
    /// Log rotation error
    RotationError(String),

    #[error("Network error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::network_error),
            help(
                "Check your network connection or the OTLP collector endpoint."
            )
        )
    )]
    /// Network error
    NetworkError(String),

    #[error("DateTime parse error: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::datetime_parse_error),
            help("RLG expects RFC 3339 / ISO 8601 timestamps.")
        )
    )]
    /// `DateTime` parse error
    DateTimeParseError(String),

    #[error("{0}")]
    #[cfg_attr(feature = "miette", diagnostic(code(rlg::custom_error)))]
    /// Custom error
    Custom(String),

    #[error("Native OS sink failure: {0}")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(rlg::native_sink_failure),
            help(
                "Check if systemd-journald is running (Linux) or if 'com.rlg.logger' subsystem is registered (macOS). Ensure RLG_FALLBACK_STDOUT is set if you want to bypass native hooks."
            )
        )
    )]
    /// Native OS sink failure
    NativeSinkError(String),
}

impl From<crate::commons::error::CommonError> for RlgError {
    fn from(err: crate::commons::error::CommonError) -> Self {
        Self::Custom(err.to_string())
    }
}

impl RlgError {
    /// Create a new custom error with a given message.
    #[must_use]
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

/// Type alias for a Result with `RlgError` as the error type.
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
    fn test_common_error_conversion() {
        let common_err =
            crate::commons::error::CommonError::custom("test");
        let rlg_err: RlgError = common_err.into();
        assert!(matches!(rlg_err, RlgError::Custom(_)));
        assert!(rlg_err.to_string().contains("test"));
    }

    #[test]
    fn test_config_error_conversion() {
        let config_err =
            ConfigError::ValidationError("Test error".to_string());
        let rlg_err: RlgError = config_err.into();
        assert!(matches!(rlg_err, RlgError::ConfigError(_)));
    }
}
