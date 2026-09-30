// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `rlg-mcp` executable.
//!
//! Everything the server does lives in the library. This binary picks
//! the transport from the command line — stdio by default, streamable
//! HTTP or the older HTTP+SSE on request — and hands the library's
//! handler to it. `transport.rs` is the same file in every Rust MCP
//! server of the suite.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[rustfmt::skip]
pub mod transport;

use std::process::ExitCode;

fn main() -> ExitCode {
    transport::run(
        "rlg-mcp",
        env!("CARGO_PKG_VERSION"),
        std::env::args().skip(1),
        rlg_mcp::LogServer::new,
    )
}
