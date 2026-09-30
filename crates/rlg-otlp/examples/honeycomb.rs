// honeycomb.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates exporting a batch of rlg records to Honeycomb through
// a local OpenTelemetry Collector. The exporter speaks plain HTTP to
// the Collector on localhost:4318; the Collector holds the API key
// and the TLS connection to Honeycomb (see the crate docs for its
// configuration).
//
// Run with a Collector listening on 127.0.0.1:4318:
//   cargo run -p rlg-otlp --example honeycomb

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::OtlpExporter;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = OtlpExporter::builder()
        .endpoint(rlg_otlp::DEFAULT_ENDPOINT)
        .timeout_secs(10)
        .max_retries(3)
        .backoff_base(Duration::from_millis(200))
        .build();

    let records: Vec<Log> = (0..5)
        .map(|i| {
            Log::info(&format!("checkout completed #{i}"))
                .component("orders")
                .with("order_id", 1000 + i)
                .with("trace_id", format!("trace-{i}"))
                .format(LogFormat::OTLP)
        })
        .collect();

    println!(
        "exporting {} record(s) to {}",
        records.len(),
        exporter.endpoint()
    );

    if let Err(e) = exporter.export_batch(&records) {
        eprintln!(
            "export failed (expected without a local Collector): {e}"
        );
    } else {
        println!("export ok");
    }
    Ok(())
}
