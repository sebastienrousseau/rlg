// honeycomb.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates exporting a batch of rlg records to a Honeycomb-style
// OTLP/HTTP endpoint.
//
// Run with:
//   HONEYCOMB_API_KEY=… cargo run -p rlg-otlp --example honeycomb

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::OtlpExporter;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("HONEYCOMB_API_KEY")
        .unwrap_or_else(|_| "no-key-supplied".into());

    let exporter = OtlpExporter::builder()
        .endpoint("https://api.honeycomb.io/v1/logs")
        .header("x-honeycomb-team", api_key)
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
            "export failed (expected without a real API key): {e}"
        );
    } else {
        println!("export ok");
    }
    Ok(())
}
