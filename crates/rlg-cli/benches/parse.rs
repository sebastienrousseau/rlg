// parse.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the CLI's hot path: parse a JSON record, run a filter,
// render it back out in a target format. This is the per-line cost
// the `rlg` binary pays for every input line at steady state.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use std::hint::black_box;

fn sample_line() -> &'static str {
    r#"{"session_id":7,"time":"2026-07-04T00:00:00.000000000Z","level":"INFO","component":"api","description":"GET /v1/users -> 200","format":"JSON","attributes":{"user_id":42,"region":"eu-west-1","latency_ms":14}}"#
}

fn bench_parse(c: &mut Criterion) {
    let line = sample_line();
    c.bench_function("rlg-cli/parse_record", |b| {
        b.iter(|| {
            let out = parse_record(black_box(line)).unwrap();
            black_box(out)
        });
    });
}

fn bench_filter_matches(c: &mut Criterion) {
    let record = parse_record(sample_line()).unwrap();
    let filter =
        Filter::new().min_level(LogLevel::INFO).component("api");
    c.bench_function("rlg-cli/filter_matches", |b| {
        b.iter(|| {
            let hit = filter.matches(black_box(&record));
            black_box(hit)
        });
    });
}

fn bench_render_pipeline(c: &mut Criterion) {
    let line = sample_line();
    let mut group = c.benchmark_group("rlg-cli/render");
    for &fmt in &[
        LogFormat::JSON,
        LogFormat::Logfmt,
        LogFormat::ECS,
        LogFormat::OTLP,
    ] {
        group.bench_function(format!("{fmt:?}"), |b| {
            b.iter(|| {
                let record = parse_record(black_box(line)).unwrap();
                let out = render(record, fmt);
                black_box(out)
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_filter_matches,
    bench_render_pipeline
);
criterion_main!(benches);
