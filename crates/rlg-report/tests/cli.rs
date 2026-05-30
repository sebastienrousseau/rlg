// cli.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End-to-end tests for the `rlg-report` binary.

#![allow(missing_docs)]
#![cfg(not(miri))]

use assert_cmd::Command;
use predicates::str::contains;

const SAMPLE: &str = concat!(
    r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#,
    "\n",
    r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":120}}"#,
    "\n",
    r#"{"session_id":3,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{"latency_ms":80}}"#,
    "\n"
);

fn rr() -> Command {
    Command::cargo_bin("rlg-report").unwrap()
}

#[test]
fn default_text_output_lists_sections() {
    rr().write_stdin(SAMPLE)
        .assert()
        .success()
        .stdout(contains("by level"))
        .stdout(contains("by component"))
        .stdout(contains("top descriptions"))
        .stdout(contains("latency"))
        .stdout(contains("ERROR"))
        .stdout(contains("boom"));
}

#[test]
fn json_output_is_valid_json() {
    let assert = rr()
        .args(["--format", "json"])
        .write_stdin(SAMPLE)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout)
        .into_owned();
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["total"], 3);
    assert_eq!(v["count_by_level"]["ERROR"], 2);
    assert_eq!(v["latency"]["samples"], 2);
}

#[test]
fn reads_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.ndjson");
    std::fs::write(&path, SAMPLE).unwrap();
    rr().arg(&path)
        .assert()
        .success()
        .stdout(contains("total records:      3"));
}

#[test]
fn top_flag_clamps_descriptions() {
    let assert = rr()
        .args(["--format", "json", "--top", "1"])
        .write_stdin(SAMPLE)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout)
        .into_owned();
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["top_descriptions"].as_array().unwrap().len(), 1);
}
