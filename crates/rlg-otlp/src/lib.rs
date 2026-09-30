// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! OpenTelemetry (OTLP) network exporter for rlg.
//!
//! rlg renders records in `LogFormat::OTLP` shape and writes them to
//! the configured `PlatformSink` (stdout, file, `os_log`,
//! `journald`). This crate adds an **HTTP exporter** that POSTs
//! OTLP-shaped records to an OpenTelemetry Collector.
//!
//! Wire format: **OTLP/HTTP JSON encoding** (`Content-Type:
//! application/json`), over plain `http://`.
//!
//! # Deployment: a local collector owns TLS
//!
//! The exporter carries no TLS stack. It sends to a Collector (or
//! any OTLP/HTTP forwarder) on the same host or pod, by default
//! [`DEFAULT_ENDPOINT`], and the Collector holds the TLS connection
//! and credentials for the backend. `https://` endpoints are refused.
//! The README has a minimal Collector configuration.
//!
//! # Example
//!
//! ```no_run
//! use rlg_otlp::OtlpExporter;
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//!
//! // Defaults to the Collector at http://localhost:4318/v1/logs.
//! let exporter = OtlpExporter::builder().timeout_secs(10).build();
//!
//! let record = Log::error("payment-service down")
//!     .component("orders")
//!     .with("trace_id", "abc")
//!     .format(LogFormat::OTLP);
//!
//! // Synchronous push. Async push is on the roadmap.
//! exporter.export_one(&record).unwrap();
//! ```

#![forbid(unsafe_code)]
// Enable `#[doc(cfg(feature = "…"))]` under docs.rs so feature-gated
// items visibly show which cargo feature they need. Requires nightly
// (`--cfg docsrs`); stable builds skip this so they compile without
// unstable features.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

/// Retry policy + full-jitter + tokens-per-window circuit breaker.
///
/// Transport-agnostic reliability primitives shared by the blocking
/// and async exporters.
pub mod backoff;

pub use crate::backoff::{CircuitBreaker, RetryPolicy};

/// The in-house HTTP/1.1 exchange behind the async exporter.
#[cfg(feature = "async")]
mod http;

/// Async HTTP/JSON transport over Tokio and plain `http://`.
/// Enable with the `async` feature.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_http;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use crate::async_http::{
    AsyncOtlpExporter, AsyncOtlpExporterBuilder,
};

/// The endpoint a builder uses when given none: the OTLP/HTTP logs
/// route of a Collector on the same host.
pub const DEFAULT_ENDPOINT: &str = "http://localhost:4318/v1/logs";

