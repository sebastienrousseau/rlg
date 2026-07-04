// layer.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the `RlgLayer` fluent-builder allocation cost and the
// `Layer::layer` wrapping cost. The layer wraps every request so
// its per-instantiation cost matters at cold-start / router-build
// time; the per-request cost is dominated by the wrapped service.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_tower::RlgLayer;
use std::hint::black_box;
use tower_layer::Layer;

#[derive(Clone, Copy)]
struct NoopService;

fn bench_builder(c: &mut Criterion) {
    c.bench_function("rlg-tower/builder", |b| {
        b.iter(|| {
            let layer = RlgLayer::new()
                .level(black_box(LogLevel::INFO))
                .format(black_box(LogFormat::JSON))
                .component("api")
                .header("traceparent");
            black_box(layer)
        });
    });
}

fn bench_layer_wrap(c: &mut Criterion) {
    let layer = RlgLayer::new()
        .level(LogLevel::INFO)
        .format(LogFormat::Logfmt);
    c.bench_function("rlg-tower/layer_wrap", |b| {
        b.iter(|| {
            let wrapped = layer.layer(black_box(NoopService));
            black_box(wrapped)
        });
    });
}

criterion_group!(benches, bench_builder, bench_layer_wrap);
criterion_main!(benches);
