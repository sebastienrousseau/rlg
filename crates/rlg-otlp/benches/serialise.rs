// serialise.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Benchmarks the `serialise_batch` hot path. Network I/O is not
// benched here because it depends on the collector's RTT.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::serialise_batch;
use std::hint::black_box;

fn sample_batch(n: usize) -> Vec<Log> {
    (0..n)
        .map(|i| {
            Log::info("checkout completed")
                .component("orders")
                .with("order_id", i as u64)
                .with("trace_id", format!("trace-{i:08x}"))
                .with("span_id", format!("span-{i:08x}"))
                .format(LogFormat::OTLP)
        })
        .collect()
}

fn bench_serialise(c: &mut Criterion) {
    let mut group = c.benchmark_group("rlg-otlp/serialise_batch");
    for &n in &[1, 10, 100, 1_000] {
        let batch = sample_batch(n);
        group.bench_function(format!("batch_{n:04}"), |b| {
            b.iter(|| {
                let body = serialise_batch(black_box(&batch)).unwrap();
                black_box(body)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_serialise);
criterion_main!(benches);
