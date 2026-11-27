// layer.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The composable [`RlgLayer`].

use super::span::{Scrub, SpanData, SpanSink, SpanStatus};
use super::{RlgVisitor, map_tracing_level};
use crate::log::Log;
use crate::log_format::LogFormat;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing_core::span::{Attributes, Id, Record};
use tracing_core::{Event, Level, Metadata, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::{LookupSpan, Scope};

/// A composable [`tracing_subscriber::Layer`] that routes events into
/// the RLG engine.
///
/// Every record carries the context of the spans it was emitted in:
/// the fields of each span in scope (root first, so an inner span's
/// field wins a clash, and the event's own fields win over all of
/// them), fields recorded later with `Span::record`, and:
///
/// | Attribute | Value |
/// | --- | --- |
/// | `span` | the innermost span's name |
/// | `span_path` | span names from the root, joined with `/` |
/// | `trace_id` | 32 hex characters, shared by the whole trace |
/// | `span_id` | 16 hex characters, the innermost span's id |
///
/// The ids are the ones a [`SpanSink`] receives, so logs and exported
/// spans link up in a collector. `LogFormat::OTLP` renders them as
/// `traceId` and `spanId`.
///
/// # Example
///
/// ```rust
/// use rlg::log_format::LogFormat;
/// use rlg::tracing::RlgLayer;
/// use tracing_subscriber::prelude::*;
///
/// let layer = RlgLayer::new().with_format(LogFormat::JSON);
/// let subscriber = tracing_subscriber::registry().with(layer);
/// tracing::subscriber::with_default(subscriber, || {
///     let request = tracing::info_span!("request", request_id = 7);
///     let _guard = request.enter();
///     tracing::info!("handled"); // carries request_id, trace_id, …
/// });
/// ```
#[derive(Clone)]
pub struct RlgLayer {
    format: LogFormat,
    scrub: Option<Arc<dyn Scrub>>,
    span_sink: Option<Arc<dyn SpanSink>>,
}

impl std::fmt::Debug for RlgLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RlgLayer")
            .field("format", &self.format)
            .field("scrub", &self.scrub.is_some())
            .field("span_sink", &self.span_sink.is_some())
            .finish()
    }
}

impl Default for RlgLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl RlgLayer {
    /// Creates a new `RlgLayer` with the default MCP format.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            format: LogFormat::MCP,
            scrub: None,
            span_sink: None,
        }
    }

    /// Sets the log output format for this layer.
    #[must_use]
    pub const fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Passes every record, and every closed span's attributes,
    /// through `scrub` before it leaves the layer. Use it for
    /// redaction (`rlg_redact::Redactor` implements [`Scrub`]).
    #[must_use]
    pub fn with_scrub(mut self, scrub: impl Scrub) -> Self {
        self.scrub = Some(Arc::new(scrub));
        self
    }

    /// Hands every closed span to `sink`, for example an
    /// `rlg_otlp::SpanExporter` sending OTLP traces.
    #[must_use]
    pub fn with_span_sink(mut self, sink: impl SpanSink) -> Self {
        self.span_sink = Some(Arc::new(sink));
        self
    }

    fn scrubbed(&self, log: Log) -> Log {
        match &self.scrub {
            Some(s) => s.scrub(log),
            None => log,
        }
    }
}

/// Collects recorded values: named fields, plus `message` if any.
fn collect(
    record: impl FnOnce(&mut RlgVisitor),
) -> BTreeMap<String, Value> {
    let mut visitor = RlgVisitor::default();
    record(&mut visitor);
    if !visitor.message.is_empty() {
        let _ = visitor.fields.insert(
            "message".to_string(),
            Value::String(visitor.message),
        );
    }
    visitor.fields
}

/// Copies the context of every span in `scope` into `attributes`.
fn merge_scope<S>(
    attributes: &mut BTreeMap<String, Value>,
    scope: Scope<'_, S>,
) where
    S: for<'a> LookupSpan<'a>,
{
    let mut path: Vec<&'static str> = Vec::new();
    let mut ids = None;
    for span in scope.from_root() {
        path.push(span.name());
        if let Some(data) = span.extensions().get::<SpanData>() {
            attributes.extend(data.fields.clone());
            ids = Some((data.trace_id.clone(), data.span_id.clone()));
        }
    }
    if let Some(name) = path.last() {
        let _ = attributes.insert("span".into(), (*name).into());
        let _ = attributes
            .insert("span_path".into(), path.join("/").into());
    }
    if let Some((trace_id, span_id)) = ids {
        let _ = attributes.insert("trace_id".into(), trace_id.into());
        let _ = attributes.insert("span_id".into(), span_id.into());
    }
}

/// Marks the span an `ERROR` event fired in as failed, unless a
/// status was already set.
fn mark_error<S>(event: &Event<'_>, ctx: &Context<'_, S>, message: &str)
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let Some(span) = ctx.event_span(event) else {
        return;
    };
    let mut extensions = span.extensions_mut();
    if let Some(data) = extensions.get_mut::<SpanData>()
        && data.status == SpanStatus::Unset
    {
        data.status = SpanStatus::Error(message.to_string());
    }
}

impl<S> tracing_subscriber::Layer<S> for RlgLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn enabled(
        &self,
        metadata: &Metadata<'_>,
        _ctx: Context<'_, S>,
    ) -> bool {
        map_tracing_level(*metadata.level()).to_numeric()
            >= crate::engine::ENGINE.filter_level()
    }

    fn on_new_span(
        &self,
        attrs: &Attributes<'_>,
        id: &Id,
        ctx: Context<'_, S>,
    ) {
        crate::engine::ENGINE.inc_spans();
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut data = span.parent().map_or_else(
            || SpanData::new(None),
            |parent| SpanData::new(parent.extensions().get()),
        );
        data.absorb(collect(|v| attrs.record(v)));
        span.extensions_mut().insert(data);
    }

    fn on_record(
        &self,
        id: &Id,
        values: &Record<'_>,
        ctx: Context<'_, S>,
    ) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let fields = collect(|v| values.record(v));
        if let Some(data) = span.extensions_mut().get_mut::<SpanData>()
        {
            data.absorb(fields);
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = RlgVisitor::default();
        event.record(&mut visitor);
        if *metadata.level() == Level::ERROR {
            mark_error(event, &ctx, &visitor.message);
        }

        let level = map_tracing_level(*metadata.level());
        let mut log = Log::build(level, &visitor.message);
        log.component =
            std::borrow::Cow::Owned(metadata.target().to_string());
        log.format = self.format;
        if let Some(scope) = ctx.event_scope(event) {
            merge_scope(&mut log.attributes, scope);
        }
        log.attributes.extend(visitor.fields);
        self.scrubbed(log).fire();
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        crate::engine::ENGINE.dec_spans();
        let Some(sink) = &self.span_sink else {
            return;
        };
        let Some(span) = ctx.span(&id) else {
            return;
        };
        let Some(data) = span.extensions_mut().remove::<SpanData>()
        else {
            return;
        };
        let metadata = span.metadata();
        sink.on_span_end(data.finish(
            metadata.name(),
            metadata.target(),
            self.scrub.as_deref(),
        ));
    }
}
