// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test utilities for downstream crates that depend on `rlg`.
//!
//! ```
//! use rlg_test::{assert_logged, capture, LogExt};
//! use rlg::log::Log;
//! use rlg::log_level::LogLevel;
//!
//! let capture = capture();
//! Log::info("user authenticated")
//!     .component("auth")
//!     .with("user_id", 42_u64)
//!     .log_to(&capture);
//!
//! assert_logged!(capture, level == LogLevel::INFO);
//! assert_logged!(capture, contains "authenticated");
//! assert_logged!(capture, attribute "user_id" => 42_u64);
//! ```
//!
//! Two patterns are supported:
//!
//! 1. **Capture handle** ([`capture`]) — explicitly route records into
//!    the handle via the `log_to` extension method or `capture.push()`.
//!    Works regardless of the global engine state.
//!
//! 2. **Global capture** (not implemented in v0.0.11) — install the
//!    handle as the engine's sink for the test scope. Tracked under
//!    the v0.0.12 roadmap; deferred so this crate doesn't reach into
//!    `rlg`'s engine internals.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use parking_lot::Mutex;
use rlg::log::Log;
use rlg::log_level::LogLevel;
use serde_json::Value;
use std::sync::Arc;

/// In-memory sink that captures every record routed to it.
///
/// Cheap to clone — internally it's an `Arc<Mutex<Vec<Log>>>`, so
/// every clone observes the same buffer.
#[derive(Debug, Clone, Default)]
pub struct Capture {
    inner: Arc<Mutex<Vec<Log>>>,
}

impl Capture {
    /// Construct a fresh, empty capture handle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a record to the captured buffer.
    pub fn push(&self, record: Log) {
        self.inner.lock().push(record);
    }

    /// Return a snapshot of the records captured so far.
    #[must_use]
    pub fn records(&self) -> Vec<Log> {
        self.inner.lock().clone()
    }

    /// How many records have been captured.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    /// Returns `true` if no records have been captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }

    /// Drop every captured record.
    pub fn clear(&self) {
        self.inner.lock().clear();
    }
}

/// Shortcut for `Capture::new()`. Reads more naturally at call sites:
/// `let capture = rlg_test::capture();`.
#[must_use]
pub fn capture() -> Capture {
    Capture::new()
}

/// Extension trait that routes a [`Log`] entry into a [`Capture`]
/// handle in-place of the global engine.
pub trait LogExt {
    /// Push `self` into `capture`.
    fn log_to(self, capture: &Capture);
}

impl LogExt for Log {
    fn log_to(self, capture: &Capture) {
        capture.push(self);
    }
}

// ---------------------------------------------------------------------------
// Predicates — exposed so the assert_logged! macro stays readable.
// ---------------------------------------------------------------------------

/// Did `capture` see at least one record with the given level?
#[must_use]
pub fn has_level(capture: &Capture, level: LogLevel) -> bool {
    capture.inner.lock().iter().any(|r| r.level == level)
}

/// Did `capture` see at least one record whose `description`
/// contains `needle`?
#[must_use]
pub fn description_contains(capture: &Capture, needle: &str) -> bool {
    capture
        .inner
        .lock()
        .iter()
        .any(|r| r.description.contains(needle))
}

/// Did `capture` see at least one record whose `key` attribute
/// equals `expected`?
pub fn attribute_eq<V>(
    capture: &Capture,
    key: &str,
    expected: V,
) -> bool
where
    V: Into<Value>,
{
    let want = expected.into();
    capture
        .inner
        .lock()
        .iter()
        .any(|r| r.attributes.get(key) == Some(&want))
}

/// Did `capture` see at least one record whose `component` matches?
#[must_use]
pub fn has_component(capture: &Capture, component: &str) -> bool {
    capture
        .inner
        .lock()
        .iter()
        .any(|r| r.component.as_ref() == component)
}

// ---------------------------------------------------------------------------
// assert_logged! macro
// ---------------------------------------------------------------------------

