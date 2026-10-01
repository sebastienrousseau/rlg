// ops.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The four operations as plain functions over a file path, callable
//! and testable without a transport.

use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ---------------------------------------------------------------------------
// Tool implementations — pure functions over a file path.
// ---------------------------------------------------------------------------

/// Return the last `n` parseable records from `path`, rendered in
/// `Logfmt`. Unparseable lines are skipped.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn tail_log(path: &Path, n: usize) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut all = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line) {
            all.push(render(record, LogFormat::Logfmt));
        }
    }
    let start = all.len().saturating_sub(n);
    Ok(all.split_off(start))
}

/// Apply `filter` to every record in `path` and return matches
/// rendered in `format`.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn filter_log(
    path: &Path,
    filter: &Filter,
    format: LogFormat,
) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line)
            && filter.matches(&record)
        {
            out.push(render(record, format));
        }
    }
    Ok(out)
}

/// Count error+ records grouped by component.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn summarize_errors(
    path: &Path,
) -> std::io::Result<BTreeMap<String, u64>> {
    let file = File::open(path)?;
    let mut buckets: BTreeMap<String, u64> = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line)
            && record.level.to_numeric() >= LogLevel::ERROR.to_numeric()
        {
            *buckets
                .entry(record.component.to_string())
                .or_insert(0) += 1;
        }
    }
    Ok(buckets)
}

/// Tail the last `n` records across every file matching a glob
/// pattern (e.g. `/var/log/**/*.log`), newest last, optionally keeping
/// only records at or above `level`. Matched files are read in sorted
/// path order and their records concatenated before the final `n` are
/// taken; unparseable lines are skipped.
///
/// # Errors
/// Returns an error string if the glob pattern is invalid, or if a
/// matched path cannot be read (e.g. it resolves to a directory).
pub fn tail_logs_glob(
    pattern: &str,
    n: usize,
    level: Option<LogLevel>,
) -> Result<Vec<String>, String> {
    let mut paths: Vec<std::path::PathBuf> = glob::glob(pattern)
        .map_err(|e| format!("invalid glob pattern: {e}"))?
        .filter_map(Result::ok)
        .collect();
    paths.sort();
    let mut all = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        for line in content.lines() {
            if let Ok(record) = parse_record(line) {
                if let Some(min) = level
                    && record.level.to_numeric() < min.to_numeric()
                {
                    continue;
                }
                all.push(render(record, LogFormat::Logfmt));
            }
        }
    }
    let start = all.len().saturating_sub(n);
    Ok(all.split_off(start))
}
