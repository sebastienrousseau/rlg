// grpc.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! OTLP/gRPC transport scaffold via `tonic` + `rustls`.
//!
//! Only compiled when the `grpc` feature is enabled. See
//! `docs/adr/0010-otlp-pluggable-transport.md`.
//!
//! # Status: scaffold
//!
//! The type layout, feature flag, endpoint validation, and
//! reliability primitive integration all ship in this module.
//! The actual protobuf wire-encoding of
//! `ExportLogsServiceRequest` is intentionally deferred: it
//! requires either the `opentelemetry-proto` crate (a large,
//! generated dep tree) or hand-carved `prost::Message` derives
//! covering ~50 nested types. Neither fits the phase's scope
//! discipline.
//!
//! Consumers can construct a [`GrpcOtlpExporter`], validate the
//! endpoint, and hold a reference to a live `tonic::transport::
//! Channel`. `export_one` / `export_batch` return
//! [`OtlpError::GrpcNotImplemented`] with a documented pointer to
//! the follow-up work.
//!
//! Phase 19c.1 (planned) delivers the protobuf send path.

use crate::backoff::{CircuitBreaker, RetryPolicy};
use crate::{OtlpError, OtlpResult};
use rlg::log::Log;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

/// OTLP/gRPC exporter to an OTel-compatible collector.
///
/// See the module docs for the current implementation status.
#[derive(Debug, Clone)]
pub struct GrpcOtlpExporter {
    endpoint: String,
    channel: Channel,
    // `metadata`, `retry`, `circuit` are unused until Phase 19c.1
    // wires up the protobuf send path. The scaffold accepts them
    // through the builder so the public API is fixed now and
    // future work is drop-in.
    #[allow(dead_code)]
    metadata: HashMap<String, String>,
    #[allow(dead_code)]
    retry: RetryPolicy,
    #[allow(dead_code)]
    circuit: Option<Arc<CircuitBreaker>>,
}

impl GrpcOtlpExporter {
    /// Start building a new gRPC exporter.
    #[must_use]
    pub fn builder() -> GrpcOtlpExporterBuilder {
        GrpcOtlpExporterBuilder::default()
    }

    /// Export a single record.
    ///
    /// # Errors
    /// Currently returns [`OtlpError::GrpcNotImplemented`]. Full
    /// protobuf send lands in Phase 19c.1.
    pub async fn export_one(&self, _record: &Log) -> OtlpResult<()> {
        Err(OtlpError::GrpcNotImplemented)
    }

    /// Export a batch of records in a single gRPC call.
    ///
    /// # Errors
    /// Currently returns [`OtlpError::GrpcNotImplemented`]. Full
    /// protobuf send lands in Phase 19c.1.
    pub async fn export_batch(
        &self,
        _records: &[Log],
    ) -> OtlpResult<()> {
        Err(OtlpError::GrpcNotImplemented)
    }

    /// Endpoint URL the exporter targets.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Reference to the underlying tonic channel. Exposed so
    /// consumers writing their own protobuf send code (until
    /// Phase 19c.1 lands) can reuse the validated channel.
    #[must_use]
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

/// Fluent builder for [`GrpcOtlpExporter`].
#[derive(Debug, Default, Clone)]
pub struct GrpcOtlpExporterBuilder {
    endpoint: Option<String>,
    metadata: HashMap<String, String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    backoff_base: Option<Duration>,
    circuit: Option<Arc<CircuitBreaker>>,
}

impl GrpcOtlpExporterBuilder {
    /// Set the collector endpoint URL. Must be a valid URI parseable
    /// by [`tonic::transport::Endpoint`].
    #[must_use]
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Set a custom request metadata pair (gRPC's analogue of an
    /// HTTP header). Call repeatedly for multiple pairs.
    #[must_use]
    pub fn metadata(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(name.into(), value.into());
        self
    }

    /// Per-request timeout. Default 10 s.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    /// Maximum retry attempts. Default 3.
    #[must_use]
    pub const fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    /// Base for the exponential-with-jitter backoff. Default
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

    /// Finalise the builder. Establishes the tonic channel
    /// eagerly so endpoint-configuration bugs surface at build
    /// time rather than on the first export.
    ///
    /// # Panics
    /// Panics if `.endpoint()` was not called.
    ///
    /// # Errors
    /// Returns [`OtlpError::GrpcEndpoint`] if the endpoint URL is
    /// not a valid tonic `Endpoint`, or if the channel cannot be
    /// constructed.
    pub fn build(self) -> OtlpResult<GrpcOtlpExporter> {
        let endpoint_str = self
            .endpoint
            .expect("GrpcOtlpExporterBuilder::endpoint is required");
        let timeout =
            self.timeout.unwrap_or_else(|| Duration::from_secs(10));
        let endpoint = Endpoint::from_shared(endpoint_str.clone())
            .map_err(|e| {
                OtlpError::GrpcEndpoint(format!(
                    "invalid gRPC endpoint {endpoint_str:?}: {e}"
                ))
            })?
            .timeout(timeout);
        // `connect_lazy` returns a channel that establishes the
        // connection on first RPC call. Cheap; no I/O here.
        let channel = endpoint.connect_lazy();
        let retry = RetryPolicy {
            max_retries: self.max_retries.unwrap_or(3),
            base: self
                .backoff_base
                .unwrap_or_else(|| Duration::from_millis(200)),
            max_delay: Duration::from_secs(30),
            jitter: 1.0,
        };
        Ok(GrpcOtlpExporter {
            endpoint: endpoint_str,
            channel,
            metadata: self.metadata,
            retry,
            circuit: self.circuit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn builder_defaults() {
        // `connect_lazy` needs a Tokio runtime context.
        let e = GrpcOtlpExporter::builder()
            .endpoint("http://localhost:4317")
            .build()
            .expect("valid endpoint must build");
        assert_eq!(e.endpoint(), "http://localhost:4317");
        assert_eq!(e.retry.max_retries, 3);
    }

    #[tokio::test]
    async fn builder_rejects_bogus_endpoint() {
        // A truly bogus scheme cannot construct an Endpoint.
        let res = GrpcOtlpExporter::builder()
            .endpoint("this is not a url at all")
            .build();
        assert!(matches!(res, Err(OtlpError::GrpcEndpoint(_))));
    }

    #[tokio::test]
    async fn builder_accepts_metadata_and_timeout() {
        let e = GrpcOtlpExporter::builder()
            .endpoint("http://localhost:4317")
            .metadata("x-tenant", "acme")
            .timeout_secs(5)
            .build()
            .unwrap();
        assert_eq!(e.metadata.get("x-tenant").unwrap(), "acme");
    }

    #[tokio::test]
    async fn export_returns_not_implemented() {
        let e = GrpcOtlpExporter::builder()
            .endpoint("http://localhost:4317")
            .build()
            .unwrap();
        let log = rlg::log::Log::info("msg");
        let res = e.export_one(&log).await;
        assert!(matches!(res, Err(OtlpError::GrpcNotImplemented)));
        let res = e.export_batch(&[log]).await;
        assert!(matches!(res, Err(OtlpError::GrpcNotImplemented)));
    }
}
