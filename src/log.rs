// log.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{LogFormat, LogLevel};
use dtt::datetime::DateTime;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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
///
/// Uses `Cow<'static, str>` for fields that are commonly static strings,
/// reducing heap allocations on the hot path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct Log {
    /// The session ID for the log entry (stored as u64, formatted on output).
    pub session_id: u64,
    /// The time the log entry was created (deferred: empty until flusher formats).
    pub time: Cow<'static, str>,
    /// The log level of the message.
    pub level: LogLevel,
    /// The component that generated the log message.
    pub component: Cow<'static, str>,
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
            session_id: 0,
            time: Cow::Borrowed(""),
            level: LogLevel::INFO,
            component: Cow::Borrowed(""),
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
    #[track_caller]
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

    /// Starts building a new VERBOSE level log.
    #[must_use]
    pub fn verbose(description: &str) -> Self {
        Self::build(LogLevel::VERBOSE, description)
    }

    /// Starts building a new FATAL level log.
    #[must_use]
    pub fn fatal(description: &str) -> Self {
        Self::build(LogLevel::FATAL, description)
    }

    /// Starts building a new CRITICAL level log.
    #[must_use]
    pub fn critical(description: &str) -> Self {
        Self::build(LogLevel::CRITICAL, description)
    }

    /// Starts building a new log with the given level and description.
    ///
    /// Timestamp generation is deferred — only a monotonic session ID is
    /// captured at the call site.  The flusher thread fills in the wall-clock
    /// time just before formatting, keeping the hot path low-allocation.
    #[must_use]
    pub fn build(level: LogLevel, description: &str) -> Self {
        Self {
            session_id: SESSION_COUNTER.fetch_add(1, Ordering::Relaxed),
            time: Cow::Owned(DateTime::new().to_string()),
            level,
            component: Cow::Borrowed("default"),
            description: description.to_string(),
            format: LogFormat::MCP,
            attributes: BTreeMap::new(),
        }
    }

    /// Sets the time for the log.
    #[must_use]
    pub fn time(mut self, time: &str) -> Self {
        self.time = Cow::Owned(time.to_string());
        self
    }

    /// Sets the session ID for the log.
    #[must_use]
    pub const fn session_id(mut self, session_id: u64) -> Self {
        self.session_id = session_id;
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
        self.component = Cow::Owned(component.to_string());
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
    ///
    /// Automatically captures the call site (`file:line`) via `#[track_caller]`.
    #[track_caller]
    pub fn fire(mut self) {
        let caller = std::panic::Location::caller();
        self.attributes.insert(
            "caller".to_string(),
            serde_json::Value::String(format!(
                "{}:{}",
                caller.file(),
                caller.line()
            )),
        );
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
        write!(f, "\" session_id={}", self.session_id)?;
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

// --- Per-format serialization methods ---
impl Log {
    fn fmt_clf(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SessionID={} Timestamp={} Description={} Level={} Component={}",
            self.session_id,
            self.time,
            self.description,
            self.level,
            self.component
        )
    }

    fn fmt_cef(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CEF:0|{}|{}|{}|{}|{}|CEF",
            self.session_id,
            self.time,
            self.level,
            self.component,
            self.description
        )
    }

    fn fmt_elf(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ELF:0|{}|{}|{}|{}|{}|ELF",
            self.session_id,
            self.time,
            self.level,
            self.component,
            self.description
        )
    }

    fn fmt_w3c(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "W3C:0|{}|{}|{}|{}|{}|W3C",
            self.session_id,
            self.time,
            self.level,
            self.component,
            self.description
        )
    }

    fn fmt_apache(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn fmt_log4j_xml(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
            self.component,
            self.time,
            self.level,
            self.session_id,
            self.description
        )
    }

    fn fmt_json(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\"Attributes\":")?;
        write_json_map(f, &self.attributes)?;
        f.write_str(",\"Component\":")?;
        write_json_str(f, &self.component)?;
        f.write_str(",\"Description\":")?;
        write_json_str(f, &self.description)?;
        f.write_str(",\"Format\":\"JSON\",\"Level\":")?;
        write_json_str(f, self.level.as_str())?;
        write!(f, ",\"SessionID\":{}", self.session_id)?;
        f.write_str(",\"Timestamp\":")?;
        write_json_str(f, &self.time)?;
        f.write_str("}")
    }

    fn fmt_gelf(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\"_attributes\":")?;
        write_json_map(f, &self.attributes)?;
        write!(f, ",\"_session_id\":{}", self.session_id)?;
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

    fn fmt_logstash(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        write!(f, ",\"session_id\":{}", self.session_id)?;
        f.write_str("}")
    }

    fn fmt_ndjson(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn fmt_mcp(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/log\",\"params\":{\"data\":{\"attributes\":")?;
        write_json_map(f, &self.attributes)?;
        f.write_str(",\"component\":")?;
        write_json_str(f, &self.component)?;
        f.write_str(",\"description\":")?;
        write_json_str(f, &self.description)?;
        write!(f, ",\"session_id\":{}", self.session_id)?;
        f.write_str(",\"time\":")?;
        write_json_str(f, &self.time)?;
        f.write_str("},\"level\":")?;
        write_json_str(f, self.level.as_str_lowercase())?;
        f.write_str("}}")
    }

    fn fmt_otlp(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let empty = serde_json::Value::String(String::new());
        let trace_id =
            self.attributes.get("trace_id").unwrap_or(&empty);
        let span_id = self.attributes.get("span_id").unwrap_or(&empty);
        f.write_str("{\"attributes\":")?;
        write_json_map(f, &self.attributes)?;
        f.write_str(",\"body\":{\"stringValue\":")?;
        write_json_str(f, &self.description)?;
        write!(f, "}},\"severityNumber\":{}", self.level.to_numeric())?;
        f.write_str(",\"severityText\":")?;
        write_json_str(f, self.level.as_str())?;
        write!(f, ",\"spanId\":{span_id}")?;
        f.write_str(",\"timeUnixNano\":")?;
        write_json_str(f, &self.time)?;
        write!(f, ",\"traceId\":{trace_id}}}")
    }

    fn fmt_ecs(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => self.fmt_clf(f),
            LogFormat::CEF => self.fmt_cef(f),
            LogFormat::ELF => self.fmt_elf(f),
            LogFormat::W3C => self.fmt_w3c(f),
            LogFormat::ApacheAccessLog => self.fmt_apache(f),
            LogFormat::Log4jXML => self.fmt_log4j_xml(f),
            LogFormat::JSON => self.fmt_json(f),
            LogFormat::GELF => self.fmt_gelf(f),
            LogFormat::Logstash => self.fmt_logstash(f),
            LogFormat::NDJSON => self.fmt_ndjson(f),
            LogFormat::MCP => self.fmt_mcp(f),
            LogFormat::OTLP => self.fmt_otlp(f),
            LogFormat::Logfmt => self.write_logfmt(f),
            LogFormat::ECS => self.fmt_ecs(f),
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
            .session_id(99)
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
            .session_id(100)
            .time("ts")
            .component("comp")
            .format(LogFormat::Logfmt);
        let output_no = format!("{log_no_attr}");
        assert!(!output_no.contains(" key="));
    }
}
