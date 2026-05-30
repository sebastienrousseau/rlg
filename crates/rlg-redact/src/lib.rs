// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! PII / secret redaction for `rlg` records.
//!
//! Wrap a [`rlg::log::Log`] with [`Redactor::scrub`] before firing
//! it. Every string field (description + every string attribute
//! value) is run through a chain of regex patterns; matches are
//! replaced with the configured marker (default `"[REDACTED]"`).
//!
//! Built-in patterns cover the common PII / secret classes that
//! show up in log streams. Add custom patterns with
//! [`Redactor::with_pattern`].
//!
//! # Example
//!
//! ```
//! use rlg::log::Log;
//! use rlg_redact::Redactor;
//!
//! let redactor = Redactor::with_defaults();
//! let log = Log::info("card 4111-1111-1111-1111 failed");
//! let scrubbed = redactor.scrub(log);
//! assert!(scrubbed.description.contains("[REDACTED]"));
//! assert!(!scrubbed.description.contains("4111"));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use regex::Regex;
use rlg::log::Log;
use serde_json::Value;
use std::sync::LazyLock;

/// Default replacement marker.
pub const DEFAULT_MARKER: &str = "[REDACTED]";

// ---------------------------------------------------------------------------
// Built-in patterns.
// ---------------------------------------------------------------------------

/// Visa / MC / Amex / Discover. Matches 13-19 digits, optionally
/// separated by spaces or hyphens. Luhn validation is not performed
/// — false positives are preferred over false negatives for logs.
pub const CREDIT_CARD: &str = r"\b(?:\d[ -]?){12,18}\d\b";

/// Three-segment base64url JWT (`header.payload.signature`).
pub const JWT: &str =
    r"\beyJ[A-Za-z0-9_=-]+\.[A-Za-z0-9._=-]+\.[A-Za-z0-9._=-]+\b";

/// OAuth `Authorization: Bearer <token>` headers.
pub const BEARER_TOKEN: &str = r"(?i)Bearer\s+[A-Za-z0-9._~+/-]+=*";

/// RFC 5321 email addresses (good-enough subset).
pub const EMAIL: &str =
    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b";

/// IPv4 dotted-quad addresses.
pub const IPV4: &str = r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b";

/// AWS access key IDs (`AKIA…`, `ASIA…`, etc.).
pub const AWS_ACCESS_KEY: &str =
    r"\b(?:AKIA|ASIA|AGPA|ANPA|ANVA|AROA|AIPA)[A-Z0-9]{16}\b";

/// Shared, process-lifetime compiled forms of every [built-in
/// pattern](self#built-in-patterns). Used by
/// [`Redactor::with_defaults`] so repeated default-redactor
/// construction in hot paths doesn't recompile six regexes per call.
static BUILTIN_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [CREDIT_CARD, JWT, BEARER_TOKEN, EMAIL, IPV4, AWS_ACCESS_KEY]
        .into_iter()
        .map(|p| Regex::new(p).expect("built-in regex must compile"))
        .collect()
});

// ---------------------------------------------------------------------------
// Redactor.
// ---------------------------------------------------------------------------

/// A compiled chain of regex patterns + a replacement marker.
#[derive(Debug, Clone)]
pub struct Redactor {
    patterns: Vec<Regex>,
    marker: String,
}

impl Default for Redactor {
    fn default() -> Self {
        Self::empty()
    }
}

impl Redactor {
    /// Construct a redactor with no patterns and the default
    /// marker. Add patterns via [`Self::with_pattern`].
    #[must_use]
    pub fn empty() -> Self {
        Self {
            patterns: Vec::new(),
            marker: DEFAULT_MARKER.to_string(),
        }
    }

