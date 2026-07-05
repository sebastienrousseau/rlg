// tail.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Measures `tail_log` throughput across representative log-file
// sizes. The MCP server's `tail_log` tool is the most-invoked path
// from LLM agents doing incremental log inspection.

#![allow(missing_docs)]

use criterion::{
    Criterion, Throughput, criterion_group, criterion_main,
};
use rlg_mcp::tail_log;
use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;

fn make_fixture(n: usize) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    for i in 0..n {
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
        writeln!(
            f,
            "{{\"session_id\":{i},\"time\":\"2026-07-04T00:00:00.000000000Z\",\"level\":\"{level}\",\"component\":\"{comp}\",\"description\":\"event {i}\",\"format\":\"JSON\",\"attributes\":{{}}}}",
        )
        .expect("write");
    }
    f.flush().expect("flush");
    f
}

fn bench_tail(c: &mut Criterion) {
    let mut group = c.benchmark_group("rlg-mcp/tail_log");
    for &n in &[100_usize, 1_000, 10_000] {
        let file = make_fixture(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("records_{n:05}"), |b| {
            b.iter(|| {
                let out =
                    tail_log(black_box(file.path()), 100).unwrap();
                black_box(out)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tail);
criterion_main!(benches);
