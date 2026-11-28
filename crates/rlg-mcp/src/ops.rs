// ops.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The four operations as plain functions over a file path, callable
//! and testable without a transport.
//!
//! Every read is bounded: a file is read up to [`MAX_FILE_BYTES`], a
//! line longer than [`MAX_LINE_BYTES`] is skipped, and no answer holds
//! more than [`MAX_RECORDS`] records. A tail keeps only the last `n`
//! records as it reads, instead of every record in the file.

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

/// The most bytes read from one file.
pub const MAX_FILE_BYTES: u64 = 64 << 20;
/// The longest line parsed; longer lines are skipped.
pub const MAX_LINE_BYTES: usize = 1 << 20;
/// The most records one answer holds.
pub const MAX_RECORDS: usize = 10_000;

/// Calls `each` with every parseable record of `path`, within the
/// read limits.
fn for_each_record(
    path: &Path,
    mut each: impl FnMut(Log),
) -> std::io::Result<()> {
    let mut reader =
        BufReader::new(File::open(path)?.take(MAX_FILE_BYTES));
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        if line.len() > MAX_LINE_BYTES {
            continue;
        }
        if let Ok(text) = std::str::from_utf8(&line)
            && let Ok(record) = parse_record(text.trim_end())
        {
            each(record);
        }
    }
}

/// The last `n` (at most [`MAX_RECORDS`]) items pushed.
struct Tail<T> {
    n: usize,
    items: VecDeque<T>,
}

impl<T> Tail<T> {
    fn new(n: usize) -> Self {
        let n = n.min(MAX_RECORDS);
        Self {
            n,
            items: VecDeque::with_capacity(n.min(1_024)),
        }
    }

    fn push(&mut self, item: T) {
        if self.n == 0 {
            return;
        }
        if self.items.len() == self.n {
            let _ = self.items.pop_front();
        }
        self.items.push_back(item);
    }

    fn into_vec(self) -> Vec<T> {
        self.items.into()
    }
}

/// Return the last `n` parseable records from `path`, rendered in
/// `Logfmt`. Unparseable lines are skipped.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn tail_log(path: &Path, n: usize) -> std::io::Result<Vec<String>> {
    let mut tail = Tail::new(n);
    for_each_record(path, |r| tail.push(render(r, LogFormat::Logfmt)))?;
    Ok(tail.into_vec())
}

/// Apply `filter` to every record in `path` and return the first
/// [`MAX_RECORDS`] matches rendered in `format`.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn filter_log(
    path: &Path,
    filter: &Filter,
    format: LogFormat,
) -> std::io::Result<Vec<String>> {
    let mut out = Vec::new();
    for_each_record(path, |record| {
        if out.len() < MAX_RECORDS && filter.matches(&record) {
            out.push(render(record, format));
        }
    })?;
    Ok(out)
}

/// Count error+ records grouped by component.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn summarize_errors(
    path: &Path,
) -> std::io::Result<BTreeMap<String, u64>> {
    let mut buckets: BTreeMap<String, u64> = BTreeMap::new();
    for_each_record(path, |record| {
        if record.level.to_numeric() >= LogLevel::ERROR.to_numeric() {
            *buckets
                .entry(record.component.to_string())
                .or_insert(0) += 1;
        }
    })?;
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
    let mut paths: Vec<PathBuf> = glob::glob(pattern)
        .map_err(|e| format!("invalid glob pattern: {e}"))?
        .filter_map(Result::ok)
        .collect();
    paths.sort();
    tail_files(&paths, n, level)
}

/// The last `n` records across `paths`, read in order.
pub(crate) fn tail_files(
    paths: &[PathBuf],
    n: usize,
    level: Option<LogLevel>,
) -> Result<Vec<String>, String> {
    let mut tail = Tail::new(n);
    for path in paths {
        for_each_record(path, |record| {
            let keep = level.is_none_or(|min| {
                record.level.to_numeric() >= min.to_numeric()
            });
            if keep {
                tail.push(render(record, LogFormat::Logfmt));
            }
        })
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    }
    Ok(tail.into_vec())
}
