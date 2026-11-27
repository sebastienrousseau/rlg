// source.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Source locations as records carry them in `caller`.

/// `file:line`, with `file` cut down by [`source_path`].
pub(super) fn source_location(file: &str, line: u32) -> String {
    let mut digits = itoa::Buffer::new();
    let digits = digits.format(line);
    let file = source_path(file);
    let mut out = String::with_capacity(file.len() + 1 + digits.len());
    out.push_str(file);
    out.push(':');
    out.push_str(digits);
    out
}

/// A source path fit for a log line. A relative path (a workspace
/// crate's `file!()`) is kept. An absolute one, which is what a crate
/// built from the registry or a vendored tree records, would put the
/// build machine's directories into every record, so it is cut to
/// start at the crate's directory (the one holding `src`), or to the
/// file name when there is no `src`.
pub(super) fn source_path(file: &str) -> &str {
    if !is_absolute(file) {
        return file;
    }
    crate_dir_start(file).map_or_else(
        || file.rsplit(is_sep).next().unwrap_or(file),
        |start| &file[start..],
    )
}

const fn is_sep(c: char) -> bool {
    c == '/' || c == '\\'
}

fn is_absolute(file: &str) -> bool {
    file.starts_with(is_sep) || file.as_bytes().get(1) == Some(&b':')
}

/// Where the directory holding the last `src` component starts.
fn crate_dir_start(file: &str) -> Option<usize> {
    let src =
        file.match_indices(is_sep).map(|(i, _)| i).rfind(|&i| {
            file[i + 1..].starts_with("src")
                && file[i + 4..].starts_with(is_sep)
        })?;
    file[..src].rfind(is_sep).map(|i| i + 1)
}
