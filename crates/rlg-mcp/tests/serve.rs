// serve.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A whole session through an in-memory pipe.
//!
//! `tests/server.rs` drives the real binary over stdio, which is the
//! honest end-to-end check but can only assert on lines of text. These
//! hold a session with the SDK's own client, so what is asserted is
//! what a client sees: the negotiated protocol, the catalogue of
//! tools, prompts and resources, and results with their structured
//! half.

#![allow(missing_docs)]

use rmcp::ServiceExt;
use rmcp::model::{
    CallToolRequestParams, ContentBlock, GetPromptRequestParams,
    ProtocolVersion, ReadResourceRequestParams, ResourceContents,
};
use rmcp::service::{RoleClient, RunningService};
use serde_json::{Map, Value, json};
use std::io::Write;

const INFO: &str = r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#;
const ERROR: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;

fn write_log(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

/// A client connected to a fresh server over a duplex pipe.
async fn session() -> RunningService<RoleClient, ()> {
    let (client_io, server_io) = tokio::io::duplex(1 << 16);
    // `serve` returns once the handshake is done, so the server must
    // already be waiting when the client starts talking.
    drop(tokio::spawn(async move {
        if let Ok(server) =
            rlg_mcp::LogServer::new().serve(server_io).await
        {
            let _ = server.waiting().await;
        }
    }));
    ().serve(client_io)
        .await
        .expect("client completes the handshake")
}

fn arguments(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => panic!("arguments must be an object"),
    }
}

fn text_of(content: &[ContentBlock]) -> &str {
    content
        .first()
        .and_then(ContentBlock::as_text)
        .map(|t| t.text.as_str())
        .expect("text content")
}

