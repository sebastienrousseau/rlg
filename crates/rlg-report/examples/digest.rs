// digest.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the library-mode aggregation API. The same logic
// runs inside the `rlg-report` binary.
//
// Run with: cargo run -p rlg-report --example digest

#![allow(missing_docs)]

use rlg_report::Report;

const SAMPLE: &[&str] = &[
    r#"{"session_id":1,"time":"t","level":"INFO","component":"api","description":"GET /v1/users -> 200","format":"JSON","attributes":{"http.latency_ms":14}}"#,
    r#"{"session_id":2,"time":"t","level":"INFO","component":"api","description":"GET /v1/users -> 200","format":"JSON","attributes":{"http.latency_ms":18}}"#,
    r#"{"session_id":3,"time":"t","level":"ERROR","component":"db","description":"connection timeout","format":"JSON","attributes":{"latency_ms":5012}}"#,
    r#"{"session_id":4,"time":"t","level":"WARN","component":"orchestrator","description":"queue depth high","format":"JSON","attributes":{"depth":4096}}"#,
    r#"{"session_id":5,"time":"t","level":"FATAL","component":"db","description":"connection timeout","format":"JSON","attributes":{}}"#,
];

fn main() {
    let report = Report::from_lines(SAMPLE.iter().copied());

    print!("{}", report.to_text());

    println!("\n── JSON ─────────────────────");
    println!("{}", report.to_json().unwrap());

    println!("\nerror_count() = {}", report.error_count());
    if let Some(latency) = &report.latency {
        println!("p99 latency  = {} ms", latency.p99);
    }

    println!("\n── Top-N clamp = 2 ────────────");
    let clamped =
        Report::from_lines_with_top(SAMPLE.iter().copied(), 2);
    for (description, count) in clamped.top_descriptions.iter() {
        println!("{count:>3}  {description}");
    }
    assert!(clamped.top_descriptions.len() <= 2);
}
