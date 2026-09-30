// error.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::config::ConfigError;
use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error variants for the RLG logging pipeline.
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
    /// `DateTime` parse error
    DateTimeParseError(String),

    #[error("{0}")]
    /// Custom error
    Custom(String),

    #[error("Native OS sink failure: {0}")]
    /// Native OS sink failure
    NativeSinkError(String),
}

impl From<crate::commons::error::CommonError> for RlgError {
    fn from(err: crate::commons::error::CommonError) -> Self {
        Self::Custom(err.to_string())
    }
}

impl RlgError {
    /// Create a custom error with the given message.
    #[must_use]
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }

    /// A stable, machine-readable code for this error, such as
    /// `rlg::io_error`. Suitable for matching in tooling and for
    /// indexing error documentation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::error::RlgError;
    /// let err = RlgError::RotationError("disk full".into());
    /// assert_eq!(err.code(), "rlg::rotation_error");
    /// ```
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.diagnostic().0
    }

    /// A one-line hint on how to resolve this error, when there is
    /// one.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::error::RlgError;
    /// let err = RlgError::LevelParseError("LOUD".into());
    /// assert!(err.help().unwrap().contains("TRACE"));
    /// assert!(RlgError::custom("x").help().is_none());
    /// ```
    #[must_use]
    pub const fn help(&self) -> Option<&'static str> {
        self.diagnostic().1
    }

    /// Render the error as a multi-line report: the code, the
    /// message and, when there is one, the help line.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::error::RlgError;
    /// let err = RlgError::NetworkError("timeout".into());
    /// assert_eq!(
    ///     err.report().to_string(),
    ///     "error[rlg::network_error]: Network error: timeout\n  \
    ///      help: Check your network connection or the OTLP collector endpoint.",
    /// );
    /// ```
    #[must_use]
    pub const fn report(&self) -> Report<'_> {
        Report(self)
    }

    /// The code and help line of each variant. One exhaustive match,
    /// so a new variant cannot compile without a diagnostic.
    const fn diagnostic(&self) -> Diagnostic {
        match self {
            Self::IoError(_) => ("rlg::io_error", Some(HELP_IO)),
            Self::ConfigError(_) => {
                ("rlg::config_error", Some(HELP_CONFIG))
            }
            Self::FormatParseError(_) => {
                ("rlg::format_parse_error", Some(HELP_FORMAT_PARSE))
            }
            Self::LevelParseError(_) => {
                ("rlg::level_parse_error", Some(HELP_LEVEL_PARSE))
            }
            Self::UnsupportedFormat(_) => (
                "rlg::unsupported_format",
                Some(HELP_UNSUPPORTED_FORMAT),
            ),
            Self::FormattingError(_) => {
                ("rlg::formatting_error", Some(HELP_FORMATTING))
            }
            Self::RotationError(_) => {
                ("rlg::rotation_error", Some(HELP_ROTATION))
            }
            Self::NetworkError(_) => {
                ("rlg::network_error", Some(HELP_NETWORK))
            }
            Self::DateTimeParseError(_) => {
                ("rlg::datetime_parse_error", Some(HELP_DATETIME_PARSE))
            }
            Self::Custom(_) => ("rlg::custom_error", None),
            Self::NativeSinkError(_) => {
                ("rlg::native_sink_failure", Some(HELP_NATIVE_SINK))
            }
        }
    }
}

/// An error's stable code and optional help line.
type Diagnostic = (&'static str, Option<&'static str>);

const HELP_IO: &str =
    "Ensure the log directory exists and is writable.";
const HELP_CONFIG: &str =
    "Check your configuration file or environment variables.";
const HELP_FORMAT_PARSE: &str = "Ensure the format string matches supported variants (JSON, OTLP, MCP, etc.).";
const HELP_LEVEL_PARSE: &str =
    "Supported levels: ALL, TRACE, DEBUG, INFO, WARN, ERROR, FATAL.";
const HELP_UNSUPPORTED_FORMAT: &str =
    "Visit docs.rs/rlg for a list of supported industry formats.";
const HELP_FORMATTING: &str =
    "This may happen if attributes contain non-serializable data.";
const HELP_ROTATION: &str =
    "Ensure RLG has permission to rename or delete old log files.";
const HELP_NETWORK: &str =
    "Check your network connection or the OTLP collector endpoint.";
const HELP_DATETIME_PARSE: &str =
    "RLG expects RFC 3339 / ISO 8601 timestamps.";
const HELP_NATIVE_SINK: &str = "Check if systemd-journald is running (Linux) or if 'com.rlg.logger' subsystem is registered (macOS). Ensure RLG_FALLBACK_STDOUT is set if you want to bypass native hooks.";

