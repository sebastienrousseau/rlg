// log_format.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::{RlgError, RlgResult};
use crate::utils::sanitize_log_message;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

/// Compiled regular expressions for log format validation.
static CLF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r#"^(?P<host>\S+) (?P<ident>\S+) (?P<user>\S+) \[(?P<time>[^\]]+)\] "(?P<method>\S+) (?P<path>\S+) (?P<protocol>\S+)" (?P<status>\d{3}) (?P<size>\d+|-)$"#
).expect("Failed to compile CLF regex")
});

static CEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^CEF:\d+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|.*$",
    )
    .expect("Failed to compile CEF regex")
});

static W3C_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^#Fields:.*
.+$",
    )
    .expect("Failed to compile W3C regex")
});

/// `LogFormat` is an enum representing the different structured log formats supported by the `RLG` library.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub enum LogFormat {
    /// Common Log Format (CLF)
    CLF,
    /// JavaScript Object Notation (JSON)
    JSON,
    /// Common Event Format (CEF)
    CEF,
    /// Extended Log Format (ELF)
    ELF,
    /// W3C Extended Log Format (W3C)
    W3C,
    /// Graylog Extended Log Format (GELF)
    GELF,
    /// Apache Access Log Format
    ApacheAccessLog,
    /// Logstash Format
    Logstash,
    /// Log4j XML Format
    Log4jXML,
    /// Network Data JSON (NDJSON)
    NDJSON,
    /// Model Context Protocol (MCP) - AI Native
    MCP,
    /// OpenTelemetry Logging (OTLP) - AI Native
    OTLP,
    /// Logfmt (key=value)
    Logfmt,
    /// Elastic Common Schema (ECS)
    ECS,
}

macro_rules! define_log_format_strings {
    ( $( $variant:ident => $display:expr, [ $( $key:expr ),+ ] );+ $(;)? ) => {
        impl FromStr for LogFormat {
            type Err = RlgError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $(
                        $( $key )|+ => Ok(Self::$variant),
                    )+
                    _ => Err(RlgError::FormatParseError(format!(
                        "Unknown log format: {s}"
                    ))),
                }
            }
        }

        impl fmt::Display for LogFormat {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let s = match self {
                    $( Self::$variant => $display, )+
                };
                write!(f, "{s}")
            }
        }
    };
}

define_log_format_strings! {
    CLF => "CLF", ["clf"];
    JSON => "JSON", ["json"];
    CEF => "CEF", ["cef"];
    ELF => "ELF", ["elf"];
    W3C => "W3C", ["w3c"];
    GELF => "GELF", ["gelf"];
    ApacheAccessLog => "Apache Access Log", ["apache", "apacheaccesslog"];
    Logstash => "Logstash", ["logstash"];
    Log4jXML => "Log4j XML", ["log4jxml"];
    NDJSON => "NDJSON", ["ndjson"];
    MCP => "MCP", ["mcp"];
    OTLP => "OTLP", ["otlp"];
    Logfmt => "logfmt", ["logfmt"];
    ECS => "ECS", ["ecs"];
}

impl LogFormat {
    /// Validates a log entry against the current format.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::log_format::LogFormat;
    /// let is_valid = LogFormat::CLF.validate("127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326");
    /// assert!(is_valid);
    /// ```
    #[must_use]
    pub fn validate(&self, entry: &str) -> bool {
        if entry.is_empty() {
            return false;
        }
        match self {
            Self::CLF => CLF_REGEX.is_match(entry),
            Self::CEF => CEF_REGEX.is_match(entry),
            Self::W3C => W3C_REGEX.is_match(entry),
            Self::JSON
            | Self::GELF
            | Self::Logstash
            | Self::NDJSON
            | Self::MCP
            | Self::OTLP
            | Self::ECS => {
                serde_json::from_str::<serde_json::Value>(entry).is_ok()
            }
            Self::Logfmt => {
                entry.contains('=') && !entry.starts_with('=')
            }
            Self::Log4jXML => {
                entry.contains("<log4j:event") && entry.contains('>')
            }
            Self::ELF | Self::ApacheAccessLog => true, // Basic validation for others
        }
    }

    /// Formats a log entry according to the log format.
    ///
    /// # Errors
    ///
    /// This function returns an error if the log entry is not valid JSON for JSON-based formats.
    ///
    /// # Panics
    ///
    /// This function does not panic under normal usage. The internal `expect` guards
    /// a `serde_json::to_string_pretty` call on a successfully parsed `Value`, which
    /// can only fail on out-of-memory conditions.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::log_format::LogFormat;
    /// let formatted_log = LogFormat::CLF.format_log("127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326").unwrap();
    /// ```
    pub fn format_log(&self, entry: &str) -> RlgResult<String> {
        let sanitized_entry = sanitize_log_message(entry);
        match self {
            Self::CLF
            | Self::ApacheAccessLog
            | Self::CEF
            | Self::ELF
            | Self::W3C
            | Self::Log4jXML
            | Self::Logfmt => Ok(sanitized_entry),
            Self::JSON
            | Self::Logstash
            | Self::NDJSON
            | Self::GELF
            | Self::MCP
            | Self::OTLP
            | Self::ECS => {
                let val = serde_json::from_str::<serde_json::Value>(
                    &sanitized_entry,
                )
                .map_err(|e| {
                    RlgError::FormattingError(format!(
                        "Invalid JSON: {e}"
                    ))
                })?;

                // to_string_pretty on a valid Value writing to String cannot fail
                // (the only failure mode is I/O error, which String doesn't produce).
                Ok(serde_json::to_string_pretty(&val).expect(
                    "serde_json::to_string_pretty cannot fail on a valid Value",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(
            LogFormat::from_str("json").unwrap(),
            LogFormat::JSON
        );
        assert_eq!(LogFormat::from_str("CLF").unwrap(), LogFormat::CLF);
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_format_validate() {
        let clf_log = r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326"#;
        assert!(LogFormat::CLF.validate(clf_log));
        assert!(LogFormat::JSON.validate(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_log_format_format_log() {
        let json_log = r#"{"key":"value"}"#;
        let formatted = LogFormat::JSON.format_log(json_log).unwrap();
        assert!(formatted.contains('"'));

        let clf_log = r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326"#;
        let formatted = LogFormat::CLF.format_log(clf_log).unwrap();
        assert_eq!(formatted, clf_log); // CLF should remain unchanged
    }
}
