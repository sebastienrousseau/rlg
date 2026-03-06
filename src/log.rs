// log.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{LogFormat, LogLevel, RlgResult};
use dtt::datetime::DateTime;
use hostname;
use serde::{Deserialize, Serialize};
use std::{fmt, collections::BTreeMap};
use vrd::random::Random;

/// The `Log` struct provides an easy way to log a message to the console.
/// It contains a set of defined fields to create a simple log message with a readable output format.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
)]
pub struct Log {
    /// The session ID for the log entry.
    pub session_id: String,
    /// The time the log entry was created.
    pub time: String,
    /// The log level of the message.
    pub level: LogLevel,
    /// The component that generated the log message.
    pub component: String,
    /// The description of the log message.
    pub description: String,
    /// The format of the log message.
    pub format: LogFormat,
    /// Semantic context tagging (High-cardinality attributes)
    pub attributes: BTreeMap<String, serde_json::Value>,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            session_id: String::default(),
            time: String::default(),
            level: LogLevel::INFO,
            component: String::default(),
            description: String::default(),
            format: LogFormat::CLF,
            attributes: BTreeMap::new(),
        }
    }
}

impl Log {
    /// Logs a message by ingesting it into the lock-free engine.
    ///
    /// # Errors
    ///
    /// This function returns an error if the ingestion into the lock-free engine fails.
    pub fn log(&self) -> RlgResult<()> {
        let payload = format!("{self}\n").into_bytes();
        let event = crate::engine::LogEvent {
            level: format!("{0:?}", self.level),
            payload,
        };
        crate::engine::ENGINE.ingest(event);
        Ok(())
    }

    /// Creates a new log entry with provided details.
    #[deprecated(since = "0.0.7", note = "Please use the lock-free fluent API (`.fire()`) instead.")]
    #[must_use] 
    pub fn new(
        session_id: &str,
        time: &str,
        level: &LogLevel,
        component: &str,
        description: &str,
        format: &LogFormat,
    ) -> Self {
        Self {
            session_id: session_id.to_string(),
            time: time.to_string(),
            level: *level,
            component: component.to_string(),
            description: description.to_string(),
            format: *format,
            attributes: BTreeMap::new(),
        }
    }

    /// Starts building a new INFO level log.
    #[must_use] 
    pub fn info(description: &str) -> Self {
        Self::build(LogLevel::INFO, description)
    }

    /// Starts building a new WARN level log.
    #[must_use] 
    pub fn warn(description: &str) -> Self {
        Self::build(LogLevel::WARN, description)
    }

    /// Starts building a new ERROR level log.
    #[must_use] 
    pub fn error(description: &str) -> Self {
        Self::build(LogLevel::ERROR, description)
    }

    /// Starts building a new DEBUG level log.
    #[must_use] 
    pub fn debug(description: &str) -> Self {
        Self::build(LogLevel::DEBUG, description)
    }

    /// Starts building a new TRACE level log.
    #[must_use] 
    pub fn trace(description: &str) -> Self {
        Self::build(LogLevel::TRACE, description)
    }

    /// Starts building a new FATAL level log.
    #[must_use] 
    pub fn fatal(description: &str) -> Self {
        Self::build(LogLevel::FATAL, description)
    }

    fn build(level: LogLevel, description: &str) -> Self {
        Self {
            session_id: Random::default().int(0, 1_000_000_000).to_string(),
            time: DateTime::new().to_string(),
            level,
            component: "default".to_string(),
            description: description.to_string(),
            format: LogFormat::MCP, // Phase 2/3 default
            attributes: BTreeMap::new(),
        }
    }

    /// Sets the time for the log.
    #[must_use] 
    pub fn time(mut self, time: &str) -> Self {
        self.time = time.to_string();
        self
    }

    /// Sets the session ID for the log.
    #[must_use] 
    pub fn session_id(mut self, session_id: &str) -> Self {
        self.session_id = session_id.to_string();
        self
    }

    /// Adds a semantic context attribute.
    #[must_use]
    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(val) = serde_json::to_value(value) {
            self.attributes.insert(key.to_string(), val);
        }
        self
    }

    /// Sets the component for the log.
    #[must_use] 
    pub fn component(mut self, component: &str) -> Self {
        self.component = component.to_string();
        self
    }

    /// Sets the format for the log.
    #[must_use] 
    pub const fn format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Fires the log into the lock-free background ingestion engine.
    pub fn fire(&self) {
        let payload = format!("{self}\n").into_bytes();
        let event = crate::engine::LogEvent {
            level: format!("{0:?}", self.level),
            payload,
        };
        crate::engine::ENGINE.ingest(event);
    }

    /// Writes a log entry to the log file using the provided details.
    ///
    /// # Errors
    ///
    /// This function returns an error if the ingestion into the lock-free engine fails.
    #[allow(deprecated)]
    #[deprecated(since = "0.0.7", note = "Please use the lock-free fluent API (`.fire()`) instead.")]
    pub fn write_log_entry(
        log_level: LogLevel,
        process: &str,
        message: &str,
        log_format: LogFormat,
    ) -> RlgResult<()> {
        let log_entry = Self::new(&Random::default().int(0, 1_000_000_000).to_string(), &DateTime::new().to_string(), &log_level, process, message, &log_format);

        let payload = format!("{log_entry}\n").into_bytes();
        let event = crate::engine::LogEvent {
            level: format!("{log_level:?}"),
            payload,
        };
        crate::engine::ENGINE.ingest(event);
        
        Ok(())
    }

    fn write_json(f: &mut fmt::Formatter<'_>, val: &serde_json::Value) -> fmt::Result {
        write!(f, "{val}")
    }

    fn write_logfmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "level={0} msg=\"{1}\" session_id={2} component=\"{3}\"", 
            self.level.to_string().to_lowercase(),
            self.description.replace('"', "\\\""),
            self.session_id,
            self.component
        )?;
        
        for (key, value) in &self.attributes {
            write!(f, " {key}=")?;
            match value {
                serde_json::Value::String(s) => {
                    if s.contains(' ') || s.contains('"') || s.is_empty() {
                        write!(f, "\"{0}\"", s.replace('"', "\\\""))?;
                    } else {
                        write!(f, "{s}")?;
                    }
                },
                _ => write!(f, "{value}")?,
            }
        }
        Ok(())
    }
}

