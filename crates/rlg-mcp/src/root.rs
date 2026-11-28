// root.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The directory the tools may read from.
//!
//! A tool's `path` or `glob_pattern` comes from the MCP client, which
//! may be a model acting on text it was shown. Every path is resolved
//! under one root: symlinks are followed, and anything that ends up
//! outside the root, or is not a regular file, is refused with one
//! message, so the reply does not reveal whether a file exists.

use std::io;
use std::path::{Path, PathBuf};

/// The most files one glob may match.
pub const MAX_GLOB_FILES: usize = 1_000;

/// The directory the tools read under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root(PathBuf);

impl Root {
    /// A root at `dir`, which must exist and be a directory.
    ///
    /// # Errors
    /// When `dir` cannot be resolved or is not a directory.
    pub fn new(dir: impl AsRef<Path>) -> io::Result<Self> {
        let dir = dir.as_ref().canonicalize()?;
        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{} is not a directory", dir.display()),
            ));
        }
        Ok(Self(dir))
    }

    /// The root directory, resolved.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// `requested` (relative to the root, or absolute), resolved, when
    /// it is a regular file under the root.
    ///
    /// # Errors
    /// One message for every refusal: missing, outside the root, or not
    /// a regular file.
    pub fn resolve(&self, requested: &str) -> Result<PathBuf, String> {
        self.contained(&self.0.join(requested))
            .ok_or_else(|| refused(requested))
    }

    /// The regular files under the root that `pattern` (relative to
    /// the root, or absolute) matches, sorted. Matches outside the
    /// root, directories and other non-files are left out.
    ///
    /// # Errors
    /// An invalid pattern, or more than [`MAX_GLOB_FILES`] files.
    pub fn glob(&self, pattern: &str) -> Result<Vec<PathBuf>, String> {
        let full = self.0.join(pattern);
        let text =
            full.to_str().ok_or("the glob pattern is not UTF-8")?;
        let _ = glob::Pattern::new(text)
            .map_err(|e| format!("invalid glob pattern: {e}"))?;
        // The walk starts at the pattern's literal prefix, which must
        // itself be under the root: `/**/*` would otherwise walk the
        // whole file system, even though no match outside is kept.
        if !literal_prefix(&full)
            .canonicalize()
            .is_ok_and(|start| start.starts_with(&self.0))
        {
            return Err(refused(pattern));
        }
        let matches = glob::glob(text)
            .map_err(|e| format!("invalid glob pattern: {e}"))?
            .filter_map(Result::ok);
        self.collect_files(matches)
    }

    /// The matches that are regular files under the root, sorted, or
    /// an error past [`MAX_GLOB_FILES`].
    fn collect_files(
        &self,
        matches: impl Iterator<Item = PathBuf>,
    ) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        for file in matches.filter_map(|m| self.contained(&m)) {
            if files.len() == MAX_GLOB_FILES {
                return Err(format!(
                    "the pattern matches more than {MAX_GLOB_FILES} files; \
                     narrow it"
                ));
            }
            files.push(file);
        }
        files.sort();
        files.dedup();
        Ok(files)
    }

    /// `path` resolved, when it is a regular file under the root.
    fn contained(&self, path: &Path) -> Option<PathBuf> {
        let full = path.canonicalize().ok()?;
        let is_file = full.metadata().ok()?.is_file();
        (is_file && full.starts_with(&self.0)).then_some(full)
    }
}

/// The leading components of `pattern` that hold no glob syntax.
fn literal_prefix(pattern: &Path) -> PathBuf {
    pattern
        .components()
        .take_while(|c| {
            !c.as_os_str()
                .to_string_lossy()
                .contains(['*', '?', '[', ']', '{', '}'])
        })
        .collect()
}

fn refused(requested: &str) -> String {
    format!("`{requested}` is not a log file under the log root")
}

/// The `--root <dir>` (or `--root=<dir>`) argument taken out of
/// `args`, and the arguments left for the transport.
///
/// # Errors
/// `--root` without a value.
pub fn take_root_arg(
    args: impl IntoIterator<Item = String>,
) -> Result<(Option<String>, Vec<String>), String> {
    let mut root = None;
    let mut rest = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--root" {
            root = Some(args.next().ok_or("--root needs a value")?);
        } else if let Some(value) = arg.strip_prefix("--root=") {
            root = Some(value.to_string());
        } else {
            rest.push(arg);
        }
    }
    Ok((root, rest))
}

/// Whether the transport arguments bind beyond loopback: a `--host`
/// other than `127.0.0.1`, `::1` or `localhost`.
#[must_use]
pub fn binds_beyond_loopback(args: &[String]) -> bool {
    let mut host = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--host" {
            host = iter.next().map(String::as_str);
        } else if let Some(value) = arg.strip_prefix("--host=") {
            host = Some(value);
        }
    }
    host.is_some_and(|h| {
        !matches!(h, "127.0.0.1" | "::1" | "[::1]" | "localhost")
    })
}
