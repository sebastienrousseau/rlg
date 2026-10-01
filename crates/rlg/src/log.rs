// log.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{LogFormat, LogLevel, datetime};
use euxis_commons::counter::Counter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::LazyLock;
use std::sync::atomic::Ordering;

mod write;
use write::Part::{Map, Num, Raw, Str, Value};
use write::{write_logfmt_value, write_parts};

/// `file:line` for a call site, built without the `format!` machinery.
/// The flusher calls it once per record `fire()` sent; under Miri,
/// which runs no flusher, nothing does.
#[cfg(not(miri))]
pub(crate) fn caller_string(
    caller: &std::panic::Location<'_>,
) -> String {
    let mut line = itoa::Buffer::new();
    let line = line.format(caller.line());
    let file = caller.file();
    let mut out = String::with_capacity(file.len() + 1 + line.len());
    out.push_str(file);
    out.push(':');
    out.push_str(line);
    out
}

/// Monotonic session ID counter. Incremented atomically per `build()` call.
static SESSION_COUNTER: Counter = Counter::new(1);

/// Hostname, resolved once and cached for the process lifetime.
static CACHED_HOSTNAME: LazyLock<String> =
    LazyLock::new(|| resolve_hostname(hostname::get()));

/// Pure helper for [`CACHED_HOSTNAME`] — exposed so the `"localhost"`
/// fallback branch can be unit-tested without injecting a syscall failure.
fn resolve_hostname(
    raw: std::io::Result<std::ffi::OsString>,
) -> String {
    raw.map_or_else(
        |_| "localhost".to_string(),
        |h| h.to_string_lossy().to_string(),
    )
}

