// pipeline.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the library-mode API: parse an NDJSON stream,
// filter by component + minimum level, and re-render each match
// in a different LogFormat.
//
// Same primitives that the `rlg` binary uses internally — call
// them directly from your own Rust code to drop the CLI process
// hop.
//
// Run with: cargo run -p rlg-cli --example pipeline

#![allow(missing_docs)]

use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};

const NDJSON: &[&str] = &[
    r#"{"session_id":1,"time":"2026-07-04T00:00:00.000000000Z","level":"INFO","component":"api","description":"GET /v1/users -> 200","format":"JSON","attributes":{"latency_ms":14}}"#,
    r#"{"session_id":2,"time":"2026-07-04T00:00:01.000000000Z","level":"WARN","component":"api","description":"rate-limited","format":"JSON","attributes":{"client":"acme"}}"#,
    r#"{"session_id":3,"time":"2026-07-04T00:00:02.000000000Z","level":"INFO","component":"db","description":"query 12ms","format":"JSON","attributes":{}}"#,
    r#"{"session_id":4,"time":"2026-07-04T00:00:03.000000000Z","level":"ERROR","component":"api","description":"payment refused","format":"JSON","attributes":{"provider":"stripe"}}"#,
    r#"not-json this-line-passes-through"#,
];

fn main() {
    // Keep only WARN-and-above records from the `api` component,
    // and re-render them as ECS (Elastic Common Schema).
    let filter =
        Filter::new().min_level(LogLevel::WARN).component("api");

    let mut matches = 0_u32;
    let mut passthrough = 0_u32;

    for line in NDJSON {
        match parse_record(line) {
            Ok(record) => {
                if filter.matches(&record) {
                    matches += 1;
                    println!("{}", render(record, LogFormat::ECS));
                }
            }
            Err(_) => {
                // Unparseable lines pass through verbatim so upstream
                // noise doesn't get dropped silently.
                passthrough += 1;
                println!("{line}");
            }
        }
    }

    eprintln!(
        "── summary ─────\nmatched     : {matches}\npassthrough : {passthrough}",
    );
}
