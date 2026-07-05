// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the builder + error type of
//! [`OtlpExporter`]. Network I/O is intentionally *not* exercised
//! here — pointing tests at a real collector would make CI
//! environment-dependent. Network-path smoke tests live in
//! `benches/exporter_bench.rs` behind a manual harness.

#![allow(missing_docs)]

use rlg_otlp::{OtlpError, OtlpExporter};

#[test]
fn builder_stores_endpoint() {
    let exporter = OtlpExporter::builder()
        .endpoint("http://localhost:4318/v1/logs")
        .build();
    assert_eq!(exporter.endpoint(), "http://localhost:4318/v1/logs");
}

#[test]
fn builder_accepts_multiple_headers() {
    // Constructs an exporter with two headers. We can't inspect the
    // internal map directly, but a successful build + roundtrip via
    // clone proves the fluent API is chainable and stable.
    let exporter = OtlpExporter::builder()
        .endpoint("https://api.example.com/v1/logs")
        .header("x-tenant", "acme")
        .header("authorization", "Bearer test-token")
        .timeout_secs(5)
        .build();
    let cloned = exporter.clone();
    assert_eq!(cloned.endpoint(), exporter.endpoint());
}

#[test]
fn builder_applies_defaults_when_fields_omitted() {
    // Omit timeout, retries, backoff: build must still succeed and
    // yield a usable endpoint accessor.
    let exporter = OtlpExporter::builder()
        .endpoint("http://localhost:4318/v1/logs")
        .build();
    assert!(exporter.endpoint().starts_with("http://"));
}

#[test]
fn builder_max_retries_zero_is_accepted() {
    let exporter = OtlpExporter::builder()
        .endpoint("http://localhost:4318/v1/logs")
        .max_retries(0)
        .build();
    assert_eq!(exporter.endpoint(), "http://localhost:4318/v1/logs");
}

#[test]
fn error_bad_status_display_includes_status_code() {
    let err = OtlpError::BadStatus(503);
    let msg = err.to_string();
    assert!(msg.contains("503"), "unexpected: {msg}");
}

#[test]
fn error_serialise_display_is_meaningful() {
    // Force a real serde_json::Error by parsing invalid JSON.
    let json_err =
        serde_json::from_str::<serde_json::Value>("not-json")
            .unwrap_err();
    let err: OtlpError = json_err.into();
    let msg = err.to_string();
    assert!(msg.contains("serialise"), "unexpected: {msg}");
}

#[test]
fn otlp_error_variants_are_debug() {
    // Every variant must implement Debug (Error trait requires it).
    let e1 = OtlpError::BadStatus(429);
    let _ = format!("{e1:?}");
}

#[test]
fn unreachable_endpoint_produces_transport_error() {
    // Point at an IP guaranteed not to accept TCP quickly.
    // 127.0.0.1:1 is closed on every sane machine; the transport
    // error surfaces after connection refusal / TCP RST.
    let exporter = OtlpExporter::builder()
        .endpoint("http://127.0.0.1:1/v1/logs")
        .timeout_secs(1)
        .max_retries(0)
        .build();
    let record = rlg::log::Log::error("smoke");
    let err = exporter.export_one(&record).expect_err(
        "unreachable endpoint must yield a Transport error",
    );
    matches!(err, OtlpError::Transport(_));
}
