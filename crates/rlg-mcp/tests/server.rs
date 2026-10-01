// server.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The server over a real pipe.
//!
//! The tools are unit-tested in the library and the session in
//! `tests/serve.rs`. What these cover is the process around them: the
//! stdio transport's line framing, the handshake a client performs
//! first, which inputs draw a reply at all, and that the process exits
//! cleanly at end of input. All of it is what an MCP client depends
//! on, and none of it is reachable from a unit test.

#![allow(missing_docs)]

use std::io::Write as _;
use std::process::{Command, Stdio};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#;
const INITIALIZED: &str =
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const ERROR: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;

/// Send `lines` to the server and collect the replies.
fn converse(lines: &[&str]) -> Vec<String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("server starts");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for l in lines {
            writeln!(stdin, "{l}").expect("write");
        }
    }
    // Dropping stdin signals end of input; the server must then exit.
    let out = child.wait_with_output().expect("server exits");
    assert!(
        out.status.success(),
        "server exited with {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The reply carrying `id`.
///
/// Requests are dispatched concurrently, so replies may come back in
/// any order; a client correlates them by id, and so must a test.
fn by_id(replies: &[String], id: u32) -> &str {
    let needle = format!("\"id\":{id},");
    let alt = format!("\"id\":{id}}}");
    replies
        .iter()
        .find(|r| r.contains(&needle) || r.contains(&alt))
        .unwrap_or_else(|| {
            panic!("no reply with id {id} in {replies:?}")
        })
}

fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");
    let rest = &line[line.find(&needle)? + needle.len()..];
    let rest = rest.trim_start().strip_prefix('"')?;
    rest.find('"').map(|end| &rest[..end])
}

fn log_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "{ERROR}").unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn one_request_draws_exactly_one_line() {
    let replies = converse(&[INITIALIZE]);
    assert_eq!(replies.len(), 1, "{replies:?}");
    assert!(replies[0].contains("\"id\":1"), "{}", replies[0]);
    assert_eq!(
        field(&replies[0], "protocolVersion"),
        Some("2025-11-25")
    );
    assert_eq!(field(&replies[0], "name"), Some("rlg-mcp"));
}

#[test]
fn an_older_client_is_answered_in_its_own_revision() {
    // A client pinned to 2024-11-05 gets 2024-11-05 back, not a newer
    // revision it would have to refuse.
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"old","version":"0"}}}"#,
    ]);
    assert_eq!(
        field(&replies[0], "protocolVersion"),
        Some("2024-11-05")
    );
}

#[test]
fn every_request_in_a_session_is_answered() {
    let f = log_file();
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"summarize_errors","arguments":{{"path":"{}"}}}}}}"#,
        f.path().display()
    );
    let replies = converse(&[
        INITIALIZE,
        INITIALIZED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        &call,
        r#"{"jsonrpc":"2.0","id":4,"method":"prompts/list"}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"resources/list"}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"resources/templates/list"}"#,
    ]);
    // Six requests, one notification: six replies.
    assert_eq!(replies.len(), 6, "{replies:?}");
    assert!(by_id(&replies, 1).contains("protocolVersion"));
    assert!(by_id(&replies, 2).contains("tail_logs_glob"));
    let call = by_id(&replies, 3);
    assert!(call.contains("\"db\""), "{call}");
    assert!(call.contains("structuredContent"), "{call}");
    assert!(by_id(&replies, 4).contains("triage_error_spike"));
    assert!(by_id(&replies, 5).contains("rlg://log-levels"));
    assert!(by_id(&replies, 6).contains("rlg://tail/{path}"));
}

#[test]
fn a_stateless_client_needs_no_handshake() {
    // The 2026-07-28 revision has no `initialize`: every request names
    // its protocol version in `_meta`, and the first one may be the
    // real work.
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}"#,
    ]);
    assert_eq!(replies.len(), 2, "{replies:?}");
    assert!(by_id(&replies, 1).contains("tail_log"));
    let discover = by_id(&replies, 2);
    assert!(discover.contains("2026-07-28"), "{discover}");
    assert!(discover.contains("2025-11-25"), "{discover}");
}

#[test]
fn blank_lines_are_skipped_rather_than_answered() {
    // Some clients pad the stream; a reply to a blank line would
    // desynchronise the whole conversation.
    let replies = converse(&["", "   ", INITIALIZE, ""]);
    assert_eq!(replies.len(), 1, "{replies:?}");
}

#[test]
fn a_notification_alone_draws_no_reply_and_exits_cleanly() {
    let replies = converse(&[INITIALIZED]);
    assert!(replies.is_empty(), "{replies:?}");
}

#[test]
fn malformed_input_does_not_stop_the_server() {
    // The connection must survive a bad line: an MCP client would
    // otherwise see the whole server die on one typo. The SDK skips
    // bytes that are not JSON without a reply, and answers JSON that
    // is not a JSON-RPC message with an error.
    let replies = converse(&[
        "this is not json",
        INITIALIZE,
        INITIALIZED,
        r#"{"jsonrpc":"2.0","id":7}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    ]);
    assert_eq!(replies.len(), 3, "{replies:?}");
    assert!(by_id(&replies, 1).contains("protocolVersion"));
    // The SDK cannot bind that error to the request it could not
    // read, so it answers with a null id and the invalid-request code.
    assert!(
        replies.iter().any(|r| r.contains("-32600")),
        "{replies:?}"
    );
    assert!(by_id(&replies, 2).contains("tail_log"));
}

#[test]
fn an_unknown_method_and_tool_are_reported_distinctly() {
    let replies = converse(&[
        INITIALIZE,
        INITIALIZED,
        r#"{"jsonrpc":"2.0","id":2,"method":"no/such"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"no_such_tool","arguments":{}}}"#,
    ]);
    assert_eq!(replies.len(), 3, "{replies:?}");
    // A method the protocol does not have is a JSON-RPC error; a tool
    // the server does not have is a result the model can read.
    assert!(by_id(&replies, 2).contains("-32601"));
    let tool = by_id(&replies, 3);
    assert!(tool.contains("\"isError\":true"), "{tool}");
    assert!(tool.contains("Unknown tool: no_such_tool"), "{tool}");
}

#[test]
fn a_missing_file_is_a_tool_error_not_a_crash() {
    let replies = converse(&[
        INITIALIZE,
        INITIALIZED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"tail_log","arguments":{"path":"/definitely/does/not/exist.ndjson"}}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#,
    ]);
    assert_eq!(replies.len(), 3, "{replies:?}");
    let call = by_id(&replies, 2);
    assert!(call.contains("\"isError\":true"), "{call}");
    assert!(call.contains("exist.ndjson"), "{call}");
    // The session went on afterwards.
    assert!(by_id(&replies, 3).contains("result"));
}

#[test]
fn no_input_at_all_exits_successfully() {
    assert!(converse(&[]).is_empty());
}

#[test]
fn version_and_help_print_and_exit() {
    let out = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .arg("--version")
        .output()
        .expect("runs");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        text.trim(),
        format!("rlg-mcp {}", env!("CARGO_PKG_VERSION"))
    );

    let out = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .arg("--help")
        .output()
        .expect("runs");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("--transport"), "{text}");
    assert!(text.contains("/mcp"), "{text}");
}

#[test]
fn a_bad_argument_is_a_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .args(["--transport", "telepathy"])
        .output()
        .expect("runs");
    assert_eq!(out.status.code(), Some(2));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("telepathy"), "{text}");
    assert!(text.contains("Usage:"), "{text}");
}
