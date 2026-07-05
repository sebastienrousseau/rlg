// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the [`Redactor`] end-to-end scrub path,
//! exercising built-in patterns, custom patterns, marker override,
//! recursive JSON scrubbing, and non-string attribute pass-through.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_redact::{
    AWS_ACCESS_KEY, BEARER_TOKEN, CREDIT_CARD, EMAIL, IPV4, JWT,
    Redactor,
};

#[test]
fn with_defaults_scrubs_credit_card_from_description() {
    let r = Redactor::with_defaults();
    let log = Log::info("card 4111-1111-1111-1111 failed");
    let scrubbed = r.scrub(log);
    assert!(scrubbed.description.contains("[REDACTED]"));
    assert!(!scrubbed.description.contains("4111"));
}

#[test]
fn with_defaults_scrubs_email_from_description() {
    let r = Redactor::with_defaults();
    let log = Log::info("bounce for user@example.com");
    let scrubbed = r.scrub(log);
    assert!(scrubbed.description.contains("[REDACTED]"));
    assert!(!scrubbed.description.contains("user@example.com"));
}

#[test]
fn with_defaults_scrubs_jwt_from_description() {
    let r = Redactor::with_defaults();
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.abcdef";
    let log = Log::info(&format!("Authorization: {jwt}"));
    let scrubbed = r.scrub(log);
    assert!(scrubbed.description.contains("[REDACTED]"));
    assert!(!scrubbed.description.contains(jwt));
}

#[test]
fn with_defaults_scrubs_string_attributes() {
    let r = Redactor::with_defaults();
    let log = Log::info("login failed")
        .with("email", "user@example.com")
        .with("client_ip", "10.0.1.42");
    let scrubbed = r.scrub(log);
    let email = scrubbed.attributes.get("email").unwrap();
    let ip = scrubbed.attributes.get("client_ip").unwrap();
    assert_eq!(email.as_str().unwrap(), "[REDACTED]");
    assert_eq!(ip.as_str().unwrap(), "[REDACTED]");
}

#[test]
fn non_string_attributes_pass_through_unchanged() {
    let r = Redactor::with_defaults();
    let log = Log::info("event")
        .with("user_id", 42_u64)
        .with("elapsed_ms", 1234_i64)
        .with("active", true);
    let scrubbed = r.scrub(log);
    assert_eq!(
        scrubbed.attributes.get("user_id").unwrap().as_u64(),
        Some(42)
    );
    assert_eq!(
        scrubbed.attributes.get("elapsed_ms").unwrap().as_i64(),
        Some(1234)
    );
    assert_eq!(
        scrubbed.attributes.get("active").unwrap().as_bool(),
        Some(true)
    );
}

#[test]
fn nested_json_array_is_scrubbed_recursively() {
    let r = Redactor::with_defaults();
    let payload = serde_json::json!([
        "4111-1111-1111-1111",
        "safe",
        "user@example.com"
    ]);
    let log = Log::info("event").with("payload", payload);
    let scrubbed = r.scrub(log);
    let arr = scrubbed
        .attributes
        .get("payload")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(arr[0].as_str().unwrap(), "[REDACTED]");
    assert_eq!(arr[1].as_str().unwrap(), "safe");
    assert_eq!(arr[2].as_str().unwrap(), "[REDACTED]");
}

#[test]
fn nested_json_object_is_scrubbed_recursively() {
    let r = Redactor::with_defaults();
    let payload = serde_json::json!({
        "card": "4111 1111 1111 1111",
        "note": "no PII here"
    });
    let log = Log::info("event").with("payload", payload);
    let scrubbed = r.scrub(log);
    let obj = scrubbed
        .attributes
        .get("payload")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        obj.get("card").unwrap().as_str().unwrap(),
        "[REDACTED]"
    );
    assert_eq!(
        obj.get("note").unwrap().as_str().unwrap(),
        "no PII here"
    );
}

#[test]
fn custom_pattern_replaces_matches() {
    let r = Redactor::empty()
        .with_pattern(r"(?i)password=\S+")
        .expect("valid regex");
    let out = r.apply("login password=hunter2 attempted");
    assert!(out.contains("[REDACTED]"));
    assert!(!out.contains("hunter2"));
}

#[test]
fn custom_marker_replaces_default() {
    let r = Redactor::with_defaults().marker("***");
    let out = r.apply("ip 192.168.0.1");
    assert!(out.contains("***"));
    assert!(!out.contains("[REDACTED]"));
    assert!(!out.contains("192.168.0.1"));
}

#[test]
fn invalid_regex_returns_error() {
    let err = Redactor::empty().with_pattern("[unclosed");
    assert!(err.is_err());
}

#[test]
fn empty_redactor_has_no_patterns() {
    let r = Redactor::empty();
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);
    let log = Log::info("4111-1111-1111-1111");
    let out = r.scrub(log.clone());
    assert_eq!(out.description, log.description);
}

#[test]
fn built_in_pattern_constants_are_exported() {
    // Compile-time contract: the crate exports these constants so
    // downstream projects can reference them in composed redactors.
    let _: &str = CREDIT_CARD;
    let _: &str = JWT;
    let _: &str = BEARER_TOKEN;
    let _: &str = EMAIL;
    let _: &str = IPV4;
    let _: &str = AWS_ACCESS_KEY;
}

#[test]
fn scrub_is_fluent_over_log() {
    let r = Redactor::with_defaults();
    let log = Log::warn("hit AKIAIOSFODNN7EXAMPLE").component("audit");
    let scrubbed = r.scrub(log);
    assert!(!scrubbed.description.contains("AKIAIOSFODNN7EXAMPLE"));
    assert_eq!(scrubbed.component.as_ref(), "audit");
}
