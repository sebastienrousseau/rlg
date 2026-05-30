// aggregate.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Benchmarks `Report::from_lines` over varying input sizes. The
// aggregation passes JSON parsing on every line, so the throughput
// number is a fair proxy for what `rlg-report <file>` will achieve.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg_report::Report;
use std::hint::black_box;

fn make_lines(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            let level = match i % 5 {
                0 => "ERROR",
                1 => "WARN",
                _ => "INFO",
            };
            let comp = match i % 3 {
                0 => "api",
                1 => "db",
                _ => "orchestrator",
            };
            format!(
                "{{\"session_id\":{i},\"time\":\"t\",\"level\":\"{level}\",\"component\":\"{comp}\",\"description\":\"event {kind}\",\"format\":\"JSON\",\"attributes\":{{\"latency_ms\":{lat}}}}}",
                kind = i % 10,
                lat = (i % 500) as u64,
            )
        })
        .collect()
}

fn bench_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("rlg-report/from_lines");
    for &n in &[100, 1_000, 10_000] {
        let lines = make_lines(n);
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        group.bench_function(format!("lines_{n:05}"), |b| {
            b.iter(|| {
                let report = Report::from_lines(
                    black_box(refs.iter().copied()),
                );
                black_box(report)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_aggregate);
criterion_main!(benches);
