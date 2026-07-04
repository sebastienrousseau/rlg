// build.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the cost of `RlgWasm` handle construction and cloning.
// The `.info()` / `.warn()` / `.error()` emit paths call
// `eprintln!` on the non-wasm host target, so benchmarking them
// here would pollute output — those paths are exercised by the
// unit tests instead.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg_wasm::RlgWasm;
use std::hint::black_box;

fn bench_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("rlg-wasm/new");
    for name in &["JSON", "Logfmt", "OTLP", "ECS", "NotAFormat"] {
        group.bench_function(*name, |b| {
            b.iter(|| {
                let r =
                    RlgWasm::new(black_box("worker"), black_box(name));
                black_box(r)
            });
        });
    }
    group.finish();
}

fn bench_clone(c: &mut Criterion) {
    let handle = RlgWasm::new("api", "JSON");
    c.bench_function("rlg-wasm/clone", |b| {
        b.iter(|| {
            let c = black_box(&handle).clone();
            black_box(c)
        });
    });
}

criterion_group!(benches, bench_new, bench_clone);
criterion_main!(benches);
