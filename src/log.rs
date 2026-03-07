// log.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{LogFormat, LogLevel};
use dtt::datetime::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic session ID counter (allocation-free).
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Cached hostname to avoid repeated syscalls.
static CACHED_HOSTNAME: LazyLock<String> = LazyLock::new(|| {
    hostname::get().map_or_else(
        |_| "localhost".to_string(),
        |h| h.to_string_lossy().to_string(),
    )
});

/// The `Log` struct provides an easy way to log a message to the console.
/// It contains a set of defined fields to create a simple log message with a readable output format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
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
    /// Formatting is deferred to the background flusher thread.
    /// This borrows `self` and clones — prefer [`fire()`](Self::fire) to avoid the clone.
    pub fn log(&self) {
        crate::engine::ENGINE.inc_format(self.format);
        let event = crate::engine::LogEvent {
            level: self.level,
            level_num: self.level.to_numeric(),
            log: self.clone(),
        };
        crate::engine::ENGINE.ingest(event);
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

    /// Starts building a new log with the given level and description.
    #[must_use]
    pub fn build(level: LogLevel, description: &str) -> Self {
        Self {
            session_id: SESSION_COUNTER
                .fetch_add(1, Ordering::Relaxed)
                .to_string(),
            time: DateTime::new().to_string(),
            level,
            component: "default".to_string(),
            description: description.to_string(),
            format: LogFormat::MCP,
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

    /// Fires the log into the lock-free background ingestion engine, consuming it.
    ///
    /// Formatting is deferred to the background flusher thread — the caller
    /// only pays for a `Log` move (~128-byte memcpy), not serialization.
    pub fn fire(self) {
        crate::engine::ENGINE.inc_format(self.format);
        let event = crate::engine::LogEvent {
            level: self.level,
            level_num: self.level.to_numeric(),
            log: self,
        };
        crate::engine::ENGINE.ingest(event);
    }

    fn write_logfmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("level=")?;
        f.write_str(self.level.as_str_lowercase())?;
        f.write_str(" msg=\"")?;
        f.write_str(&self.description.replace('"', "\\\""))?;
        f.write_str("\" session_id=")?;
        f.write_str(&self.session_id)?;
        f.write_str(" component=\"")?;
        f.write_str(&self.component)?;
        f.write_str("\"")?;

        for (key, value) in &self.attributes {
            write!(f, " {key}=")?;
            match value {
                serde_json::Value::String(s) => {
                    if s.contains(' ')
                        || s.contains('"')
                        || s.is_empty()
                    {
                        write!(f, "\"{0}\"", s.replace('"', "\\\""))?;
                    } else {
                        write!(f, "{s}")?;
                    }
                }
                _ => write!(f, "{value}")?,
            }
        }
        Ok(())
    }
}

/// Writes a JSON-escaped string (with surrounding quotes) to the formatter.
fn write_json_str(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if c.is_control() => write!(f, "\\u{:04x}", c as u32)?,
            c => write!(f, "{c}")?,
        }
    }
    f.write_str("\"")
}

/// Writes a `BTreeMap<String, serde_json::Value>` as a JSON object.
fn write_json_map(
    f: &mut fmt::Formatter<'_>,
    map: &BTreeMap<String, serde_json::Value>,
) -> fmt::Result {
    f.write_str("{")?;
    let mut first = true;
    for (key, value) in map {
        if !first {
            f.write_str(",")?;
        }
        first = false;
        write_json_str(f, key)?;
        // serde_json::Value Display already produces valid JSON
        write!(f, ":{value}")?;
    }
    f.write_str("}")
}

