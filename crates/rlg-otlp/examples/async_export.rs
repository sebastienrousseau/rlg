// async_export.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the async OTLP/HTTP transport (reqwest + rustls).
// The exporter is fully async; a batch of records is serialised
// once and posted with retry + optional circuit-breaker on top.
//
// Run with:
//   cargo run -p rlg-otlp --example async_export --features async

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::{AsyncOtlpExporter, CircuitBreaker};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Attach a circuit breaker: 3 failures inside a 30-second
    // window trips the breaker; the next requests short-circuit
    // to `OtlpError::CircuitOpen` without touching the network.
    let breaker =
        Arc::new(CircuitBreaker::new(3, Duration::from_secs(30)));

    let exporter = AsyncOtlpExporter::builder()
        .endpoint("https://otel-collector.internal/v1/logs")
        .header("x-api-key", "not-a-real-key")
        .timeout_secs(5)
        .circuit(Arc::clone(&breaker))
        .build()?;

    let records: Vec<Log> = (0..3)
        .map(|i| {
            Log::info(&format!("async-checkout #{i}"))
                .component("orders")
                .with("order_id", 2000 + i)
                .format(LogFormat::OTLP)
        })
        .collect();

    println!("posting {} record(s)…", records.len());
    if let Err(e) = exporter.export_batch(&records).await {
        // Expected without a real collector — the example is
        // primarily a compile/API-shape smoke test.
        eprintln!("export failed (expected offline): {e}");
    } else {
        println!("export ok");
    }
    Ok(())
}
