// async_http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async HTTP transport for OTLP/JSON via `reqwest` + `rustls`.
//!
//! Only compiled when the `async` feature is enabled. See
//! `docs/adr/0010-otlp-pluggable-transport.md`.

use crate::backoff::{
    CircuitBreaker, RetryPolicy, cheap_random_0_to_1,
};
use crate::{OtlpError, OtlpResult, serialise_batch};
use rlg::log::Log;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Async HTTP/JSON exporter to an OTLP-compatible collector.
///
/// Same wire format and reliability primitives as the blocking
/// [`OtlpExporter`](crate::OtlpExporter); the difference is the
/// I/O path — `reqwest` + a caller-provided Tokio runtime instead
/// of `ureq`.
///
/// Retry with full jitter and the tokens-per-window circuit
/// breaker are shared with the blocking exporter via
/// [`crate::backoff`].
#[derive(Debug, Clone)]
pub struct AsyncOtlpExporter {
    endpoint: String,
    headers: HashMap<String, String>,
    timeout: Duration,
    retry: RetryPolicy,
    circuit: Option<Arc<CircuitBreaker>>,
    client: reqwest::Client,
}

impl AsyncOtlpExporter {
    /// Start building a new async exporter.
    #[must_use]
    pub fn builder() -> AsyncOtlpExporterBuilder {
        AsyncOtlpExporterBuilder::default()
    }

    /// Export a single record.
    ///
    /// # Errors
    /// See [`Self::export_batch`].
    pub async fn export_one(&self, record: &Log) -> OtlpResult<()> {
        self.export_batch(std::slice::from_ref(record)).await
    }

    /// Export a batch of records in a single HTTP POST.
    ///
    /// # Errors
    /// Returns [`OtlpError::Serialise`] on serialisation failure,
    /// [`OtlpError::BadStatus`] on non-2xx response,
    /// [`OtlpError::CircuitOpen`] if the breaker is tripped, or
    /// [`OtlpError::Transport`] on a `reqwest` transport failure.
    pub async fn export_batch(
        &self,
        records: &[Log],
    ) -> OtlpResult<()> {
        let body = serialise_batch(records)?;
        self.post(body).await
    }

    async fn post(&self, body: String) -> OtlpResult<()> {
        if let Some(cb) = &self.circuit
            && !cb.allow()
        {
            return Err(OtlpError::CircuitOpen);
        }

        let mut attempt: u32 = 0;
        loop {
            let mut req = self
                .client
                .post(&self.endpoint)
                .header("content-type", "application/json")
                .timeout(self.timeout);
            for (k, v) in &self.headers {
                req = req.header(k.as_str(), v.as_str());
            }
            match req.body(body.clone()).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    if status >= 500 || status == 429 {
                        if attempt < self.retry.max_retries {
                            self.sleep_for_attempt(attempt).await;
                            attempt += 1;
                            continue;
                        }
                        if let Some(cb) = &self.circuit {
                            cb.record_failure();
                        }
                        return Err(OtlpError::BadStatus(status));
                    }
                    if !(200..300).contains(&status) {
                        if let Some(cb) = &self.circuit {
                            cb.record_failure();
                        }
                        return Err(OtlpError::BadStatus(status));
                    }
                    if let Some(cb) = &self.circuit {
                        cb.record_success();
                    }
                    return Ok(());
                }
                Err(e) => {
                    if attempt < self.retry.max_retries {
                        self.sleep_for_attempt(attempt).await;
                        attempt += 1;
                        continue;
                    }
                    if let Some(cb) = &self.circuit {
                        cb.record_failure();
                    }
                    return Err(OtlpError::AsyncTransport(Box::new(e)));
                }
            }
        }
    }

    async fn sleep_for_attempt(&self, attempt: u32) {
        tokio::time::sleep(
            self.retry.delay(attempt, cheap_random_0_to_1()),
        )
        .await;
    }

    /// Endpoint URL the exporter posts to.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Fluent builder for [`AsyncOtlpExporter`].
