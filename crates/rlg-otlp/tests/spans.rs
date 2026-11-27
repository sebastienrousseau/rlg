// spans.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End to end: spans closed under `RlgLayer` reach a fake collector's
//! `/v1/traces` as OTLP JSON, with ids that match the log records.

#![cfg(feature = "tracing")]
#![allow(missing_docs)]

use rlg::log::Log;
use rlg::tracing::{RlgLayer, SpanRecord, SpanSink};
use rlg_otlp::{SpanExportStats, SpanExporter};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use tracing_subscriber::prelude::*;

/// Answers `n` requests with `status`, returning (request line, body).
fn collector(
    n: usize,
    status: u16,
) -> (String, JoinHandle<Vec<(String, String)>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url =
        format!("http://{}/v1/traces", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        (0..n)
            .map(|_| {
                let (sock, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(sock);
                let request = read_request(&mut reader);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: 0\r\n\r\n"
                );
                reader.get_mut().write_all(reply.as_bytes()).unwrap();
                request
            })
            .collect()
    });
    (url, server)
}

fn read_request(reader: &mut impl BufRead) -> (String, String) {
    let mut line = String::new();
    let _ = reader.read_line(&mut line).unwrap();
    let mut len = 0;
    loop {
        let mut header = String::new();
        let _ = reader.read_line(&mut header).unwrap();
        if header == "\r\n" {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            len = value.trim().parse().unwrap();
        }
    }
    let mut body = vec![0; len];
    reader.read_exact(&mut body).unwrap();
    (line, String::from_utf8(body).unwrap())
}

fn spans_in(body: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(body).unwrap();
    v["resourceSpans"][0]["scopeSpans"][0]["spans"]
        .as_array()
        .unwrap()
        .clone()
}

fn by_name<'a>(spans: &'a [Value], name: &str) -> &'a Value {
    spans.iter().find(|s| s["name"] == name).unwrap()
}

#[test]
#[cfg_attr(miri, ignore)]
fn closed_spans_reach_the_collector_linked_to_logs() {
    let (url, server) = collector(1, 200);
    let exporter = SpanExporter::builder()
        .endpoint(&url)
        .service_name("nalufx")
        .flush_interval(Duration::from_secs(60))
        .build()
        .unwrap();
    let logs = Arc::new(Mutex::new(Vec::<Log>::new()));
    let seen = Arc::clone(&logs);
    let layer = RlgLayer::new()
        .with_scrub(move |log: Log| {
            seen.lock().unwrap().push(log.clone());
            log
        })
        .with_span_sink(exporter.clone());
    tracing::subscriber::with_default(
        tracing_subscriber::registry().with(layer),
        || {
            let request = tracing::error_span!(
                "request",
                otel.kind = "server",
                route = "/v1/report"
            );
            let _r = request.enter();
            let attempt =
                tracing::error_span!("llm.attempt", attempt = 1u64);
            let _a = attempt.enter();
            tracing::error!("provider refused");
        },
    );
    assert!(exporter.flush(Duration::from_secs(5)));

    let requests = server.join().unwrap();
    assert!(requests[0].0.starts_with("POST /v1/traces "));
    let spans = spans_in(&requests[0].1);
    let (request, attempt) =
        (by_name(&spans, "request"), by_name(&spans, "llm.attempt"));
    assert_eq!(request["kind"], 2);
    assert_eq!(attempt["traceId"], request["traceId"]);
    assert_eq!(attempt["parentSpanId"], request["spanId"]);
    assert_eq!(attempt["status"]["code"], 2);

    let log = logs.lock().unwrap()[0].clone();
    assert_eq!(log.attributes["trace_id"], request["traceId"]);
    assert_eq!(log.attributes["span_id"], attempt["spanId"]);
    let stats = exporter.stats();
    assert_eq!(
        stats,
        SpanExportStats {
            exported: 2,
            dropped: 0,
            failed: 0
        }
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn full_batch_posts_without_a_flush_and_refusals_count_as_failed() {
    let (url, server) = collector(1, 400);
    let exporter = SpanExporter::builder()
        .endpoint(&url)
        .max_batch(2)
        .max_retries(0)
        .flush_interval(Duration::from_secs(60))
        .build()
        .unwrap();
    for _ in 0..2 {
        exporter.on_span_end(record());
    }
    let requests = server.join().unwrap();
    assert_eq!(spans_in(&requests[0].1).len(), 2);
    assert!(exporter.flush(Duration::from_secs(5)));
    assert_eq!(exporter.stats().failed, 2);
}

#[test]
#[cfg_attr(miri, ignore)]
fn interval_posts_and_drop_drains() {
    let (url, server) = collector(2, 200);
    let exporter = SpanExporter::builder()
        .endpoint(&url)
        .flush_interval(Duration::from_millis(50))
        .build()
        .unwrap();
    exporter.on_span_end(record());
    std::thread::sleep(Duration::from_millis(300));
    exporter.on_span_end(record());
    drop(exporter);
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
}

#[test]
#[cfg_attr(miri, ignore)]
fn a_full_queue_drops_and_counts() {
    // Nothing listens, so the worker is stuck retrying the first
    // batch while the one-slot queue fills.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url =
        format!("http://{}/v1/traces", listener.local_addr().unwrap());
    let exporter = SpanExporter::builder()
        .endpoint(&url)
        .max_batch(1)
        .queue_capacity(1)
        .timeout_secs(1)
        .max_retries(0)
        .build()
        .unwrap();
    for _ in 0..50 {
        exporter.on_span_end(record());
    }
    assert!(exporter.stats().dropped > 0);
    drop(listener);
}

fn record() -> SpanRecord {
    SpanRecord {
        trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
        span_id: "b7ad6b7169203331".into(),
        parent_span_id: None,
        name: "unit",
        target: "spans",
        start_unix_nanos: 1,
        end_unix_nanos: 2,
        attributes: std::collections::BTreeMap::new(),
        kind: rlg::tracing::SpanKind::Internal,
        status: rlg::tracing::SpanStatus::Unset,
    }
}
