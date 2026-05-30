// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `rlg-mcp` — Model Context Protocol server over stdio.
//!
//! Read one JSON-RPC 2.0 request per line on stdin; write one
//! response (or nothing, for notifications) per line on stdout.
//! Dispatch goes through [`rlg_mcp::dispatch`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use anyhow::Context;
use std::io::{self, BufRead, Write};

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.context("read stdin")?;
        if line.trim().is_empty() {
            continue;
        }
        let request: rlg_mcp::Request =
            match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let resp = rlg_mcp::Response::err(
                        None,
                        -32_700,
                        format!("parse error: {e}"),
                    );
                    writeln!(out, "{}", serde_json::to_string(&resp)?)?;
                    out.flush()?;
                    continue;
                }
            };
        if let Some(response) = rlg_mcp::dispatch(&request) {
            writeln!(out, "{}", serde_json::to_string(&response)?)?;
            out.flush()?;
        }
    }
    Ok(())
}