impl fmt::Display for Log {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => write!(
                f,
                "SessionID={} Timestamp={} Description={} Level={} Component={}",
                self.session_id,
                self.time,
                self.description,
                self.level,
                self.component
            ),
            LogFormat::CEF => write!(
                f,
                "CEF:0|{}|{}|{}|{}|{}|CEF",
                self.session_id,
                self.time,
                self.level,
                self.component,
                self.description
            ),
            LogFormat::ELF => write!(
                f,
                "ELF:0|{}|{}|{}|{}|{}|ELF",
                self.session_id,
                self.time,
                self.level,
                self.component,
                self.description
            ),
            LogFormat::W3C => write!(
                f,
                "W3C:0|{}|{}|{}|{}|{}|W3C",
                self.session_id,
                self.time,
                self.level,
                self.component,
                self.description
            ),
            LogFormat::ApacheAccessLog => {
                write!(
                    f,
                    "{} - - [{}] \"{}\" {} {}",
                    &*CACHED_HOSTNAME,
                    self.time,
                    self.description,
                    self.level,
                    self.component
                )
            }
            LogFormat::Log4jXML => write!(
                f,
                r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
                self.component,
                self.time,
                self.level,
                self.session_id,
                self.description
            ),
            LogFormat::JSON => {
                // Keys in alphabetical order to match previous serde_json output
                f.write_str("{\"Attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"Component\":")?;
                write_json_str(f, &self.component)?;
                f.write_str(",\"Description\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"Format\":\"JSON\",\"Level\":")?;
                write_json_str(f, self.level.as_str())?;
                f.write_str(",\"SessionID\":")?;
                write_json_str(f, &self.session_id)?;
                f.write_str(",\"Timestamp\":")?;
                write_json_str(f, &self.time)?;
                f.write_str("}")
            }
            LogFormat::GELF => {
                // Keys in alphabetical order
                f.write_str("{\"_attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"_session_id\":")?;
                write_json_str(f, &self.session_id)?;
                f.write_str(",\"full_message\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"host\":")?;
                write_json_str(f, &self.component)?;
                write!(f, ",\"level\":{}", self.level.to_numeric())?;
                f.write_str(",\"short_message\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"timestamp\":")?;
                write_json_str(f, &self.time)?;
                f.write_str(",\"version\":\"1.1\"}")
            }
            LogFormat::Logstash => {
                // Keys in alphabetical order
                f.write_str("{\"@timestamp\":")?;
                write_json_str(f, &self.time)?;
                f.write_str(",\"attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"component\":")?;
                write_json_str(f, &self.component)?;
                f.write_str(",\"level\":")?;
                write_json_str(f, self.level.as_str())?;
                f.write_str(",\"message\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"session_id\":")?;
                write_json_str(f, &self.session_id)?;
                f.write_str("}")
            }
            LogFormat::NDJSON => {
                // Keys in alphabetical order
                f.write_str("{\"attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"component\":")?;
                write_json_str(f, &self.component)?;
                f.write_str(",\"level\":")?;
                write_json_str(f, self.level.as_str())?;
                f.write_str(",\"message\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"timestamp\":")?;
                write_json_str(f, &self.time)?;
                f.write_str("}")
            }
            LogFormat::MCP => {
                f.write_str("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/log\",\"params\":{\"data\":{\"attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"component\":")?;
                write_json_str(f, &self.component)?;
                f.write_str(",\"description\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"session_id\":")?;
                write_json_str(f, &self.session_id)?;
                f.write_str(",\"time\":")?;
                write_json_str(f, &self.time)?;
                f.write_str("},\"level\":")?;
                write_json_str(f, self.level.as_str_lowercase())?;
                f.write_str("}}")
            }
            LogFormat::OTLP => {
                let empty = serde_json::Value::String(String::new());
                let trace_id =
                    self.attributes.get("trace_id").unwrap_or(&empty);
                let span_id =
                    self.attributes.get("span_id").unwrap_or(&empty);
                f.write_str("{\"attributes\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"body\":{\"stringValue\":")?;
                write_json_str(f, &self.description)?;
                write!(
                    f,
                    "}},\"severityNumber\":{}",
                    self.level.to_numeric()
                )?;
                f.write_str(",\"severityText\":")?;
                write_json_str(f, self.level.as_str())?;
                write!(f, ",\"spanId\":{span_id}")?;
                f.write_str(",\"timeUnixNano\":")?;
                write_json_str(f, &self.time)?;
                write!(f, ",\"traceId\":{trace_id}}}")
            }
            LogFormat::Logfmt => self.write_logfmt(f),
            LogFormat::ECS => {
                // Keys in alphabetical order
                f.write_str("{\"@timestamp\":")?;
                write_json_str(f, &self.time)?;
                f.write_str(",\"labels\":")?;
                write_json_map(f, &self.attributes)?;
                f.write_str(",\"log.level\":")?;
                write_json_str(f, self.level.as_str_lowercase())?;
                f.write_str(",\"log.logger\":\"rlg\",\"message\":")?;
                write_json_str(f, &self.description)?;
                f.write_str(",\"process.name\":")?;
                write_json_str(f, &self.component)?;
                f.write_str("}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_format::LogFormat;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_log_write_logfmt_with_attributes() {
        let mut log = Log::build(LogLevel::INFO, "desc")
            .session_id("sid")
            .time("ts")
            .component("comp")
            .format(LogFormat::Logfmt);
        log.attributes
            .insert("key".to_string(), serde_json::json!("value"));
        log.attributes.insert(
            "space".to_string(),
            serde_json::json!("has space"),
        );
        log.attributes
            .insert("num".to_string(), serde_json::json!(42));
        log.attributes
            .insert("empty".to_string(), serde_json::json!(""));

        let output = format!("{log}");
        assert!(output.contains("key=value"));
        assert!(output.contains("space=\"has space\""));
        assert!(output.contains("num=42"));
        assert!(output.contains("empty=\"\""));

        // Case with no attributes to cover the other branch
        let log_no_attr = Log::build(LogLevel::INFO, "desc")
            .session_id("sid")
            .time("ts")
            .component("comp")
            .format(LogFormat::Logfmt);
        let output_no = format!("{log_no_attr}");
        assert!(!output_no.contains(" key="));
    }
}
