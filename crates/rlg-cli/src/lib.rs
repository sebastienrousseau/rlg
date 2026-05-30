// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Shared parsing, filtering, and re-emission for the `rlg` CLI.
//!
//! Splitting these helpers into a library lets `rlg-mcp` (and any
//! other downstream tool) reuse the same record pipeline without
//! depending on `clap` or the binary entry point.
//!
//! # Example
//!
//! ```
//! use rlg_cli::{parse_record, Filter};
//! use rlg::log_level::LogLevel;
//!
//! let line = r#"{"session_id":1,"time":"2026-05-30T00:00:00.000000000Z","level":"INFO","component":"svc","description":"hello","format":"JSON","attributes":{}}"#;
//! let record = parse_record(line).unwrap();
//! let filter = Filter::new().min_level(LogLevel::WARN);
//! assert!(!filter.matches(&record));
//! ```

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

/// Parse a single line as an [`rlg::log::Log`] record.
///
/// Accepts the canonical `LogFormat::JSON` shape — the same one that
/// `rlg`'s `Display` impl emits for `LogFormat::JSON` and `NDJSON`.
/// Other rlg formats (MCP, OTLP, ECS, …) wrap the underlying record
/// inside a transport envelope and are not yet parsed back to a
/// `Log` (see [crate-level docs](crate) for the roadmap).
///
/// # Errors
///
/// Returns `serde_json::Error` if the input is not valid JSON in the
/// canonical shape.
pub fn parse_record(line: &str) -> Result<Log, serde_json::Error> {
    serde_json::from_str(line.trim())
}

/// Filter criteria applied to each record as it streams past.
#[derive(Debug, Default, Clone)]
pub struct Filter {
    /// Minimum log level (inclusive). Records below this level are
    /// dropped.
    pub min_level: Option<LogLevel>,
    /// Optional component name. Only records with this exact
    /// `component` value pass.
    pub component: Option<String>,
    /// Optional `(key, value)` attribute match. Only records whose
    /// attributes map contains this exact pairing pass.
    pub attribute: Option<(String, serde_json::Value)>,
}

impl Filter {
    /// Construct a filter that lets every record through.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_level: None,
            component: None,
            attribute: None,
        }
    }

    /// Set the minimum severity.
    #[must_use]
    pub const fn min_level(mut self, level: LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Restrict to a single component.
    #[must_use]
    pub fn component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Restrict to records carrying a specific attribute key/value.
    #[must_use]
    pub fn attribute(
        mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.attribute = Some((key.into(), value));
        self
    }

    /// Does this record satisfy every active criterion?
    #[must_use]
    pub fn matches(&self, record: &Log) -> bool {
        if let Some(min) = self.min_level
            && record.level.to_numeric() < min.to_numeric()
        {
            return false;
        }
        if let Some(comp) = &self.component
            && record.component.as_ref() != comp.as_str()
        {
            return false;
        }
        if let Some((key, val)) = &self.attribute
            && record.attributes.get(key) != Some(val)
        {
            return false;
        }
        true
    }
}

/// Re-emit a record using the chosen [`LogFormat`].
///
/// The record's `format` field is overwritten so the wire shape
/// follows `format`, not whatever was in the input.
#[must_use]
pub fn render(mut record: Log, format: LogFormat) -> String {
    record.format = format;
    format!("{record}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "session_id": 7,
            "time": "2026-05-30T00:00:00.000000000Z",
            "level": "INFO",
            "component": "svc",
            "description": "hi",
            "format": "JSON",
            "attributes": {"user_id": 42, "region": "eu-west-1"}
        }"#
    }

    #[test]
    fn parse_returns_log_struct() {
        let log = parse_record(sample_json()).unwrap();
        assert_eq!(log.session_id, 7);
        assert_eq!(log.component.as_ref(), "svc");
        assert_eq!(log.level, LogLevel::INFO);
    }

    #[test]
    fn filter_default_passes_everything() {
        let log = parse_record(sample_json()).unwrap();
        assert!(Filter::new().matches(&log));
    }

    #[test]
    fn filter_min_level_drops_below() {
        let log = parse_record(sample_json()).unwrap();
        assert!(!Filter::new().min_level(LogLevel::WARN).matches(&log));
        assert!(Filter::new().min_level(LogLevel::INFO).matches(&log));
        assert!(Filter::new().min_level(LogLevel::DEBUG).matches(&log));
    }

    #[test]
    fn filter_component_exact_match() {
        let log = parse_record(sample_json()).unwrap();
        assert!(Filter::new().component("svc").matches(&log));
        assert!(!Filter::new().component("other").matches(&log));
    }

    #[test]
    fn filter_attribute_exact_match() {
        let log = parse_record(sample_json()).unwrap();
        assert!(
            Filter::new()
                .attribute("user_id", serde_json::json!(42))
                .matches(&log)
        );
        assert!(
            !Filter::new()
                .attribute("user_id", serde_json::json!(99))
                .matches(&log)
        );
        assert!(
            !Filter::new()
                .attribute("missing", serde_json::json!(true))
                .matches(&log)
        );
    }

    #[test]
    fn render_overrides_format() {
        let log = parse_record(sample_json()).unwrap();
        let out = render(log, LogFormat::Logfmt);
        assert!(out.contains("level=info"));
        assert!(out.contains("session_id=7"));
    }

    #[test]
    fn render_to_mcp_wraps_in_jsonrpc() {
        let log = parse_record(sample_json()).unwrap();
        let out = render(log, LogFormat::MCP);
        assert!(out.contains("\"jsonrpc\":\"2.0\""));
        assert!(out.contains("notifications/log"));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_record("not json at all").is_err());
    }
}
