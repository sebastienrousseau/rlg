// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Downstream-consumer integration test.
//!
//! Verifies the crate's public surface (`capture`, `LogExt`,
//! `assert_logged!`, free predicates) works when consumed from a
//! separate crate — the same call-site pattern real downstream
//! projects use, exercised through Rust's black-box integration
//! test model.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_level::LogLevel;
use rlg_test::{
    Capture, LogExt, assert_logged, attribute_eq, capture,
    description_contains, has_component, has_level,
};

#[test]
fn capture_starts_empty() {
    let c = capture();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
    assert!(c.records().is_empty());
}

#[test]
fn log_ext_routes_records_into_handle() {
    let c = capture();
    Log::info("first").log_to(&c);
    Log::warn("second").log_to(&c);
    Log::error("third").log_to(&c);
    assert_eq!(c.len(), 3);
    assert!(!c.is_empty());
}

#[test]
fn assert_logged_level_matches() {
    let c = capture();
    Log::info("hi").log_to(&c);
    assert_logged!(c, level == LogLevel::INFO);
}

#[test]
fn assert_logged_contains_matches() {
    let c = capture();
    Log::info("user authenticated").log_to(&c);
    assert_logged!(c, contains "authenticated");
}

#[test]
fn assert_logged_component_matches() {
    let c = capture();
    Log::info("boot").component("api").log_to(&c);
    assert_logged!(c, component "api");
}

#[test]
fn assert_logged_attribute_matches() {
    let c = capture();
    Log::info("payment").with("user_id", 42_u64).log_to(&c);
    assert_logged!(c, attribute "user_id" => 42_u64);
}

#[test]
fn assert_logged_len_matches() {
    let c = capture();
    for i in 0..5_u32 {
        Log::info(&format!("event-{i}")).log_to(&c);
    }
    assert_logged!(c, len == 5);
}

#[test]
fn free_predicates_are_accessible() {
    let c = capture();
    Log::error("boom")
        .component("orders")
        .with("order_id", "xyz")
        .log_to(&c);

    assert!(has_level(&c, LogLevel::ERROR));
    assert!(!has_level(&c, LogLevel::INFO));

    assert!(description_contains(&c, "boom"));
    assert!(!description_contains(&c, "missing"));

    assert!(has_component(&c, "orders"));
    assert!(!has_component(&c, "billing"));

    assert!(attribute_eq(&c, "order_id", "xyz"));
    assert!(!attribute_eq(&c, "order_id", "other"));
}

#[test]
fn capture_clone_shares_buffer() {
    // The docs promise cheap clones that share the underlying buffer.
    let c1 = capture();
    let c2 = c1.clone();
    Log::info("hi").log_to(&c1);
    assert_eq!(c2.len(), 1);
    Log::warn("bye").log_to(&c2);
    assert_eq!(c1.len(), 2);
}

#[test]
fn clear_drops_all_records() {
    let c = capture();
    Log::info("one").log_to(&c);
    Log::info("two").log_to(&c);
    assert_eq!(c.len(), 2);
    c.clear();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}

#[test]
fn default_construction_matches_new() {
    let a: Capture = Capture::default();
    let b: Capture = Capture::new();
    assert!(a.is_empty());
    assert!(b.is_empty());
}

#[test]
#[should_panic(expected = "expected a captured record at level")]
fn assert_logged_panics_on_missing_level() {
    let c = capture();
    Log::info("only info").log_to(&c);
    // ERROR was never logged — assertion must panic.
    assert_logged!(c, level == LogLevel::ERROR);
}
