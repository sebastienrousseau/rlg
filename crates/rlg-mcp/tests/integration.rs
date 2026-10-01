// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the four MCP tools as plain functions
//! (`tail_log`, `filter_log`, `summarize_errors`, `tail_logs_glob`).
//!
//! The tools are pure functions over a file path, so the tests
//! write NDJSON fixtures to a temp file and assert on the tool
//! outputs directly — no transport involved. The protocol around
//! them is covered by `tests/serve.rs` (an in-memory session),
//! `tests/server.rs` (the binary over stdio) and `tests/http.rs`
//! (the two HTTP transports).

#![allow(missing_docs)]

use rlg_cli::Filter;
use rlg_mcp::{filter_log, summarize_errors, tail_log, tail_logs_glob};
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

/// Write NDJSON fixtures to a temp file. Each record is the
/// canonical `LogFormat::JSON` shape that `parse_record` accepts.
fn fixture(records: &[serde_json::Value]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    for r in records {
        writeln!(f, "{r}").expect("write");
    }
    f.flush().expect("flush");
    f
}

fn record(
    session_id: u64,
    level: &str,
    component: &str,
    description: &str,
) -> serde_json::Value {
    json!({
        "session_id": session_id,
        "time": "2026-07-04T00:00:00.000000000Z",
        "level": level,
        "component": component,
        "description": description,
        "format": "JSON",
        "attributes": {}
    })
}

// ---------------------------------------------------------------------------
// tail_log
// ---------------------------------------------------------------------------

#[test]
fn tail_log_returns_last_n_records() {
    let f = fixture(&[
        record(1, "INFO", "svc", "first"),
        record(2, "INFO", "svc", "second"),
        record(3, "INFO", "svc", "third"),
        record(4, "INFO", "svc", "fourth"),
    ]);
    let out = tail_log(f.path(), 2).expect("tail_log");
    assert_eq!(out.len(), 2);
    assert!(out[0].contains("third"));
    assert!(out[1].contains("fourth"));
}

#[test]
fn tail_log_returns_all_when_n_exceeds_record_count() {
    let f = fixture(&[
        record(1, "INFO", "svc", "one"),
        record(2, "INFO", "svc", "two"),
    ]);
    let out = tail_log(f.path(), 100).expect("tail_log");
    assert_eq!(out.len(), 2);
}

#[test]
fn tail_log_skips_unparseable_lines() {
    let mut f = NamedTempFile::new().expect("tempfile");
    writeln!(f, "{}", record(1, "INFO", "svc", "valid")).unwrap();
    writeln!(f, "not-json garbage").unwrap();
    writeln!(f, "{}", record(2, "INFO", "svc", "also valid")).unwrap();
    f.flush().unwrap();
    let out = tail_log(f.path(), 100).expect("tail_log");
    assert_eq!(out.len(), 2);
}

#[test]
fn tail_log_bubbles_open_error() {
    let err = tail_log(
        std::path::Path::new("/definitely/does/not/exist.ndjson"),
        1,
    )
    .expect_err("missing file must error");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

// ---------------------------------------------------------------------------
// filter_log
// ---------------------------------------------------------------------------

#[test]
fn filter_log_matches_component() {
    let f = fixture(&[
        record(1, "INFO", "auth", "login"),
        record(2, "INFO", "db", "connect"),
        record(3, "INFO", "auth", "logout"),
    ]);
    let filter = Filter::new().component("auth");
    let out = filter_log(
        f.path(),
        &filter,
        rlg::log_format::LogFormat::Logfmt,
    )
    .expect("filter_log");
    assert_eq!(out.len(), 2);
    // Every rendered line must mention "auth" — the concrete Logfmt
    // key ("component", "svc", etc.) is a render-detail we don't
    // pin here, but the substring must survive filtering.
    assert!(out.iter().all(|line| line.contains("auth")));
}

#[test]
fn filter_log_matches_min_level() {
    let f = fixture(&[
        record(1, "INFO", "svc", "info-level"),
        record(2, "WARN", "svc", "warn-level"),
        record(3, "ERROR", "svc", "error-level"),
    ]);
    let filter =
        Filter::new().min_level(rlg::log_level::LogLevel::WARN);
    let out = filter_log(
        f.path(),
        &filter,
        rlg::log_format::LogFormat::Logfmt,
    )
    .expect("filter_log");
    assert_eq!(out.len(), 2);
}

#[test]
fn filter_log_empty_filter_matches_everything() {
    let f = fixture(&[
        record(1, "INFO", "a", "x"),
        record(2, "WARN", "b", "y"),
    ]);
    let filter = Filter::new();
    let out =
        filter_log(f.path(), &filter, rlg::log_format::LogFormat::JSON)
            .expect("filter_log");
    assert_eq!(out.len(), 2);
}

// ---------------------------------------------------------------------------
// summarize_errors
// ---------------------------------------------------------------------------

#[test]
fn summarize_errors_groups_by_component() {
    let f = fixture(&[
        record(1, "INFO", "auth", "not counted"),
        record(2, "ERROR", "auth", "boom"),
        record(3, "ERROR", "auth", "boom again"),
        record(4, "FATAL", "db", "kaboom"),
        record(5, "WARN", "auth", "not counted"),
    ]);
    let buckets = summarize_errors(f.path()).expect("summarize");
    assert_eq!(buckets.get("auth"), Some(&2));
    assert_eq!(buckets.get("db"), Some(&1));
    assert!(!buckets.contains_key("svc"));
}

#[test]
fn summarize_errors_on_empty_file_yields_empty_map() {
    let f = NamedTempFile::new().unwrap();
    let buckets = summarize_errors(f.path()).expect("summarize");
    assert!(buckets.is_empty());
}

// ---------------------------------------------------------------------------
// tail_logs_glob
// ---------------------------------------------------------------------------

#[test]
fn tail_logs_glob_reads_files_in_path_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.log"),
        format!("{}\n", record(1, "INFO", "svc", "from a")),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.log"),
        format!("{}\n", record(2, "ERROR", "svc", "from b")),
    )
    .unwrap();
    let pattern = format!("{}/*.log", dir.path().display());
    let out = tail_logs_glob(&pattern, 10, None).expect("glob");
    assert_eq!(out.len(), 2);
    assert!(out[0].contains("from a"));
    assert!(out[1].contains("from b"));

    let errors = tail_logs_glob(
        &pattern,
        10,
        Some(rlg::log_level::LogLevel::ERROR),
    )
    .expect("glob");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("from b"));
}

#[test]
fn tail_logs_glob_matching_nothing_is_empty_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let pattern = format!("{}/*.nothing", dir.path().display());
    let out = tail_logs_glob(&pattern, 10, None).expect("glob");
    assert!(out.is_empty());
}
