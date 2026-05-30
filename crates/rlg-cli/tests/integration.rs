// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! End-to-end CLI tests: spawn the `rlg` binary with `assert_cmd`,
//! feed it JSON-shaped records, and assert on stdout.

#![allow(missing_docs)]

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

const SAMPLE_INFO: &str = r#"{"session_id":1,"time":"2026-05-30T00:00:00.000000000Z","level":"INFO","component":"svc","description":"hello","format":"JSON","attributes":{"user_id":42}}"#;
const SAMPLE_ERROR: &str = r#"{"session_id":2,"time":"2026-05-30T00:00:00.000000000Z","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;

fn rlg() -> Command {
    Command::cargo_bin("rlg").unwrap()
}

#[test]
fn default_logfmt_output_round_trips() {
    rlg()
        .write_stdin(SAMPLE_INFO)
        .assert()
        .success()
        .stdout(contains("level=info"))
        .stdout(contains("session_id=1"))
        .stdout(contains("component=\"svc\""));
}

#[test]
fn min_level_drops_lower_severity() {
    let input = format!("{SAMPLE_INFO}\n{SAMPLE_ERROR}\n");
    rlg()
        .args(["--min-level", "error"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("level=error"))
        .stdout(contains("boom"))
        .stdout(contains("hello").not());
}

#[test]
fn component_filter_is_exact() {
    let input = format!("{SAMPLE_INFO}\n{SAMPLE_ERROR}\n");
    rlg()
        .args(["--component", "db"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("boom"))
        .stdout(contains("hello").not());
}

#[test]
fn attr_filter_parses_value_as_json() {
    let input = format!("{SAMPLE_INFO}\n{SAMPLE_ERROR}\n");
    rlg()
        .args(["--attr", "user_id=42"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("hello"))
        .stdout(contains("boom").not());
}

#[test]
fn unparseable_lines_pass_through_verbatim() {
    let input = "this is not json\n";
    rlg()
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("this is not json"));
}

#[test]
fn format_json_is_canonical() {
    rlg()
        .args(["--format", "json"])
        .write_stdin(SAMPLE_INFO)
        .assert()
        .success()
        .stdout(contains("\"Format\":\"JSON\""))
        .stdout(contains("\"Level\":\"INFO\""));
}

#[test]
fn format_mcp_wraps_in_jsonrpc() {
    rlg()
        .args(["--format", "mcp"])
        .write_stdin(SAMPLE_INFO)
        .assert()
        .success()
        .stdout(contains("\"jsonrpc\":\"2.0\""))
        .stdout(contains("notifications/log"));
}

#[test]
fn input_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.ndjson");
    std::fs::write(&path, format!("{SAMPLE_INFO}\n{SAMPLE_ERROR}\n"))
        .unwrap();
    rlg()
        .arg(&path)
        .assert()
        .success()
        .stdout(contains("hello"))
        .stdout(contains("boom"));
}

#[test]
fn malformed_attr_flag_errors() {
    rlg()
        .args(["--attr", "no_equals_sign"])
        .write_stdin(SAMPLE_INFO)
        .assert()
        .failure();
}
