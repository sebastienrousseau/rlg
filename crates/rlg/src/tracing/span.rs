// span.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Span context for [`RlgLayer`](super::RlgLayer): the fields, ids and
//! timing the layer keeps for each open span, and the hooks it calls
//! ([`Scrub`] before a record leaves, [`SpanSink`] when a span closes).

use crate::log::Log;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Field names the layer reads as span metadata rather than as
/// attributes, following the `tracing-opentelemetry` convention.
const OTEL_KIND: &str = "otel.kind";
const OTEL_STATUS_CODE: &str = "otel.status_code";
const OTEL_STATUS_MESSAGE: &str = "otel.status_message";
/// A root span field carrying an incoming W3C `traceparent`.
const TRACEPARENT: &str = "traceparent";

/// Rewrites a record before it leaves the layer: redaction, field
/// filtering, enrichment.
///
/// [`RlgLayer`](super::RlgLayer) applies it to every log record it
/// builds and to the attributes of every closed span it hands to a
/// [`SpanSink`], so one scrubber covers both paths. Closures
/// `Fn(Log) -> Log` implement it; so does `rlg_redact::Redactor`
/// with that crate's `tracing` feature.
pub trait Scrub: Send + Sync + 'static {
    /// Returns the record to emit in place of `log`.
    fn scrub(&self, log: Log) -> Log;
}

impl<F> Scrub for F
where
    F: Fn(Log) -> Log + Send + Sync + 'static,
{
    fn scrub(&self, log: Log) -> Log {
        self(log)
    }
}

/// Receives each span when it closes. `rlg_otlp::SpanExporter`
/// implements it to send spans to a collector as OTLP traces.
///
/// It is called on the thread that closes the span, so an
/// implementation must hand the record off without blocking.
pub trait SpanSink: Send + Sync + 'static {
    /// Takes ownership of a closed span.
    fn on_span_end(&self, span: SpanRecord);
}

/// The role of a span in a trace, as OTLP's `SpanKind`. Set it with
/// an `otel.kind` field (`"server"`, `"client"`, `"producer"`,
/// `"consumer"` or `"internal"`); the default is internal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpanKind {
    /// An operation inside the application.
    #[default]
    Internal,
    /// Handling a request from a remote client.
    Server,
    /// A request to a remote service.
    Client,
    /// Sending a message to a broker.
    Producer,
    /// Receiving a message from a broker.
    Consumer,
}

impl SpanKind {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "internal" => Some(Self::Internal),
            "server" => Some(Self::Server),
            "client" => Some(Self::Client),
            "producer" => Some(Self::Producer),
            "consumer" => Some(Self::Consumer),
            _ => None,
        }
    }
}

/// How a span ended, as OTLP's `Status`.
///
/// A span becomes [`SpanStatus::Error`] when an `ERROR` event fires
/// directly inside it, or when its `otel.status_code` field is
/// `"error"`; `"ok"` sets [`SpanStatus::Ok`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SpanStatus {
    /// No status was set.
    #[default]
    Unset,
    /// The operation was marked as successful.
    Ok,
    /// The operation failed; the message says why.
    Error(String),
}

/// A closed span, as handed to a [`SpanSink`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanRecord {
    /// 32 lowercase hex characters, shared by every span in the trace.
    pub trace_id: String,
    /// 16 lowercase hex characters.
    pub span_id: String,
    /// The enclosing span's id; `None` for a root span.
    pub parent_span_id: Option<String>,
    /// The span's name, as given to `tracing::span!`.
    pub name: &'static str,
    /// The span's target (usually its module path).
    pub target: &'static str,
    /// Wall-clock creation time, nanoseconds since the Unix epoch.
    pub start_unix_nanos: u64,
    /// Wall-clock close time, nanoseconds since the Unix epoch.
    pub end_unix_nanos: u64,
    /// The span's fields, after [`Scrub`].
    pub attributes: BTreeMap<String, Value>,
    /// The span's role in the trace.
    pub kind: SpanKind,
    /// How the span ended.
    pub status: SpanStatus,
}

