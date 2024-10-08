// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(missing_docs)]
use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};

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
            let log = Log::new(
                "123",
                "2023-01-23 14:04:09.881393 +00:00:00",
                &LogLevel::INFO,
                "test",
                "test log message",
                &LogFormat::CLF,
            );
            black_box(log);
        })
    });
}

// Benchmark formatting Log structs to string
fn format_benchmark(c: &mut Criterion) {
    let clf_log = Log::new(
        "123",
        "2023-01-23 14:04:09.881393 +00:00:00",
        &LogLevel::INFO,
        "test",
        "test log message",
        &LogFormat::CLF,
    );

    let json_log = Log::new(
        "123",
        "2023-01-23 14:04:09.881393 +00:00:00",
        &LogLevel::INFO,
        "test",
        "test log message",
        &LogFormat::JSON,
    );

    c.bench_function("clf_format", |b| b.iter(|| format!("{clf_log}")));
    c.bench_function("json_format", |b| {
        b.iter(|| format!("{json_log}"))
    });
}

// Benchmark async writing logs to files
fn write_benchmark(c: &mut Criterion) {
    // Create test CLF log
    let clf_log = Log::new(
        "123",
        "2023-01-23 14:04:09.881393 +00:00:00",
        &LogLevel::INFO,
        "test",
        "test log message",
        &LogFormat::CLF,
    );

    c.bench_function("clf_write", |b| {
        b.iter(|| {
            // Runtime for async block to write to file
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Open file and write log
                let mut file =
                    tokio::fs::File::create("log.txt").await.unwrap();
                let _ = file
                    .write_all(format!("{clf_log}").as_bytes())
                    .await;
            })
        })
    });
}

// Group benchmarks together
criterion_group!(
    benches,
    new_benchmark,
    format_benchmark,
    write_benchmark
);
criterion_main!(benches);
