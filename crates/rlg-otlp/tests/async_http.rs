// async_http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The async exporter against a fake collector on a local socket:
//! request framing, status handling, retries, timeouts and
//! build-time validation.

#![cfg(feature = "async")]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_otlp::{AsyncOtlpExporter, CircuitBreaker, OtlpError};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn sample(level: LogLevel) -> Log {
    Log::build(level, "msg")
        .component("svc")
        .session_id(7)
        .time("2026-05-30T00:00:00.000000000Z")
        .format(LogFormat::OTLP)
}

/// A collector on an ephemeral port that answers the n-th
/// connection with `responses[n]` (or never, for `None`) and
/// hands back every request it read.
async fn fake_collector(
    responses: Vec<Option<&'static [u8]>>,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener =
        tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url =
        format!("http://{}/v1/logs", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let mut seen = Vec::new();
        for response in responses {
            let (mut sock, _) = listener.accept().await.unwrap();
            seen.push(read_request(&mut sock).await);
            match response {
                Some(bytes) => sock.write_all(bytes).await.unwrap(),
                None => {
                    tokio::time::sleep(Duration::from_secs(5)).await
                }
            }
        }
        seen
    });
    (url, task)
}

async fn read_request(sock: &mut TcpStream) -> String {
    let mut buf = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let n = sock.read(&mut chunk).await.unwrap();
        buf.extend_from_slice(&chunk[..n]);
        let text = String::from_utf8_lossy(&buf).to_string();
        if let Some(end) = text.find("\r\n\r\n") {
            let len: usize = text
                .lines()
                .find_map(|l| l.strip_prefix("Content-Length: "))
                .unwrap()
                .parse()
                .unwrap();
            if buf.len() >= end + 4 + len || n == 0 {
                return text;
            }
        }
    }
}

fn exporter(url: &str, retries: u32) -> AsyncOtlpExporter {
    AsyncOtlpExporter::builder()
        .endpoint(url)
        .header("x-api-key", "k")
        .timeout_secs(1)
        .max_retries(retries)
        .backoff_base(Duration::ZERO)
        .build()
        .unwrap()
}

#[tokio::test]
async fn posts_framed_otlp_json_and_accepts_2xx() {
    let (url, server) =
        fake_collector(vec![Some(b"HTTP/1.1 200 OK\r\n\r\n")]).await;
    exporter(&url, 0)
        .export_one(&sample(LogLevel::INFO))
        .await
        .unwrap();
    let requests = server.await.unwrap();
    let req = &requests[0];
    assert!(req.starts_with("POST /v1/logs HTTP/1.1\r\n"));
    assert!(req.contains("\r\nx-api-key: k\r\n"));
    assert!(req.contains("\r\nContent-Type: application/json\r\n"));
    let body = &req[req.find("\r\n\r\n").unwrap() + 4..];
    let json: serde_json::Value = serde_json::from_str(body).unwrap();
    assert!(
        json["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0]
            .is_object()
    );
}

#[tokio::test]
async fn retries_a_503_then_succeeds_and_closes_the_breaker() {
    let cb = Arc::new(CircuitBreaker::new(1, Duration::from_secs(60)));
    let (url, server) = fake_collector(vec![
        Some(b"HTTP/1.1 503 Service Unavailable\r\n\r\n"),
        Some(b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 204 No Content\r\n\r\n"),
    ])
    .await;
    let e = AsyncOtlpExporter::builder()
        .endpoint(url)
        .max_retries(1)
        .backoff_base(Duration::ZERO)
        .circuit(Arc::clone(&cb))
        .build()
        .unwrap();
    e.export_one(&sample(LogLevel::INFO)).await.unwrap();
    assert_eq!(server.await.unwrap().len(), 2);
    assert!(cb.allow());
}

#[tokio::test]
async fn a_4xx_is_final_and_not_retried() {
    let (url, server) =
        fake_collector(vec![Some(b"HTTP/1.1 400 Bad Request\r\n\r\n")])
            .await;
    let res =
        exporter(&url, 3).export_one(&sample(LogLevel::INFO)).await;
    assert!(matches!(res, Err(OtlpError::BadStatus(400))));
    assert_eq!(server.await.unwrap().len(), 1);
}

#[tokio::test]
async fn exhausted_5xx_retries_report_the_status() {
    let (url, _server) = fake_collector(vec![
        Some(b"HTTP/1.1 500 Oops\r\n\r\n"),
        Some(b"HTTP/1.1 429 Slow Down\r\n\r\n"),
    ])
    .await;
    let res =
        exporter(&url, 1).export_one(&sample(LogLevel::INFO)).await;
    assert!(matches!(res, Err(OtlpError::BadStatus(429))));
}

#[tokio::test]
async fn a_silent_collector_times_out() {
    let (url, _server) = fake_collector(vec![None]).await;
    let res =
        exporter(&url, 0).export_one(&sample(LogLevel::INFO)).await;
    match res {
        Err(OtlpError::AsyncTransport(e)) => {
            assert_eq!(e.kind(), io::ErrorKind::TimedOut);
        }
        other => panic!("expected a timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn a_non_http_answer_is_a_transport_error() {
    let (url, _server) =
        fake_collector(vec![Some(b"SSH-2.0-OpenSSH_9.6\r\n\r\n")])
            .await;
    let res =
        exporter(&url, 0).export_one(&sample(LogLevel::INFO)).await;
    match res {
        Err(OtlpError::AsyncTransport(e)) => {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
        }
        other => panic!("expected invalid data, got {other:?}"),
    }
}

#[test]
fn build_defaults_to_the_local_collector() {
    let e = AsyncOtlpExporter::builder().build().unwrap();
    assert_eq!(e.endpoint(), "http://localhost:4318/v1/logs");
}

#[test]
fn build_rejects_https_and_unsafe_headers() {
    let https = AsyncOtlpExporter::builder()
        .endpoint("https://api.honeycomb.io/v1/logs")
        .build();
    assert!(matches!(https, Err(OtlpError::InvalidEndpoint(_))));
    let injected = AsyncOtlpExporter::builder()
        .header("x-api-key", "k\r\nHost: evil")
        .build();
    assert!(matches!(injected, Err(OtlpError::InvalidHeader(_))));
}
