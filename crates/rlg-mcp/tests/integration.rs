// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the three MCP tools (`tail_log`,
//! `filter_log`, `summarize_errors`) and the JSON-RPC 2.0
//! dispatcher.
//!
//! The tools are pure functions over a file path, so the tests
//! write NDJSON fixtures to a temp file and assert on the tool
//! outputs directly — no need to spawn the stdio transport loop.

#![allow(missing_docs)]

use rlg_cli::Filter;
use rlg_mcp::{
    Request, dispatch, filter_log, summarize_errors, tail_log,
};
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
// dispatch (JSON-RPC 2.0)
// ---------------------------------------------------------------------------

fn req(method: &str, id: Option<serde_json::Value>) -> Request {
    Request {
        jsonrpc: "2.0".into(),
        id,
        method: method.into(),
        params: json!({}),
    }
}

#[test]
fn dispatch_initialize_returns_protocol_version() {
    let r = dispatch(&req("initialize", Some(json!(1))))
        .expect("initialize has a response");
    let result = r.result.expect("ok result");
    assert_eq!(result["protocolVersion"], "2025-06-18");
    assert!(
        result["serverInfo"]["name"]
            .as_str()
            .unwrap()
            .starts_with("rlg-mcp")
    );
}

#[test]
fn dispatch_tools_list_returns_three_tools() {
    let r = dispatch(&req("tools/list", Some(json!(2))))
        .expect("tools/list has a response");
    let result = r.result.expect("ok result");
    let tools = result["tools"].as_array().expect("tools array");
    let names: Vec<&str> =
        tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"tail_log"));
    assert!(names.contains(&"filter_log"));
    assert!(names.contains(&"summarize_errors"));
}

#[test]
fn dispatch_notifications_return_none() {
    assert!(
        dispatch(&req("notifications/initialized", None)).is_none()
    );
    assert!(dispatch(&req("notifications/cancelled", None)).is_none());
}

#[test]
fn dispatch_unknown_method_returns_error() {
    let r = dispatch(&req("bogus/method", Some(json!(3))))
        .expect("has a response");
    assert!(r.error.is_some(), "expected error variant");
    assert!(r.result.is_none());
}
