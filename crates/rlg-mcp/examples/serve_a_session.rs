// serve_a_session.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates holding an MCP session against `rlg-mcp` without a
// client process: the server that wraps the four tools speaks MCP
// over anything that reads and writes bytes, so a session can be
// held through an in-memory pipe — which is what makes it testable,
// and what this example shows. A host process does the same over the
// server's stdin and stdout, or over HTTP with `--transport`.
//
// Run with: cargo run -p rlg-mcp --example serve_a_session

#![allow(missing_docs)]

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Map, Value, json};
use std::io::Write;

const FIXTURE: &[&str] = &[
    r#"{"session_id":1,"time":"2026-07-04T00:00:00.000000000Z","level":"INFO","component":"auth","description":"login ok","format":"JSON","attributes":{}}"#,
    r#"{"session_id":2,"time":"2026-07-04T00:00:01.000000000Z","level":"ERROR","component":"auth","description":"token expired","format":"JSON","attributes":{}}"#,
    r#"{"session_id":3,"time":"2026-07-04T00:00:02.000000000Z","level":"ERROR","component":"db","description":"deadlock retry","format":"JSON","attributes":{}}"#,
];

fn arguments(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => unreachable!("arguments are an object"),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut file = tempfile::NamedTempFile::new()?;
    for line in FIXTURE {
        writeln!(file, "{line}")?;
    }
    file.flush()?;
    let path = file.path().to_str().expect("utf-8 path").to_owned();

    // The server side runs in its own task: `serve` returns once the
    // handshake is done, so it must already be waiting when the client
    // starts talking.
    let (client_io, server_io) = tokio::io::duplex(1 << 16);
    drop(tokio::spawn(async move {
        if let Ok(server) =
            rlg_mcp::LogServer::new().serve(server_io).await
        {
            let _ = server.waiting().await;
        }
    }));
    let client =
        ().serve(client_io).await.expect("the client completes it");

    let info = client.peer_info().expect("initialize result");
    let server_info = info.server_info.as_ref().expect("serverInfo");
    println!(
        "connected to {} {} speaking MCP {}",
        server_info.name, server_info.version, info.protocol_version
    );

    let tools = client.list_all_tools().await.expect("tools/list");
    let names: Vec<&str> =
        tools.iter().map(|t| t.name.as_ref()).collect();
    println!("tools: {}", names.join(", "));
    assert_eq!(names.len(), 4);

    let result = client
        .call_tool(
            CallToolRequestParams::new("summarize_errors")
                .with_arguments(arguments(json!({"path": path}))),
        )
        .await
        .expect("tools/call");
    let summary = result.structured_content.expect("structured");
    println!(
        "summarize_errors: {} error(s), by component {}",
        summary["total"], summary["by_component"]
    );
    assert_eq!(summary["total"], 2);

    // A tool that cannot do the job says so as a result the model can
    // read, not as a protocol error.
    let missing = client
        .call_tool(
            CallToolRequestParams::new("tail_log").with_arguments(arguments(
                json!({"path": "/definitely/does/not/exist.ndjson"}),
            )),
        )
        .await
        .expect("a result, not a protocol error");
    assert_eq!(missing.is_error, Some(true));
    println!(
        "a missing file is an isError result: {:?}",
        missing.content[0]
    );

    let _ = client.cancel().await.expect("clean close");
    Ok(())
}
