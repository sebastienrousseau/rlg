// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `rlg-report` — aggregation digest for rlg log streams.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use clap::{Parser, ValueEnum};
use rlg_report::Report;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

/// `rlg-report` — log digest / analytics for rlg streams.
#[derive(Parser, Debug)]
#[command(name = "rlg-report", version, about, long_about = None)]
struct Cli {
    /// Input file. Reads from stdin when omitted.
    input: Option<PathBuf>,

    /// Output shape.
    #[arg(short, long, value_enum, default_value_t = OutputShape::Text)]
    format: OutputShape,

    /// How many top descriptions to surface.
    #[arg(long, default_value_t = 10)]
    top: usize,
}

/// Output shapes the binary can emit.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputShape {
    /// Pretty terminal-friendly tables.
    Text,
    /// JSON for piping into a dashboard.
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let lines: Vec<String> = match cli.input.as_ref() {
        Some(path) => BufReader::new(File::open(path)?)
            .lines()
            .collect::<Result<_, _>>()?,
        None => stdin
            .lock()
            .lines()
            .collect::<Result<_, _>>()?,
    };
    let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let report =
        Report::from_lines_with_top(line_refs, cli.top);
    match cli.format {
        OutputShape::Text => println!("{}", report.to_text()),
        OutputShape::Json => println!("{}", report.to_json()?),
    }
    Ok(())
}
