// test_file_safety.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Log files are not world-readable, and rotation never overwrites an
//! archive.

#![allow(missing_docs)]
#![cfg(not(miri))]

use rlg::config::LogRotation;
use rlg::rotation::RotatingFile;
use std::fs;

fn files_in(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn rotations_in_the_same_second_keep_every_archive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.log");
    let mut rf =
        RotatingFile::open(&path, LogRotation::Count(1)).unwrap();
    // Each write rotates; all three happen well inside one second.
    for line in [b"one\n", b"two\n", b"333\n"] {
        rf.write_batch(line, 1).unwrap();
    }
    let names = files_in(dir.path());
    assert_eq!(
        names.len(),
        4,
        "three archives plus the live file: {names:?}"
    );
    let mut contents: Vec<String> = names
        .iter()
        .filter(|n| *n != "app.log")
        .map(|n| fs::read_to_string(dir.path().join(n)).unwrap())
        .collect();
    contents.sort();
    assert_eq!(contents, ["333\n", "one\n", "two\n"]);
}

#[cfg(unix)]
#[test]
fn log_files_are_not_readable_by_others() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private.log");
    let mut rf =
        RotatingFile::open(&path, LogRotation::Count(1)).unwrap();
    let mode = |p: &std::path::Path| {
        fs::metadata(p).unwrap().permissions().mode()
    };
    assert_eq!(mode(&path) & 0o007, 0, "{:o}", mode(&path));
    rf.write_batch(b"x\n", 1).unwrap(); // rotates and reopens
    assert_eq!(mode(&path) & 0o007, 0, "{:o}", mode(&path));

    let nested = dir.path().join("a/b/c.log");
    let mut config = rlg::config::Config::default();
    config.logging_destinations =
        vec![rlg::config::LoggingDestination::File(nested.clone())];
    config.ensure_paths().unwrap();
    assert_eq!(mode(&nested) & 0o007, 0, "{:o}", mode(&nested));
    assert_eq!(
        mode(&dir.path().join("a/b")) & 0o007,
        0,
        "{:o}",
        mode(&dir.path().join("a/b"))
    );
}

#[test]
fn config_debug_and_diff_hide_env_var_values() {
    let mut a = rlg::config::Config::default();
    let _ = a
        .env_vars
        .insert("AWS_SECRET_ACCESS_KEY".into(), "s3cr3t-old".into());
    let mut b = a.clone();
    let _ = b
        .env_vars
        .insert("AWS_SECRET_ACCESS_KEY".into(), "s3cr3t-new".into());
    let debug = format!("{a:?}");
    assert!(debug.contains("AWS_SECRET_ACCESS_KEY"), "{debug}");
    assert!(!debug.contains("s3cr3t"), "{debug}");
    let diff = rlg::config::Config::diff(&a, &b);
    let shown = format!("{diff:?}");
    assert!(diff.contains_key("env_vars"), "{shown}");
    assert!(!shown.contains("s3cr3t"), "{shown}");
}