/// Nanoseconds since the Unix epoch, saturating.
pub(super) fn unix_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
}

/// What the layer stores in each span's extensions.
#[derive(Debug)]
pub(super) struct SpanData {
    pub(super) trace_id: String,
    pub(super) span_id: String,
    pub(super) parent_span_id: Option<String>,
    pub(super) start_unix_nanos: u64,
    pub(super) fields: BTreeMap<String, Value>,
    pub(super) kind: SpanKind,
    pub(super) status: SpanStatus,
    /// The last `otel.status_message`, kept for an error status
    /// that may be recorded later.
    status_message: Option<String>,
}

impl SpanData {
    /// A new span: a child of `parent` when given, else a trace root.
    pub(super) fn new(parent: Option<&Self>) -> Self {
        let trace_id = parent
            .map_or_else(crate::utils::generate_trace_id, |p| {
                p.trace_id.clone()
            });
        Self {
            trace_id,
            span_id: crate::utils::generate_span_id(),
            parent_span_id: parent.map(|p| p.span_id.clone()),
            start_unix_nanos: unix_nanos(),
            fields: BTreeMap::new(),
            kind: SpanKind::Internal,
            status: SpanStatus::Unset,
            status_message: None,
        }
    }

    /// Takes in recorded fields, moving the `otel.*` ones into
    /// `kind` and `status`.
    pub(super) fn absorb(&mut self, fields: BTreeMap<String, Value>) {
        self.fields.extend(fields);
        if let Some(kind) = self.take_str(OTEL_KIND) {
            self.kind = SpanKind::parse(&kind).unwrap_or(self.kind);
        }
        if let Some(message) = self.take_str(OTEL_STATUS_MESSAGE) {
            if let SpanStatus::Error(current) = &mut self.status {
                current.clone_from(&message);
            }
            self.status_message = Some(message);
        }
        if let Some(code) = self.take_str(OTEL_STATUS_CODE) {
            self.status = match code.to_ascii_lowercase().as_str() {
                "ok" => SpanStatus::Ok,
                "error" => SpanStatus::Error(
                    self.status_message.clone().unwrap_or_default(),
                ),
                _ => SpanStatus::Unset,
            };
        }
    }

    /// For a root span: adopts the trace and remote parent of a valid
    /// `traceparent` field, consuming the field. An invalid value
    /// leaves the span a new trace root.
    pub(super) fn adopt_traceparent(&mut self) {
        if self.parent_span_id.is_some() {
            return;
        }
        let header = self.take_str(TRACEPARENT);
        if let Some((trace_id, parent)) =
            header.as_deref().and_then(super::traceparent::parse)
        {
            self.trace_id = trace_id;
            self.parent_span_id = Some(parent);
        }
    }

    fn take_str(&mut self, key: &str) -> Option<String> {
        match self.fields.remove(key)? {
            Value::String(s) => Some(s),
            other => Some(other.to_string()),
        }
    }

    /// The closed span, its attributes passed through `scrub`.
    pub(super) fn finish(
        self,
        name: &'static str,
        target: &'static str,
        scrub: Option<&dyn Scrub>,
    ) -> SpanRecord {
        let attributes = match scrub {
            Some(s) => scrub_attributes(s, self.fields),
            None => self.fields,
        };
        SpanRecord {
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            name,
            target,
            start_unix_nanos: self.start_unix_nanos,
            end_unix_nanos: unix_nanos(),
            attributes,
            kind: self.kind,
            status: self.status,
        }
    }
}

/// Runs span attributes through a [`Scrub`] by carrying them in an
/// otherwise empty record.
fn scrub_attributes(
    scrub: &dyn Scrub,
    attributes: BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let carrier = Log {
        attributes,
        ..Log::default()
    };
    scrub.scrub(carrier).attributes
}
