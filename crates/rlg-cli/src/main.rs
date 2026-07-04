// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `rlg` — tail, filter, and convert structured log streams.
//!
//! The binary reads canonical JSON-shaped `rlg` records (one per
//! line) from stdin or a file, applies the `--min-level`,
//! `--component`, and `--attr` filters, and re-emits each surviving
//! record in the format selected with `--format`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use clap::{Parser, ValueEnum};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

/// `rlg` — `jq` for structured logs.
#[derive(Parser, Debug)]
#[command(name = "rlg", version, about, long_about = None)]
struct Cli {
    /// Input file. Reads from stdin if omitted.
    input: Option<PathBuf>,

    /// Output `LogFormat`. Defaults to `Logfmt` for terminal-friendly
    /// reading.
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Logfmt)]
    format: OutputFormat,

    /// Drop records below this level.
    #[arg(short = 'l', long)]
    min_level: Option<LevelArg>,

    /// Restrict to a single component.
    #[arg(short = 'c', long)]
    component: Option<String>,

    /// Filter by a single attribute, formatted as `key=value`. Value
    /// is parsed as JSON (so `key=42` is a number, `key="x"` is a
    /// string, etc.).
    #[arg(short = 'a', long)]
    attr: Option<String>,
}

/// CLI-friendly mirror of [`LogFormat`].
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Clf,
    Cef,
    Elf,
    W3c,
    Apache,
    Log4jXml,
    Json,
    Gelf,
    Logstash,
    Ndjson,
    Mcp,
    Otlp,
    Logfmt,
    Ecs,
}

impl From<OutputFormat> for LogFormat {
    fn from(o: OutputFormat) -> Self {
        match o {
            OutputFormat::Clf => Self::CLF,
            OutputFormat::Cef => Self::CEF,
            OutputFormat::Elf => Self::ELF,
            OutputFormat::W3c => Self::W3C,
            OutputFormat::Apache => Self::ApacheAccessLog,
            OutputFormat::Log4jXml => Self::Log4jXML,
            OutputFormat::Json => Self::JSON,
            OutputFormat::Gelf => Self::GELF,
            OutputFormat::Logstash => Self::Logstash,
            OutputFormat::Ndjson => Self::NDJSON,
            OutputFormat::Mcp => Self::MCP,
            OutputFormat::Otlp => Self::OTLP,
            OutputFormat::Logfmt => Self::Logfmt,
            OutputFormat::Ecs => Self::ECS,
        }
    }
}

/// CLI-friendly mirror of [`LogLevel`].
#[derive(Copy, Clone, Debug, ValueEnum)]
enum LevelArg {
    Trace,
    Debug,
    Verbose,
    Info,
    Warn,
    Error,
    Fatal,
    Critical,
}

impl From<LevelArg> for LogLevel {
    fn from(l: LevelArg) -> Self {
        match l {
            LevelArg::Trace => Self::TRACE,
            LevelArg::Debug => Self::DEBUG,
            LevelArg::Verbose => Self::VERBOSE,
            LevelArg::Info => Self::INFO,
            LevelArg::Warn => Self::WARN,
            LevelArg::Error => Self::ERROR,
            LevelArg::Fatal => Self::FATAL,
            LevelArg::Critical => Self::CRITICAL,
        }
    }
}

fn build_filter(cli: &Cli) -> anyhow::Result<Filter> {
    let mut filter = Filter::new();
    if let Some(level) = cli.min_level {
        filter = filter.min_level(level.into());
    }
    if let Some(comp) = &cli.component {
        filter = filter.component(comp);
    }
    if let Some(spec) = &cli.attr {
        let (k, raw) = spec.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "--attr must be `key=value` (got: `{spec}`)"
            )
        })?;
        let v: serde_json::Value = serde_json::from_str(raw)
            .unwrap_or_else(|_| {
                serde_json::Value::String(raw.to_string())
            });
        filter = filter.attribute(k, v);
    }
    Ok(filter)
}

fn run<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    filter: &Filter,
    format: LogFormat,
) -> anyhow::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(record) = parse_record(&line) else {
            // Pass through unparseable lines verbatim so the caller
            // can still see them — `tail -f` of a non-rlg log
            // shouldn't silently drop entries.
            writeln!(writer, "{line}")?;
            continue;
        };
        if !filter.matches(&record) {
            continue;
        }
        writeln!(writer, "{}", render(record, format))?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let filter = build_filter(&cli)?;
    let format: LogFormat = cli.format.into();

    let stdout = io::stdout();
    let writer = stdout.lock();

    match cli.input.as_ref() {
        Some(path) => {
            let file = File::open(path)?;
            run(BufReader::new(file), writer, &filter, format)
        }
        None => {
            let stdin = io::stdin();
            run(stdin.lock(), writer, &filter, format)
        }
    }
}
