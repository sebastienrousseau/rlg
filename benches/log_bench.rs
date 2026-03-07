// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Divan benchmark suite for the Log, LogLevel, and LogFormat APIs.
//!
//! Proves zero performance regressions from the macro-driven refactoring by
//! measuring allocation counts, throughput across payload sizes, and the cost
//! of macro-generated `FromStr`/`Display` implementations.
//!
//! Run:  `cargo bench --bench log_bench`
//! Alloc report:  every benchmark row shows `alloc:` and `dealloc:` columns.

#![allow(missing_docs)]

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Payload sizes used across throughput benchmarks.
// Small = typical one-liner, Medium = structured context, Large = JSON trace.
// ---------------------------------------------------------------------------

const PAYLOADS: &[&str] = &[
    "short msg", // 10 B
    "A moderately sized log message with some operational context", // ~60 B
];

fn large_payload() -> String {
    // ~10 KB JSON-style trace payload
    let entry =
        r#"{"trace_id":"abc123","span_id":"def456","key":"val"},"#;
    let mut buf = String::with_capacity(10240);
    buf.push('[');
    while buf.len() < 10200 {
        buf.push_str(entry);
    }
    buf.truncate(buf.len() - 1); // remove trailing comma
    buf.push(']');
    buf
}

// ===========================================================================
// Module: Log::build fluent API — construction cost + allocation tracking
// ===========================================================================

mod log_build {
    use divan::{Bencher, black_box};
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    #[divan::bench(args = super::PAYLOADS)]
    fn build_only(bencher: Bencher, msg: &&str) {
        bencher.bench(|| {
            black_box(
                Log::build(black_box(LogLevel::INFO), black_box(msg))
                    .component("bench-component")
                    .format(LogFormat::MCP),
            );
        });
    }

