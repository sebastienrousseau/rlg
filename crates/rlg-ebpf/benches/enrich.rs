// enrich.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures the per-record overhead of `ProcessEnricher::enrich`.
// Target from the plan: <5 µs per record.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::log::Log;
use rlg_ebpf::{Enricher, ProcessEnricher};
use std::hint::black_box;

fn bench_enrich(c: &mut Criterion) {
    let e = ProcessEnricher::new();
    c.bench_function("rlg-ebpf/process-enrich", |b| {
        let log = Log::info("checkout completed").component("orders");
        b.iter(|| {
            let enriched = e.enrich(black_box(log.clone()));
            black_box(enriched)
        });
    });
}

criterion_group!(benches, bench_enrich);
criterion_main!(benches);
