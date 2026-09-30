// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `log.rs`.

use super::*;
use crate::log_format::LogFormat;

#[test]
fn resolve_hostname_uses_localhost_on_error() {
    let err = Err(std::io::Error::other("no hostname"));
    assert_eq!(resolve_hostname(err), "localhost");
}

#[test]
fn resolve_hostname_returns_provided_value() {
    let raw = Ok(std::ffi::OsString::from("test-host"));
    assert_eq!(resolve_hostname(raw), "test-host");
}

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
    log.attributes
        .insert("space".to_string(), serde_json::json!("has space"));
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