    #[divan::bench]
    fn build_with_attributes(bencher: Bencher) {
        bencher.bench(|| {
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
    }

    #[divan::bench]
    fn build_large_payload(bencher: Bencher) {
        let payload = super::large_payload();
        bencher.bench(|| {
            black_box(
                Log::build(
                    black_box(LogLevel::DEBUG),
                    black_box(payload.as_str()),
                )
                .component("trace-collector")
                .format(LogFormat::OTLP),
            );
        });
    }

    /// Baseline: raw struct init (no fluent API) to compare allocation counts.
    #[divan::bench(args = super::PAYLOADS)]
    fn raw_struct_init(bencher: Bencher, msg: &&str) {
        bencher.bench(|| {
            black_box(Log {
                session_id: String::from("1"),
                time: String::from("2025-01-01T00:00:00Z"),
                level: LogLevel::INFO,
                component: String::from("bench-component"),
                description: String::from(black_box(*msg)),
                format: LogFormat::MCP,
                attributes: std::collections::BTreeMap::new(),
            });
        });
    }
}

// ===========================================================================
// Module: Log Display (format!) — serialization throughput per format
// ===========================================================================

mod log_display {
    use divan::{Bencher, black_box};
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    const FORMATS: &[LogFormat] = &[
        LogFormat::CLF,
        LogFormat::JSON,
        LogFormat::GELF,
        LogFormat::MCP,
        LogFormat::OTLP,
        LogFormat::Logfmt,
        LogFormat::ECS,
    ];

    #[divan::bench(args = FORMATS)]
    fn display_format(bencher: Bencher, fmt: &LogFormat) {
        let log = Log::build(LogLevel::INFO, "benchmark display test")
            .session_id("bench-sid")
            .time("2025-01-01T00:00:00Z")
            .component("bench")
            .format(*fmt);

        bencher.bench(|| {
            black_box(format!("{}", black_box(&log)));
        });
    }

    #[divan::bench(args = FORMATS)]
    fn display_format_with_attrs(bencher: Bencher, fmt: &LogFormat) {
        let log = Log::build(LogLevel::WARN, "structured event")
            .session_id("bench-sid")
            .time("2025-01-01T00:00:00Z")
            .component("bench")
            .with("request_id", "req-abc-123")
            .with("user_id", 42)
            .with("ok", true)
            .format(*fmt);

        bencher.bench(|| {
            black_box(format!("{}", black_box(&log)));
        });
    }
}

// ===========================================================================
// Module: LogLevel — macro-generated FromStr / Display / to_numeric
// ===========================================================================

mod log_level_conversion {
    use divan::{Bencher, black_box};
    use rlg::log_level::LogLevel;
    use std::str::FromStr;

    const LEVEL_STRINGS: &[&str] = &[
        "ALL", "TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR",
        "FATAL", "CRITICAL", "NONE",
    ];

    #[divan::bench(args = LEVEL_STRINGS)]
    fn from_str(bencher: Bencher, s: &&str) {
        bencher.bench(|| {
            black_box(LogLevel::from_str(black_box(s)).unwrap());
        });
    }

    #[divan::bench(args = LEVEL_STRINGS)]
    fn from_str_lowercase(bencher: Bencher, s: &&str) {
        let lower = s.to_lowercase();
        bencher.bench(|| {
            black_box(LogLevel::from_str(black_box(&lower)).unwrap());
        });
    }

    const ALL_LEVELS: &[LogLevel] = &[
        LogLevel::ALL,
        LogLevel::TRACE,
        LogLevel::DEBUG,
        LogLevel::VERBOSE,
        LogLevel::INFO,
        LogLevel::WARN,
        LogLevel::ERROR,
        LogLevel::FATAL,
        LogLevel::CRITICAL,
        LogLevel::NONE,
    ];

    #[divan::bench(args = ALL_LEVELS)]
    fn display(bencher: Bencher, level: &LogLevel) {
        bencher.bench(|| {
            black_box(format!("{}", black_box(level)));
        });
    }

    #[divan::bench(args = ALL_LEVELS)]
    fn as_str(bencher: Bencher, level: &LogLevel) {
        bencher.bench(|| {
            black_box(black_box(level).as_str());
        });
    }

    #[divan::bench(args = ALL_LEVELS)]
    fn as_str_lowercase(bencher: Bencher, level: &LogLevel) {
        bencher.bench(|| {
            black_box(black_box(level).as_str_lowercase());
        });
    }

    #[divan::bench(args = ALL_LEVELS)]
    fn to_numeric(bencher: Bencher, level: &LogLevel) {
        bencher.bench(|| {
            black_box(black_box(*level).to_numeric());
        });
    }

    const NUMERIC_VALUES: &[u8] = &[0, 3, 6, 8, 10, 11];

    #[divan::bench(args = NUMERIC_VALUES)]
    fn from_numeric(bencher: Bencher, n: &u8) {
        bencher.bench(|| {
            black_box(LogLevel::from_numeric(black_box(*n)));
        });
    }

    #[divan::bench(args = ALL_LEVELS)]
    fn includes(bencher: Bencher, level: &LogLevel) {
        bencher.bench(|| {
            black_box(
                black_box(*level).includes(black_box(LogLevel::DEBUG)),
            );
        });
    }
}

// ===========================================================================
// Module: LogFormat — macro-generated FromStr / Display
// ===========================================================================

mod log_format_conversion {
    use divan::{Bencher, black_box};
    use rlg::log_format::LogFormat;
    use std::str::FromStr;

    const FORMAT_STRINGS: &[&str] = &[
        "clf", "json", "cef", "elf", "w3c", "gelf", "apache",
        "logstash", "log4jxml", "ndjson", "mcp", "otlp", "logfmt",
        "ecs",
    ];

    #[divan::bench(args = FORMAT_STRINGS)]
    fn from_str(bencher: Bencher, s: &&str) {
        bencher.bench(|| {
            black_box(LogFormat::from_str(black_box(s)).unwrap());
        });
    }

    const ALL_FORMATS: &[LogFormat] = &[
        LogFormat::CLF,
        LogFormat::JSON,
        LogFormat::CEF,
        LogFormat::GELF,
        LogFormat::MCP,
        LogFormat::OTLP,
        LogFormat::Logfmt,
        LogFormat::ECS,
    ];

    #[divan::bench(args = ALL_FORMATS)]
    fn display(bencher: Bencher, fmt: &LogFormat) {
        bencher.bench(|| {
            black_box(format!("{}", black_box(fmt)));
        });
    }
}

// ===========================================================================
// Module: Engine ingestion — end-to-end throughput without I/O
// ===========================================================================

mod engine_ingest {
    use divan::{Bencher, black_box};
    use rlg::engine::{ENGINE, LogEvent};
    use rlg::log_level::LogLevel;

    #[divan::bench(args = [10, 64, 256, 1024])]
    fn ingest_payload(bencher: Bencher, size: usize) {
        let payload = vec![b'X'; size];
        bencher.bench(|| {
            let event = LogEvent {
                level: LogLevel::INFO,
                level_num: 6,
                payload: black_box(payload.clone()),
            };
            ENGINE.ingest(black_box(event));
        });
    }
}

// ===========================================================================
// Module: Full pipeline — build + format + ingest (realistic hot-path)
// ===========================================================================

mod full_pipeline {
    use divan::{Bencher, black_box};
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    #[divan::bench(args = super::PAYLOADS)]
    fn build_format_ingest(bencher: Bencher, msg: &&str) {
        bencher.bench(|| {
            let log =
                Log::build(black_box(LogLevel::INFO), black_box(msg))
                    .component("pipeline-bench")
                    .format(LogFormat::MCP);
            let payload = format!("{log}\n").into_bytes();
            let event = rlg::engine::LogEvent {
                level: LogLevel::INFO,
                level_num: 6,
                payload: black_box(payload),
            };
            rlg::engine::ENGINE.ingest(black_box(event));
        });
    }

    #[divan::bench]
    fn fire_info(bencher: Bencher) {
        bencher.bench(|| {
            Log::info(black_box("fire benchmark"))
                .component("bench")
                .fire();
        });
    }
}