impl fmt::Display for Log {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => write!(
                f,
                "SessionID={} Timestamp={} Description={} Level={} Component={}",
                self.session_id, self.time, self.description, self.level, self.component
            ),
            LogFormat::CEF => write!(
                f,
                "CEF:0|{}|{}|{}|{}|{}|CEF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ELF => write!(
                f,
                "ELF:0|{}|{}|{}|{}|{}|ELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::W3C => write!(
                f,
                "W3C:0|{}|{}|{}|{}|{}|W3C",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ApacheAccessLog => {
                let host = hostname::get().map_or_else(|_| "localhost".to_string(), |h| h.to_string_lossy().to_string());
                write!(
                    f,
                    "{} - - [{}] \"{}\" {} {}",
                    host,
                    self.time,
                    self.description,
                    self.level,
                    self.component
                )
            },
            LogFormat::Log4jXML => write!(
                f,
                r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
                self.component, self.time, self.level, self.session_id, self.description
            ),
            LogFormat::JSON => Self::write_json(f, &serde_json::json!({
                "SessionID": self.session_id,
                "Timestamp": self.time,
                "Level": self.level,
                "Component": self.component,
                "Description": self.description,
                "Format": "JSON",
                "Attributes": self.attributes
            })),
            LogFormat::GELF => Self::write_json(f, &serde_json::json!({
                "version": "1.1",
                "host": self.component,
                "short_message": self.description,
                "full_message": self.description,
                "timestamp": self.time,
                "level": self.level.to_numeric(),
                "_session_id": self.session_id,
                "_attributes": self.attributes,
            })),
            LogFormat::Logstash => Self::write_json(f, &serde_json::json!({
                "@timestamp": self.time,
                "level": self.level,
                "component": self.component,
                "message": self.description,
                "session_id": self.session_id,
                "attributes": self.attributes,
            })),
            LogFormat::NDJSON => Self::write_json(f, &serde_json::json!({
                "timestamp": self.time,
                "level": self.level,
                "component": self.component,
                "message": self.description,
                "attributes": self.attributes
            })),
            LogFormat::MCP => Self::write_json(f, &serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/log",
                "params": {
                    "level": self.level.to_string().to_lowercase(),
                    "data": {
                        "session_id": self.session_id,
                        "time": self.time,
                        "component": self.component,
                        "description": self.description,
                        "attributes": self.attributes
                    }
                }
            })),
            LogFormat::OTLP => {
                let trace_id = self.attributes.get("trace_id").cloned().unwrap_or_else(|| serde_json::Value::String(String::new()));
                let span_id = self.attributes.get("span_id").cloned().unwrap_or_else(|| serde_json::Value::String(String::new()));
                Self::write_json(f, &serde_json::json!({
                    "timeUnixNano": self.time,
                    "severityText": self.level.to_string(),
                    "severityNumber": self.level.to_numeric(),
                    "body": { "stringValue": self.description },
                    "attributes": self.attributes,
                    "traceId": trace_id,
                    "spanId": span_id,
                }))
            },
            LogFormat::Logfmt => self.write_logfmt(f),
            LogFormat::ECS => Self::write_json(f, &serde_json::json!({
                "@timestamp": self.time,
                "log.level": self.level.to_string().to_lowercase(),
                "message": self.description,
                "process.name": self.component,
                "log.logger": "rlg",
                "labels": self.attributes,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_level::LogLevel;
    use crate::log_format::LogFormat;

    #[test]
    #[allow(deprecated)]
    fn test_log_write_logfmt_with_attributes() {
        let mut log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::Logfmt);
        log.attributes.insert("key".to_string(), serde_json::json!("value"));
        log.attributes.insert("space".to_string(), serde_json::json!("has space"));
        log.attributes.insert("num".to_string(), serde_json::json!(42));
        log.attributes.insert("empty".to_string(), serde_json::json!(""));
        
        let output = format!("{log}");
        assert!(output.contains("key=value"));
        assert!(output.contains("space=\"has space\""));
        assert!(output.contains("num=42"));
        assert!(output.contains("empty=\"\""));

        // Case with no attributes to cover the other branch
        let log_no_attr = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::Logfmt);
        let output_no = format!("{log_no_attr}");
        assert!(!output_no.contains(" key="));
    }
}
