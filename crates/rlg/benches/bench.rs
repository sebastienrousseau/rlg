// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(missing_docs)]
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// Import Log struct for benchmarking
use rlg::log::Log;

// Import tokio for async file writing
use tokio::io::AsyncWriteExt;

// Import LogFormat for benchmarking log formatting
use rlg::log_format::LogFormat;

// Import LogLevel for benchmarking log formatting
use rlg::log_level::LogLevel;

// Benchmark creating new Log instances
fn new_benchmark(c: &mut Criterion) {
    c.bench_function("new", |b| {
        b.iter(|| {
            let log = Log::build(LogLevel::INFO, "test log message")
                .session_id(123)
                .time("2023-01-23 14:04:09.881393 +00:00:00")
                .component("test")
                .format(LogFormat::CLF);
            black_box(log);
        })
    });
}

// Benchmark formatting Log structs to string
fn format_benchmark(c: &mut Criterion) {
    let clf_log = Log::build(LogLevel::INFO, "test log message")
        .session_id(123)
        .time("2023-01-23 14:04:09.881393 +00:00:00")
        .component("test")
        .format(LogFormat::CLF);

    let json_log = Log::build(LogLevel::INFO, "test log message")
        .session_id(123)
        .time("2023-01-23 14:04:09.881393 +00:00:00")
        .component("test")
        .format(LogFormat::JSON);

    c.bench_function("clf_format", |b| b.iter(|| format!("{clf_log}")));
    c.bench_function("json_format", |b| {
        b.iter(|| format!("{json_log}"))
    });
}

// Benchmark async writing logs to files vs Lock-Free Engine
fn write_benchmark(c: &mut Criterion) {
    // Create test CLF log
    let clf_log = Log::build(LogLevel::INFO, "test log message")
        .session_id(123)
        .time("2023-01-23 14:04:09.881393 +00:00:00")
        .component("test")
        .format(LogFormat::CLF);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("Ingestion Speed");

    group.bench_function("Legacy Tokio File IO", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut file = tokio::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("bench_legacy.log")
                    .await
                    .unwrap();
                let _ = file
                    .write_all(format!("{clf_log}").as_bytes())
                    .await;
            })
        })
    });

    group.bench_function("Brutalist Lock-Free Engine", |b| {
        b.iter(|| {
            let event = rlg::engine::LogEvent {
                level: LogLevel::INFO,
                level_num: 6,
                log: clf_log.clone(),
            };
            rlg::engine::ENGINE.ingest(black_box(event));
        })
    });

    group.finish();
}

// Group benchmarks together
criterion_group!(
    benches,
    new_benchmark,
    format_benchmark,
    write_benchmark
);
criterion_main!(benches);
