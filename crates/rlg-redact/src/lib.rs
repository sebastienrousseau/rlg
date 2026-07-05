// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! PII / secret redaction for `rlg` records.
//!
//! Wrap a [`rlg::log::Log`] with [`Redactor::scrub`] before firing
//! it. Every string field (description + every string attribute
//! value) is scanned once against a **fused alternation of all
//! loaded patterns**; matches are replaced with the configured
//! marker (default `"[REDACTED]"`).
//!
//! Built-in patterns cover the common PII / secret classes that
//! show up in log streams. Add custom patterns with
//! [`Redactor::with_pattern`].
//!
//! # Performance model
//!
//! Every mutator ([`Redactor::with_defaults`], [`Redactor::with_pattern`])
//! compiles all currently-loaded patterns into a single `Regex`
//! alternation. `scrub` performs one `replace_all` pass instead of
//! `N` (once per pattern) — the DFA engine handles the union
//! internally. Result: single-pass throughput, no repeated
//! traversal of the input string.
//!
//! The compilation cost is amortised at construction. Constructing
//! [`Redactor::with_defaults`] is O(1) past the first call via a
//! process-lifetime `LazyLock`. Ad-hoc `with_pattern` chains
//! recompile the fused regex at each step; if you compose many
//! patterns, build the full chain once and reuse the redactor.
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

/// Source strings of the six built-in patterns, in the order they
/// are fused into the default alternation.
const DEFAULT_SOURCES: [&str; 6] =
    [CREDIT_CARD, JWT, BEARER_TOKEN, EMAIL, IPV4, AWS_ACCESS_KEY];

/// Process-lifetime fused alternation of every built-in pattern.
/// Amortises compilation to a single first-touch cost regardless of
/// how many times [`Redactor::with_defaults`] is called.
static DEFAULT_COMBINED: LazyLock<Regex> = LazyLock::new(|| {
    build_combined(&DEFAULT_SOURCES)
        .expect("built-in patterns must compile as an alternation")
});

/// Build a fused alternation `(?:p1)|(?:p2)|…|(?:pN)`. Individual
/// patterns are wrapped in non-capturing groups so the top-level
/// alternation composes cleanly regardless of internal grouping.
fn build_combined<S: AsRef<str>>(
    sources: &[S],
) -> Result<Regex, regex::Error> {
    debug_assert!(!sources.is_empty(), "must not build from empty set");
    let alternation = sources
        .iter()
        .map(|p| format!("(?:{})", p.as_ref()))
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&alternation)
}

// ---------------------------------------------------------------------------
// Redactor.
// ---------------------------------------------------------------------------

/// A fused-alternation redactor: one regex, one pass, N patterns.
///
/// See the crate-level docs for the performance model.
#[derive(Debug, Clone)]
pub struct Redactor {
    /// Source patterns kept for [`len`](Self::len) reporting and
    /// re-composition when a new pattern is appended.
    sources: Vec<String>,
    /// Fused alternation of `sources`. `None` when `sources` is
    /// empty — the fast path returns the input unchanged without
    /// touching the regex engine.
    combined: Option<Regex>,
    /// Replacement marker.
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
            sources: Vec::new(),
            combined: None,
            marker: DEFAULT_MARKER.to_string(),
        }
    }

    /// Construct a redactor pre-loaded with every built-in pattern:
    /// credit card, JWT, OAuth bearer, email, IPv4, AWS key.
    ///
    /// All six patterns are fused into a single alternation regex
    /// at process start-up (via [`LazyLock`]). Subsequent calls to
    /// this constructor clone the cached `Regex` — construction is
    /// O(1) past the first call.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            sources: DEFAULT_SOURCES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            combined: Some(DEFAULT_COMBINED.clone()),
            marker: DEFAULT_MARKER.to_string(),
        }
    }

    /// Append a custom regex pattern.
    ///
    /// The pattern is validated in isolation before being fused into
    /// the combined alternation, so a compile error surfaces the
    /// specific bad pattern rather than the fused string.
    ///
    /// # Errors
    /// Returns [`regex::Error`] if the pattern fails to compile.
    pub fn with_pattern(
        mut self,
        pattern: &str,
    ) -> Result<Self, regex::Error> {
        // Validate the standalone pattern first — surfaces a
        // targeted error message.
        let _ = Regex::new(pattern)?;
        self.sources.push(pattern.to_string());
        // Recompile the fused alternation. Individual patterns are
        // known-valid; alternation should compile unless the user
        // exceeds regex-engine size limits, which we surface too.
        self.combined = Some(build_combined(&self.sources)?);
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

    /// Apply the fused pattern to a single string.
    ///
    /// One `replace_all` pass through the regex DFA replaces every
    /// match — regardless of which pattern each match originates
    /// from — with the configured marker.
    #[must_use]
    pub fn apply(&self, input: &str) -> String {
        match &self.combined {
            None => input.to_string(),
            Some(re) => {
                re.replace_all(input, self.marker.as_str()).into_owned()
            }
        }
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
        self.sources.len()
    }

    /// Returns `true` if no patterns are loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
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

    // ---- Fusion-boundary regression tests (Phase 17) -----------------

    #[test]
    fn fusion_scans_all_pattern_kinds_in_one_pass() {
        // Every built-in pattern class appears once in a single
        // input. A per-pattern loop would iterate the input six
        // times; the fused alternation touches it once.
        let r = Redactor::with_defaults();
        let out = r.apply(
            "cc 4111-1111-1111-1111, jwt \
             eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abcd, \
             Bearer xyz.abc, alice@example.com, 10.0.0.1, \
             AKIAIOSFODNN7EXAMPLE",
        );
        for needle in [
            "4111",
            "eyJhbGciOiJIUzI1NiJ9",
            "xyz.abc",
            "alice@example.com",
            "10.0.0.1",
            "AKIAIOSFODNN7EXAMPLE",
        ] {
            assert!(
                !out.contains(needle),
                "fusion missed {needle:?} in output {out:?}"
            );
        }
        assert!(out.matches("[REDACTED]").count() >= 6);
    }

    #[test]
    fn fusion_prefers_leftmost_match_across_pattern_kinds() {
        // The fused regex uses leftmost-first semantics, so the
        // whole sensitive span is replaced with a single marker per
        // longest match — one marker per distinct span.
        let r = Redactor::empty()
            .with_pattern(CREDIT_CARD)
            .unwrap()
            .with_pattern(IPV4)
            .unwrap();
        let out = r.apply("card 4111-1111-1111-1111 from 10.0.0.1");
        assert!(!out.contains("4111"));
        assert!(!out.contains("10.0.0.1"));
        assert_eq!(out.matches("[REDACTED]").count(), 2);
    }

    #[test]
    fn fusion_compiles_alternation_from_chained_with_pattern() {
        // Chained with_pattern calls must produce a redactor whose
        // fused regex covers every appended source. Regression test
        // for the recompilation invariant.
        let r = Redactor::empty()
            .with_pattern(r"AAA-\d+")
            .unwrap()
            .with_pattern(r"BBB-\d+")
            .unwrap()
            .with_pattern(r"CCC-\d+")
            .unwrap();
        assert_eq!(r.len(), 3);
        let out = r.apply("AAA-1 BBB-2 CCC-3 DDD-4");
        assert!(out.contains("[REDACTED] [REDACTED] [REDACTED] DDD-4"));
    }
}
