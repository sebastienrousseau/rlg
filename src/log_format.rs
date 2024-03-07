// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

/// An enumeration of the different log formats that can be used.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogFormat {
    /// The log format is set to CLF.
    CLF,
    /// The log format is set to JSON.
    JSON,
    /// The log format is set to CEF.
    CEF,
    /// The log format is set to ELF.
    ELF,
    /// The log format is set to W3C.
    W3C,
    /// The log format is set to GELF.
    GELF,
    /// The log format is set to Apache Access Log.
    ApacheAccessLog,
    /// The log format is set to Logstash.
    Logstash,
    /// The log format is set to Log4j XML.
    Log4jXML,
    /// The log format is set to NDJSON (Newline Delimited JSON).
    NDJSON,
}

impl fmt::Display for LogFormat {
    /// Implements [`LogFormat`] to display the log format as a string.
    /// It allows the LogFormat enumeration to be used with the write! macro.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::CEF => write!(f, "CEF"),
            LogFormat::CLF => write!(f, "CLF"),
            LogFormat::ELF => write!(f, "ELF"),
            LogFormat::GELF => write!(f, "GELF"),
            LogFormat::JSON => write!(f, "JSON"),
            LogFormat::W3C => write!(f, "W3C"),
            LogFormat::ApacheAccessLog => write!(f, "Apache Access Log"),
            LogFormat::Logstash => write!(f, "Logstash"),
            LogFormat::Log4jXML => write!(f, "Log4j XML"),
            LogFormat::NDJSON => write!(f, "NDJSON"),
        }
    }
}