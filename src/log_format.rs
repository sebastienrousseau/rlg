// log_format.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Custom error type for parsing LogFormat.
#[derive(Debug, Clone)]
pub struct ParseLogFormatError {
    details: String,
}

impl ParseLogFormatError {
    /// Creates a new `ParseLogFormatError` with a specified message.
    fn new(msg: &str) -> ParseLogFormatError {
        ParseLogFormatError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseLogFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

/// Compiled regular expressions for log format validation.
static CLF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
    r#"^(?P<host>\S+) (?P<ident>\S+) (?P<user>\S+) \[(?P<time>[^\]]+)\] "(?P<method>\S+) (?P<path>\S+) (?P<protocol>\S+)" (?P<status>\d{3}) (?P<size>\d+|-)$"#
).unwrap()
});

static CEF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^CEF:\d+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|[^|]+\|.*$"#,
    )
    .unwrap()
});

static W3C_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^#Fields:.*
.+$"#,
    )
    .unwrap()
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
    /// Newline Delimited JSON.
    NDJSON,
}

impl FromStr for LogFormat {
    type Err = ParseLogFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clf" => Ok(LogFormat::CLF),
            "json" => Ok(LogFormat::JSON),
            "cef" => Ok(LogFormat::CEF),
            "elf" => Ok(LogFormat::ELF),
            "w3c" => Ok(LogFormat::W3C),
            "gelf" => Ok(LogFormat::GELF),
            "apacheaccesslog" => Ok(LogFormat::ApacheAccessLog),
            "logstash" => Ok(LogFormat::Logstash),
            "log4jxml" => Ok(LogFormat::Log4jXML),
            "ndjson" => Ok(LogFormat::NDJSON),
            _ => Err(ParseLogFormatError::new("Unknown log format")),
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
            LogFormat::CLF | LogFormat::ApacheAccessLog => {
                CLF_REGEX.is_match(input)
            }
            LogFormat::JSON
            | LogFormat::Logstash
            | LogFormat::NDJSON => {
                serde_json::from_str::<serde_json::Value>(input).is_ok()
            }
            LogFormat::CEF => CEF_REGEX.is_match(input),
            LogFormat::ELF | LogFormat::W3C => {
                W3C_REGEX.is_match(input)
            }
            LogFormat::GELF => {
                serde_json::from_str::<serde_json::Value>(input).is_ok()
            }
            LogFormat::Log4jXML => {
                input.trim_start().starts_with("<log4j:event")
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
    /// A `Result<String, serde_json::Error>` containing the formatted log entry or an error if the formatting fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rlg::log_format::LogFormat;
    /// let formatted_log = LogFormat::CLF.format_log("127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326").unwrap();
    /// ```
    pub fn format_log(
        &self,
        entry: &str,
    ) -> Result<String, ParseLogFormatError> {
        match self {
            LogFormat::CLF
            | LogFormat::ApacheAccessLog
            | LogFormat::CEF
            | LogFormat::ELF
            | LogFormat::W3C
            | LogFormat::Log4jXML => Ok(entry.to_string()),
            LogFormat::JSON
            | LogFormat::Logstash
            | LogFormat::NDJSON
            | LogFormat::GELF => serde_json::to_string_pretty(
                &serde_json::from_str::<serde_json::Value>(entry)
                    .map_err(|e| {
                        ParseLogFormatError::new(&format!(
                            "Invalid JSON: {}",
                            e
                        ))
                    })?,
            )
            .map_err(|e| {
                ParseLogFormatError::new(&format!(
                    "JSON formatting error: {}",
                    e
                ))
            }),
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogFormat::CLF => "CLF",
            LogFormat::JSON => "JSON",
            LogFormat::CEF => "CEF",
            LogFormat::ELF => "ELF",
            LogFormat::W3C => "W3C",
            LogFormat::GELF => "GELF",
            LogFormat::ApacheAccessLog => "Apache Access Log",
            LogFormat::Logstash => "Logstash",
            LogFormat::Log4jXML => "Log4j XML",
            LogFormat::NDJSON => "NDJSON",
        };
        write!(f, "{}", s)
    }
}
