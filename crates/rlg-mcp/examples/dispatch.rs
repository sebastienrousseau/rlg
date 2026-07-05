// dispatch.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates driving the MCP server's JSON-RPC 2.0 dispatch
// loop directly from Rust — the same code path the stdio server
// runs, minus the transport. Useful for embedding, custom
// transports (SSE / HTTP), or unit-testing.
//
// Run with: cargo run -p rlg-mcp --example dispatch

#![allow(missing_docs)]

use rlg_mcp::{Request, Response, dispatch};
use serde_json::json;

fn send(method: &str, params: serde_json::Value, id: i64) -> Response {
    let req = Request {
        jsonrpc: "2.0".into(),
        id: Some(json!(id)),
        method: method.into(),
        params,
    };
    dispatch(&req).expect("method calls always return a Response")
}

fn main() {
    // `initialize` handshake — MCP mandates this before any
    // `tools/*` call.
    let init = send("initialize", json!({}), 1);
    println!("initialize -> {}", serde_json::to_string(&init).unwrap());

    // `tools/list` returns the tool catalogue an LLM host will
    // surface to the model.
    let list = send("tools/list", json!({}), 2);
    println!("tools/list -> {}", serde_json::to_string(&list).unwrap());

    // Unknown methods flow back as JSON-RPC error responses. The
    // `-32601` code is JSON-RPC 2.0's reserved "method not found".
    let unknown = send("tools/does_not_exist", json!({}), 3);
    println!(
        "unknown method -> {}",
        serde_json::to_string(&unknown).unwrap()
    );
    assert!(unknown.error.is_some());

    // MCP notifications — `notifications/initialized` and
    // `notifications/cancelled` — return `None`. Per JSON-RPC 2.0
    // semantics the peer must not send a response.
    let notif = Request {
        jsonrpc: "2.0".into(),
        id: None,
        method: "notifications/initialized".into(),
        params: json!({}),
    };
    assert!(dispatch(&notif).is_none());
    println!("notification produced no response (correct)");
}