use crate::backoff::{
    cheap_random_0_to_1, is_retriable, record_outcome, status_outcome,
};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Errors raised by [`OtlpExporter`].
#[derive(Debug, Error)]
pub enum OtlpError {
    /// The HTTP transport failed before a response arrived (DNS, TCP,
    /// timeout, or an `https://` endpoint, which is not supported).
    #[error("OTLP transport error: {0}")]
    Transport(#[from] Box<ureq::Error>),
    /// The collector responded with a non-2xx status code.
    #[error("OTLP collector returned status {0}")]
    BadStatus(u16),
    /// Serialising a record to OTLP/JSON failed.
    #[error("OTLP serialise error: {0}")]
    Serialise(#[from] serde_json::Error),
    /// The circuit breaker is tripped for the current window. The
    /// request was rejected without touching the network.
    #[error("OTLP circuit breaker tripped (too many recent failures)")]
    CircuitOpen,
    /// The async HTTP transport failed: the collector could not be
    /// reached, timed out, or did not answer with HTTP/1.x. Only
    /// produced when the `async` feature is enabled and an
    /// [`AsyncOtlpExporter`] is in use.
    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    #[error("OTLP async transport error: {0}")]
    AsyncTransport(std::io::Error),
    /// The endpoint is not a usable `http://` URL. `https://` is
    /// refused: TLS belongs to the local collector.
    #[error("OTLP endpoint error: {0}")]
    InvalidEndpoint(String),
    /// A caller-supplied header has an invalid name, a line break
    /// in its value, or is one the exporter sets itself.
    #[error("OTLP header error: {0}")]
    InvalidHeader(String),
}

/// A `Result` alias with [`OtlpError`] as the error variant.
pub type OtlpResult<T> = Result<T, OtlpError>;

/// Synchronous HTTP/JSON exporter to an OTLP-compatible collector.
///
/// Construct via [`OtlpExporter::builder`]. Send records one at a
/// time with [`OtlpExporter::export_one`] or in a batch with
/// [`OtlpExporter::export_batch`].
#[derive(Debug, Clone)]
pub struct OtlpExporter {
    endpoint: String,
    headers: HashMap<String, String>,
    timeout: Duration,
    retry: RetryPolicy,
    /// Optional per-exporter circuit breaker. When set, every
    /// `post` consults it before touching the network and updates
    /// it based on the outcome. When unset, no circuit-breaking
    /// behaviour applies.
    circuit: Option<Arc<CircuitBreaker>>,
}

impl OtlpExporter {
    /// Start building a new exporter.
    #[must_use]
    pub fn builder() -> OtlpExporterBuilder {
        OtlpExporterBuilder::default()
    }

    /// Export a single record. Renders the record as
    /// `LogFormat::OTLP`, wraps it in the OTLP/HTTP envelope, and
    /// POSTs it to the configured endpoint.
    ///
    /// # Errors
    /// Returns [`OtlpError`] on transport failure, non-2xx response,
    /// or serialisation failure.
    pub fn export_one(&self, record: &Log) -> OtlpResult<()> {
        self.export_batch(std::slice::from_ref(record))
    }

    /// Export a batch of records in a single HTTP POST.
    ///
    /// # Errors
    /// See [`Self::export_one`].
    pub fn export_batch(&self, records: &[Log]) -> OtlpResult<()> {
        let body = serialise_batch(records)?;
        self.post(&body)
    }

    fn post(&self, body: &str) -> OtlpResult<()> {
        // A tripped breaker rejects without touching the network.
        if let Some(cb) = &self.circuit
            && !cb.allow()
        {
            return Err(OtlpError::CircuitOpen);
        }
        // ureq turns every non-2xx status into an `Err` by default,
        // which would retry a 4xx and report a 5xx as a transport
        // error. `status_outcome` classifies the status instead.
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .http_status_as_error(false)
            .build()
            .new_agent();
        let mut attempt: u32 = 0;
        let result = loop {
            let result = self.send(&agent, body);
            if !is_retriable(&result)
                || attempt >= self.retry.max_retries
            {
                break result;
            }
            self.sleep_for_attempt(attempt);
            attempt += 1;
        };
        record_outcome(self.circuit.as_deref(), &result);
        result
    }

    /// One attempt: POST `body` and classify the answer.
    fn send(&self, agent: &ureq::Agent, body: &str) -> OtlpResult<()> {
        let mut req = agent
            .post(&self.endpoint)
            .header("content-type", "application/json");
        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let response = req
            .send(body)
            .map_err(|e| OtlpError::Transport(Box::new(e)))?;
        status_outcome(response.status().as_u16())
    }

    /// Sleep for the delay computed by the retry policy, including
    /// full jitter. Delegates the schedule math to
    /// [`RetryPolicy::delay`] in `backoff.rs`.
    fn sleep_for_attempt(&self, attempt: u32) {
        std::thread::sleep(
            self.retry.delay(attempt, cheap_random_0_to_1()),
        );
    }

    /// Endpoint URL the exporter posts to.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Fluent builder for [`OtlpExporter`].
#[derive(Debug, Default, Clone)]
pub struct OtlpExporterBuilder {
    endpoint: Option<String>,
    headers: HashMap<String, String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    backoff_base: Option<Duration>,
    circuit: Option<Arc<CircuitBreaker>>,
}

impl OtlpExporterBuilder {
    /// Set the collector endpoint URL. The default is
    /// [`DEFAULT_ENDPOINT`], a Collector on this host. Use `http://`:
    /// this transport has no TLS.
    #[must_use]
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Set a custom request header. Call repeatedly for multiple
    /// headers (auth tokens, dataset identifiers, etc.).
    #[must_use]
    pub fn header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the per-request timeout in seconds. Default is 10 s.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Maximum number of retry attempts after a 5xx / 429 response or
    /// transport error. Default is 3 (so up to 4 total attempts).
    ///
    /// Set to `0` to disable retries.
    #[must_use]
    pub const fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    /// Base for the exponential backoff between retries.
    /// `delay = base * 2^attempt`, capped at 30 s and modulated by
    /// full jitter. Default base is 200 ms, which gives a
    /// `[0, 200 ms]` / `[0, 400 ms]` / `[0, 800 ms]` schedule under
    /// the default 3 retries.
    #[must_use]
    pub const fn backoff_base(mut self, base: Duration) -> Self {
        self.backoff_base = Some(base);
        self
    }

    /// Attach a circuit breaker. When set, every export consults the
    /// breaker before touching the network; a tripped breaker
    /// rejects immediately with [`OtlpError::CircuitOpen`].
    ///
    /// Cloning the returned exporter shares the same breaker state
    /// across clones (the field is `Arc<CircuitBreaker>`).
    #[must_use]
    pub fn circuit(mut self, cb: Arc<CircuitBreaker>) -> Self {
        self.circuit = Some(cb);
        self
    }

    /// Finalise the builder.
    #[must_use]
    pub fn build(self) -> OtlpExporter {
        let retry = RetryPolicy {
            max_retries: self.max_retries.unwrap_or(3),
            base: self
                .backoff_base
                .unwrap_or_else(|| Duration::from_millis(200)),
            max_delay: Duration::from_secs(30),
            jitter: 1.0,
        };
        OtlpExporter {
            endpoint: self
                .endpoint
                .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string()),
            headers: self.headers,
            timeout: self
                .timeout
                .unwrap_or_else(|| Duration::from_secs(10)),
            retry,
            circuit: self.circuit,
        }
    }
}

/// Serialise a batch of records into a single OTLP/HTTP JSON envelope.
///
/// Each record is first rendered via its `LogFormat::OTLP` `Display`
/// impl, which produces an `attributes` / `body` / `severityNumber`
/// / `severityText` / `spanId` / `timeUnixNano` / `traceId` shape.
/// Those individual record JSON blobs are then nested inside the
/// OTLP `resourceLogs[].scopeLogs[].logRecords[]` envelope the
/// collector expects.
///
/// # Errors
/// Returns [`OtlpError::Serialise`] if a record fails to parse back
/// as JSON (should be impossible — the `OTLP` Display impl emits
/// valid JSON by construction).
pub fn serialise_batch(records: &[Log]) -> OtlpResult<String> {
    let mut log_records = Vec::with_capacity(records.len());
    for record in records {
        let mut copy = record.clone();
        copy.format = LogFormat::OTLP;
        let rendered = format!("{copy}");
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered)?;
        log_records.push(parsed);
    }
    let envelope = serde_json::json!({
        "resourceLogs": [{
            "resource": {
                "attributes": [{
                    "key": "service.name",
                    "value": { "stringValue": "rlg" }
                }]
            },
            "scopeLogs": [{
                "scope": {
                    "name": "rlg-otlp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "logRecords": log_records
            }]
        }]
    });
    Ok(envelope.to_string())
}

#[cfg(test)]
mod tests;
