// circuit_breaker.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates attaching a CircuitBreaker to an OtlpExporter and
// walking the breaker through its three states: closed (all
// requests allowed), open (all requests rejected after N failures
// in the window), and half-open (single probe request allowed
// after the cool-down window).
//
// Run with: cargo run -p rlg-otlp --example circuit_breaker

#![allow(missing_docs)]

use rlg_otlp::{CircuitBreaker, OtlpExporter};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    // Three failures within a 30-second window trip the breaker;
    // once open, wait 5 s before allowing a probe.
    let breaker =
        Arc::new(CircuitBreaker::new(3, Duration::from_secs(30)));

    let _exporter = OtlpExporter::builder()
        .endpoint("https://otel-collector.internal/v1/logs")
        .circuit(Arc::clone(&breaker))
        .build();

    println!("── walking breaker states ─────");

    // Closed: allow() returns true for every request.
    println!("start (closed)           allow()={}", breaker.allow());

    // Record 3 failures — hits the threshold.
    for i in 1..=3 {
        breaker.record_failure();
        println!(
            "after failure #{i:<2}         allow()={}",
            breaker.allow()
        );
    }

    // One success closes the breaker again.
    breaker.record_success();
    println!("after success (closed)   allow()={}", breaker.allow());
}
