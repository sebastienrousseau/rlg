// hello.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Host-side smoke test for the rlg-wasm shape. The same `RlgWasm`
// type is reachable from JavaScript when built via `wasm-pack`;
// here we exercise the API in native Rust for CI convenience.

#![allow(missing_docs)]

use rlg_wasm::RlgWasm;

fn main() {
    let logger = RlgWasm::new("worker", "JSON");
    logger.info("worker booted", None);
    logger.info(
        "request handled",
        Some(r#"{"url":"/api","status":200,"latency_ms":12}"#.to_string()),
    );
    logger.warn("rate limited", None);
    logger.error(
        "db timeout",
        Some(r#"{"db":"primary","elapsed_ms":5012}"#.to_string()),
    );
}
