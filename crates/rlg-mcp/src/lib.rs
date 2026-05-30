// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MCP (Model Context Protocol) tool implementations and the
//! minimal JSON-RPC 2.0 dispatcher that wires them to a Server.
//!
//! The wire format follows the
//! [MCP specification](https://modelcontextprotocol.io/specification/2025-06-18)'s
//! stdio transport: one JSON-RPC request per line on stdin, one
//! response per line on stdout.
//!
//! The three tools (`tail_log`, `filter_log`, `summarize_errors`)
//! are pure functions over a file path so they can be unit-tested
//! without spinning up the transport loop.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! let recent = rlg_mcp::tail_log(Path::new("/var/log/app.ndjson"), 10).unwrap();
//! for line in recent { println!("{line}"); }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use serde::{Deserialize, Serialize};
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

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 transport (minimal subset of MCP).
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Request {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct Response {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

impl Response {
    /// Construct a successful response.
    #[must_use]
    pub const fn ok(
        id: Option<serde_json::Value>,
        result: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Construct an error response.
    #[must_use]
    pub fn err(
        id: Option<serde_json::Value>,
        code: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(ResponseError {
                code,
                message: message.into(),
            }),
        }
    }
}

/// Dispatch one JSON-RPC request to the appropriate handler.
///
/// Returns `None` for *notifications* (requests without an `id`) —
/// MCP requires no response for notifications.
#[must_use]
pub fn dispatch(req: &Request) -> Option<Response> {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        "initialize" => Ok(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rlg-mcp", "version": env!("CARGO_PKG_VERSION") }
        })),
        "tools/list" => Ok(serde_json::json!({
            "tools": [
                {
                    "name": "tail_log",
                    "description": "Return the last N parseable rlg records from a file (Logfmt).",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "n": { "type": "integer", "minimum": 1, "default": 100 }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "filter_log",
                    "description": "Filter records by level/component/attribute, render in any LogFormat.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "min_level": { "type": "string", "enum": ["TRACE","DEBUG","VERBOSE","INFO","WARN","ERROR","FATAL","CRITICAL"] },
                            "component": { "type": "string" },
                            "format": { "type": "string", "default": "Logfmt" }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "summarize_errors",
                    "description": "Group ERROR-and-above records by component and count them.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"]
                    }
                }
            ]
        })),
        "tools/call" => dispatch_tool_call(&req.params),
        "notifications/initialized" | "notifications/cancelled" => {
            // MCP notifications — no response required.
            return None;
        }
        other => Err(format!("unknown method: {other}")),
    };

    Some(match result {
        Ok(v) => Response::ok(id, v),
        Err(e) => Response::err(id, -32_603, e),
    })
}

fn dispatch_tool_call(
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let name = params
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "missing `name`".to_string())?;
    let args = params.get("arguments").cloned().unwrap_or_default();

    match name {
        "tail_log" => {
            let path = arg_path(&args)?;
            let n = args
                .get("n")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(100) as usize;
            let lines =
                tail_log(&path, n).map_err(|e| e.to_string())?;
            Ok(wrap_text(lines.join("\n")))
        }
        "filter_log" => {
            let path = arg_path(&args)?;
            let mut filter = Filter::new();
            if let Some(lvl) = args
                .get("min_level")
                .and_then(serde_json::Value::as_str)
            {
                let level: LogLevel =
                    lvl.parse().map_err(|e| format!("{e:?}"))?;
                filter = filter.min_level(level);
            }
            if let Some(c) = args
                .get("component")
                .and_then(serde_json::Value::as_str)
            {
                filter = filter.component(c);
            }
            let format = args
                .get("format")
                .and_then(serde_json::Value::as_str)
                .map_or(Ok(LogFormat::Logfmt), |s| {
                    s.parse::<LogFormat>().map_err(|e| e.to_string())
                })?;
            let lines = filter_log(&path, &filter, format)
                .map_err(|e| e.to_string())?;
            Ok(wrap_text(lines.join("\n")))
        }
        "summarize_errors" => {
            let path = arg_path(&args)?;
            let buckets =
                summarize_errors(&path).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "content": [ {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&buckets).unwrap_or_default()
                } ]
            }))
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn arg_path(
    args: &serde_json::Value,
) -> Result<std::path::PathBuf, String> {
    args.get("path")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "missing `path`".to_string())
}

