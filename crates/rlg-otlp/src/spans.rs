// spans.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`SpanExporter`]: closed spans from `rlg::tracing::RlgLayer`, sent
//! to a collector as OTLP traces in batches from a background thread.

use crate::traces::{DEFAULT_TRACES_ENDPOINT, serialise_spans};
use env::Env;

mod env;
use crate::{OtlpError, OtlpExporter, OtlpExporterBuilder, OtlpResult};
use rlg::tracing::{SpanRecord, SpanSink};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{
    self, Receiver, RecvTimeoutError, SyncSender, TrySendError,
};
use std::time::{Duration, Instant};

/// Counters a [`SpanExporter`] keeps; read them with
/// [`SpanExporter::stats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpanExportStats {
    /// Spans the collector accepted.
    pub exported: u64,
    /// Spans dropped because the queue was full.
    pub dropped: u64,
    /// Spans in batches the collector refused or never received.
    pub failed: u64,
}

#[derive(Debug, Default)]
struct Counters {
    exported: AtomicU64,
    dropped: AtomicU64,
    failed: AtomicU64,
}

enum Msg {
    Span(SpanRecord),
    Flush(mpsc::Sender<()>),
}

/// Sends closed spans to an OpenTelemetry Collector as OTLP/HTTP JSON
/// traces (`/v1/traces`). Hand it to `RlgLayer::with_span_sink`.
///
/// Closing a span only queues it (a bounded queue; a full queue drops
/// the span and counts it), so the application thread never waits on
/// the network. A background thread posts a batch when it reaches
/// the batch size, when the flush interval passes, on
/// [`flush`](Self::flush), and when the last clone is dropped.
/// Retries and timeouts are the [`OtlpExporter`]'s. Clones share one
/// queue and thread: keep one to flush at shutdown.
///
/// ```no_run
/// use rlg::tracing::RlgLayer;
/// use rlg_otlp::SpanExporter;
/// use std::time::Duration;
/// use tracing_subscriber::prelude::*;
///
/// let spans = SpanExporter::from_env().expect("valid OTLP settings");
/// let mut layer = RlgLayer::new();
/// if let Some(exporter) = &spans {
///     layer = layer.with_span_sink(exporter.clone());
/// }
/// tracing_subscriber::registry().with(layer).init();
/// // … at shutdown:
/// if let Some(exporter) = &spans {
///     let _ = exporter.flush(Duration::from_secs(5));
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SpanExporter {
    tx: SyncSender<Msg>,
    counters: Arc<Counters>,
    endpoint: Arc<str>,
}

impl std::fmt::Debug for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Span(_) => "Span",
            Self::Flush(_) => "Flush",
        })
    }
}

impl SpanExporter {
    /// Start building an exporter.
    #[must_use]
    pub fn builder() -> SpanExporterBuilder {
        SpanExporterBuilder::default()
    }

    /// An exporter configured from the standard OpenTelemetry
    /// environment variables, or `Ok(None)` when none names an
    /// endpoint (or `OTEL_SDK_DISABLED=true`). See
    /// [`SpanExporterBuilder::from_lookup`] for the variables read.
    ///
    /// # Errors
    /// As [`SpanExporterBuilder::build`].
    pub fn from_env() -> OtlpResult<Option<Self>> {
        SpanExporterBuilder::from_lookup(|k| std::env::var(k).ok())
            .map(SpanExporterBuilder::build)
            .transpose()
    }

    /// Post every queued span now, waiting up to `timeout` for the
    /// export to finish. Returns `false` if it did not finish in time.
    #[must_use = "a false return means spans may still be queued"]
    pub fn flush(&self, timeout: Duration) -> bool {
        let (ack, done) = mpsc::channel();
        self.tx.send(Msg::Flush(ack)).is_ok()
            && done.recv_timeout(timeout).is_ok()
    }

    /// Spans exported, dropped and failed so far.
    #[must_use]
    pub fn stats(&self) -> SpanExportStats {
        SpanExportStats {
            exported: self.counters.exported.load(Ordering::Relaxed),
            dropped: self.counters.dropped.load(Ordering::Relaxed),
            failed: self.counters.failed.load(Ordering::Relaxed),
        }
    }

