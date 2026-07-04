// capture.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the two hot paths of `rlg-test`:
// 1. Routing records into the capture handle (`Capture::push`).
// 2. Predicate lookup against a populated buffer.

#![allow(missing_docs)]

use criterion::{
    Criterion, Throughput, criterion_group, criterion_main,
};
use rlg::log::Log;
use rlg::log_level::LogLevel;
use rlg_test::{
    Capture, LogExt, capture, description_contains, has_level,
};
use std::hint::black_box;

fn bench_push(c: &mut Criterion) {
    c.bench_function("rlg-test/push", |b| {
        let cap = capture();
        b.iter(|| {
            Log::info(black_box("event"))
                .component("bench")
                .with("user_id", 42_u64)
                .log_to(&cap);
        });
    });
}

fn populate(n: usize) -> Capture {
    let cap = capture();
    for i in 0..n {
        Log::info(&format!("event {i}"))
            .component("bench")
            .with("i", i as u64)
            .log_to(&cap);
    }
    cap
}

fn bench_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("rlg-test/predicates");
    for &n in &[100_usize, 1_000, 10_000] {
        let cap = populate(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("has_level_{n:05}"), |b| {
            b.iter(|| {
                let hit = has_level(black_box(&cap), LogLevel::INFO);
                black_box(hit)
            });
        });
        group.bench_function(
            format!("description_contains_{n:05}"),
            |b| {
                b.iter(|| {
                    let hit = description_contains(
                        black_box(&cap),
                        "event 42",
                    );
                    black_box(hit)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_push, bench_predicates);
criterion_main!(benches);