/// A structured log entry with a chainable builder API.
///
/// Fields use `Cow<'static, str>` and `u64` where possible to
/// minimize heap allocations on the ingestion hot path.
///
/// Construct via level shortcuts ([`Log::info`], [`Log::error`], ...)
/// or the generic [`Log::build`]. Dispatch with [`Log::fire`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct Log {
    /// Monotonic counter assigned at `build()` time.
    pub session_id: u64,
    /// Wall-clock timestamp. Populated at build time; override with `.time()`.
    pub time: Cow<'static, str>,
    /// Severity level (`INFO`, `ERROR`, etc.).
    pub level: LogLevel,
    /// Originating service or module name. Defaults to `"default"`.
    pub component: Cow<'static, str>,
    /// Human-readable message body.
    pub description: String,
    /// Output format applied during `Display` serialization.
    pub format: LogFormat,
    /// Arbitrary key-value attributes for structured context.
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
    /// Ingest this entry into the engine by cloning it.
    ///
    /// **Prefer [`fire()`](Self::fire)**, which consumes `self` and avoids
    /// the clone. Use `log()` only when you need to retain the entry.
    #[track_caller]
    pub fn log(&self) {
        crate::engine::ENGINE
            .ingest(crate::engine::LogEvent::new(self.clone()));
    }

    /// Build an INFO-level log entry.
    #[must_use]
    pub fn info(description: &str) -> Self {
        Self::build(LogLevel::INFO, description)
    }

    /// Build a WARN-level log entry.
    #[must_use]
    pub fn warn(description: &str) -> Self {
        Self::build(LogLevel::WARN, description)
    }

    /// Build an ERROR-level log entry.
    #[must_use]
    pub fn error(description: &str) -> Self {
        Self::build(LogLevel::ERROR, description)
    }

    /// Build a DEBUG-level log entry.
    #[must_use]
    pub fn debug(description: &str) -> Self {
        Self::build(LogLevel::DEBUG, description)
    }

    /// Build a TRACE-level log entry.
    #[must_use]
    pub fn trace(description: &str) -> Self {
        Self::build(LogLevel::TRACE, description)
    }

    /// Build a VERBOSE-level log entry.
    #[must_use]
    pub fn verbose(description: &str) -> Self {
        Self::build(LogLevel::VERBOSE, description)
    }

    /// Build a FATAL-level log entry.
    #[must_use]
    pub fn fatal(description: &str) -> Self {
        Self::build(LogLevel::FATAL, description)
    }

    /// Build a CRITICAL-level log entry.
    #[must_use]
    pub fn critical(description: &str) -> Self {
        Self::build(LogLevel::CRITICAL, description)
    }

    /// Build a log entry with an explicit level and description.
    ///
    /// Assigns a monotonic `session_id` and captures the current wall-clock
    /// time. Defaults to `LogFormat::MCP` and component `"default"`.
    #[must_use]
    pub fn build(level: LogLevel, description: &str) -> Self {
        Self {
            session_id: SESSION_COUNTER.fetch_add(1, Ordering::Relaxed),
            time: Cow::Owned(datetime::now_iso8601()),
            level,
            component: Cow::Borrowed("default"),
            description: description.to_string(),
            format: LogFormat::MCP,
            attributes: BTreeMap::new(),
        }
    }

    /// Override the timestamp for this entry.
    #[must_use]
    pub fn time(mut self, time: &str) -> Self {
        self.time = Cow::Owned(time.to_string());
        self
    }

    /// Override the auto-assigned session ID.
    #[must_use]
    pub const fn session_id(mut self, session_id: u64) -> Self {
        self.session_id = session_id;
        self
    }

    /// Attach a key-value attribute. Accepts any `T: Serialize`.
    #[must_use]
    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(val) = serde_json::to_value(value) {
            self.attributes.insert(key.to_string(), val);
        }
        self
    }

    /// Tag the originating service or module.
    #[must_use]
    pub fn component(mut self, component: &str) -> Self {
        self.component = Cow::Owned(component.to_string());
        self
    }

    /// Set the output format for this entry.
    #[must_use]
    pub const fn format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Consume this entry and push it into the ring buffer.
    ///
    /// Cost: one `Log` move (~128 bytes). Serialization is deferred.
    /// Automatically captures `file:line` via `#[track_caller]`; the
    /// flusher adds it as the `caller` attribute.
    #[track_caller]
    pub fn fire(self) {
        crate::engine::ENGINE.ingest(self.into_fired_event());
    }

    /// The event `fire()` ingests: this entry plus its call site,
    /// which stays a `&'static Location` until the flusher renders it.
    #[track_caller]
    fn into_fired_event(self) -> crate::engine::LogEvent {
        crate::engine::LogEvent {
            caller: Some(std::panic::Location::caller()),
            ..crate::engine::LogEvent::new(self)
        }
    }

    fn write_logfmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "level={} msg=\"{}\" session_id={} component=\"{}\"",
            self.level.as_str_lowercase(),
            self.description.replace('"', "\\\""),
            self.session_id,
            self.component,
        )?;
        for (key, value) in &self.attributes {
            write!(f, " {key}=")?;
            write_logfmt_value(f, value)?;
        }
        Ok(())
    }
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
            *CACHED_HOSTNAME,
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
        write_parts(
            f,
            &[
                Raw("{\"Attributes\":"),
                Map(&self.attributes),
                Raw(",\"Component\":"),
                Str(&self.component),
                Raw(",\"Description\":"),
                Str(&self.description),
                Raw(",\"Format\":\"JSON\",\"Level\":"),
                Str(self.level.as_str()),
                Raw(",\"SessionID\":"),
                Num(self.session_id),
                Raw(",\"Timestamp\":"),
                Str(&self.time),
                Raw("}"),
            ],
        )
    }

    fn fmt_gelf(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(
            f,
            &[
                Raw("{\"_attributes\":"),
                Map(&self.attributes),
                Raw(",\"_session_id\":"),
                Num(self.session_id),
                Raw(",\"full_message\":"),
                Str(&self.description),
                Raw(",\"host\":"),
                Str(&self.component),
                Raw(",\"level\":"),
                Num(u64::from(self.level.to_numeric())),
                Raw(",\"short_message\":"),
                Str(&self.description),
                Raw(",\"timestamp\":"),
                Str(&self.time),
                Raw(",\"version\":\"1.1\"}"),
            ],
        )
    }

    fn fmt_logstash(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(
            f,
            &[
                Raw("{\"@timestamp\":"),
                Str(&self.time),
                Raw(",\"attributes\":"),
                Map(&self.attributes),
                Raw(",\"component\":"),
                Str(&self.component),
                Raw(",\"level\":"),
                Str(self.level.as_str()),
                Raw(",\"message\":"),
                Str(&self.description),
                Raw(",\"session_id\":"),
                Num(self.session_id),
                Raw("}"),
            ],
        )
    }

    fn fmt_ndjson(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(
            f,
            &[
                Raw("{\"attributes\":"),
                Map(&self.attributes),
                Raw(",\"component\":"),
                Str(&self.component),
                Raw(",\"level\":"),
                Str(self.level.as_str()),
                Raw(",\"message\":"),
                Str(&self.description),
                Raw(",\"timestamp\":"),
                Str(&self.time),
                Raw("}"),
            ],
        )
    }

    fn fmt_mcp(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(
            f,
            &[
                Raw(
                    "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/log\",\"params\":{\"data\":{\"attributes\":",
                ),
                Map(&self.attributes),
                Raw(",\"component\":"),
                Str(&self.component),
                Raw(",\"description\":"),
                Str(&self.description),
                Raw(",\"session_id\":"),
                Num(self.session_id),
                Raw(",\"time\":"),
                Str(&self.time),
                Raw("},\"level\":"),
                Str(self.level.as_str_lowercase()),
                Raw("}}"),
            ],
        )
    }

    fn fmt_otlp(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let empty = serde_json::Value::String(String::new());
        let trace_id =
            self.attributes.get("trace_id").unwrap_or(&empty);
        let span_id = self.attributes.get("span_id").unwrap_or(&empty);
        write_parts(
            f,
            &[
                Raw("{\"attributes\":"),
                Map(&self.attributes),
                Raw(",\"body\":{\"stringValue\":"),
                Str(&self.description),
                Raw("},\"severityNumber\":"),
                Num(u64::from(self.level.to_numeric())),
                Raw(",\"severityText\":"),
                Str(self.level.as_str()),
                Raw(",\"spanId\":"),
                Value(span_id),
                Raw(",\"timeUnixNano\":"),
                Str(&self.time),
                Raw(",\"traceId\":"),
                Value(trace_id),
                Raw("}"),
            ],
        )
    }

    fn fmt_ecs(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(
            f,
            &[
                Raw("{\"@timestamp\":"),
                Str(&self.time),
                Raw(",\"labels\":"),
                Map(&self.attributes),
                Raw(",\"log.level\":"),
                Str(self.level.as_str_lowercase()),
                Raw(",\"log.logger\":\"rlg\",\"message\":"),
                Str(&self.description),
                Raw(",\"process.name\":"),
                Str(&self.component),
                Raw("}"),
            ],
        )
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
mod tests;