    /// Endpoint URL the exporter posts to.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl SpanSink for SpanExporter {
    fn on_span_end(&self, span: SpanRecord) {
        if let Err(
            TrySendError::Full(_) | TrySendError::Disconnected(_),
        ) = self.tx.try_send(Msg::Span(span))
        {
            let _ =
                self.counters.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Fluent builder for [`SpanExporter`].
#[derive(Debug, Clone)]
pub struct SpanExporterBuilder {
    http: OtlpExporterBuilder,
    endpoint: String,
    service_name: String,
    max_batch: usize,
    queue_capacity: usize,
    flush_interval: Duration,
    /// A problem found in the environment, reported by `build`.
    env_error: Option<String>,
}

impl Default for SpanExporterBuilder {
    fn default() -> Self {
        Self {
            http: OtlpExporter::builder(),
            endpoint: DEFAULT_TRACES_ENDPOINT.to_string(),
            service_name: "unknown_service".to_string(),
            max_batch: 512,
            queue_capacity: 2048,
            flush_interval: Duration::from_secs(5),
            env_error: None,
        }
    }
}

impl SpanExporterBuilder {
    /// A builder from OpenTelemetry environment variables, read
    /// through `lookup`; `None` when no endpoint is named or
    /// `OTEL_SDK_DISABLED` is `true`. Empty values count as unset.
    ///
    /// - `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`: the full URL, or else
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT` with `/v1/traces` appended;
    /// - `OTEL_SERVICE_NAME`: the resource's `service.name`;
    /// - `OTEL_EXPORTER_OTLP_TRACES_HEADERS`, or else
    ///   `OTEL_EXPORTER_OTLP_HEADERS`: `key=value` pairs separated
    ///   by commas, values percent-decoded; a malformed pair makes
    ///   [`build`](Self::build) fail with `InvalidHeader`.
    #[must_use]
    pub fn from_lookup(
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Option<Self> {
        let env = Env(&lookup);
        if env
            .get("OTEL_SDK_DISABLED")
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("true"))
        {
            return None;
        }
        let mut builder = Self::default().endpoint(env.endpoint()?);
        if let Some(name) = env.get("OTEL_SERVICE_NAME") {
            builder = builder.service_name(name);
        }
        match env.headers() {
            Ok(headers) => Some(
                headers
                    .into_iter()
                    .fold(builder, |b, (k, v)| b.header(k, v)),
            ),
            Err(e) => {
                builder.env_error = Some(e);
                Some(builder)
            }
        }
    }

    /// The collector's traces URL; default [`DEFAULT_TRACES_ENDPOINT`].
    /// Use `http://`: this transport has no TLS.
    #[must_use]
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = url.into();
        self
    }

    /// The resource's `service.name`; default `unknown_service`.
    #[must_use]
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// A request header, such as an auth token for the collector.
    #[must_use]
    pub fn header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.http = self.http.header(name, value);
        self
    }

    /// Per-request timeout in seconds; default 10.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.http = self.http.timeout_secs(secs);
        self
    }

    /// Retries after a 5xx, 429 or transport error; default 3.
    #[must_use]
    pub fn max_retries(mut self, n: u32) -> Self {
        self.http = self.http.max_retries(n);
        self
    }

    /// Spans per request; default 512 (at least 1).
    #[must_use]
    pub fn max_batch(mut self, n: usize) -> Self {
        self.max_batch = n.max(1);
        self
    }

    /// Spans the queue holds before it drops; default 2048 (at least 1).
    #[must_use]
    pub fn queue_capacity(mut self, n: usize) -> Self {
        self.queue_capacity = n.max(1);
        self
    }

    /// Longest a span waits before its batch is posted; default 5 s.
    #[must_use]
    pub const fn flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// Validate the settings and start the export thread.
    ///
    /// # Errors
    /// [`OtlpError::InvalidEndpoint`] for an endpoint that is not an
    /// `http://` URL, [`OtlpError::InvalidHeader`] for an unsafe
    /// header, [`OtlpError::Transport`] if the thread cannot start.
    pub fn build(self) -> OtlpResult<SpanExporter> {
        if let Some(e) = self.env_error {
            return Err(OtlpError::InvalidHeader(e));
        }
        let _ = crate::http::Endpoint::parse(&self.endpoint)
            .map_err(OtlpError::InvalidEndpoint)?;
        let http = self.http.endpoint(self.endpoint.clone()).build();
        crate::http::validate_headers(&http.headers)
            .map_err(OtlpError::InvalidHeader)?;
        let (tx, rx) = mpsc::sync_channel(self.queue_capacity);
        let counters = Arc::new(Counters::default());
        let worker = Worker {
            http,
            service_name: self.service_name,
            max_batch: self.max_batch,
            flush_interval: self.flush_interval,
            counters: Arc::clone(&counters),
        };
        let _ = std::thread::Builder::new()
            .name("rlg-otlp-spans".into())
            .spawn(move || worker.run(&rx))
            .map_err(OtlpError::Transport)?;
        Ok(SpanExporter {
            tx,
            counters,
            endpoint: self.endpoint.into(),
        })
    }
}

struct Worker {
    http: OtlpExporter,
    service_name: String,
    max_batch: usize,
    flush_interval: Duration,
    counters: Arc<Counters>,
}

