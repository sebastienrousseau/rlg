// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Aggregation helpers for `rlg` log streams.
//!
//! The companion `rlg-report` binary wires these helpers to a CLI.
//! The library surface is exposed so dashboards / web UIs / tests
//! can build their own front-ends on top.
//!
//! ```
//! use rlg_report::Report;
//!
//! let lines = [
//!     r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#,
//!     r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":120}}"#,
//!     r#"{"session_id":3,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":80}}"#,
//! ];
//! let report = Report::from_lines(lines.iter().copied());
//! assert_eq!(report.total, 3);
//! assert_eq!(report.count_by_level.get("ERROR").copied(), Some(2));
//! assert_eq!(report.count_by_component.get("db").copied(), Some(2));
//! assert_eq!(report.top_descriptions[0].0, "boom");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use rlg::log_level::LogLevel;
use rlg_cli::parse_record;
use std::collections::BTreeMap;

/// Aggregated digest of a log stream.
#[derive(Debug, Default, Clone)]
pub struct Report {
    /// Total parseable records seen.
    pub total: u64,
    /// Records that failed to parse as canonical JSON `Log`.
    pub unparseable: u64,
    /// Count of records grouped by level (string keys: `INFO`, `ERROR`, …).
    pub count_by_level: BTreeMap<String, u64>,
    /// Count grouped by `component`.
    pub count_by_component: BTreeMap<String, u64>,
    /// Top descriptions by frequency, descending. Top 10 by default.
    pub top_descriptions: Vec<(String, u64)>,
    /// Latency stats (ms) extracted from the `latency_ms` /
    /// `http.latency_ms` attribute when present. `None` when no
    /// records carried such an attribute.
    pub latency: Option<LatencyStats>,
}

/// p50 / p95 / p99 / max latency percentiles, all in milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatencyStats {
    /// Number of records that carried a `latency_ms` attribute.
    pub samples: usize,
    /// 50th percentile (median).
    pub p50: u64,
    /// 95th percentile.
    pub p95: u64,
    /// 99th percentile.
    pub p99: u64,
    /// Maximum.
    pub max: u64,
}

impl Report {
    /// Aggregate a stream of JSON-shaped record lines into a [`Report`].
    /// Lines that don't parse as canonical JSON `Log` are counted as
    /// `unparseable` and otherwise ignored.
    pub fn from_lines<'a, I>(lines: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self::from_lines_with_top(lines, 10)
    }

    /// Same as [`Self::from_lines`] but lets the caller pick how many
    /// top descriptions to keep.
    pub fn from_lines_with_top<'a, I>(lines: I, top_n: usize) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut report = Self::default();
        let mut descriptions: BTreeMap<String, u64> = BTreeMap::new();
        let mut latencies: Vec<u64> = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(record) = parse_record(trimmed) else {
                report.unparseable += 1;
                continue;
            };
            report.total += 1;
            *report
                .count_by_level
                .entry(record.level.to_string())
                .or_insert(0) += 1;
            *report
                .count_by_component
                .entry(record.component.to_string())
                .or_insert(0) += 1;
            *descriptions
                .entry(record.description.clone())
                .or_insert(0) += 1;
            for key in ["latency_ms", "http.latency_ms"] {
                if let Some(v) = record.attributes.get(key)
                    && let Some(ms) = v.as_u64()
                {
                    latencies.push(ms);
                }
            }
        }

        let mut top: Vec<(String, u64)> =
            descriptions.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        top.truncate(top_n);
        report.top_descriptions = top;

        if !latencies.is_empty() {
            report.latency = Some(percentiles(&mut latencies));
        }
        report
    }

    /// Render the report as a human-readable text table.
    #[must_use]
    pub fn to_text(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "── rlg report ───────────────────────────────────────────"
        );
        let _ = writeln!(out, "total records:      {}", self.total);
        let _ =
            writeln!(out, "unparseable lines:  {}", self.unparseable);
        let _ = writeln!(out, "\n── by level ─────────────────");
        for (level, count) in &self.count_by_level {
            let _ = writeln!(out, "  {level:<10} {count}");
        }
        let _ = writeln!(out, "\n── by component ─────────────");
        for (component, count) in &self.count_by_component {
            let _ = writeln!(out, "  {component:<10} {count}");
        }
        let _ = writeln!(out, "\n── top descriptions ─────────");
        for (desc, count) in &self.top_descriptions {
            let _ = writeln!(out, "  {count:>5}  {desc}");
        }
        if let Some(l) = &self.latency {
            let _ = writeln!(out, "\n── latency (ms) ─────────────");
            let _ = writeln!(out, "  samples  {}", l.samples);
            let _ = writeln!(out, "  p50      {}", l.p50);
            let _ = writeln!(out, "  p95      {}", l.p95);
            let _ = writeln!(out, "  p99      {}", l.p99);
            let _ = writeln!(out, "  max      {}", l.max);
        }
        out
    }

    /// Render the report as JSON.
    ///
    /// # Errors
    /// Returns `serde_json::Error` only if the serialiser fails,
    /// which cannot happen for the report's fixed shape.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let value = serde_json::json!({
            "total":              self.total,
            "unparseable":        self.unparseable,
            "count_by_level":     self.count_by_level,
            "count_by_component": self.count_by_component,
            "top_descriptions":   self.top_descriptions,
            "latency":            self.latency.map(|l| serde_json::json!({
                "samples": l.samples,
                "p50":     l.p50,
                "p95":     l.p95,
                "p99":     l.p99,
                "max":     l.max,
            })),
        });
        serde_json::to_string_pretty(&value)
    }

    /// Count of records at level ERROR-and-above.
    #[must_use]
    pub fn error_count(&self) -> u64 {
        let threshold = LogLevel::ERROR.to_numeric();
        self.count_by_level
            .iter()
            .filter_map(|(name, count)| {
                name.parse::<LogLevel>()
                    .ok()
                    .filter(|l| l.to_numeric() >= threshold)
                    .map(|_| *count)
            })
            .sum()
    }
}

