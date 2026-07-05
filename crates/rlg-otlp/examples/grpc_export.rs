// grpc_export.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the OTLP/gRPC transport scaffold (tonic + prost +
// rustls). The gRPC path currently returns `OtlpError::NotWired`
// on export — see docs/adr/0010 and Phase 19c.1 for the
// deferred protobuf integration.
//
// Run with:
//   cargo run -p rlg-otlp --example grpc_export --features grpc

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_otlp::{CircuitBreaker, GrpcOtlpExporter};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let breaker =
        Arc::new(CircuitBreaker::new(5, Duration::from_secs(60)));

    let exporter = GrpcOtlpExporter::builder()
        .endpoint("https://otel-collector.internal:4317")
        .metadata("x-tenant-id", "acme")
        .timeout_secs(5)
        .circuit(Arc::clone(&breaker))
        .build()?;

    println!("gRPC exporter built for {}", exporter.endpoint());

    let record = Log::info("startup complete").component("api");
    match exporter.export_one(&record).await {
        Ok(()) => println!("export ok"),
        Err(e) => {
            // Phase 19c is a scaffold; the wire path returns
            // `NotWired` today. Print to demonstrate the error
            // channel is threaded through correctly.
            eprintln!("scaffold response: {e}");
        }
    }
    Ok(())
}
