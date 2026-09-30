// async_http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async HTTP transport for OTLP/JSON over plain `http://`.
//!
//! Only compiled when the `async` feature is enabled. The HTTP/1.1
//! exchange is the crate's own (`src/http.rs`) over a Tokio
//! `TcpStream`; there is no TLS. Point the exporter at a local
//! OpenTelemetry Collector and let it handle TLS towards the
//! backend. See `docs/adr/0015-otlp-local-collector-transport.md`.

use crate::backoff::{
    CircuitBreaker, RetryPolicy, cheap_random_0_to_1,
};
use crate::http::{self, Endpoint};
use crate::{DEFAULT_ENDPOINT, OtlpError, OtlpResult, serialise_batch};
use rlg::log::Log;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Async HTTP/JSON exporter to an OTLP-compatible collector.
///
/// Same wire format and reliability primitives as the blocking
/// [`OtlpExporter`](crate::OtlpExporter); the difference is the
/// I/O path, which runs on the caller's Tokio runtime.
///
/// Retry with full jitter and the tokens-per-window circuit
/// breaker are shared with the blocking exporter via
/// [`crate::backoff`].
#[derive(Debug, Clone)]
pub struct AsyncOtlpExporter {
    endpoint: String,
    target: Endpoint,
    headers: HashMap<String, String>,
    timeout: Duration,
    retry: RetryPolicy,
    circuit: Option<Arc<CircuitBreaker>>,
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
    /// [`OtlpError::AsyncTransport`] when the collector cannot be
    /// reached, times out, or answers with something other than
    /// HTTP/1.x.
    pub async fn export_batch(
        &self,
        records: &[Log],
    ) -> OtlpResult<()> {
        let body = serialise_batch(records)?;
        self.post(&body).await
    }

    async fn post(&self, body: &str) -> OtlpResult<()> {
        if let Some(cb) = &self.circuit
            && !cb.allow()
        {
            return Err(OtlpError::CircuitOpen);
        }
        let request =
            http::encode_post(&self.target, &self.headers, body);
        let mut attempt: u32 = 0;
        let result = loop {
            let result = self.send(&request).await;
            if !is_retriable(&result)
                || attempt >= self.retry.max_retries
            {
                break result;
            }
            self.sleep_for_attempt(attempt).await;
            attempt += 1;
        };
        if let Some(cb) = &self.circuit {
            if result.is_ok() {
                cb.record_success();
            } else {
                cb.record_failure();
            }
        }
        result
    }

    /// One attempt: 2xx is success, any other status is
    /// [`OtlpError::BadStatus`].
    async fn send(&self, request: &[u8]) -> OtlpResult<()> {
        let status =
            tokio::time::timeout(self.timeout, self.exchange(request))
                .await
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "collector timed out",
                    )
                })
                .and_then(|r| r)
                .map_err(OtlpError::AsyncTransport)?;
        if (200..300).contains(&status) {
            Ok(())
        } else {
            Err(OtlpError::BadStatus(status))
        }
    }

    /// Connect, write the request, and read until the final status
    /// line and headers have arrived.
    async fn exchange(&self, request: &[u8]) -> io::Result<u16> {
        let target = (self.target.host.as_str(), self.target.port);
        let mut stream = TcpStream::connect(target).await?;
        stream.write_all(request).await?;
        let mut buf = Vec::with_capacity(512);
        let mut chunk = [0_u8; 1024];
        loop {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(status) = http::parse_status(&buf)? {
                return Ok(status);
            }
            if buf.len() > http::MAX_RESPONSE_HEAD {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "collector response headers too large",
                ));
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

/// Transport failures, 5xx and 429 are worth another attempt; a 4xx
/// or success is final.
const fn is_retriable(result: &OtlpResult<()>) -> bool {
    match result {
        Err(OtlpError::AsyncTransport(_)) => true,
        Err(OtlpError::BadStatus(status)) => {
            *status >= 500 || *status == 429
        }
        _ => false,
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
    /// Set the collector endpoint URL. Must be `http://`; the
    /// default is [`DEFAULT_ENDPOINT`], a Collector on this host.
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

    /// Per-request timeout, covering connect, send and the
    /// response head. Default is 10 s.
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
    /// # Errors
    /// Returns [`OtlpError::InvalidEndpoint`] if the endpoint is not
    /// an `http://` URL (`https://` included: TLS belongs to the
    /// collector), or [`OtlpError::InvalidHeader`] if a header name
    /// is invalid, a value holds a line break, or the header is one
    /// the exporter sets itself.
    pub fn build(self) -> OtlpResult<AsyncOtlpExporter> {
        let endpoint = self
            .endpoint
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        let target = Endpoint::parse(&endpoint)
            .map_err(OtlpError::InvalidEndpoint)?;
        http::validate_headers(&self.headers)
            .map_err(OtlpError::InvalidHeader)?;
        let retry = RetryPolicy {
            max_retries: self.max_retries.unwrap_or(3),
            base: self
                .backoff_base
                .unwrap_or_else(|| Duration::from_millis(200)),
            max_delay: Duration::from_secs(30),
            jitter: 1.0,
        };
        Ok(AsyncOtlpExporter {
            endpoint,
            target,
            headers: self.headers,
            timeout: self
                .timeout
                .unwrap_or_else(|| Duration::from_secs(10)),
            retry,
            circuit: self.circuit,
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

    #[tokio::test]
    async fn retry_loop_exhausts_attempts() {
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://127.0.0.1:1/v1/logs")
            .timeout_secs(1)
            .max_retries(2)
            .backoff_base(Duration::ZERO)
            .build()
            .unwrap();
        let res = e.export_one(&sample(LogLevel::INFO)).await;
        assert!(matches!(
            res,
            Err(OtlpError::AsyncTransport(_))
                | Err(OtlpError::BadStatus(_))
        ));
    }

    #[tokio::test]
    async fn export_batch_multiple_records() {
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://127.0.0.1:1/v1/logs")
            .timeout_secs(1)
            .max_retries(0)
            .build()
            .unwrap();
        let batch =
            vec![sample(LogLevel::INFO), sample(LogLevel::ERROR)];
        let res = e.export_batch(&batch).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn builder_max_retries_and_backoff_base() {
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://x")
            .max_retries(5)
            .backoff_base(Duration::from_millis(50))
            .build()
            .unwrap();
        assert_eq!(e.retry.max_retries, 5);
        assert_eq!(e.retry.base, Duration::from_millis(50));
    }

    #[tokio::test]
    async fn circuit_records_failure_on_shared_breaker() {
        let cb =
            Arc::new(CircuitBreaker::new(3, Duration::from_secs(60)));
        let e = AsyncOtlpExporter::builder()
            .endpoint("http://127.0.0.1:1/v1/logs")
            .timeout_secs(1)
            .max_retries(0)
            .circuit(cb.clone())
            .build()
            .unwrap();
        let _ = e.export_one(&sample(LogLevel::INFO)).await;
        // Shared breaker allows still: 3 tokens, 1 consumed.
        assert!(cb.allow());
    }
}
