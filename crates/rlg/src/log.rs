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

mod source;
use rlg_core::{Format, Record};

/// `file:line` for a call site, built without the `format!` machinery.
/// The flusher calls it once per record `fire()` sent; under Miri,
/// which runs no flusher, nothing does.
#[cfg(not(miri))]
pub(crate) fn caller_string(
    caller: &std::panic::Location<'_>,
) -> String {
    source::source_location(caller.file(), caller.line())
}

/// `file:line` for a source location, with an absolute path cut to
/// start at its crate (see `source::source_path`).
#[cfg(feature = "tracing-layer")]
pub(crate) fn source_location(file: &str, line: u32) -> String {
    source::source_location(file, line)
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
}

// --- Rendering, delegated to `rlg-core` ---
impl Log {
    /// This entry as an `rlg-core` record in `format`. The host name
    /// is resolved only for the Apache format, the one that prints it.
    fn record(&self, format: Format) -> Record<'_> {
        let host = if format == Format::ApacheAccessLog {
            CACHED_HOSTNAME.as_str()
        } else {
            ""
        };
        Record {
            session_id: self.session_id,
            time: &self.time,
            level: self.level,
            component: &self.component,
            description: &self.description,
            format,
            attributes: &self.attributes,
            host,
        }
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `Format` declares the same variants as `LogFormat`, in the
        // same order; `log/tests.rs` checks every pair exhaustively.
        self.record(Format::ALL[self.format as usize]).write_to(f)
    }
}

#[cfg(test)]
mod tests;