fn wrap_text(s: String) -> serde_json::Value {
    serde_json::json!({
        "content": [ { "type": "text", "text": s } ]
    })
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_log(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    const INFO: &str = r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#;
    const ERROR: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;
    const FATAL: &str = r#"{"session_id":3,"time":"t","level":"FATAL","component":"db","description":"down","format":"JSON","attributes":{}}"#;

    #[test]
    fn tail_log_returns_last_n() {
        let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
        let out = tail_log(f.path(), 2).unwrap();
        assert_eq!(out.len(), 2);
        assert!(out[0].contains("boom"));
        assert!(out[1].contains("down"));
    }

    #[test]
    fn tail_log_handles_short_files() {
        let f = write_log(&format!("{INFO}\n"));
        let out = tail_log(f.path(), 100).unwrap();
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn filter_log_drops_below_min_level() {
        let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
        let filter = Filter::new().min_level(LogLevel::ERROR);
        let out =
            filter_log(f.path(), &filter, LogFormat::Logfmt).unwrap();
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn summarize_errors_groups_by_component() {
        let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
        let buckets = summarize_errors(f.path()).unwrap();
        assert_eq!(buckets.get("db"), Some(&2));
        assert_eq!(buckets.get("svc"), None);
    }

    fn req(method: &str, params: serde_json::Value) -> Request {
        Request {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: method.to_string(),
            params,
        }
    }

    #[test]
    fn dispatch_initialize_returns_capabilities() {
        let r = dispatch(&req("initialize", serde_json::json!({})))
            .expect("response");
        let result = r.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-06-18");
        assert_eq!(result["serverInfo"]["name"], "rlg-mcp");
    }

    #[test]
    fn dispatch_tools_list_returns_three_tools() {
        let r = dispatch(&req("tools/list", serde_json::json!({})))
            .expect("response");
        let tools =
            r.result.unwrap()["tools"].as_array().unwrap().len();
        assert_eq!(tools, 3);
    }

    #[test]
    fn dispatch_unknown_method_returns_error() {
        let r = dispatch(&req("nope", serde_json::json!({})))
            .expect("response");
        assert!(r.error.is_some());
    }

    #[test]
    fn dispatch_notification_returns_none() {
        let mut r =
            req("notifications/initialized", serde_json::json!({}));
        r.id = None;
        assert!(dispatch(&r).is_none());
    }

    #[test]
    fn dispatch_tools_call_tail_log() {
        let f = write_log(&format!("{INFO}\n{ERROR}\n"));
        let call = req(
            "tools/call",
            serde_json::json!({
                "name": "tail_log",
                "arguments": { "path": f.path().to_str().unwrap(), "n": 5 }
            }),
        );
        let r = dispatch(&call).expect("response");
        assert!(r.error.is_none());
        let text = r.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(text.contains("hi"));
        assert!(text.contains("boom"));
    }

    #[test]
    fn dispatch_tools_call_unknown_name_errors() {
        let call = req(
            "tools/call",
            serde_json::json!({ "name": "nope", "arguments": {} }),
        );
        let r = dispatch(&call).expect("response");
        assert!(r.error.is_some());
    }

    #[test]
    fn dispatch_tools_call_missing_path_errors() {
        let call = req(
            "tools/call",
            serde_json::json!({ "name": "tail_log", "arguments": {} }),
        );
        let r = dispatch(&call).expect("response");
        assert!(r.error.is_some());
    }
}
