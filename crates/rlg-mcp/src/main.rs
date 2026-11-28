// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `rlg-mcp` executable.
//!
//! Everything the server does lives in the library. This binary takes
//! `--root` (the directory the tools may read) off the command line,
//! then picks the transport from the rest — stdio by default,
//! streamable HTTP or the older HTTP+SSE on request — and hands the
//! library's handler to it. `transport.rs` is the same file in every
//! Rust MCP server of the suite, so `--root` is handled here.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use rlg_mcp::{LogServer, Root, binds_beyond_loopback, take_root_arg};
use std::process::ExitCode;

const NAME: &str = "rlg-mcp";

fn main() -> ExitCode {
    let (root_arg, rest) = match take_root_arg(std::env::args().skip(1))
    {
        Ok(split) => split,
        Err(e) => {
            eprintln!("{NAME}: {e}");
            return ExitCode::from(2);
        }
    };
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", rlg_mcp::transport::usage(NAME));
        println!(
            "\n  --root <dir>        the directory the tools may read \
             (default: RLG_MCP_ROOT, else the current directory)"
        );
        return ExitCode::SUCCESS;
    }
    let root = match open_root(root_arg) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{NAME}: {e}");
            return ExitCode::from(2);
        }
    };
    if binds_beyond_loopback(&rest) {
        eprintln!(
            "{NAME}: warning: the HTTP transports have no authentication \
             and this --host is not loopback; anyone who can reach it can \
             read log files under {}",
            root.path().display()
        );
    }
    rlg_mcp::transport::run(
        NAME,
        env!("CARGO_PKG_VERSION"),
        rest,
        move || LogServer::with_root(root.clone()),
    )
}

/// The root from `--root`, else `RLG_MCP_ROOT`, else the current
/// directory with a warning. Running without a root was deprecated in
/// 0.0.21; `--root` becomes required in the first release on or after
/// 2027-02-26 (see `DEPRECATIONS.md`).
fn open_root(arg: Option<String>) -> Result<Root, String> {
    let chosen = arg.or_else(|| {
        std::env::var("RLG_MCP_ROOT").ok().filter(|v| !v.is_empty())
    });
    let explicit = chosen.is_some();
    let dir = chosen.unwrap_or_else(|| ".".to_string());
    let root =
        Root::new(&dir).map_err(|e| format!("--root {dir}: {e}"))?;
    if !explicit {
        eprintln!(
            "{NAME}: warning: no --root given; the tools read only under \
             {}. Pass --root <dir> or set RLG_MCP_ROOT (required from the first \
             release on or after 2027-02-26).",
            root.path().display()
        );
    }
    Ok(root)
}
