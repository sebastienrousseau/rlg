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

/// An enumeration of the different log formats that can be used.
///
/// # Variants
/// * `CLF` - Common Log Format.
/// * `JSON` - JavaScript Object Notation.
/// * `CEF` - Common Event Format.
/// * `ELF` - Extended Log Format.
/// * `W3C` - W3C Extended Log File Format.
/// * `GELF` - Graylog Extended Log Format.
/// * `ApacheAccessLog` - Apache HTTP server access logs.
/// * `Logstash` - Logstash JSON format.
/// * `Log4jXML` - Log4j's XML format.
/// * `NDJSON` - Newline Delimited JSON.
///
/// # Examples
/// ```
/// use rlg::log_format::LogFormat;
/// let format: LogFormat = "CLF".parse().unwrap();
/// assert_eq!(format, LogFormat::CLF);
/// ```
#[non_exhaustive]
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
pub enum LogFormat {
    /// Common Log Format.
    CLF,
    /// JavaScript Object Notation.
    JSON,
    /// Common Event Format.
    CEF,
    /// Extended Log Format.
    ELF,
    /// W3C Extended Log File Format.
    W3C,
    /// Graylog Extended Log Format.
    GELF,
    /// Apache HTTP server access logs.
    ApacheAccessLog,
    /// Logstash JSON format.
    Logstash,
    /// Log4j's XML format.
    Log4jXML,
    /// NDJSON (Newline Delimited JSON).
    NDJSON,
    /// Model Context Protocol (MCP) format.
    MCP,
    /// OpenTelemetry (OTLP) Log Data Model.
    OTLP,
    /// logfmt (Key-Value) format.
    Logfmt,
    /// Elastic Common Schema (ECS) format.
    ECS,
}

impl FromStr for LogFormat {
    type Err = RlgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clf" => Ok(Self::CLF),
            "json" => Ok(Self::JSON),
            "cef" => Ok(Self::CEF),
            "elf" => Ok(Self::ELF),
            "w3c" => Ok(Self::W3C),
            "gelf" => Ok(Self::GELF),
            "apacheaccesslog" => Ok(Self::ApacheAccessLog),
            "logstash" => Ok(Self::Logstash),
            "log4jxml" => Ok(Self::Log4jXML),
            "ndjson" => Ok(Self::NDJSON),
            "mcp" => Ok(Self::MCP),
            "otlp" => Ok(Self::OTLP),
            "logfmt" => Ok(Self::Logfmt),
            "ecs" => Ok(Self::ECS),
            _ => Err(RlgError::FormatParseError(format!(
                "Unknown log format: {s}"
            ))),
        }
    }
}

impl LogFormat {
    /// Validates if a given string adheres to a particular log format.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice that holds the log entry to be validated.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the input matches the log format, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use rlg::log_format::LogFormat;
    /// let is_valid = LogFormat::CLF.validate("127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326");
    /// assert!(is_valid);
    /// ```
    pub fn validate(&self, input: &str) -> bool {
        match self {
            Self::CLF | Self::ApacheAccessLog => {
                CLF_REGEX.is_match(input)
            }
            Self::JSON
            | Self::Logstash
            | Self::NDJSON
            | Self::MCP
            | Self::GELF
            | Self::OTLP
            | Self::ECS => {
                serde_json::from_str::<serde_json::Value>(input).is_ok()
            }
            Self::CEF => CEF_REGEX.is_match(input),
            Self::ELF | Self::W3C => {
                W3C_REGEX.is_match(input)
            }
            Self::Log4jXML => {
                input.trim_start().starts_with("<log4j:event")
            }
            Self::Logfmt => {
                // Basic check for logfmt (key=value or msg="value")
                input.contains('=')
            }
        }
    }

    /// Formats a log entry according to the specified log format.
    ///
    /// # Arguments
    ///
    /// * `entry` - A string slice that holds the log entry to be formatted.
    ///
    /// # Returns
    ///
    /// A `RlgResult<String>` containing the formatted log entry or an error if the formatting fails.
    ///
    /// # Example
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
                let val = serde_json::from_str::<serde_json::Value>(&sanitized_entry)
                    .map_err(|e| RlgError::FormattingError(format!("Invalid JSON: {e}")))?;
                
                let res = serde_json::to_string_pretty(&val);
                #[cfg(not(test))]
                return res.map_err(|e| RlgError::FormattingError(format!("JSON formatting error: {e}")));
                
                #[cfg(test)]
                return Ok(res.unwrap());
            }
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CLF => "CLF",
            Self::JSON => "JSON",
            Self::CEF => "CEF",
            Self::ELF => "ELF",
            Self::W3C => "W3C",
            Self::GELF => "GELF",
            Self::ApacheAccessLog => "Apache Access Log",
            Self::Logstash => "Logstash",
            Self::Log4jXML => "Log4j XML",
            Self::NDJSON => "NDJSON",
            Self::MCP => "MCP",
            Self::OTLP => "OTLP",
            Self::Logfmt => "logfmt",
            Self::ECS => "ECS",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("clf").unwrap(), LogFormat::CLF);
        assert_eq!(
            LogFormat::from_str("JSON").unwrap(),
            LogFormat::JSON
        );
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_format_validate() {
        let clf_log = r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326"#;
        assert!(LogFormat::CLF.validate(clf_log));

        let json_log = r#"{"level":"info","message":"Test log","timestamp":"2023-05-17T12:34:56Z"}"#;
        assert!(LogFormat::JSON.validate(json_log));
    }

    #[test]
    fn test_log_format_format_log() {
        let json_log = r#"{"level":"info","message":"Test log","timestamp":"2023-05-17T12:34:56Z"}"#;
        let formatted = LogFormat::JSON.format_log(json_log).unwrap();
        assert!(formatted.contains("{\n")); // Check if it's pretty-printed

        let clf_log = r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326"#;
        let formatted = LogFormat::CLF.format_log(clf_log).unwrap();
        assert_eq!(formatted, clf_log); // CLF should remain unchanged
    }
}