#[derive(Debug, Default, Clone)]
pub struct AsyncOtlpExporterBuilder {
    endpoint: Option<String>,
    headers: HashMap<String, String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    backoff_base: Option<Duration>,
    circuit: Option<Arc<CircuitBreaker>>,
}

impl AsyncOtlpExporterBuilder {
    /// Set the collector endpoint URL.
    #[must_use]
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Set a custom request header. Call repeatedly for multiple.
    #[must_use]
    pub fn header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Per-request timeout. Default is 10 s.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Maximum retry attempts. Default is 3.
    #[must_use]
    pub const fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    /// Base for the exponential-with-jitter backoff. Default is
    /// 200 ms.
    #[must_use]
    pub const fn backoff_base(mut self, base: Duration) -> Self {
        self.backoff_base = Some(base);
        self
    }

    /// Attach an optional circuit breaker.
    #[must_use]
    pub fn circuit(mut self, cb: Arc<CircuitBreaker>) -> Self {
        self.circuit = Some(cb);
        self
    }

    /// Finalise the builder.
    ///
    /// # Panics
    /// Panics if `.endpoint()` was not called.
    ///
    /// # Errors
    /// Returns [`OtlpError::AsyncTransport`] if the underlying
    /// `reqwest::Client` cannot be constructed (e.g. rustls TLS
    /// stack initialisation failure).
    pub fn build(self) -> OtlpResult<AsyncOtlpExporter> {
        let retry = RetryPolicy {
            max_retries: self.max_retries.unwrap_or(3),
            base: self
                .backoff_base
                .unwrap_or_else(|| Duration::from_millis(200)),
            max_delay: Duration::from_secs(30),
            jitter: 1.0,
        };
        let timeout =
            self.timeout.unwrap_or_else(|| Duration::from_secs(10));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| OtlpError::AsyncTransport(Box::new(e)))?;
        Ok(AsyncOtlpExporter {
            endpoint: self.endpoint.expect(
                "AsyncOtlpExporterBuilder::endpoint is required",
            ),
            headers: self.headers,
            timeout,
            retry,
            circuit: self.circuit,
            client,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    fn sample(level: LogLevel) -> Log {
        Log::build(level, "msg")
            .component("svc")
            .session_id(7)
            .time("2026-05-30T00:00:00.000000000Z")
            .format(LogFormat::OTLP)
    }

    #[tokio::test]
    async fn builder_defaults_to_sensible_values() {
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://x/v1/logs")
            .build()
            .expect("builder must succeed with an endpoint");
        assert_eq!(e.retry.max_retries, 3);
        assert_eq!(e.retry.base, Duration::from_millis(200));
        assert_eq!(e.timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn builder_sets_headers_and_timeout() {
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://x/v1/logs")
            .header("x-honeycomb-team", "key123")
            .timeout_secs(30)
            .build()
            .unwrap();
        assert_eq!(
            e.headers.get("x-honeycomb-team").unwrap(),
            "key123"
        );
        assert_eq!(e.timeout, Duration::from_secs(30));
        assert_eq!(e.endpoint(), "http://x/v1/logs");
    }

    #[tokio::test]
    async fn export_one_against_unreachable_endpoint_errors() {
        // localhost:1 is closed on every sane machine; the
        // transport error surfaces after connection refusal.
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://127.0.0.1:1/v1/logs")
            .timeout_secs(1)
            .max_retries(0)
            .build()
            .unwrap();
        let res = e.export_one(&sample(LogLevel::INFO)).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn circuit_breaker_short_circuits_when_tripped() {
        let cb =
            Arc::new(CircuitBreaker::new(1, Duration::from_secs(60)));
        // Pre-fail the breaker so the very first call is rejected.
        cb.record_failure();
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://127.0.0.1:1/v1/logs")
            .max_retries(0)
            .circuit(cb)
            .build()
            .unwrap();
        let res = e.export_one(&sample(LogLevel::INFO)).await;
        assert!(matches!(res, Err(OtlpError::CircuitOpen)));
    }
}
