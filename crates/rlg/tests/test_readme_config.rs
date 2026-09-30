// test_readme_config.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `rlg.toml` example in README.md loads: documentation that is
//! executed cannot drift from the types it describes.

#![cfg(not(miri))]

use rlg::config::{Config, LogRotation, LoggingDestination};

/// The fenced TOML block that starts with `# rlg.toml`.
fn readme_example() -> String {
    let readme = include_str!("../README.md");
    let start = readme
        .find("```toml\n# rlg.toml\n")
        .expect("README has an rlg.toml example")
        + "```toml\n".len();
    let len = readme[start..].find("```").expect("example is closed");
    readme[start..start + len].to_string()
}

#[test]
fn the_readme_config_example_loads() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("rlg.log");
    let toml = readme_example()
        .replace("/var/log/rlg.log", log.to_str().unwrap());
    let path = dir.path().join("rlg.toml");
    std::fs::write(&path, toml).unwrap();

    let config =
        Config::load(Some(&path)).expect("README example loads");
    let config = config.read();
    assert_eq!(config.profile, "production");
    assert_eq!(
        config.logging_destinations,
        vec![LoggingDestination::File(log), LoggingDestination::Stdout]
    );
    assert!(matches!(config.log_rotation, Some(LogRotation::Size(_))));
}
