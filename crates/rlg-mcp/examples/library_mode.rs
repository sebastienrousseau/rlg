// library_mode.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates invoking the three MCP tools (`tail_log`,
// `filter_log`, `summarize_errors`) as pure functions from Rust,
// without spinning up the JSON-RPC 2.0 stdio transport.
//
// This is the same code path the MCP server executes under the
// hood; calling it directly is useful for embedding rlg-mcp's
// analytics into another Rust process, or unit-testing custom
// tooling that composes with the same primitives.
//
// Run with: cargo run -p rlg-mcp --example library_mode

#![allow(missing_docs)]

use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::Filter;
use rlg_mcp::{filter_log, summarize_errors, tail_log};
use std::io::Write;

const FIXTURE: &[&str] = &[
    r#"{"session_id":1,"time":"2026-07-04T00:00:00.000000000Z","level":"INFO","component":"auth","description":"login ok","format":"JSON","attributes":{}}"#,
    r#"{"session_id":2,"time":"2026-07-04T00:00:01.000000000Z","level":"ERROR","component":"auth","description":"token expired","format":"JSON","attributes":{}}"#,
    r#"{"session_id":3,"time":"2026-07-04T00:00:02.000000000Z","level":"WARN","component":"db","description":"slow query","format":"JSON","attributes":{"latency_ms":812}}"#,
    r#"{"session_id":4,"time":"2026-07-04T00:00:03.000000000Z","level":"ERROR","component":"db","description":"deadlock retry","format":"JSON","attributes":{}}"#,
    r#"{"session_id":5,"time":"2026-07-04T00:00:04.000000000Z","level":"FATAL","component":"auth","description":"OIDC issuer unreachable","format":"JSON","attributes":{}}"#,
];

fn main() -> std::io::Result<()> {
    // Write the NDJSON fixture to a real temp file so the tools
    // see the same interface (`&Path`) an MCP client would give
    // them.
    let mut file = tempfile::NamedTempFile::new()?;
    for line in FIXTURE {
        writeln!(file, "{line}")?;
    }
    file.flush()?;
    let path = file.path();

    println!("── tail_log (last 3, Logfmt) ─────");
    for line in tail_log(path, 3)? {
        println!("{line}");
    }

    println!("\n── filter_log (min_level=ERROR, format=JSON) ─────");
    let filter = Filter::new().min_level(LogLevel::ERROR);
    for line in filter_log(path, &filter, LogFormat::JSON)? {
        println!("{line}");
    }

    println!("\n── summarize_errors (by component) ─────");
    for (component, count) in summarize_errors(path)? {
        println!("{component:<12} {count}");
    }

    Ok(())
}
