// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Span context and span hand-off in `RlgLayer`. Records are caught
//! by a capturing [`Scrub`], closed spans by a capturing [`SpanSink`].

use super::{RlgLayer, SpanKind, SpanRecord, SpanSink, SpanStatus};
use crate::engine::ENGINE;
use crate::log::Log;
use crate::log_level::LogLevel;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tracing_subscriber::prelude::*;

#[derive(Clone, Default)]
struct Captured {
    logs: Arc<Mutex<Vec<Log>>>,
    spans: Arc<Mutex<Vec<SpanRecord>>>,
}

struct Sink(Arc<Mutex<Vec<SpanRecord>>>);

impl SpanSink for Sink {
    fn on_span_end(&self, span: SpanRecord) {
        self.0.lock().unwrap().push(span);
    }
}

impl Captured {
    fn layer(&self) -> RlgLayer {
        let logs = Arc::clone(&self.logs);
        RlgLayer::new()
            .with_scrub(move |log: Log| {
                logs.lock().unwrap().push(log.clone());
                log
            })
            .with_span_sink(Sink(Arc::clone(&self.spans)))
    }

    /// Runs `f` under a registry with the capturing layer, with the
    /// engine filter open, and returns what was caught.
    fn run(f: impl FnOnce()) -> Self {
        let captured = Self::default();
        let prior = ENGINE.filter_level();
        ENGINE.set_filter(LogLevel::ALL.to_numeric());
        let subscriber =
            tracing_subscriber::registry().with(captured.layer());
        tracing::subscriber::with_default(subscriber, f);
        ENGINE.set_filter(prior);
        captured
    }

    fn log(&self, i: usize) -> Log {
        self.logs.lock().unwrap()[i].clone()
    }

    fn span(&self, name: &str) -> SpanRecord {
        let spans = self.spans.lock().unwrap();
        spans.iter().find(|s| s.name == name).cloned().unwrap()
    }
}