    /// Construct a redactor pre-loaded with every built-in pattern:
    /// credit card, JWT, OAuth bearer, email, IPv4, AWS key.
    ///
    /// The built-in regexes are compiled once per process via
    /// `LazyLock` and shared (`Arc`-cloned in effect since `Regex` is
    /// `Clone` over an inner `Arc`). Constructing a default redactor
    /// is O(`builtins.len()`) and allocation-free past the first call.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            patterns: BUILTIN_REGEXES.clone(),
            marker: DEFAULT_MARKER.to_string(),
        }
    }

    /// Append a custom regex pattern.
    ///
    /// # Errors
    /// Returns [`regex::Error`] if the pattern fails to compile.
    pub fn with_pattern(
        mut self,
        pattern: &str,
    ) -> Result<Self, regex::Error> {
        self.patterns.push(Regex::new(pattern)?);
        Ok(self)
    }

    /// Override the replacement marker (default: `"[REDACTED]"`).
    #[must_use]
    pub fn marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = marker.into();
        self
    }

    /// Scrub a record in-place. Returns the (possibly modified)
    /// record for fluent chaining.
    #[must_use]
    pub fn scrub(&self, mut log: Log) -> Log {
        log.description = self.apply(&log.description);
        for value in log.attributes.values_mut() {
            *value =
                self.scrub_value(std::mem::replace(value, Value::Null));
        }
        log
    }

    /// Apply every pattern to a single string.
    pub fn apply(&self, input: &str) -> String {
        let mut out = input.to_string();
        for re in &self.patterns {
            out =
                re.replace_all(&out, self.marker.as_str()).into_owned();
        }
        out
    }

    fn scrub_value(&self, v: Value) -> Value {
        match v {
            Value::String(s) => Value::String(self.apply(&s)),
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|i| self.scrub_value(i))
                    .collect(),
            ),
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.scrub_value(v)))
                    .collect(),
            ),
            other => other,
        }
    }

    /// How many patterns are loaded.
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Returns `true` if no patterns are loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlg::log_level::LogLevel;

    #[test]
    fn empty_redactor_is_a_no_op() {
        let r = Redactor::empty();
        let log = Log::info("4111-1111-1111-1111");
        let out = r.scrub(log.clone());
        assert_eq!(out.description, log.description);
    }

    #[test]
    fn default_marker_is_redacted() {
        assert_eq!(DEFAULT_MARKER, "[REDACTED]");
        let r = Redactor::with_defaults();
        let out = r.apply("email me at user@example.com");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("user@example.com"));
    }

    #[test]
    fn custom_marker_replaces_default() {
        let r = Redactor::with_defaults().marker("***");
        let out = r.apply("ip 192.168.0.1 down");
        assert!(out.contains("***"));
        assert!(!out.contains("192.168.0.1"));
    }

    #[test]
    fn credit_card_pattern_matches_visa() {
        let r = Redactor::empty().with_pattern(CREDIT_CARD).unwrap();
        assert!(!r.apply("4111-1111-1111-1111").contains("4111"));
        assert!(!r.apply("4111 1111 1111 1111").contains("4111"));
        assert!(!r.apply("4111111111111111").contains("4111"));
    }

    #[test]
    fn jwt_pattern_matches() {
        let r = Redactor::empty().with_pattern(JWT).unwrap();
        let token =
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.abcdEFGHijk";
        let out = r.apply(&format!("auth={token} ok"));
        assert!(!out.contains("eyJ"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn bearer_token_pattern_matches() {
        let r = Redactor::empty().with_pattern(BEARER_TOKEN).unwrap();
        let out = r.apply("Authorization: Bearer abc123XYZ.foo");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("abc123XYZ"));
    }

    #[test]
    fn email_pattern_matches() {
        let r = Redactor::empty().with_pattern(EMAIL).unwrap();
        let out = r.apply("sent to alice+test@example.co.uk today");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("alice"));
    }

    #[test]
    fn ipv4_pattern_matches() {
        let r = Redactor::empty().with_pattern(IPV4).unwrap();
        let out = r.apply("client 10.0.1.42 disconnected");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("10.0.1.42"));
    }

    #[test]
    fn aws_key_pattern_matches() {
        let r = Redactor::empty().with_pattern(AWS_ACCESS_KEY).unwrap();
        let out = r.apply("AKIAIOSFODNN7EXAMPLE leaked");
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn scrub_walks_string_attributes() {
        let r = Redactor::with_defaults();
        let log = Log::build(LogLevel::INFO, "user@host.com signed in")
            .with("email", "other@host.com")
            .with("session_id_num", 42_u64);
        let out = r.scrub(log);
        assert!(!out.description.contains("user@host.com"));
        // Numeric attributes pass through unchanged.
        assert_eq!(
            out.attributes.get("session_id_num"),
            Some(&serde_json::json!(42_u64))
        );
        // String attributes are scrubbed.
        let email = out.attributes.get("email").unwrap();
        assert!(email.as_str().unwrap().contains("[REDACTED]"));
    }

    #[test]
    fn scrub_recurses_into_nested_json() {
        let r = Redactor::with_defaults();
        let log = Log::info("x").with(
            "payload",
            serde_json::json!({
                "user": { "email": "x@y.com" },
                "ips": ["10.0.0.1", "192.168.0.1"]
            }),
        );
        let out = r.scrub(log);
        let payload = out.attributes.get("payload").unwrap();
        let serialised = payload.to_string();
        assert!(!serialised.contains("x@y.com"));
        assert!(!serialised.contains("10.0.0.1"));
    }

    #[test]
    fn with_pattern_rejects_invalid_regex() {
        let r = Redactor::empty().with_pattern("[unclosed");
        assert!(r.is_err());
    }

    #[test]
    fn len_reflects_patterns_loaded() {
        let r = Redactor::with_defaults();
        assert_eq!(r.len(), 6);
        assert!(!r.is_empty());
        assert!(Redactor::empty().is_empty());
    }
}