impl Worker {
    fn run(&self, rx: &Receiver<Msg>) {
        let mut batch = Vec::with_capacity(self.max_batch);
        let mut deadline = Instant::now() + self.flush_interval;
        loop {
            let wait =
                deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(wait) {
                Ok(Msg::Span(span)) => {
                    batch.push(span);
                    if batch.len() >= self.max_batch {
                        self.post(&mut batch);
                    }
                }
                Ok(Msg::Flush(ack)) => {
                    self.post(&mut batch);
                    let _ = ack.send(());
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.post(&mut batch);
                    deadline = Instant::now() + self.flush_interval;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.post(&mut batch);
                    return;
                }
            }
        }
    }

    fn post(&self, batch: &mut Vec<SpanRecord>) {
        if batch.is_empty() {
            return;
        }
        let body = serialise_spans(batch, &self.service_name);
        let n = batch.len() as u64;
        let counter = match self.http.post(&body) {
            Ok(()) => &self.counters.exported,
            Err(_) => &self.counters.failed,
        };
        let _ = counter.fetch_add(n, Ordering::Relaxed);
        batch.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(
        vars: &[(&str, &str)],
    ) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn env_without_endpoint_is_off() {
        assert!(
            SpanExporterBuilder::from_lookup(lookup(&[])).is_none()
        );
        let empty = lookup(&[("OTEL_EXPORTER_OTLP_ENDPOINT", " ")]);
        assert!(SpanExporterBuilder::from_lookup(empty).is_none());
    }

    #[test]
    fn env_base_endpoint_gets_the_traces_path() {
        let b = SpanExporterBuilder::from_lookup(lookup(&[
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318/"),
            ("OTEL_SERVICE_NAME", "nalufx"),
            ("OTEL_EXPORTER_OTLP_HEADERS", "x-a=1, x-b = two"),
        ]))
        .unwrap();
        assert_eq!(b.endpoint, "http://collector:4318/v1/traces");
        assert_eq!(b.service_name, "nalufx");
        let http = b.http.build();
        assert_eq!(http.headers["x-a"], "1");
        assert_eq!(http.headers["x-b"], "two");
        assert_eq!(http.headers.len(), 2);
    }

    #[test]
    fn env_header_values_are_percent_decoded() {
        let b = SpanExporterBuilder::from_lookup(lookup(&[
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c:4318"),
            (
                "OTEL_EXPORTER_OTLP_HEADERS",
                "Authorization=Bearer%20abc%3D%3D,x-team=r%26d",
            ),
        ]))
        .unwrap();
        let http = b.http.build();
        assert_eq!(http.headers["Authorization"], "Bearer abc==");
        assert_eq!(http.headers["x-team"], "r&d");
    }

    #[test]
    fn malformed_env_headers_fail_the_build() {
        for raw in [
            "junk",
            "x-a=1,junk",
            "x=%zz",
            "x=%e9",
            "x=%0D%0AHost:%20evil",
        ] {
            let b = SpanExporterBuilder::from_lookup(lookup(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c:4318"),
                ("OTEL_EXPORTER_OTLP_HEADERS", raw),
            ]))
            .unwrap();
            assert!(
                matches!(b.build(), Err(OtlpError::InvalidHeader(_))),
                "{raw}"
            );
        }
    }

    #[test]
    fn env_traces_specific_settings_win() {
        let b = SpanExporterBuilder::from_lookup(lookup(&[
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://base:4318"),
            (
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
                "http://t:4318/custom",
            ),
            ("OTEL_EXPORTER_OTLP_HEADERS", "x-a=base"),
            ("OTEL_EXPORTER_OTLP_TRACES_HEADERS", "x-a=traces"),
        ]))
        .unwrap();
        assert_eq!(b.endpoint, "http://t:4318/custom");
        assert_eq!(b.http.build().headers["x-a"], "traces");
    }

    #[test]
    fn env_sdk_disabled_turns_it_off() {
        let vars = lookup(&[
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://base:4318"),
            ("OTEL_SDK_DISABLED", "TRUE"),
        ]);
        assert!(SpanExporterBuilder::from_lookup(vars).is_none());
    }

    #[test]
    fn build_refuses_https_and_bad_headers() {
        let https = SpanExporter::builder()
            .endpoint("https://x/v1/traces")
            .build();
        assert!(matches!(https, Err(OtlpError::InvalidEndpoint(_))));
        let header =
            SpanExporter::builder().header("bad name", "v").build();
        assert!(matches!(header, Err(OtlpError::InvalidHeader(_))));
    }

    #[test]
    fn builder_floors_sizes_at_one() {
        let b = SpanExporter::builder().max_batch(0).queue_capacity(0);
        assert_eq!((b.max_batch, b.queue_capacity), (1, 1));
    }
}
