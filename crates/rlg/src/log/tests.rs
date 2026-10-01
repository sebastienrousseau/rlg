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

#[track_caller]
fn here() -> &'static std::panic::Location<'static> {
    std::panic::Location::caller()
}

#[test]
fn caller_string_is_file_colon_line() {
    for location in [here(), here(), here()] {
        assert_eq!(
            caller_string(location),
            format!("{}:{}", location.file(), location.line())
        );
    }
    let (location, line) = (here(), line!());
    assert!(
        caller_string(location).ends_with(&format!("tests.rs:{line}"))
    );
}

/// `fire()` leaves the attributes alone on the caller's thread and
/// carries the call site to the flusher as a `&'static Location`.
#[test]
fn fired_event_carries_the_call_site_unrendered() {
    let line = line!() + 1;
    let event = Log::info("msg").with("k", 1).into_fired_event();
    let caller = event.caller.expect("fire() records its call site");
    assert!(caller.file().ends_with("tests.rs"), "{}", caller.file());
    assert_eq!(caller.line(), line);
    assert!(!event.log.attributes.contains_key("caller"));
    assert_eq!(event.level_num, LogLevel::INFO.to_numeric());
}