/// One-stop assertion macro for verifying that a [`Capture`] handle
/// observed a record matching some predicate.
///
/// Supported forms:
///
/// | Syntax | Predicate |
/// | --- | --- |
/// | `assert_logged!(c, level == L::INFO)` | a record at level INFO exists |
/// | `assert_logged!(c, contains "needle")` | a record's description contains the substring |
/// | `assert_logged!(c, attribute "k" == v)` | a record's `k` attribute equals `v` |
/// | `assert_logged!(c, component "auth")` | a record's component equals `"auth"` |
/// | `assert_logged!(c, len == 3)` | exactly 3 records were captured |
#[macro_export]
macro_rules! assert_logged {
    ($capture:expr, level == $level:expr) => {{
        assert!(
            $crate::has_level(&$capture, $level),
            "expected a captured record at level {:?}, got: {:?}",
            $level,
            $capture.records()
        );
    }};
    ($capture:expr, contains $needle:expr) => {{
        assert!(
            $crate::description_contains(&$capture, $needle),
            "expected a captured record description to contain {:?}, got: {:?}",
            $needle,
            $capture.records()
        );
    }};
    ($capture:expr, attribute $key:expr => $expected:expr) => {{
        assert!(
            $crate::attribute_eq(&$capture, $key, $expected),
            "expected a captured record with attribute {:?} == {:?}, got: {:?}",
            $key,
            $expected,
            $capture.records()
        );
    }};
    ($capture:expr, component $component:expr) => {{
        assert!(
            $crate::has_component(&$capture, $component),
            "expected a captured record with component {:?}, got: {:?}",
            $component,
            $capture.records()
        );
    }};
    ($capture:expr, len == $n:expr) => {{
        let n = $capture.len();
        assert!(
            n == $n,
            "expected {} captured records, got {}: {:?}",
            $n,
            n,
            $capture.records()
        );
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlg::log::Log;

    #[test]
    fn capture_starts_empty() {
        let c = capture();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn log_to_appends() {
        let c = capture();
        Log::info("hi").component("svc").log_to(&c);
        Log::warn("oops").log_to(&c);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn clear_drops_records() {
        let c = capture();
        Log::info("x").log_to(&c);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn predicates_match_levels() {
        let c = capture();
        Log::error("boom").log_to(&c);
        assert!(has_level(&c, LogLevel::ERROR));
        assert!(!has_level(&c, LogLevel::INFO));
    }

    #[test]
    fn predicates_match_components() {
        let c = capture();
        Log::info("x").component("auth").log_to(&c);
        assert!(has_component(&c, "auth"));
        assert!(!has_component(&c, "other"));
    }

    #[test]
    fn predicates_match_descriptions() {
        let c = capture();
        Log::info("authenticated user").log_to(&c);
        assert!(description_contains(&c, "authenticated"));
        assert!(!description_contains(&c, "missing"));
    }

    #[test]
    fn predicates_match_attributes() {
        let c = capture();
        Log::info("x")
            .with("user_id", 42_u64)
            .with("region", "eu-west-1")
            .log_to(&c);
        assert!(attribute_eq(&c, "user_id", 42_u64));
        assert!(attribute_eq(&c, "region", "eu-west-1"));
        assert!(!attribute_eq(&c, "user_id", 99_u64));
    }

    #[test]
    fn macro_succeeds_on_match() {
        let c = capture();
        Log::info("hello")
            .component("svc")
            .with("k", 1_u64)
            .log_to(&c);
        assert_logged!(c, level == LogLevel::INFO);
        assert_logged!(c, contains "hello");
        assert_logged!(c, component "svc");
        assert_logged!(c, attribute "k" => 1_u64);
        assert_logged!(c, len == 1);
    }

    #[test]
    #[should_panic(expected = "expected a captured record at level")]
    fn macro_panics_on_missing_level() {
        let c = capture();
        Log::info("x").log_to(&c);
        assert_logged!(c, level == LogLevel::ERROR);
    }

    #[test]
    #[should_panic(expected = "expected 3 captured records")]
    fn macro_panics_on_wrong_count() {
        let c = capture();
        Log::info("x").log_to(&c);
        assert_logged!(c, len == 3);
    }

    #[test]
    fn capture_handle_is_cheap_to_clone() {
        let a = capture();
        let b = a.clone();
        Log::info("x").log_to(&a);
        assert_eq!(b.len(), 1, "clones share the same buffer");
    }
}
