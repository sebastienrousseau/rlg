// blocking_http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The blocking exporter against a fake collector on a local socket:
//! 2xx succeeds, 5xx and 429 are retried, any other status is final,
//! and the circuit breaker hears about each outcome.

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_otlp::{CircuitBreaker, OtlpError, OtlpExporter};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

fn sample() -> Log {
    Log::build(LogLevel::INFO, "msg")
        .component("svc")
        .session_id(7)
        .time("2026-05-30T00:00:00.000000000Z")
        .format(LogFormat::OTLP)
}

/// A collector that answers the n-th request with `statuses[n]` and
/// hands back every request line and body it read.
fn fake_collector(
    statuses: &'static [u16],
) -> (String, JoinHandle<Vec<(String, String)>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url =
        format!("http://{}/v1/logs", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        statuses
            .iter()
            .map(|status| {
                let (sock, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(sock);
                let request = read_request(&mut reader);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
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

fn exporter(url: &str, retries: u32) -> OtlpExporter {
    OtlpExporter::builder()
        .endpoint(url)
        .header("x-api-key", "k")
        .timeout_secs(2)
        .max_retries(retries)
        .backoff_base(Duration::ZERO)
        .build()
}

#[test]
fn a_2xx_succeeds_with_an_otlp_envelope() {
    let (url, server) = fake_collector(&[200]);
    exporter(&url, 0).export_one(&sample()).unwrap();
    let requests = server.join().unwrap();
    let (line, body) = &requests[0];
    assert!(line.starts_with("POST /v1/logs HTTP/1.1"), "{line}");
    let json: serde_json::Value = serde_json::from_str(body).unwrap();
    assert!(
        json["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0]
            .is_object()
    );
}

#[test]
fn a_503_is_retried_until_it_succeeds() {
    let cb = Arc::new(CircuitBreaker::new(1, Duration::from_secs(60)));
    let (url, server) = fake_collector(&[503, 429, 204]);
    let e = OtlpExporter::builder()
        .endpoint(url)
        .max_retries(2)
        .backoff_base(Duration::ZERO)
        .circuit(Arc::clone(&cb))
        .build();
    e.export_one(&sample()).unwrap();
    assert_eq!(server.join().unwrap().len(), 3);
    assert!(cb.allow(), "a success must not consume the breaker");
}

#[test]
fn exhausted_retries_report_the_last_status() {
    let cb = Arc::new(CircuitBreaker::new(1, Duration::from_secs(60)));
    let (url, server) = fake_collector(&[500, 502]);
    let e = OtlpExporter::builder()
        .endpoint(url)
        .max_retries(1)
        .backoff_base(Duration::ZERO)
        .circuit(Arc::clone(&cb))
        .build();
    let res = e.export_one(&sample());
    assert!(matches!(res, Err(OtlpError::BadStatus(502))), "{res:?}");
    assert_eq!(server.join().unwrap().len(), 2);
    assert!(!cb.allow(), "the failure must reach the breaker");
}

#[test]
fn a_4xx_is_final_and_not_retried() {
    let (url, server) = fake_collector(&[400]);
    let res = exporter(&url, 3).export_one(&sample());
    assert!(matches!(res, Err(OtlpError::BadStatus(400))), "{res:?}");
    assert_eq!(server.join().unwrap().len(), 1);
}

#[test]
fn a_tripped_breaker_rejects_without_a_request() {
    let cb = Arc::new(CircuitBreaker::new(1, Duration::from_secs(60)));
    cb.record_failure();
    let e = OtlpExporter::builder()
        .endpoint("http://127.0.0.1:1/v1/logs")
        .circuit(cb)
        .build();
    assert!(matches!(
        e.export_one(&sample()),
        Err(OtlpError::CircuitOpen)
    ));
}