#[allow(clippy::cast_possible_truncation)]
fn percentiles(values: &mut [u64]) -> LatencyStats {
    values.sort_unstable();
    let samples = values.len();
    let pick = |q: f64| -> u64 {
        let idx = (((samples as f64) * q).ceil() as usize)
            .saturating_sub(1)
            .min(samples - 1);
        values[idx]
    };
    LatencyStats {
        samples,
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        max: *values.last().unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INFO: &str = r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#;
    const ERROR_BOOM_1: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":120}}"#;
    const ERROR_BOOM_2: &str = r#"{"session_id":3,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":80}}"#;
    const FATAL: &str = r#"{"session_id":4,"time":"t","level":"FATAL","component":"db","description":"crash","format":"JSON","attributes":{}}"#;
    const HTTP: &str = r#"{"session_id":5,"time":"t","level":"INFO","component":"api","description":"GET /","format":"JSON","attributes":{"http.latency_ms":42}}"#;

    #[test]
    fn empty_input_is_a_zero_report() {
        let r = Report::from_lines(std::iter::empty::<&str>());
        assert_eq!(r.total, 0);
        assert!(r.count_by_level.is_empty());
        assert!(r.latency.is_none());
    }

    #[test]
    fn counts_records_by_level_and_component() {
        let r = Report::from_lines([
            INFO,
            ERROR_BOOM_1,
            ERROR_BOOM_2,
            FATAL,
            HTTP,
        ]);
        assert_eq!(r.total, 5);
        assert_eq!(r.count_by_level.get("INFO").copied(), Some(2));
        assert_eq!(r.count_by_level.get("ERROR").copied(), Some(2));
        assert_eq!(r.count_by_level.get("FATAL").copied(), Some(1));
        assert_eq!(r.count_by_component.get("db").copied(), Some(3));
        assert_eq!(r.count_by_component.get("svc").copied(), Some(1));
        assert_eq!(r.count_by_component.get("api").copied(), Some(1));
    }

    #[test]
    fn top_descriptions_ranks_by_frequency() {
        let r = Report::from_lines([
            INFO,
            ERROR_BOOM_1,
            ERROR_BOOM_2,
            FATAL,
        ]);
        // "boom" appears twice, "hi" and "crash" once each.
        assert_eq!(r.top_descriptions[0].0, "boom");
        assert_eq!(r.top_descriptions[0].1, 2);
    }

    #[test]
    fn latency_percentiles_are_sorted() {
        let r = Report::from_lines([ERROR_BOOM_1, ERROR_BOOM_2, HTTP]);
        let l = r.latency.expect("latency stats present");
        assert_eq!(l.samples, 3);
        assert_eq!(l.max, 120);
        // p50 of [42, 80, 120] is 80; p95 / p99 land on 120.
        assert_eq!(l.p50, 80);
        assert_eq!(l.p95, 120);
        assert_eq!(l.p99, 120);
    }

    #[test]
    fn unparseable_lines_are_counted_separately() {
        let r =
            Report::from_lines([INFO, "not json at all", "", FATAL]);
        assert_eq!(r.total, 2);
        assert_eq!(r.unparseable, 1);
    }

    #[test]
    fn to_text_contains_section_headers() {
        let r = Report::from_lines([INFO, ERROR_BOOM_1]);
        let text = r.to_text();
        assert!(text.contains("by level"));
        assert!(text.contains("by component"));
        assert!(text.contains("top descriptions"));
        assert!(text.contains("latency"));
    }

    #[test]
    fn to_json_round_trips() {
        let r = Report::from_lines([INFO, ERROR_BOOM_1]);
        let json = r.to_json().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["total"], 2);
        assert_eq!(v["count_by_level"]["INFO"], 1);
    }

    #[test]
    fn error_count_sums_error_and_above() {
        let r = Report::from_lines([
            INFO,
            ERROR_BOOM_1,
            ERROR_BOOM_2,
            FATAL,
        ]);
        assert_eq!(r.error_count(), 3);
    }

    #[test]
    fn top_n_can_be_clamped() {
        let lines = [INFO, ERROR_BOOM_1, ERROR_BOOM_2, FATAL, HTTP];
        let r = Report::from_lines_with_top(lines, 1);
        assert_eq!(r.top_descriptions.len(), 1);
    }

    #[test]
    fn http_latency_attribute_is_picked_up() {
        let r = Report::from_lines([HTTP]);
        let l = r.latency.expect("latency present");
        assert_eq!(l.samples, 1);
        assert_eq!(l.max, 42);
    }
}
