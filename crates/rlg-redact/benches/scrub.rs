// scrub.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the per-record cost of running every default regex
// pattern across a typical log description + attribute payload.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log::Log;
use rlg_redact::Redactor;
use std::hint::black_box;

fn no_pii_payload() -> Log {
    Log::info("checkout completed in 142ms")
        .component("orders")
        .with("order_id", 42_u64)
        .with("region", "eu-west-1")
        .with("latency_ms", 142_u64)
}

fn heavy_pii_payload() -> Log {
    Log::info("card 4111-1111-1111-1111 declined for alice@example.com")
        .component("payments")
        .with("client_ip", "192.0.2.42")
        .with("session_token", "Bearer abc123XYZdef.ghi")
        .with("aws_key", "AKIAIOSFODNN7EXAMPLE")
}

fn bench_redact(c: &mut Criterion) {
    let redactor = Redactor::with_defaults();
    let mut group = c.benchmark_group("rlg-redact/scrub");

    group.bench_function("no_pii_match", |b| {
        let log = no_pii_payload();
        b.iter(|| {
            let out = redactor.scrub(black_box(log.clone()));
            black_box(out)
        });
    });

    group.bench_function("heavy_pii_match", |b| {
        let log = heavy_pii_payload();
        b.iter(|| {
            let out = redactor.scrub(black_box(log.clone()));
            black_box(out)
        });
    });

    group.bench_function("with_defaults_construction", |b| {
        b.iter(|| black_box(Redactor::with_defaults()));
    });

    group.finish();
}

criterion_group!(benches, bench_redact);
criterion_main!(benches);