#[tokio::test]
async fn the_handshake_negotiates_a_current_revision() {
    let client = session().await;
    let info = client.peer_info().expect("initialize result");
    let server_info = info.server_info.as_ref().expect("serverInfo");
    assert_eq!(server_info.name, "rlg-mcp");
    assert_eq!(server_info.version, env!("CARGO_PKG_VERSION"));
    assert!(info.capabilities.tools.is_some(), "tools capability");
    assert!(info.capabilities.prompts.is_some(), "prompts capability");
    assert!(
        info.capabilities.resources.is_some(),
        "resources capability"
    );
    // The SDK client asks for its latest revision that still has an
    // `initialize` handshake; the server must agree to it rather than
    // fall back. (`LATEST` may name a revision with no handshake.)
    assert_eq!(
        info.protocol_version,
        ProtocolVersion::LATEST_WITH_INITIALIZE
    );
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn the_catalogue_is_complete_and_annotated() {
    let client = session().await;
    let tools = client.list_all_tools().await.expect("tools/list");
    let mut names: Vec<&str> =
        tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "filter_log",
            "summarize_errors",
            "tail_log",
            "tail_logs_glob"
        ]
    );
    for tool in &tools {
        assert!(
            tool.description.is_some(),
            "{} undescribed",
            tool.name
        );
        assert!(
            tool.output_schema.is_some(),
            "{} no outputSchema",
            tool.name
        );
        let a = tool.annotations.as_ref().expect("annotations");
        assert_eq!(
            a.read_only_hint,
            Some(true),
            "{} read-only",
            tool.name
        );
    }

    let prompts =
        client.list_all_prompts().await.expect("prompts/list");
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "triage_error_spike");

    let resources =
        client.list_all_resources().await.expect("resources");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].uri, "rlg://log-levels");

    let templates = client
        .list_all_resource_templates()
        .await
        .expect("resources/templates/list");
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].uri_template, "rlg://tail/{path}");
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn a_call_returns_text_and_structured_content() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n"));
    let client = session().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("summarize_errors")
                .with_arguments(arguments(
                    json!({"path": f.path().to_str().unwrap()}),
                )),
        )
        .await
        .expect("tools/call");
    assert_ne!(result.is_error, Some(true), "{result:?}");
    assert!(text_of(&result.content).contains("\"db\": 1"));
    assert_eq!(
        result.structured_content,
        Some(json!({"total": 1, "by_component": {"db": 1}}))
    );
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn a_tool_failure_is_a_result_the_model_can_read() {
    let client = session().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("tail_log").with_arguments(arguments(
                json!({"path": "/definitely/does/not/exist.ndjson"}),
            )),
        )
        .await
        .expect("a missing file is a result, not a protocol error");
    assert_eq!(result.is_error, Some(true));
    assert!(text_of(&result.content).contains("exist.ndjson"));
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn protocol_mistakes_are_readable_results() {
    let client = session().await;
    // A tool the server does not have is a result the model can read,
    // naming the tools it does have.
    let unknown = client
        .call_tool(CallToolRequestParams::new("no_such_tool"))
        .await
        .expect("a result, not a protocol error");
    assert_eq!(unknown.is_error, Some(true));
    let text = text_of(&unknown.content);
    assert!(text.contains("no_such_tool"), "{text}");
    assert!(text.contains("tail_log"), "{text}");

    // A required argument missing is a tool failure naming the field,
    // so the model can supply it.
    let missing = client
        .call_tool(CallToolRequestParams::new("tail_log"))
        .await
        .expect("a result, not a protocol error");
    assert_eq!(missing.is_error, Some(true));
    assert!(text_of(&missing.content).contains("path"), "{missing:?}");
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn every_advertised_tool_is_callable() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n"));
    let path = f.path().to_str().unwrap();
    let dir = f.path().parent().unwrap().display().to_string();
    let name = f.path().file_name().unwrap().to_str().unwrap();
    let pattern = format!("{dir}/{name}");
    let client = session().await;
    for (tool, args) in [
        ("tail_log", json!({"path": path})),
        ("filter_log", json!({"path": path, "min_level": "ERROR"})),
        ("summarize_errors", json!({"path": path})),
        ("tail_logs_glob", json!({"glob_pattern": pattern})),
    ] {
        let result = client
            .call_tool(
                CallToolRequestParams::new(tool)
                    .with_arguments(arguments(args)),
            )
            .await
            .unwrap_or_else(|e| panic!("{tool} rejected: {e}"));
        assert_ne!(result.is_error, Some(true), "{tool}: {result:?}");
        assert!(
            result.structured_content.is_some(),
            "{tool} unstructured"
        );
    }
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn the_prompt_and_resources_are_served() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n"));
    let client = session().await;

    let prompt = client
        .get_prompt(
            GetPromptRequestParams::new("triage_error_spike").with_arguments(
                arguments(
                    json!({"path": "/var/log/app.log", "window_minutes": "15"}),
                ),
            ),
        )
        .await
        .expect("prompts/get");
    let text = prompt.messages[0]
        .content
        .as_text()
        .map(|t| t.text.clone())
        .expect("text content");
    assert!(text.contains("/var/log/app.log"), "{text}");
    assert!(text.contains("15 minutes"), "{text}");

    let levels = client
        .read_resource(ReadResourceRequestParams::new(
            "rlg://log-levels",
        ))
        .await
        .expect("resources/read");
    match &levels.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => {
            assert!(text.contains("CRITICAL"), "{text}");
        }
        other => panic!("unexpected contents {other:?}"),
    }

    let tail = client
        .read_resource(ReadResourceRequestParams::new(format!(
            "rlg://tail/{}",
            f.path().to_str().unwrap()
        )))
        .await
        .expect("resources/read");
    match &tail.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => {
            assert!(text.contains("boom"), "{text}");
        }
        other => panic!("unexpected contents {other:?}"),
    }

    // A URI the server does not serve is a protocol error: nothing ran.
    let missing = client
        .read_resource(ReadResourceRequestParams::new("rlg://mystery"))
        .await;
    assert!(missing.is_err(), "{missing:?}");
    let _ = client.cancel().await.expect("clean close");
}

#[tokio::test]
async fn unknown_prompts_and_unreadable_logs_are_protocol_errors() {
    let client = session().await;

    let prompt = client
        .get_prompt(GetPromptRequestParams::new("no_such_prompt"))
        .await;
    let message = format!("{prompt:?}");
    assert!(
        message.contains("unknown prompt: no_such_prompt"),
        "{message}"
    );
    assert!(message.contains("triage_error_spike"), "{message}");

    let tail = client
        .read_resource(ReadResourceRequestParams::new(
            "rlg://tail//nonexistent/rlg-mcp/app.log",
        ))
        .await;
    let message = format!("{tail:?}");
    assert!(tail.is_err(), "{message}");
    assert!(
        message.contains("/nonexistent/rlg-mcp/app.log"),
        "{message}"
    );
    let _ = client.cancel().await.expect("clean close");
}