fn attr<'a>(log: &'a Log, key: &str) -> &'a Value {
    log.attributes.get(key).unwrap_or(&Value::Null)
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn event_carries_the_parent_span_fields() {
    let c = Captured::run(|| {
        let span = tracing::info_span!(
            "request",
            request_id = "r-1",
            status = 200u64
        );
        let _g = span.enter();
        tracing::info!("handled");
    });
    let log = c.log(0);
    assert_eq!(log.description, "handled");
    assert_eq!(attr(&log, "request_id"), &json!("r-1"));
    assert_eq!(attr(&log, "status"), &json!(200));
    assert_eq!(attr(&log, "span"), &json!("request"));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn nested_spans_merge_root_first_inner_wins() {
    let c = Captured::run(|| {
        let outer = tracing::info_span!(
            "request",
            request_id = "r-2",
            method = "POST"
        );
        let _o = outer.enter();
        let inner = tracing::info_span!(
            "allocate",
            method = "hrp",
            rows = 12u64
        );
        let _i = inner.enter();
        tracing::info!("solved");
    });
    let log = c.log(0);
    assert_eq!(attr(&log, "request_id"), &json!("r-2"));
    assert_eq!(attr(&log, "method"), &json!("hrp"));
    assert_eq!(attr(&log, "rows"), &json!(12));
    assert_eq!(attr(&log, "span"), &json!("allocate"));
    assert_eq!(attr(&log, "span_path"), &json!("request/allocate"));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn field_recorded_after_creation_is_carried() {
    let c = Captured::run(|| {
        let span = tracing::info_span!(
            "request",
            status = tracing::field::Empty
        );
        let _g = span.enter();
        tracing::info!("before");
        span.record("status", 503u64);
        tracing::info!("after");
    });
    assert_eq!(attr(&c.log(0), "status"), &Value::Null);
    assert_eq!(attr(&c.log(1), "status"), &json!(503));
    assert_eq!(c.span("request").attributes["status"], json!(503));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn event_field_overrides_span_field() {
    let c = Captured::run(|| {
        let span =
            tracing::info_span!("llm.attempt", status = "pending");
        let _g = span.enter();
        tracing::info!(status = "ok", "attempt done");
    });
    assert_eq!(attr(&c.log(0), "status"), &json!("ok"));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn ids_link_logs_and_spans() {
    let c = Captured::run(|| {
        let outer = tracing::info_span!("request");
        let _o = outer.enter();
        let inner = tracing::info_span!("report");
        let _i = inner.enter();
        tracing::info!("inside");
    });
    let (outer, inner, log) =
        (c.span("request"), c.span("report"), c.log(0));
    assert_eq!(outer.trace_id.len(), 32);
    assert_eq!(inner.span_id.len(), 16);
    assert_eq!(inner.trace_id, outer.trace_id);
    assert_ne!(inner.span_id, outer.span_id);
    assert_eq!(outer.parent_span_id, None);
    assert_eq!(
        inner.parent_span_id.as_deref(),
        Some(outer.span_id.as_str())
    );
    assert_eq!(attr(&log, "trace_id"), &json!(outer.trace_id));
    assert_eq!(attr(&log, "span_id"), &json!(inner.span_id));
    assert!(inner.start_unix_nanos <= inner.end_unix_nanos);
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn separate_roots_get_separate_traces() {
    let c = Captured::run(|| {
        drop(tracing::info_span!("a"));
        drop(tracing::info_span!("b"));
    });
    assert_ne!(c.span("a").trace_id, c.span("b").trace_id);
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn otel_fields_set_kind_and_status() {
    let c = Captured::run(|| {
        let server = tracing::info_span!(
            "request",
            otel.kind = "server",
            otel.status_code = "ok"
        );
        let _s = server.enter();
        let client =
            tracing::info_span!("llm.attempt", otel.kind = "client");
        let _c = client.enter();
        tracing::error!("provider timed out");
    });
    let (server, client) = (c.span("request"), c.span("llm.attempt"));
    assert_eq!(server.kind, SpanKind::Server);
    assert_eq!(server.status, SpanStatus::Ok);
    assert!(!server.attributes.contains_key("otel.kind"));
    assert_eq!(client.kind, SpanKind::Client);
    assert_eq!(
        client.status,
        SpanStatus::Error("provider timed out".into())
    );
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn otel_status_error_carries_its_message() {
    let c = Captured::run(|| {
        let span = tracing::info_span!(
            "forecast",
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty
        );
        span.record("otel.status_message", "no data");
        span.record("otel.status_code", "ERROR");
        let late = tracing::info_span!(
            "report",
            otel.status_code = "error",
            otel.status_message = tracing::field::Empty
        );
        late.record("otel.status_message", "quota");
    });
    assert_eq!(
        c.span("forecast").status,
        SpanStatus::Error("no data".into())
    );
    assert_eq!(
        c.span("report").status,
        SpanStatus::Error("quota".into())
    );
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn scrub_rewrites_logs_and_span_attributes() {
    let spans = Arc::new(Mutex::new(Vec::new()));
    let layer = RlgLayer::new()
        .with_scrub(|mut log: Log| {
            if log.attributes.contains_key("secret") {
                let _ = log
                    .attributes
                    .insert("secret".into(), json!("[REDACTED]"));
            }
            log
        })
        .with_span_sink(Sink(Arc::clone(&spans)));
    let prior = ENGINE.filter_level();
    ENGINE.set_filter(LogLevel::ALL.to_numeric());
    tracing::subscriber::with_default(
        tracing_subscriber::registry().with(layer),
        || {
            drop(tracing::info_span!("request", secret = "sk-123"));
        },
    );
    ENGINE.set_filter(prior);
    let span = spans.lock().unwrap()[0].clone();
    assert_eq!(span.attributes["secret"], json!("[REDACTED]"));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn event_outside_spans_has_no_span_context() {
    let c = Captured::run(|| tracing::info!(k = 1u64, "bare"));
    let log = c.log(0);
    assert_eq!(attr(&log, "k"), &json!(1));
    assert!(!log.attributes.contains_key("trace_id"));
    assert!(!log.attributes.contains_key("span"));
}

#[test]
fn debug_shows_which_hooks_are_set() {
    let plain = format!("{:?}", RlgLayer::new());
    assert!(plain.contains("scrub: false"));
    let hooked =
        format!("{:?}", RlgLayer::new().with_scrub(|l: Log| l));
    assert!(hooked.contains("scrub: true"));
}

#[test]
#[cfg_attr(miri, ignore)]
#[serial_test::serial(engine_filter)]
fn caller_is_the_event_call_site_not_the_layer() {
    let line = line!() + 1;
    let c = Captured::run(|| tracing::info!("here"));
    let expected = format!("{}:{line}", file!());
    assert_eq!(attr(&c.log(0), "caller"), &json!(expected));
}