/// A displayable diagnostic report for an [`RlgError`], returned by
/// [`RlgError::report`].
#[derive(Debug, Clone, Copy)]
pub struct Report<'a>(&'a RlgError);

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}]: {}", self.0.code(), self.0)?;
        if let Some(help) = self.0.help() {
            write!(f, "\n  help: {help}")?;
        }
        Ok(())
    }
}

/// Convenience alias: `Result<T, RlgError>`.
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

    #[test]
    fn test_io_error_variant() {
        let io_err =
            io::Error::new(io::ErrorKind::NotFound, "file missing");
        let rlg_err: RlgError = io_err.into();
        assert!(matches!(rlg_err, RlgError::IoError(_)));
        assert!(rlg_err.to_string().contains("file missing"));
    }

    #[test]
    fn test_format_parse_error_variant() {
        let err = RlgError::FormatParseError("bad format".into());
        assert_eq!(
            err.to_string(),
            "Log format parse error: bad format"
        );
    }

    #[test]
    fn test_level_parse_error_variant() {
        let err = RlgError::LevelParseError("bad level".into());
        assert_eq!(err.to_string(), "Log level parse error: bad level");
    }

    #[test]
    fn test_unsupported_format_variant() {
        let err = RlgError::UnsupportedFormat("XML".into());
        assert_eq!(err.to_string(), "Unsupported log format: XML");
    }

    #[test]
    fn test_formatting_error_variant() {
        let err = RlgError::FormattingError("template".into());
        assert_eq!(err.to_string(), "Log formatting error: template");
    }

    #[test]
    fn test_rotation_error_variant() {
        let err = RlgError::RotationError("disk full".into());
        assert_eq!(err.to_string(), "Log rotation error: disk full");
    }

    #[test]
    fn test_network_error_variant() {
        let err = RlgError::NetworkError("timeout".into());
        assert_eq!(err.to_string(), "Network error: timeout");
    }

    #[test]
    fn test_datetime_parse_error_variant() {
        let err = RlgError::DateTimeParseError("bad date".into());
        assert_eq!(err.to_string(), "DateTime parse error: bad date");
    }

    #[test]
    fn test_native_sink_error_variant() {
        let err = RlgError::NativeSinkError("journald down".into());
        assert_eq!(
            err.to_string(),
            "Native OS sink failure: journald down"
        );
    }

    #[test]
    fn test_error_debug_all_variants() {
        let variants: Vec<RlgError> = vec![
            RlgError::IoError(io::Error::other("test")),
            RlgError::ConfigError(ConfigError::ValidationError(
                "v".into(),
            )),
            RlgError::FormatParseError("f".into()),
            RlgError::LevelParseError("l".into()),
            RlgError::UnsupportedFormat("u".into()),
            RlgError::FormattingError("fm".into()),
            RlgError::RotationError("r".into()),
            RlgError::NetworkError("n".into()),
            RlgError::DateTimeParseError("d".into()),
            RlgError::Custom("c".into()),
            RlgError::NativeSinkError("ns".into()),
        ];
        for err in &variants {
            let dbg = format!("{err:?}");
            assert!(!dbg.is_empty());
        }
    }

    #[test]
    fn test_every_variant_has_a_distinct_code() {
        let variants: Vec<RlgError> = vec![
            RlgError::IoError(io::Error::other("test")),
            RlgError::ConfigError(ConfigError::ValidationError(
                "v".into(),
            )),
            RlgError::FormatParseError("f".into()),
            RlgError::LevelParseError("l".into()),
            RlgError::UnsupportedFormat("u".into()),
            RlgError::FormattingError("fm".into()),
            RlgError::RotationError("r".into()),
            RlgError::NetworkError("n".into()),
            RlgError::DateTimeParseError("d".into()),
            RlgError::Custom("c".into()),
            RlgError::NativeSinkError("ns".into()),
        ];
        let codes: std::collections::HashSet<_> =
            variants.iter().map(RlgError::code).collect();
        assert_eq!(codes.len(), variants.len());
        for err in &variants {
            assert!(err.code().starts_with("rlg::"));
            let report = err.report().to_string();
            assert!(
                report.starts_with(&format!("error[{}]: ", err.code()))
            );
            assert_eq!(
                report.contains("\n  help: "),
                err.help().is_some()
            );
        }
    }

    #[test]
    fn test_custom_error_has_no_help() {
        let err = RlgError::custom("boom");
        assert_eq!(err.help(), None);
        assert_eq!(
            err.report().to_string(),
            "error[rlg::custom_error]: boom"
        );
    }

    #[test]
    fn test_error_is_std_error() {
        let err = RlgError::NetworkError("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_rlg_result_ok() {
        let r: RlgResult<i32> = Ok(42);
        assert!(matches!(r, Ok(42)));
    }

    #[test]
    fn test_rlg_result_err() {
        let r: RlgResult<i32> = Err(RlgError::custom("fail"));
        assert!(r.is_err());
    }
}
