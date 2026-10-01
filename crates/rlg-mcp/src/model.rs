// model.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! What the tools take and return: argument types, whose field docs
//! are what a client shows the model, and the structured results.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Structured outputs.
// ---------------------------------------------------------------------------

/// A slice of rendered log records: what `tail_log`, `filter_log` and
/// `tail_logs_glob` return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Records {
    /// How many records are returned.
    pub count: usize,
    /// The records, oldest first, one rendered record each.
    pub records: Vec<String>,
}

impl From<Vec<String>> for Records {
    fn from(records: Vec<String>) -> Self {
        Self {
            count: records.len(),
            records,
        }
    }
}

impl fmt::Display for Records {
    /// One record per line. When nothing matched, say so: an empty
    /// string would read to a model as a file with nothing in it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.records.is_empty() {
            return f.write_str("No parseable rlg records matched.");
        }
        f.write_str(&self.records.join("\n"))
    }
}

/// ERROR-and-above records counted per component: what
/// `summarize_errors` returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ErrorSummary {
    /// How many ERROR-and-above records the file holds.
    pub total: u64,
    /// The count per component, sorted by component name.
    pub by_component: BTreeMap<String, u64>,
}

impl From<BTreeMap<String, u64>> for ErrorSummary {
    fn from(by_component: BTreeMap<String, u64>) -> Self {
        Self {
            total: by_component.values().sum(),
            by_component,
        }
    }
}

impl fmt::Display for ErrorSummary {
    /// The `component → count` map as pretty-printed JSON, which is
    /// the text this tool has always returned.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(&self.by_component)
            .unwrap_or_default();
        f.write_str(&text)
    }
}

// ---------------------------------------------------------------------------
// Tool arguments.
// ---------------------------------------------------------------------------
//
// The doc comments on the fields are the descriptions a client shows
// the model, kept word for word from the previous release. The
// examples are what an auditor or a client with no log of its own
// sends: a path a log file can be written to on any Unix host.

fn default_count() -> usize {
    100
}

fn default_format() -> String {
    "Logfmt".to_owned()
}

/// Arguments of `tail_log`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TailLogArgs {
    /// Filesystem path to an rlg log file (Logfmt/JSON records, one per line).
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
    /// How many of the most recent parseable records to return (default 100).
    #[serde(default = "default_count")]
    #[schemars(range(min = 1), example = &10)]
    pub n: usize,
}

/// Arguments of `filter_log`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilterLogArgs {
    /// Filesystem path to an rlg log file to read.
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
    /// Keep only records at or above this severity. Omit to keep all levels.
    #[serde(default)]
    #[schemars(
        with = "String",
        extend("enum" = ["TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR", "FATAL", "CRITICAL"]),
        example = &"WARN"
    )]
    pub min_level: Option<String>,
    /// Keep only records whose component matches this exact value. Omit to keep all components.
    #[serde(default)]
    #[schemars(with = "String", example = &"db")]
    pub component: Option<String>,
    /// rlg LogFormat name to render matched records in (e.g. Logfmt, JSON). Defaults to Logfmt.
    #[serde(default = "default_format")]
    #[schemars(example = &"JSON")]
    pub format: String,
}

/// Arguments of `summarize_errors`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SummarizeErrorsArgs {
    /// Filesystem path to an rlg log file to scan for ERROR-and-above records.
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
}

/// Arguments of `tail_logs_glob`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TailLogsGlobArgs {
    /// Glob pattern matching one or more rlg log files, e.g. `/var/log/**/*.log`.
    #[schemars(example = &"/tmp/rlg/*.ndjson")]
    pub glob_pattern: String,
    /// How many of the most recent parseable records to return across all matched files (default 100).
    #[serde(default = "default_count")]
    #[schemars(range(min = 1), example = &10)]
    pub lines: usize,
    /// Keep only records at or above this severity. Omit to keep all levels.
    #[serde(default)]
    #[schemars(
        with = "String",
        extend("enum" = ["TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR", "FATAL", "CRITICAL"]),
        example = &"ERROR"
    )]
    pub level: Option<String>,
}
