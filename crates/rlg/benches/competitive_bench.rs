// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Competitive benchmark suite comparing RLG against the `log` and `tracing`
//! facade ecosystems.
//!
//! Run: `cargo bench --bench competitive_bench`

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Null backends for fair comparison
// ---------------------------------------------------------------------------

/// A no-op `log::Log` implementation for baseline measurement.
struct NullLogger;

impl log::Log for NullLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, _record: &log::Record<'_>) {}
    fn flush(&self) {}
}

static NULL_LOGGER: NullLogger = NullLogger;

/// Installs the null logger for the `log` facade (best-effort, idempotent).
fn ensure_null_logger() {
    let _ = log::set_logger(&NULL_LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}

// ---------------------------------------------------------------------------
// Group 1: Simple emission (single string message)
// ---------------------------------------------------------------------------

fn simple_emission(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple Emission");

    group.bench_function("RLG fire()", |b| {
        b.iter(|| {
            rlg::log::Log::info(black_box("benchmark message"))
                .component("bench")
                .fire();
        });
    });

    // tracing with a scoped no-op subscriber
    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_writer(std::io::sink)
            .finish(),
    );
    group.bench_function("tracing::info!", |b| {
        b.iter(|| {
            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("benchmark message");
            });
        });
    });

    // log crate with null backend
    ensure_null_logger();
    group.bench_function("log::info!", |b| {
        b.iter(|| {
            log::info!("benchmark message");
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: Structured emission (3 key-value pairs)
// ---------------------------------------------------------------------------

fn structured_emission(c: &mut Criterion) {
    let mut group = c.benchmark_group("Structured Emission");

    group.bench_function("RLG fire() + 3 attrs", |b| {
        b.iter(|| {
            rlg::log::Log::info(black_box("structured event"))
                .component("bench")
                .with("user_id", 42)
                .with("path", "/api/v1")
                .with("latency_ms", 12.5)
                .fire();
        });
    });

    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_writer(std::io::sink)
            .finish(),
    );
    group.bench_function("tracing::info! + 3 fields", |b| {
        b.iter(|| {
            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!(
                    user_id = 42,
                    path = "/api/v1",
                    latency_ms = 12.5,
                    "structured event"
                );
            });
        });
    });

    ensure_null_logger();
    group.bench_function("log::info! + format args", |b| {
        b.iter(|| {
            log::info!(
                "structured event user_id={} path={} latency_ms={}",
                42,
                "/api/v1",
                12.5
            );
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 3: Burst 10k events
// ---------------------------------------------------------------------------

fn burst_10k(c: &mut Criterion) {
    let mut group = c.benchmark_group("Burst 10k");
    group.sample_size(20);

    group.bench_function("RLG fire() x10k", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                rlg::log::Log::info(black_box("burst event"))
                    .with("seq", i)
                    .fire();
            }
        });
    });

    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_writer(std::io::sink)
            .finish(),
    );
    group.bench_function("tracing::info! x10k", |b| {
        b.iter(|| {
            tracing::dispatcher::with_default(&dispatch, || {
                for i in 0..10_000 {
                    tracing::info!(seq = i, "burst event");
                }
            });
        });
    });

    ensure_null_logger();
    group.bench_function("log::info! x10k", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                log::info!("burst event seq={}", i);
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 4: Latency distribution (P50/P95/P99 via Criterion statistics)
// ---------------------------------------------------------------------------

fn latency_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Latency Distribution");
    group.sample_size(1000);

    group.bench_function("RLG fire() latency", |b| {
        b.iter(|| {
            rlg::log::Log::info(black_box("latency probe"))
                .component("bench")
                .fire();
        });
    });

    let dispatch = tracing::Dispatch::new(
        tracing_subscriber::fmt()
            .with_writer(std::io::sink)
            .finish(),
    );
    group.bench_function("tracing::info! latency", |b| {
        b.iter(|| {
            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("latency probe");
            });
        });
    });

    ensure_null_logger();
    group.bench_function("log::info! latency", |b| {
        b.iter(|| {
            log::info!("latency probe");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    simple_emission,
    structured_emission,
    burst_10k,
    latency_distribution,
);
criterion_main!(benches);
