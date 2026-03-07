// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Criterion benchmark suite for the Log, LogLevel, and LogFormat APIs.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use std::hint::black_box;
use std::str::FromStr;

fn bench_log_build(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_build");

    g.bench_function("build_short", |b| {
        b.iter(|| {
            black_box(
                Log::build(
                    black_box(LogLevel::INFO),
                    black_box("short msg"),
                )
                .component("bench-component")
                .format(LogFormat::MCP),
            );
        });
    });

    g.bench_function("build_with_attributes", |b| {
        b.iter(|| {
            black_box(
                Log::build(
                    black_box(LogLevel::ERROR),
                    black_box("request failed"),
                )
                .component("api-gateway")
                .with("status", 502)
                .with("latency_ms", 142)
                .with("path", "/v1/ingest")
                .format(LogFormat::JSON),
            );
        });
    });

    g.bench_function("raw_struct_init", |b| {
        b.iter(|| {
            black_box(Log {
                session_id: 1,
                time: std::borrow::Cow::Borrowed(
                    "2025-01-01T00:00:00Z",
                ),
                level: LogLevel::INFO,
                component: std::borrow::Cow::Borrowed(
                    "bench-component",
                ),
                description: String::from(black_box("short msg")),
                format: LogFormat::MCP,
                attributes: std::collections::BTreeMap::new(),
            });
        });
    });

    g.finish();
}

fn bench_log_display(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_display");

    let formats = [
        ("CLF", LogFormat::CLF),
        ("JSON", LogFormat::JSON),
        ("GELF", LogFormat::GELF),
        ("MCP", LogFormat::MCP),
        ("OTLP", LogFormat::OTLP),
        ("Logfmt", LogFormat::Logfmt),
        ("ECS", LogFormat::ECS),
    ];

    for (name, fmt) in &formats {
        let log = Log::build(LogLevel::INFO, "benchmark display test")
            .session_id(1)
            .time("2025-01-01T00:00:00Z")
            .component("bench")
            .format(*fmt);

        g.bench_function(*name, |b| {
            b.iter(|| {
                black_box(format!("{}", black_box(&log)));
            });
        });
    }

    g.finish();
}

fn bench_log_level(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_level");

    g.bench_function("from_str", |b| {
        b.iter(|| {
            black_box(LogLevel::from_str(black_box("INFO")).unwrap());
        });
    });

    g.bench_function("to_numeric", |b| {
        b.iter(|| {
            black_box(black_box(LogLevel::INFO).to_numeric());
        });
    });

    g.bench_function("as_str", |b| {
        b.iter(|| {
            black_box(black_box(LogLevel::INFO).as_str());
        });
    });

    g.finish();
}

fn bench_log_format(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_format");

    g.bench_function("from_str", |b| {
        b.iter(|| {
            black_box(LogFormat::from_str(black_box("json")).unwrap());
        });
    });

    g.bench_function("display", |b| {
        b.iter(|| {
            black_box(format!("{}", black_box(LogFormat::JSON)));
        });
    });

    g.finish();
}

fn bench_engine_ingest(c: &mut Criterion) {
    let mut g = c.benchmark_group("engine_ingest");

    g.bench_function("ingest_short", |b| {
        b.iter(|| {
            let log = Log::build(LogLevel::INFO, "short")
                .component("bench")
                .format(LogFormat::MCP);
            let event = rlg::engine::LogEvent {
                level: LogLevel::INFO,
                level_num: 6,
                log: black_box(log),
            };
            rlg::engine::ENGINE.ingest(black_box(event));
        });
    });

    g.bench_function("fire_info", |b| {
        b.iter(|| {
            Log::info(black_box("fire benchmark"))
                .component("bench")
                .fire();
        });
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_log_build,
    bench_log_display,
    bench_log_level,
    bench_log_format,
    bench_engine_ingest,
);
criterion_main!(benches);
